use std::io::Write;
use crate::utilities;
use std::collections::LinkedList;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

struct Buffer
{
    locks_acquired: u32,
    locks_released: u32,
    queue: LinkedList<(u64, String)>,
}

impl Buffer
{
    fn new() -> Self {
        Self {
            locks_released: 0,
            locks_acquired: 0,
            queue: LinkedList::new(),
        }
    }
}

pub struct Logger
{
    stdout: Arc<Mutex<Buffer>>,
    file_io: Arc<Mutex<Buffer>>,
}

impl Logger
{
    fn buffer() -> Arc<Mutex<Buffer>> {
        Arc::new(Mutex::new(Buffer::new()))
    }

    pub fn new() -> Self {
        Self {
            stdout: Self::buffer(),
            file_io: Self::buffer(),
        }
    }
}

// Output Buffered Messages
// Claims the original backing buffer, clearing it in the process
impl Logger
{
    pub fn print_outputs(&self) {
        let mut lock = self.stdout.lock().unwrap();
        let swapped = std::mem::replace(&mut lock.queue, LinkedList::new());
        drop(lock);

        let mut elements: Vec<_> = swapped.into_iter().collect();
        elements.sort_by_key(|x| x.0); // Sort by timestamp
        for (_, msg) in elements {
            println!("{}", msg);
        }
    }

    pub fn write_log(&self) {
        let mut lock = self.file_io.lock().unwrap();
        let swapped = std::mem::replace(&mut lock.queue, LinkedList::new());
        drop(lock);

        const HASH_LOG: &str = "hash.log";
        let path = Path::new(HASH_LOG);
        let file = File::create(path);
        let mut writer = match file {
            Ok(file) => BufWriter::new(file),
            Err(e) => panic!("Filed to create log file:\n{}", e)
        };

        let mut elements: Vec<_> = swapped.into_iter().collect();
        elements.sort_by_key(|x| x.0); // Sort by timestamp
        for (_, msg) in elements {
            let result = writeln!(writer, "{}", msg);
            if let Err(e) = result {
                panic!("Failed to log: '{}'\nError:\n{}", msg, e)
            }
        }

        if let Err(e) = writer.flush() {
            panic!("Failed to flush log:\n{}", e)
        }
    }
}

// Stdout Buffering
//
// Needs a DRY refactor
// https://doc.rust-lang.org/rust-by-example/macros/variadics.html
// Look into how format and println work
// Maybe composing macros with each other
impl Logger
{
    pub fn print_insert(&self, hash: u32, key: Arc<String>, salary: u32) {
        let time = utilities::current_timestamp();
        let queue = Arc::clone(&self.stdout);
        thread::spawn(move || {
            let msg = format!(
                "Inserted {},{},{}",
                hash, key, salary
            );
            let entry = (time, msg);
            let mut lock = queue.lock().unwrap();
            lock.queue.push_back(entry);
            drop(lock);
        });
    }

    pub fn print_insert_failed(&self, hash: u32) {
        let time = utilities::current_timestamp();
        let queue = Arc::clone(&self.stdout);
        thread::spawn(move || {
            let msg = format!(
                "Insert failed. Entry {} is a duplicate.",
                hash
            );
            let entry = (time, msg);
            let mut lock = queue.lock().unwrap();
            lock.queue.push_back(entry);
            drop(lock);
        });
    }

    pub fn print_update(&self, hash: u32, key: Arc<String>, old_salary: u32, new_salary: u32) {
        let time = utilities::current_timestamp();
        let queue = Arc::clone(&self.stdout);
        thread::spawn(move || {
            let msg = format!(
                "Updated record {0} from {0},{1},{2} to {0},{1},{3}",
                hash, key, old_salary, new_salary
            );
            let entry = (time, msg);
            let mut lock = queue.lock().unwrap();
            lock.queue.push_back(entry);
            drop(lock);
        });
    }

    pub fn print_update_failed(&self, hash: u32) {
        let time = utilities::current_timestamp();
        let queue = Arc::clone(&self.stdout);
        thread::spawn(move || {
            let msg = format!(
                "Update failed. Entry {} not found.",
                hash
            );
            let entry = (time, msg);
            let mut lock = queue.lock().unwrap();
            lock.queue.push_back(entry);
            drop(lock);
        });
    }

    pub fn print_delete(&self, hash: u32, key: Arc<String>, salary: u32) {
        let time = utilities::current_timestamp();
        let queue = Arc::clone(&self.stdout);
        thread::spawn(move || {
            let msg = format!(
                "Deleted record for {},{},{}",
                hash, key, salary
            );
            let entry = (time, msg);
            let mut lock = queue.lock().unwrap();
            lock.queue.push_back(entry);
            drop(lock);
        });
    }

    pub fn print_delete_failed(&self, hash: u32) {
        let time = utilities::current_timestamp();
        let queue = Arc::clone(&self.stdout);
        thread::spawn(move || {
            let msg = format!(
                "Entry {} not deleted. Not in database.",
                hash
            );
            let entry = (time, msg);
            let mut lock = queue.lock().unwrap();
            lock.queue.push_back(entry);
            drop(lock);
        });
    }

    pub fn print_search(&self, hash: u32, key: Arc<String>, salary: u32) {
        let time = utilities::current_timestamp();
        let queue = Arc::clone(&self.stdout);
        thread::spawn(move || {
            let msg = format!(
                "Found: {},{},{}",
                hash, key, salary
            );
            let entry = (time, msg);
            let mut lock = queue.lock().unwrap();
            lock.queue.push_back(entry);
            drop(lock);
        });
    }

    pub fn print_search_failed(&self, key: Arc<String>) {
        let time = utilities::current_timestamp();
        let queue = Arc::clone(&self.stdout);
        thread::spawn(move || {
            let msg = format!(
                "{} not found.",
                key
            );
            let entry = (time, msg);
            let mut lock = queue.lock().unwrap();
            lock.queue.push_back(entry);
            drop(lock);
        });
    }

    pub fn print_database(&self,
                          snapshot: Arc<Mutex<LinkedList<(u32, Arc<String>, u32)>>>,
                          time: u64)
    {
        let queue = Arc::clone(&self.stdout);
        thread::spawn(move || {
            // Sort entries by hashcode
            let snapshot = snapshot.lock().unwrap();
            let mut elements: Vec<_> = snapshot.iter().collect();
            elements.sort_by_key(|x| x.0);

            // Prepare to insert into the buffer
            let mut lock = queue.lock().unwrap();
            let header = (time, String::from("Current Database:"));
            lock.queue.push_back(header);
            drop(lock);

            // Enter entries corresponding to the print command snapshot
            for (hash, name, salary) in &elements {
                let msg = format!(
                    "{},{},{}",
                    hash, name, salary
                );
                let entry = (time, msg);
                let mut lock = queue.lock().unwrap();
                lock.queue.push_back(entry);
                drop(lock);
            }
            drop(snapshot);
        });
    }
}

// File IO Buffering
//
// Needs a DRY refactor
// https://doc.rust-lang.org/rust-by-example/macros/variadics.html
// Look into how format and println work
// Maybe composing macros with each other
impl Logger
{
    pub fn log_waiting(&self, priority: u32) {
        let time = utilities::current_timestamp();
        let queue = Arc::clone(&self.file_io);
        thread::spawn(move || {
            let msg = format!("{}: THREAD {} WAITING FOR MY TURN", time, priority);
            let entry = (time, msg);
            let mut lock = queue.lock().unwrap();
            lock.queue.push_back(entry);
            drop(lock);
        });
    }

    pub fn log_awakened(&self, priority: u32) {
        let time = utilities::current_timestamp();
        let queue = Arc::clone(&self.file_io);
        thread::spawn(move || {
            let msg = format!("{}: THREAD {} AWAKENED FOR WORK", time, priority);
            let entry = (time, msg);
            let mut lock = queue.lock().unwrap();
            lock.queue.push_back(entry);
            drop(lock);
        });
    }

    pub fn log_insert(&self, key: Arc<String>, salary: u32, priority: u32, hash: u32) {
        let time = utilities::current_timestamp();
        let queue = Arc::clone(&self.file_io);
        thread::spawn(move || {
            let msg = format!(
                "{}: THREAD {} INSERT,{},{},{}",
                time, priority, hash, key, salary
            );
            let entry = (time, msg);
            let mut lock = queue.lock().unwrap();
            lock.queue.push_back(entry);
            drop(lock);
        });
    }

    pub fn log_update(&self, key: Arc<String>, salary: u32, priority: u32, hash: u32) {
        let time = utilities::current_timestamp();
        let queue = Arc::clone(&self.file_io);
        thread::spawn(move || {
            let msg = format!(
                "{}: THREAD {} UPDATE,{},{},{}",
                time, priority, hash, key, salary
            );
            let entry = (time, msg);
            let mut lock = queue.lock().unwrap();
            lock.queue.push_back(entry);
            drop(lock);
        });
    }

    pub fn log_delete(&self, key: Arc<String>, priority: u32, hash: u32) {
        let time = utilities::current_timestamp();
        let queue = Arc::clone(&self.file_io);
        thread::spawn(move || {
            let msg = format!(
                "{}: THREAD {} DELETE,{},{}",
                time, priority, hash, key
            );
            let entry = (time, msg);
            let mut lock = queue.lock().unwrap();
            lock.queue.push_back(entry);
            drop(lock);
        });
    }

    pub fn log_search(&self, key: Arc<String>, priority: u32, hash: u32) {
        let time = utilities::current_timestamp();
        let queue = Arc::clone(&self.file_io);
        thread::spawn(move || {
            let msg = format!(
                "{}: THREAD {} SEARCH,{},{}",
                time, priority, hash, key
            );
            let entry = (time, msg);
            let mut lock = queue.lock().unwrap();
            lock.queue.push_back(entry);
            drop(lock);
        });
    }

    pub fn log_print(&self, priority: u32) {
        let time = utilities::current_timestamp();
        let queue = Arc::clone(&self.file_io);
        thread::spawn(move || {
            let msg = format!(
                "{}: THREAD {} PRINT",
                time, priority
            );
            let entry = (time, msg);
            let mut lock = queue.lock().unwrap();
            lock.queue.push_back(entry);
            drop(lock);
        });
    }

    pub fn log_write_lock_acquired(&self, priority: u32) {
        let time = utilities::current_timestamp();
        let queue = Arc::clone(&self.file_io);
        thread::spawn(move || {
            let msg = format!("{}: THREAD {} WRITE LOCK ACQUIRED", time, priority);
            let entry = (time, msg);
            let mut lock = queue.lock().unwrap();
            lock.queue.push_back(entry);
            lock.locks_acquired += 1;
            drop(lock);
        });
    }

    pub fn log_write_lock_released(&self, priority: u32) {
        let time = utilities::current_timestamp();
        let queue = Arc::clone(&self.file_io);
        thread::spawn(move || {
            let msg = format!("{}: THREAD {} WRITE LOCK RELEASED", time, priority);
            let entry = (time, msg);
            let mut lock = queue.lock().unwrap();
            lock.queue.push_back(entry);
            lock.locks_released += 1;
            drop(lock);
        });
    }

    pub fn log_read_lock_acquired(&self, priority: u32) {
        let time = utilities::current_timestamp();
        let queue = Arc::clone(&self.file_io);
        thread::spawn(move || {
            let msg = format!("{}: THREAD {} READ LOCK ACQUIRED", time, priority);
            let entry = (time, msg);
            let mut lock = queue.lock().unwrap();
            lock.queue.push_back(entry);
            lock.locks_acquired += 1;
            drop(lock);
        });
    }

    pub fn log_read_lock_released(&self, priority: u32) {
        let time = utilities::current_timestamp();
        let queue = Arc::clone(&self.file_io);
        thread::spawn(move || {
            let msg = format!("{}: THREAD {} READ LOCK RELEASED", time, priority);
            let entry = (time, msg);
            let mut lock = queue.lock().unwrap();
            lock.queue.push_back(entry);
            lock.locks_released += 1;
            drop(lock);
        });
    }

    pub fn log_lock_counts(&self) {
        let time = u64::MAX - 1;
        let queue = Arc::clone(&self.file_io);

        // Non-Threaded
        // Ran by main thread
        let newline = (time, String::from(""));
        let mut lock = queue.lock().unwrap();
        lock.queue.push_back(newline);
        let acquired = lock.locks_acquired;
        let released = lock.locks_released;
        lock.queue.push_back((
            time, format!("Number of lock acquisitions: {}", acquired),
        ));
        lock.queue.push_back((
            time, format!("Number of locks released: {}", released),
        ));
        drop(lock);
    }

    pub fn log_final_table(&self, entries: Arc<Vec<String>>) {
        let time = u64::MAX;
        let queue = Arc::clone(&self.file_io);
        let n = entries.len();
        let mut lock  = queue.lock().unwrap();
        lock.queue.push_back((time, String::from("Final Table:")));
        for i in 0..n {
            let entry = &entries[i];
            lock.queue.push_back((time, entry.clone()));
        }
        drop(lock);
    }
    
    pub fn print_final_table(&self, entries: Arc<Vec<String>>) {
        let n = entries.len();
        println!("Final Table:");
        for i in 0..n {
            let entry = &entries[i];
            println!("{}", entry);
        }
    }
}
