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
(pending)