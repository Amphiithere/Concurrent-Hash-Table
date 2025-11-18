use crate::commands_io::{collect_commands, Command};
use crate::map::ConcurrentEmployeeSalaryMap as Map;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

mod hashing;
mod commands_io;
mod map;
mod primes;
mod utilities;

fn main()
{
    let (commands, inserts) = collect_commands();
    let capacity = primes::search_closest_larger(inserts as u32 * 2);
    let hashmap = Map::new_defaulted(capacity as usize);
    spawn_threads(commands, hashmap);
}



pub fn current_timestamp() -> u64 {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64;
}

fn spawn_threads(mut commands: Vec<Command>, hashmap: Map)
{
    commands.reverse();
    let hashmap = Arc::new(hashmap);

    // Scheduling ordering
    let first = if let Some(first) = commands.last() {
        first.priority()
    } else {
        u32::MAX // Empty commands
    };
    let mutex = Arc::new(Mutex::new(first));
    let cv = Arc::new(Condvar::new());

    // Collect the handles so the main thread can wait on these threads
    let capacity = commands.len() + 1;
    let mut handles = Vec::with_capacity(capacity);

    // Thread spawning loop
    while let Some(command) = commands.pop()
    {
        // Increment reference counters for the loop iteration scope
        let hashmap = hashmap.clone();
        let mutex = mutex.clone();
        let cv = cv.clone();

        // The next priority thread to be scheduled
        let next: u32 = if let Some(next) = commands.last() {
            next.priority()
        } else {
            u32::MAX // Last command has no next
        };

        // Spawn thread with scoped data
        let handle = thread::spawn(move || {
            let priority = command.priority();

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
                    hashmap.insert(name, salary, priority);
                }
                Command::Update { name, salary, priority } => {
                    hashmap.update(name, salary, priority);
                }
                Command::Delete { name, priority } => {
                    hashmap.delete(name, priority);
                }
                Command::Search { name, priority } => {
                    hashmap.search(name, priority);
                }
                Command::Print { priority } => {
                    hashmap.print(priority);
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
