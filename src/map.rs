use crate::primes;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

struct HashRecord
{
    hash: u32,
    name: String,
    salary: u32,
    next: Option<Arc<HashRecord>>
}

impl HashRecord
{
    pub fn new(hash: u32, name: String, salary: u32) -> Self {
        return Self {
            hash,
            name,
            salary,
            next: None
        };
    }
}

impl HashRecord
{
    fn has_next(&self) -> bool {
        return self.next.is_some();
    }
}



pub struct Bucket
{
    entries: Option<Arc<HashRecord>>
}

impl Bucket
{
    fn new() -> Self {
        return Self {
            entries: None
        }
    }
}

impl Bucket
{
    fn is_empty(&self) -> bool {
        return self.entries.is_none();
    }

    fn is_chained(&self) -> bool {
        return self.entries.is_some() && self.entries.as_ref().unwrap().has_next();
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
        return Self {
            vector
        };
    }
}



pub struct ConcurrentEmployeeSalaryMap
{
    // Attributes
    migrating: Arc<AtomicPtr<Backing>>,
    backing: Arc<AtomicPtr<Backing>>,
    capacity: AtomicUsize,
    size: AtomicUsize,
    threshold: f32,
    scaling: f32,
    is_resizing: AtomicBool,
    is_migrating: AtomicBool,
}

// Constructing
impl ConcurrentEmployeeSalaryMap
{
    pub fn new(capacity: usize, threshold: f32, scaling: f32) -> Self
    {
        if capacity <= 0 {
            panic!("Initial capacity must be greater than 0");
        }

        let initial_backing = Arc::new(Backing::new(capacity));
        let pointer = Arc::into_raw(initial_backing) as *mut Backing;
        let atomic = Arc::new(AtomicPtr::new(pointer));

        // 'initial_backing'
        return Self {
            migrating: Arc::new(AtomicPtr::new(ptr::null_mut())),
            backing: atomic,
            capacity: AtomicUsize::new(capacity),
            size: AtomicUsize::new(0),
            threshold,
            scaling,
            is_resizing: AtomicBool::new(false),
            is_migrating: AtomicBool::new(false)
        };
    }

    pub fn new_defaulted(capacity: usize) -> Self {
        return Self::new(capacity, 0.75, 2.0);
    }

    pub fn new_fixed_size(capacity: usize) -> Self {
        return Self::new(capacity, f32::INFINITY, 2.0);
    }
}

// Commands
impl ConcurrentEmployeeSalaryMap
{
    pub fn insert(&self, key: String, salary: u32, priority: u32) -> Option<u32> {
        todo!()
    }

    pub fn update(&self, key: String, salary: u32, priority: u32) -> Option<u32> {
        todo!()
    }

    pub fn delete(&self, key: String, priority: u32) -> Option<u32> {
        todo!()
    }

    pub fn search(&self, key: String, priority: u32) -> Option<u32> {
        todo!()
    }
    
    pub fn print(&self, priority: u32) {
        todo!()
    }
}

// Attributes
impl ConcurrentEmployeeSalaryMap
{
    pub fn capacity(&self) -> usize {
        return self.capacity.load(Ordering::Acquire);
    }

    pub fn size(&self) -> usize {
        return self.size.load(Ordering::Acquire);
    }
}

// Internals
impl ConcurrentEmployeeSalaryMap
{
    // fn resize_old(&self)
    // {
    //     // Only allow one writer to handle resizing and migration
    //     if self.is_resizing.compare_exchange(
    //         false, true, Ordering::AcqRel, Ordering::Acquire
    //     ).is_err()
    //     {
    //         return;
    //     }
    //
    //     let migratory_capacity = self.capacity();
    //     let mut scaled = (migratory_capacity as f32 * self.scaling) as usize;
    //     if !primes::exceeds_largest(scaled as u32) {
    //         scaled = primes::search_closest_larger(scaled as u32) as usize
    //     }
    //
    //     // Create new backing
    //     let new_backing = Arc::new(Backing::new(scaled));
    //     let new_backing_pointer = Arc::into_raw(new_backing) as *mut Backing;
    //     let migrating = self.backing.swap(new_backing_pointer, Ordering::AcqRel);
    //
    //     // Store the backing and related attribute data
    //     self.capacity.store(scaled, Ordering::Release);
    //     self.migrating.store(migrating, Ordering::Release);
    //     self.is_migrating.store(true, Ordering::Release);
    //
    //     // Obtain the vector backing the old backing via raw pointer dereference
    //     let migratory_vector: Vec<RwLock<Bucket>>;
    //     unsafe {
    //         migratory_vector = ptr::read(migrating).vector;
    //     }
    //
    //     // Iterate each bucket to copy them to the new backing
    //     for i in 0..migratory_capacity
    //     {
    //         // Claim the write lock, since it will also be wiped
    //         let mut bucket = migratory_vector[i].write().unwrap();
    //         // Take from the Option to return contents and overwrite with None
    //         let mut current = bucket.entries.take();
    //         while current.is_some()
    //         {
    //             // Migrate this specific record
    //             let record = current.unwrap();
    //             self.migratory_insert(
    //                 record.hash,
    //                 &record.name,
    //                 record.salary
    //             );
    //
    //             // Make it go to the next node
    //             // The Option is technically cloned, but this is cheap
    //             // The encapsulated Arc has its reference incremented
    //             // the internal type T is NOT cloned
    //             current = record.next.clone().take();
    //         }
    //         // Drop write lock
    //         drop(bucket);
    //     }
    //
    //     // The old migrating pointer is no longer needed as it has been fully migrated,
    //     // so drop it to reclaim the memory
    //     self.is_migrating.store(false, Ordering::Release);
    //     self.migrating.store(ptr::null_mut(), Ordering::Release);
    //     unsafe {
    //         drop(Arc::from_raw(migrating));
    //     }
    //     // Resizing has completed
    //     self.is_resizing.store(false, Ordering::Release);
    // }

    fn compress(&self, hash: u32) -> usize {
        return (hash % self.capacity() as u32) as usize;
    }

    fn migratory_insert(&self, hash: u32, name: &String, salary: u32) {
        todo!()
    }
}
