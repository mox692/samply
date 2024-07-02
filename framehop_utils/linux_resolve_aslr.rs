//! An example linux program that get a process_map and calc aslr slice.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process;

fn read_process_map() -> Result<u64, std::io::Error> {
    let path = "/proc/self/maps";
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut start_address: u64 = 0;

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            let start_address_str = line.split('-').next().unwrap();
            start_address = u64::from_str_radix(start_address_str, 16).unwrap();
            println!("The start address of the first memory range is: {:x}", start_address);
        }
        println!("{}", line);
    }

    Ok(start_address)
}

fn main() {
    let pid = process::id();
    println!("PID is {}", pid);

    match read_process_map() {
        Ok(start_address) => {
            println!("main:          {:p}", main as *const ());
            println!("start_address: {:x}", start_address);
            println!("relative main: {:p}",  (main as usize as u64 - start_address) as *const () );
        },
        Err(e) => eprintln!("Failed to read process map: {}", e),
    }
}

