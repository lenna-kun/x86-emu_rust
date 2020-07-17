#[macro_use]
extern crate log;

use env_logger;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use std::{
    env,
    fmt,
    fs::File,
    io::Read,
    process,
};

const MEMORY_SIZE: usize = 1024 * 1024;

#[derive(Debug, FromPrimitive)]
enum Register {
    EAX,
    ECX,
    EDX,
    EBX,
    ESP,
    EBP,
    ESI,
    EDI,
    RegisterCount,
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self) // fmt::Debug::fmt(self, f)
    }
}

struct Emulator {
    registers: [u32; Register::RegisterCount as usize],
    eflags: u32,
    memory: Vec<u8>,
    eip: u32,
}

impl Emulator {
    fn create(size: usize, eip: u32, esp: u32) -> Emulator {
        Emulator {
            registers: {
                let mut regs = [0; Register::RegisterCount as usize];
                regs[Register::ESP as usize] = esp;
                regs
            },
            eflags: 0,
            memory: vec![0; size],
            eip: eip,
        }
    }

    fn dump_registers(&self) {
        for i in 0..Register::RegisterCount as usize {
            println!("{} = {:08x}", Register::from_usize(i).unwrap(), self.registers[i]);
        }
        println!("EIP = {:08x}", self.eip);
    }

    fn get_code8(&self, index: usize) -> u8 {
        self.memory[self.eip as usize + index] as u8
    }

    fn get_sign_code8(&self, index: usize) -> i8 {
        self.memory[self.eip as usize + index] as i8
    }

    fn get_code32(&self, index: usize) -> u32 {
        let mut ret = 0;
        for i in 0..4 {
            ret |= (self.get_code8(index + i) as u32) << (i * 8);
        }
        ret
    }

    fn get_sign_code32(&self, index: usize) -> i32 {
        self.get_code32(index) as i32
    }

    fn mov_r32_imm32(&mut self) {
        let reg = self.get_code8(0) - 0xB8;
        let value = self.get_code32(1);
        self.registers[reg as usize] = value;
        self.eip += 5;
    }

    fn short_jump(&mut self) {
        let diff = self.get_sign_code8(1) as i32;
        self.eip = (self.eip as i32 + diff + 2) as u32; // be careful if diff minus
    }

    fn near_jump(&mut self) {
        let diff = self.get_sign_code32(1);
        self.eip = (self.eip as i32 + diff + 5) as u32
    }
}
fn init_instructions() -> Vec<Option<fn(&mut Emulator)>> {
    let mut instructions: Vec<Option<fn(&mut Emulator)>> = (0..256).map(|_| None).collect();

    for i in 0..8 {
        instructions[0xB8 + i] = Some(Emulator::mov_r32_imm32);
    }
    instructions[0xE9] = Some(Emulator::near_jump);
    instructions[0xEB] = Some(Emulator::short_jump);

    instructions
}

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