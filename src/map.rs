use std::ptr;
use crate::hashing;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

pub struct HashRecord
{
    hash: u32,
    pub(crate) name: String,
    salary: u32,
    pub(crate) next: Option<Box<HashRecord>>
}

impl HashRecord
{
    pub fn new<T: Into<String>>(name: T, salary: u32) -> Self
    {
        let name = name.into();
        return Self {
            hash: hashing::one_at_a_time(&name),
            name,
            salary,
            next: None
        };
    }

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
        return if let Some(root) = &self.entries {
            root.has_next()
        } else {
            false
        };
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
        }
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

        // Arc heap allocates the argument
        let initial_backing = Arc::new(Backing::new(capacity));
        // This allows the pointer to the allocation on the heap to be intentionally exposed
        let pointer = Arc::into_raw(initial_backing) as *mut Backing;
        // Wraps the pointer into a managed atomic pointer which will maintain the
        // reference count and allow it to be shared between threads
        let atomic = Arc::new(AtomicPtr::new(pointer));

        // 'initial_backing'
        return Self {
            migrating: Arc::new(AtomicPtr::new(ptr::null_mut())),
            backing: atomic,
            capacity: AtomicUsize::new(capacity),
            size: AtomicUsize::new(0),
            threshold,
            scaling,
            is_migrating: AtomicBool::new(false)
        };
    }

    pub fn new_defaulted(capacity: usize) -> Self {
        return Self::new(capacity, 0.75, 2.0);
    }
}

// Commands
impl ConcurrentEmployeeSalaryMap
{
    pub fn insert(&self, key: &String, salary: u32, priority: u32) -> Option<u32> {
        todo!()
    }

    pub fn delete(&self, key: &String, priority: u32) -> Option<u32> {
        todo!()
    }

    pub fn search(&self, key: &String, priority: u32) -> Option<u32> {
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
        return self.capacity.load(Ordering::Relaxed);
    }

    pub fn size(&self) -> usize {
        return self.size.load(Ordering::Relaxed);
    }

    fn compress(&self, hash: u32) -> usize {
        return (hash % self.capacity() as u32) as usize;
    }
}
