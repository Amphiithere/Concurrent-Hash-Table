
use crate::hashing;
use crate::primes;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex, RwLock};

struct HashRecord
{
    hash: u32,
    name: String,
    salary: u32
}

impl HashRecord
{
    // Constructor
    // ============================================================================================

    pub fn new<T: Into<String>>(name: T, salary: u32) -> Self
    {
        let name = name.into();
        Self {
            hash: hashing::str_one_at_a_time(&name),
            name,
            salary
        }
    }
}

impl PartialEq for HashRecord
{
    fn eq(&self, other: &Self) -> bool {
        return self.hash == other.hash
            && self.name == other.name
            && self.salary == other.salary;
    }
}

// Concurrent Hash Map
// ================================================================================================
// ================================================================================================
// ================================================================================================

pub struct Bucket
{
    head: Option<Rc<RefCell<HashRecord>>>,
    tail: Option<Rc<RefCell<HashRecord>>>,
}

impl Bucket
{
}

pub struct ConcurrentEmployeeSalaryMap
{
    // Attributes
    map: Vec<RwLock<Bucket>>,
    capacity: AtomicUsize,
    size: AtomicUsize,
    threshold: f32,
    scaling: f32,
    // Private Concurrency Internals
    active_readers: AtomicUsize,
    active_writers: AtomicUsize,
    is_resizing: Mutex<bool>,
    resize_cv: Condvar,
}

impl ConcurrentEmployeeSalaryMap
{
}
