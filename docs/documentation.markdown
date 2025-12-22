---
layout: documentation
title: Documentation
permalink: /documentation
---
# Prerequisites
Rust 1.91.1 is required to install and execute this program. You can install Rust through Rustup [here](https://rustup.rs/). Alternatively, you can follow the installation instructions provided by [rust-lang.org](https://rust-lang.org/).

# Installation
1. Clone the repository
2. Navigate to the project directory
3. Build the program using `cargo build`
4. Run the program using `cargo run main.rs` within the `src` directory

# Usage
Upon execution, the program takes input from a file, `commands.txt`, within its folder location, `src`, and simulates multiple threads concurrently accessing a list of employee salaries.

Your input file should begin with `threads,<thread_count>,0`

All subsequent lines will be one of 5 options:

| **Command**                         || **Usage**                                                              |
| ----------------------------------- || ---------------------------------------------------------------------- |
| `insert,<name>,<salary>,<priority>` || Insert `<name>` with `<salary>` into the list                          |
| `delete,<name>,0,<priority>`        || Delete `<name>` from the list                                          |
| `update,<name>,<salary>,<priority>` || Update `<salary>` of a given entry                                     |
| `search,<name>,0,<priority>`        || Searches for `<name>` within the list and prints result to the console |
| `print,0,0,<priority>`              || Prints all entries in the list to the console                          |

The program initially processes the input file, sorting each command by ascending priority. After processing, the program spawns all corresponding threads and begins simulation. The results of each completed thread are printed to the console. A separate log of each completed thread is printed to `hash.log`, located in the same directory as the program.

# Authors
* Main program development – Matthew Santos ([Github](https://github.com/Santos-Matthew))
* Comments, touch-ups, and documentation – Azzy Dotson ([Github](https://github.com/Amphiithere))

# Technical Breakdown
This section serves as a more technical analysis, breaking down the modular components of the program and how they function.

## Preprocessing
The program creates a vector of Commands to hold all commands compiled from the input file. Command is an enum used for strictly matching a raw line of the input file to a defined type of command. Command also provides structure and encapsulation for all supported command types and their arguments.  Each raw line is individually split via the comma (,) character, then checked for a valid format. If the command is invalid, or has the incorrect number of parameters, the program will throw an error and terminate. After the line is validated, it is converted into a Command and inserted into the previously created vector.

After all strings have been verified and inserted into the Command vector, the program sorts the vector by priority in ascending order. The program assumes that every command’s priority value is unique. The program then selects the closest prime that is larger than double the amount of inserts, defined by the return from collect_commands().

## Thread Creation
After preprocessing, the program reverses the order of Commands in the vector. A hashmap is then created with a capacity that was defined after preprocessing. The logger, which handles all hashmap and command printing, is also created. The program uses the minimum priority in the vector to schedule ordering of thread creation. A conditional variable is used to enforce ordering for thread scheduling. A mutex variable is used to lock use of the conditional variable. Mutex is updated every time a thread is successfully spawned.

The program will then sequentially spawn each command as a thread and atomically updates all relevant reference counters (hashmap, logger, mutex, and cv). Each command must be individually isolated from the vector of Commands before each reference counter is updated. Threads will wait for their priority value to match the expected next priority, mutex, before being signaled to run by the conditional variable.

## Command Execution
When a thread is scheduled to run, the program will match the name of the command with the valid list of commands. For update, delete, and search, the program will attempt to find the corresponding name within the hashmap. For insert, the program will add the new entry into the hashmap, assuming that the value will not cause a collision. For print, the program will output all current contents within the hashmap. All command outputs are deferred to the logger to avoid blocking due to I/O.

## Logging
The logger outputs to two different streams using a buffered writer, console (stdout) and a file named hash.log. The logger builds its output to the console and the output file via two different linked lists. Hash.log uses the locks_acquired and locks_released variables within Buffer to track the use of the reader and writer locks. The logger will print to console first, then force the output for hash.log via write caching.

## Cleanup
The main program will wait for all threads to complete before calling the logger to print the buffered output. Main will sleep for the loggers called by the threads to finish. Afterwards, the buffered output for both the console and hash.log will be sent to their respective streams before the program terminates.