use crate::hashing;
use crate::io::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, RwLock};

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
    backing: Option<Arc<HashRecord>>
}

impl Bucket
{
    fn new() -> Self {
        return Self {
            backing: None
        }
    }
}

impl Bucket
{
    fn is_empty(&self) -> bool {
        return self.backing.is_none();
    }

    fn is_chained(&self) -> bool {
        return if let Some(backing) = &self.backing {
            backing.has_next()
        } else {
            false
        };
    }
}



pub struct ConcurrentEmployeeSalaryMap
{
    // Attributes
    backing: Vec<RwLock<Bucket>>,
    capacity: AtomicUsize,
    size: AtomicUsize,
    threshold: f32,
    scaling: f32,
    // Concurrency Internals
    active_threads: AtomicUsize,
    is_resizing: Mutex<bool>,
    resizing_cv: Condvar,
    is_printing: Mutex<bool>,
    printing_cv: Condvar,
}

// Constructing
impl ConcurrentEmployeeSalaryMap
{
    fn create_backing(capacity: usize) -> Vec<RwLock<Bucket>>
    {
        let mut vec = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            vec.push(RwLock::new(Bucket::new()));
        }
        return vec;
    }

    pub fn new(capacity: usize, threshold: f32, scaling: f32) -> Self
    {
        if capacity <= 0 {
            panic!("Initial capacity must be greater than 0");
        }

        return Self {
            backing: Self::create_backing(capacity),
            capacity: AtomicUsize::new(capacity),
            size: AtomicUsize::new(0),
            threshold,
            scaling,
            active_threads: AtomicUsize::new(0),
            is_resizing: Mutex::new(false),
            resizing_cv: Condvar::new(),
            is_printing: Mutex::new(false),
            printing_cv: Condvar::new()
        }
    }

    pub fn new_defaulted(capacity: usize) -> Self {
        return Self::new(capacity, 0.75, 2.0);
    }
}

// Commands
impl ConcurrentEmployeeSalaryMap
{
    pub fn inserted(context: Command) {
        println!()
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

    fn active_threads(&self) -> bool {
        return self.active_threads.load(Ordering::Relaxed) > 1;
    }

    fn compress(&self, hash: u32) -> usize {
        return (hash % self.capacity() as u32) as usize;
    }

    fn increment_active_threads(&self) {
        self.active_threads.fetch_add(1, Ordering::SeqCst);
    }

    fn decrement_active_threads(&self) {
        self.active_threads.fetch_sub(1, Ordering::SeqCst);
    }
}
