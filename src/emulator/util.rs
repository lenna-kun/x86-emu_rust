use num_traits::FromPrimitive;

use log::*;

use std::process;

use super::Register;
use super::Emulator;
use super::instruction::ModRM;

impl Emulator {
    pub fn dump_registers(&self) {
        for i in 0..Register::RegisterCount as usize {
            println!("{} = {:08x}", Register::from_usize(i).unwrap(), self.registers[i]);
        }
        println!("EIP = {:08x}", self.eip);
    }

    pub fn get_code8(&self, index: usize) -> u8 {
        self.memory[self.eip as usize + index] as u8
    }

    pub fn get_sign_code8(&self, index: usize) -> i8 {
        self.memory[self.eip as usize + index] as i8
    }

    pub fn get_code32(&self, index: usize) -> u32 {
        (0..4).fold(0, |acc, i| acc | ((self.get_code8(index + i) as u32) << (i * 8)))
    }

    pub fn get_sign_code32(&self, index: usize) -> i32 {
        self.get_code32(index) as i32
    }

    pub fn calc_memory_address(&self, modrm: &ModRM) -> u32 {
        match modrm.mode {
            0 => match modrm.rm {
                4 => {
                    error!("not implemented ModRM mod = 0, rm = 4"); // TODO
                    process::exit(1);
                },
                5 => unsafe { modrm.disp.disp32 },
                _ => self.get_register32(modrm.rm as usize),
            },
            1 => match modrm.rm {
                4 => {
                    error!("not implemented ModRM mod = 1, rm = 4"); // TODO
                    process::exit(1);
                },
                _ => (self.get_register32(modrm.rm as usize) as i32 + unsafe { modrm.disp.disp8 } as i32) as u32
            },
            2 => match modrm.rm {
                4 => {
                    error!("not implemented ModRM mod = 2, rm = 4"); // TODO
                    process::exit(1);
                },
                _ => self.get_register32(modrm.rm as usize) + unsafe { modrm.disp.disp32 }
            },
            _ => {
                error!("not implemented ModRM mod = 3");
                process::exit(1);
            }
        }
    }

    pub fn get_rm32(&self, modrm: &ModRM) -> u32 {
        if modrm.mode == 0b11 {
            self.get_register32(modrm.rm as usize)
        } else {
            let address = self.calc_memory_address(modrm);
            self.get_memory32(address as usize)
        }
    }

    pub fn set_rm32(&mut self, modrm: &ModRM, value: u32) {
        if modrm.mode == 0b11 {
            self.set_register32(modrm.rm as usize, value);
        } else {
            let address = self.calc_memory_address(modrm);
            self.set_memory32(address as usize, value);
        }
    }

    pub fn get_register8(&self, index: usize) -> u8 {
        if index < 4 {
            (self.registers[index] & 0xff) as u8
        } else {
            ((self.registers[index - 4] >> 8) & 0xff) as u8
        }
    }

    pub fn set_register8(&mut self, index: usize, value: u8) {
        if index < 4 {
            self.registers[index] = (self.registers[index] & !0xff) | (value as u32);
        } else {
            self.registers[index - 4] = (self.registers[index] & !(0xff << 8)) | ((value as u32) << 8);
        }
    }

    pub fn get_register32(&self, index: usize) -> u32 {
        self.registers[index]
    }

    pub fn set_register32(&mut self, index: usize, value: u32) {
        self.registers[index] = value;
    }

    pub fn get_memory8(&self, address: usize) -> u8 {
        self.memory[address]
    }

    pub fn set_memory8(&mut self, address: usize, value: u8) {
        self.memory[address] = value;
    }

    pub fn get_memory32(&self, address: usize) -> u32 {
        (0..4).fold(0, |acc, i| acc | ((self.get_memory8(address + i) as u32) << (i * 8)))
    }

    pub fn set_memory32(&mut self, address: usize, value: u32) {
        (0..4).for_each(|i| self.set_memory8(address + i, ((value >> (i * 8)) & 0xff) as u8));
    }

    pub fn push32(&mut self, value: u32) {
        let address = self.get_register32(Register::ESP as usize) - 4;
        self.set_register32(Register::ESP as usize, address);
        self.set_memory32(address as usize, value);
    }

    pub fn pop32(&mut self) -> u32 {
        let address = self.get_register32(Register::ESP as usize);
        let ret = self.get_memory32(address as usize);
        self.set_register32(Register::ESP as usize, address + 4);
        ret
    }
}