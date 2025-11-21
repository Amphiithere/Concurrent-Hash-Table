#![allow(warnings)]

use crate::command_io::{collect_commands, Command};
use crate::map::ConcurrentEmployeeSalaryMap as Map;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

mod hashing;
mod command_io;
mod map;
mod primes;
mod utilities;
mod logging;

fn main()
{
    // Commands are already sorted by priority in increasing order
    let (commands, inserts) = collect_commands();
    let capacity = primes::search_closest_larger(inserts as u32 * 2);
    let hashmap = Map::new_fixed_size(capacity as usize);
    spawn_threads(commands, hashmap);
}

fn spawn_threads(mut commands: Vec<Command>, hashmap: Map)
{
    commands.reverse();
    let hashmap = Arc::new(hashmap);
    let logger = hashmap.logger();

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
        let hashmap = Arc::clone(&hashmap);
        let logger = Arc::clone(&logger);
        let mutex = Arc::clone(&mutex);
        let cv = Arc::clone(&cv);

        // The next priority thread to be scheduled
        let next: u32 = if let Some(next) = commands.last() {
            next.priority()
        } else {
            u32::MAX // Last command has no next
        };

        // Spawn thread with scoped data
        let handle = thread::spawn(move || {
            // Waiting
            let priority = command.priority();
            logger.log_waiting(priority);

            // Enforce ordered scheduling using conditional variables
            let mut ordering = mutex.lock().unwrap();
            while *ordering != command.priority() {
                ordering = cv.wait(ordering).unwrap();
            }
            // Awakened
            logger.log_awakened(priority);

            // Set the next priority to be scheduled after notifying
            *ordering = next;
            drop(ordering);
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

    // Final Outputs
    thread::sleep(Duration::from_millis(321));
    // Non-threaded
    hashmap.logger().log_lock_counts();
    let entries = hashmap.log_final_database();
    hashmap.logger().print_outputs();
    hashmap.logger().write_log();
    println!("Number of Entries: {}", hashmap.size());
    hashmap.logger().print_final_table(entries);
}
