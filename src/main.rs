use crate::io::{collect_commands, Command};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;
use crate::map::ConcurrentEmployeeSalaryMap as ConcurrentMap;

mod hashing;
mod io;
mod map;
mod primes;

fn main()
{
    let commands = collect_commands();
    let hashmap = ConcurrentMap::new_defaulted(commands.len());
    spawn_threads(commands, hashmap);
}

pub fn current_timestamp() -> u64 {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64
}

fn spawn_threads(commands: Vec<Command>, hashmap: ConcurrentMap)
{
    let n = commands.len();
    // Shadowed
    let commands = Arc::new(commands);
    let hashmap = Arc::new(hashmap);

    // Scheduling ordering
    let mutex = Arc::new(Mutex::new(commands[0].priority()));
    let cv = Arc::new(Condvar::new());

    // Spawn loop
    let mut handles = vec![];
    for i in 0..n
    {
        // Increment the reference counters to keep the references alive
        // between individual loop iteration scopes. Since in Rust, each
        // iteration is its own-owned scope
        let commands = Arc::clone(&commands);
        let hashmap = Arc::clone(&hashmap);
        let mutex = Arc::clone(&mutex);
        let cv = Arc::clone(&cv);

        // Spawn thread with scoped data
        let handle = thread::spawn(move || {
            // Current command being parsed
            let command = &commands[i];

            // The next priority thread to be scheduled
            let next: u32 = if i < n - 1 {
                commands[i + 1].priority()
            } else {
                u32::MAX // Case for the last thread
            };

            // Enforce ordered scheduling using conditional variables
            let mut ordering = mutex.lock().unwrap();
            while *ordering != command.priority() {
                ordering = cv.wait(ordering).unwrap();
            }

            // Set the next priority to be scheduled after notifying
            *ordering = next;
            cv.notify_all();

            // Command matching and execution
            // ====================================================================================
            match command {
                Command::Insert { name, salary, priority } => {
                }
                Command::Delete { name, priority } => {
                }
                Command::Search { name, priority } => {
                }
                Command::Print { priority } => {

                }
            }
            // ====================================================================================
        });

        handles.push(handle);
    }

    /* Makes the main thread wait on these spawned threads before proceeding.
     * This is done because the program fully terminates all active threads
     * --regardless of completion-- when the main thread ends (that is, the
     * main function finishes execution).
    */
    for handle in handles {
        handle.join().unwrap();
    }
}
