---
# Feel free to add content and custom Front Matter to this file.
# To modify the layout, see https://jekyllrb.com/docs/themes/#overriding-theme-defaults

layout: default
---
# Concurrent Hash Table
Concurrent Hash Table is a program that implements the concept of concurrent hash tables using the Rust language. A concurrent hash table uses the hash table data structure to allow concurrent access by multiple threads. This program simulates the scenario of multiple threads attempting to read or modify a list of employee salaries. Due to the nature of concurrency, the resulting output is unlikely to be the same for repeated simulations.

The hash table is of a fixed size, influenced by the number of threads specified within the input file. The program uses Jenkin’s one at a time hash function to determine data placement within the hash table. The program expects input to not cause any hash collisions. If a collision occurs, the program will terminate.

Conditional variables are used to sort operations by priority in ascending order. Locks are applied on a per-bucket basis, allowing fine-tuned locking behavior for threads accessing its contents; however, it is possible to cause race conditions within the program due to the locking system.