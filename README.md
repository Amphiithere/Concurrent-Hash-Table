# Concurrent Hash Table
Concurrent Hash Table is a program that implements the concept of concurrent hash tables using the Rust language. A concurrent hash table uses the hash table data structure to allow concurrent access by multiple threads. This program simulates the scenario of multiple threads attempting to read or modify a list of employee salaries. Due to the nature of concurrency, the resulting output is unlikely to be the same for repeated simulations.

The hash table is of a fixed size, influenced by the number of threads specified within the input file. The program uses Jenkin’s one at a time hash function to determine data placement within the hash table. The program expects input to not cause any hash collisions. If a collision occurs, the program will terminate.

Conditional variables are used to sort operations by priority in ascending order. Locks are applied on a per-bucket basis, allowing fine-tuned locking behavior for threads accessing its contents; however, it is possible to cause race conditions within the program due to the locking system.

# Quickstart Guide
## Prerequisites
Rust 1.91.1 is required to install and execute this program. You can install Rust through Rustup [here](https://rustup.rs/). Alternatively, you can follow the installation instructions provided by [rust-lang.org](https://rust-lang.org/).

## Installation
1. Clone the repository
2. Navigate to the project directory
3. Build the program using `cargo build`
4. Run the program using `cargo run main.rs` within the `src` directory

## Usage
Upon execution, the program takes input from a file, `commands.txt`, within its folder location, `src`, and simulates multiple threads concurrently accessing a list of employee salaries. Commands are sorted by priority and the results are printed to the console. A separate log of each completed thread is printed to `hash.log`, located in the same directory as the program.

## Authors
* Main program development – Matthew Santos ([Github](https://github.com/Santos-Matthew))
* Comments, touch-ups, and documentation – Azzy Dotson ([Github](https://github.com/Amphiithere))