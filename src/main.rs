mod cpu;
mod bus;
mod dram;

use std::{io, env};
use std::fs::File;
use std::io::prelude::*;

// Default DRAM size (128 MiB)

use cpu::*;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("Usage: cargo run <filename>");
    }
    let mut file = File::open(&args[1])?;
    let mut code = Vec::new();
    file.read_to_end(&mut code)?;

    let mut cpu = Cpu::new(code);


    loop {
        // 1. Fetch
        let instruction = match cpu.fetch() {
            Ok(inst) => inst,
            Err(_) => break,
        };

        // 2. Add 4 to the program counter
        cpu.pc += 4;

        // 3. Decode
        // 4. Execute
        match cpu.execute(instruction) {
            Ok(_) => (),
            Err(_) => break,
        }

        if cpu.pc == 0 {
            break;
        }
    }

    Ok(())
}
