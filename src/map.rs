use crate::hashing;
use std::collections::LinkedList;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, RwLock};

struct HashRecord
{
    hash: u32,
    name: String,
    salary: u32
}

impl HashRecord
{
    pub fn new<T: Into<String>>(name: T, salary: u32) -> Self
    {
        let name = name.into();
        Self {
            hash: hashing::one_at_a_time(&name),
            name,
            salary
        }
    }
}



pub struct Bucket
{
    backing: LinkedList<Arc<HashRecord>>
}

impl Bucket
{
    fn new() -> Self {
        return Self {
            backing: LinkedList::new()
        }
    }
}

impl Bucket
{
    fn is_empty(&self) -> bool {
        return self.backing.is_empty()
    }

    fn is_chained(&self) -> bool {
        return self.backing.len() > 1;
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

impl ConcurrentEmployeeSalaryMap
{

}

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
        self.active_threads.fetch_and(1, Ordering::Relaxed);
    }

    fn decrement_active_threads(&self) {
        self.active_threads.fetch_sub(1, Ordering::Relaxed);
    }
}
