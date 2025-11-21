use std::collections::LinkedList;
use crate::logging::Logger;
use crate::{hashing, utilities};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

struct HashRecord
{
    hash: u32,
    name: Arc<String>,
    salary: u32,
    next: Option<Arc<Self>>
}

impl HashRecord
{
    pub fn new(hash: u32, name: Arc<String>, salary: u32) -> Self {
        Self {
            hash,
            name,
            salary,
            next: None
        }
    }
}

impl HashRecord
{
    // This is OK in the context of a writer lock being held for bucket access, since the
    // access WILL be exclusive to that thread. And the records are protected by the
    // buckets write lock.
    //
    // This is NOT safe to use for a reader lock (and should never need to be used by one)
    unsafe fn update_salary(&self, new_salary: u32) -> u32 {
        let old_salary = self.salary;
        let pointer = self as *const _ as *mut HashRecord;
        (*pointer).salary = new_salary;
        old_salary
    }

    fn has_next(&self) -> bool {
        self.next.is_some()
    }
}



pub struct Bucket
{
    entries: Option<Arc<HashRecord>>
}

impl Bucket
{
    fn new() -> Self {
        Self {
            entries: None
        }
    }
}

impl Bucket
{
    fn is_empty(&self) -> bool {
        self.entries.is_none()
    }
}



struct Backing
{
    vector: Vec<RwLock<Bucket>>
}

impl Backing
{
    fn new(capacity: usize) -> Self {
        let mut vector: Vec<RwLock<Bucket>> = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            vector.push(RwLock::new(Bucket::new()));
        }
        Self {
            vector
        }
    }
}



pub struct ConcurrentEmployeeSalaryMap
{
    logger: Arc<Logger>,
    // Core
    backing: Arc<Backing>,
    capacity: AtomicUsize,
    size: AtomicUsize
}

// Constructing
impl ConcurrentEmployeeSalaryMap
{
    fn backing(capacity: usize) -> Arc<Backing> {
        Arc::new(Backing::new(capacity))
    }

    pub fn new_fixed_size(capacity: usize) -> Self
    {
        Self {
            logger: Arc::new(Logger::new()),
            // Core
            backing: Self::backing(capacity),
            capacity: AtomicUsize::new(capacity),
            size: AtomicUsize::new(0)
        }
    }
}

// Commands
impl ConcurrentEmployeeSalaryMap
{
    pub fn insert(&self, key: String, salary: u32, priority: u32) {
        // Internal reference counter
        let key = Arc::new(key);
        let borrowed_key = Arc::clone(&key);

        // Hash computation
        let hash = hashing::one_at_a_time(&*key);
        self.logger.log_insert(Arc::clone(&borrowed_key), salary, priority, hash);
        let mut insertion = HashRecord::new(hash, key, salary);

        // Resolve bucket
        let backing = Arc::clone(&self.backing);
        let index = self.compress(hash);

        // Writer lock acquisition
        let mut bucket = backing.vector[index].write().unwrap();
        self.logger.log_write_lock_acquired(priority);
        if bucket.is_empty() {
            bucket.entries = Some(Arc::new(insertion));
            self.logger.print_insert(hash, borrowed_key, salary);
            self.size.fetch_add(1, Ordering::SeqCst);
            drop(bucket);
            self.logger.log_write_lock_released(priority);
            return;
        }

        // Search for duplicate
        let mut current = Option::clone(&bucket.entries);
        while let Some(record) = current {
            if record.hash == hash {
                self.logger.print_insert_failed(hash);
                drop(bucket);
                self.logger.log_write_lock_released(priority);
                return;
            }
            current = Option::clone(&record.next);
        }

        insertion.next = bucket.entries.take();
        bucket.entries = Some(Arc::new(insertion));
        self.logger.print_insert(hash, borrowed_key, salary);
        self.size.fetch_add(1, Ordering::SeqCst);
        drop(bucket);
        self.logger.log_write_lock_released(priority);
    }

    pub fn update(&self, key: String, salary: u32, priority: u32) -> Option<u32> {
        // Internal reference counter
        let key = Arc::new(key);
        let borrowed_key = Arc::clone(&key);

        // Hash computation
        let hash = hashing::one_at_a_time(&*key);
        self.logger.log_update(Arc::clone(&borrowed_key), salary, priority, hash);

        // Resolve bucket
        let backing = Arc::clone(&self.backing);
        let index = self.compress(hash);

        // Writer lock acquisition
        let mut bucket = backing.vector[index].write().unwrap();
        self.logger.log_write_lock_acquired(priority);
        if bucket.is_empty() {
            self.logger.print_update_failed(hash);
            drop(bucket);
            self.logger.log_write_lock_released(priority);
            return None;
        }

        // Search for entry
        let mut current = Option::clone(&bucket.entries);
        while let Some(mut record) = current {
            if record.hash == hash {
                let old_salary = unsafe {
                    record.update_salary(salary)
                };
                self.logger.print_update(hash, borrowed_key, old_salary, hash);
                drop(bucket);
                self.logger.log_write_lock_released(priority);
                return Some(old_salary);
            }
            current = Option::clone(&record.next);
        }

        // Bucket did not contain the entry
        self.logger.print_update_failed(hash);
        drop(bucket);
        self.logger.log_write_lock_released(priority);
        None
    }

    pub fn delete(&self, key: String, priority: u32) -> Option<u32> {
        // Internal reference counter
        let key = Arc::new(key);
        let borrowed_key = Arc::clone(&key);

        // Hash computation
        let hash = hashing::one_at_a_time(&*key);
        self.logger.log_delete(Arc::clone(&borrowed_key), priority, hash);

        // Resolve bucket
        let backing = Arc::clone(&self.backing);
        let index = self.compress(hash);

        // Writer lock acquisition
        let mut bucket = backing.vector[index].write().unwrap();
        self.logger.log_write_lock_acquired(priority);
        if bucket.is_empty() {
            self.logger.print_delete_failed(hash);
            drop(bucket);
            self.logger.log_write_lock_released(priority);
            return None;
        }

        // Manually work with the buckets entries
        let chain = bucket.entries.take();
        let head = chain.as_ref().unwrap();

        // Check if the head is the record to delete
        if head.hash == hash {
            let salary = head.salary;
            bucket.entries = Option::clone(&head.next);
            self.logger.print_delete(hash, borrowed_key, salary);
            self.size.fetch_sub(1, Ordering::SeqCst);
            drop(bucket);
            self.logger.log_write_lock_released(priority);
            return Some(salary);
        }

        // By this point, since the head is NOT the record to delete, check if it is
        // chained with more entries, if not then deletion fails
        if head.next.is_none() {
            self.logger.print_delete_failed(hash);
            bucket.entries = chain;
            drop(bucket);
            self.logger.log_write_lock_released(priority);
            return None;
        }

        // Search for if the record is somewhere within the chaining
        let mut previous = Arc::as_ptr(head) as *mut HashRecord;
        let next = Option::as_ref(&head.next).unwrap();
        let mut current = Arc::as_ptr(next) as *mut HashRecord;

        // Use raw pointers to manually manipulate the chain structure
        // Safe due to exclusive ownership by the thread claiming a writer lock
        loop {
            unsafe {
                let record = &*current;

                if record.hash == hash {
                    let salary = record.salary;
                    (*previous).next = Option::clone(&record.next);
                    bucket.entries = chain;
                    self.logger.print_delete(hash, borrowed_key, salary);
                    self.size.fetch_sub(1, Ordering::SeqCst);
                    drop(bucket);
                    self.logger.log_write_lock_released(priority);
                    return Some(salary);
                }

                // Update pointers
                previous = current;
                current = if let Some(ref next) = record.next {
                    Arc::as_ptr(next) as *mut HashRecord
                } else {
                    break;
                }
            }
        }

        // Bucket did not contain the entry
        bucket.entries = chain; // Restore bucket
        self.logger.print_delete_failed(hash);
        drop(bucket);
        self.logger.log_write_lock_released(priority);
        None
    }

    pub fn search(&self, key: String, priority: u32) -> Option<u32> {
        // Internal reference counter
        let key = Arc::new(key);
        let borrowed_key = Arc::clone(&key);

        // Hash computation
        let hash = hashing::one_at_a_time(&*key);
        self.logger.log_search(Arc::clone(&borrowed_key), priority, hash);

        // Resolve bucket
        let backing = Arc::clone(&self.backing);
        let index = self.compress(hash);

        // Reader lock acquisition
        let mut bucket = backing.vector[index].read().unwrap();
        self.logger.log_read_lock_acquired(priority);
        if bucket.is_empty() {
            self.logger.print_search_failed(key);
            drop(bucket);
            self.logger.log_read_lock_released(priority);
            return None;
        }

        // Search the chain if the bucket is not empty
        let mut current = Option::clone(&bucket.entries);
        while let Some(record) = current {
            if record.hash == hash {
                self.logger.print_search(hash, borrowed_key, record.salary);
                let salary = record.salary;
                drop(bucket);
                self.logger.log_read_lock_released(priority);
                return Some(salary);
            }
            current = Option::clone(&record.next);
        }

        // Key was not found
        self.logger.print_search_failed(key);
        drop(bucket);
        self.logger.log_read_lock_released(priority);
        None
    }

    pub fn print(&self, priority: u32) {
        self.logger.log_print(priority);
        // Multithreaded logging
        let time = utilities::current_timestamp();
        // We're basically storing a snapshot of the database
        let buffer = Arc::new(Mutex::new(
            LinkedList::<(u32, Arc<String>, u32)>::new()
        ));

        // Bucket Accessing
        let backing = Arc::clone(&self.backing);
        for i in 0..self.capacity() {
            // For each bucket check its records (if any)
            let bucket = backing.vector[i].read().unwrap();
            self.logger.log_read_lock_acquired(priority);
            let buffer = Arc::clone(&buffer);

            // Read the records in the bucket (if any)
            let mut current = Option::clone(&bucket.entries);
            while let Some(record) = current {
                // Record the necessary information for the printing
                let hash = record.hash;
                let name = Arc::clone(&record.name);
                let salary = record.salary;
                let buffer = Arc::clone(&buffer);
                thread::spawn(move || {
                    let entry = (hash, name, salary);
                    let mut lock = buffer.lock().unwrap();
                    lock.push_back(entry);
                    drop(lock);
                });
                current = Option::clone(&record.next);
            }
            self.logger.log_read_lock_released(priority);
        }

        // Process the buffer
        self.logger.print_database(Arc::clone(&buffer), time);
    }

    pub fn log_final_database(&self) -> Arc<Vec<String>> {

        // We're basically storing a snapshot of the database
        let mut buffer: LinkedList<(u32, String)> = LinkedList::new();

        // Bucket Accessing
        let backing = Arc::clone(&self.backing);
        for i in 0..self.capacity() {
            // For each bucket check its records (if any)
            let bucket = backing.vector[i].read().unwrap();

            // Read the records in the bucket (if any)
            let mut current = Option::clone(&bucket.entries);
            while let Some(record) = current {
                // Create entry and enter
                let entry = format!("{},{},{}", record.hash, record.name, record.salary);
                buffer.push_back((record.hash, entry));
                current = Option::clone(&record.next);
            }
        }

        // Process the buffer
        let mut entries: Vec<_> = buffer.into_iter().collect();
        entries.sort_by_key(|x| x.0); // Sort by hash
        let entries: Vec<String> = entries.into_iter().map(|x| x.1).collect();
        let entries = Arc::new(entries);
        self.logger.log_final_table(Arc::clone(&entries));
        entries
    }
}

// Attributes
impl ConcurrentEmployeeSalaryMap
{
    pub fn capacity(&self) -> usize {
        self.capacity.load(Ordering::Acquire)
    }

    pub fn size(&self) -> usize {
        self.size.load(Ordering::Acquire)
    }
}

// Internals
impl ConcurrentEmployeeSalaryMap
{
    pub fn logger(&self) -> Arc<Logger> {
        Arc::clone(&self.logger)
    }
    
    fn compress(&self, hash: u32) -> usize {
        (hash % self.capacity() as u32) as usize
    }
}
