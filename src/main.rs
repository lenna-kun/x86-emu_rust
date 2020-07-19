#[macro_use]
extern crate log;

use env_logger;

use std::{
    env,
    fs::File,
    io::Read,
    process,
};

use x86_emu_rust::emulator::{
    Emulator, 
    init_instructions,
};

const MEMORY_SIZE: usize = 1024 * 1024;

fn main() {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        error!("Bad nunber of arguments. [filename]");
        process::exit(1);
    }

    let mut emu = Emulator::create(MEMORY_SIZE, 0x7c00, 0x7c00);

    let filename = &args[1];
    if let Ok(f) = File::open(filename) {
        f.bytes().enumerate().for_each(|(i, byte)| {
            emu.memory[i + 0x7c00] = byte.unwrap();
        });
    } else {
        error!("Failed to open {}.", filename);
        process::exit(1);
    }

    let instructions = init_instructions();

    while emu.eip < MEMORY_SIZE as u32 {
        let opcode = emu.get_code8(0);
        debug!("EIP = {:0x}, Opcode = {:02x}", emu.eip, opcode);

        if let Some(instruction) = instructions[opcode as usize] {
            instruction(&mut emu);
        } else {
            error!("Not Implemented: {:0x}", opcode);
            process::exit(1);
        }

        if emu.eip == 0x00 {
            info!("end of program.\n");
            break;
        }
    }

    emu.dump_registers();
}