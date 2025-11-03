use crate::io::collect_commands;

mod hashing;
mod io;
mod map;
mod primes;

fn main()
{
    let commands = collect_commands();
    println!("Running commands:");
    for command in commands {
        println!("{}", command)
    }
}
