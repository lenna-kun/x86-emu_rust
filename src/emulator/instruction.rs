use log::*;

use std::process;

use super::Emulator;
use super::Register;

#[repr(C)]
pub union Disp {
    pub disp8: i8,
    pub disp32: u32,
}

#[repr(C)]
pub union Reg {
    pub opcode: u8,
    pub reg: u8,
}

pub struct ModRM {
    pub mode: u8,
    pub reg: Reg,
    pub rm: u8,
    pub sib: u8,
    pub disp: Disp,
}

impl ModRM {
    pub fn parse_modrm(emu: &mut Emulator) -> Self {
        let mut modrm = ModRM {
            mode: 0,
            reg: Reg { opcode: 0 },
            rm: 0,
            sib: 0,
            disp: Disp { disp32: 0 },
        };

        let code = emu.get_code8(0);
        modrm.mode = (code & (0b11 << 6)) >> 6;
        modrm.reg.opcode = (code & (0b111 << 3)) >> 3;
        modrm.rm = code & 0b111;

        emu.eip += 1;

        if modrm.mode != 0b11 && modrm.rm == 0b100 {
            modrm.sib = emu.get_code8(0);
            emu.eip += 1;
        }

        if (modrm.mode == 0b00 && modrm.rm == 0b101) || modrm.mode == 0b10 {
            modrm.disp.disp32 = emu.get_sign_code32(0) as u32;
            emu.eip += 4;
        } else if modrm.mode == 0b01 {
            modrm.disp.disp8 = emu.get_sign_code8(0);
            emu.eip += 1;
        }

        modrm
    }
}

impl Emulator {
    pub fn mov_r32_imm32(&mut self) {
        let reg = self.get_code8(0) - 0xB8;
        let value = self.get_code32(1);
        self.registers[reg as usize] = value;
        self.eip += 5;
    }

    pub fn mov_rm32_imm32(&mut self) {
        self.eip += 1;
        let modrm = ModRM::parse_modrm(self);
        let value = self.get_code32(0);
        self.eip += 4;
        self.set_rm32(&modrm, value);
    }

    pub fn mov_rm32_r32(&mut self) {
        self.eip += 1;
        let modrm = ModRM::parse_modrm(self);
        let r32 = self.get_register32(unsafe { modrm.reg.reg } as usize);
        self.set_rm32(&modrm, r32);
    }

    pub fn mov_r32_rm32(&mut self) {
        self.eip += 1;
        let modrm = ModRM::parse_modrm(self);
        let rm32 = self.get_rm32(&modrm);
        self.set_register32(unsafe { modrm.reg.reg } as usize, rm32);
    }

    pub fn code_83(&mut self) {
        self.eip += 1;
        let modrm = ModRM::parse_modrm(self);
        match unsafe { modrm.reg.opcode } {
            0 => self.add_rm32_imm8(&modrm),
            5 => self.sub_rm32_imm8(&modrm),
            opcode @ _ => {
                error!("not implemented: 83 /{}", opcode);
                process::exit(1);
            }
        }
    }

    pub fn code_ff(&mut self) {
        self.eip += 1;
        let modrm = ModRM::parse_modrm(self);
        match unsafe { modrm.reg.opcode } {
            0 => self.inc_rm32(&modrm),
            opcode @ _ => {
                error!("not implemented: ff /{}", opcode);
                process::exit(1);
            }
        }
    }

    pub fn add_rm32_r32(&mut self) {
        self.eip += 1;
        let modrm = ModRM::parse_modrm(self);
        let r32 = self.get_register32(unsafe { modrm.reg.reg } as usize);
        let rm32 = self.get_rm32(&modrm);
        self.set_rm32(&modrm, rm32 + r32);
    }

    pub fn add_rm32_imm8(&mut self, modrm: &ModRM) {
        let rm32 = self.get_rm32(modrm);
        let imm8 = self.get_sign_code8(0) as u32;
        self.eip += 1;
        self.set_rm32(modrm, rm32 + imm8);
    }

    pub fn sub_rm32_imm8(&mut self, modrm: &ModRM) {
        let rm32 = self.get_rm32(&modrm);
        let imm8 = self.get_sign_code8(0);
        self.eip += 1;
        self.set_rm32(modrm, (rm32 as i32 - imm8 as i32) as u32);
    }

    pub fn inc_rm32(&mut self, modrm: &ModRM) {
        let value = self.get_rm32(modrm);
        self.set_rm32(modrm, value + 1);
    }

    pub fn short_jump(&mut self) {
        let diff = self.get_sign_code8(1) as i32;
        self.eip = (self.eip as i32 + diff + 2) as u32; // be careful if diff minus
    }

    pub fn near_jump(&mut self) {
        let diff = self.get_sign_code32(1);
        self.eip = (self.eip as i32 + diff + 5) as u32
    }

    pub fn push_r32(&mut self) {
        let reg = self.get_code8(0) - 0x50;
        self.push32(self.get_register32(reg as usize));
        self.eip += 1;
    }

    pub fn push_imm32(&mut self) {
        let value = self.get_code32(1);
        self.push32(value);
        self.eip += 5;
    }

    pub fn push_imm8(&mut self) {
        let value = self.get_code8(1) as u32;
        self.push32(value);
        self.eip += 2;
    }

    pub fn pop_r32(&mut self) {
        let reg = self.get_code8(0) - 0x58;
        let value = self.pop32();
        self.set_register32(reg as usize, value);
        self.eip += 1;
    }

    pub fn call_rel32(&mut self) {
        let diff = self.get_sign_code32(1);
        self.push32(self.eip + 5);
        self.eip = (self.eip as i32 + diff + 5) as u32;
    }

    pub fn ret(&mut self) {
        self.eip = self.pop32();
    }

    pub fn leave(&mut self) {
        self.set_register32(Register::ESP as usize, self.get_register32(Register::EBP as usize));
        let value = self.pop32();
        self.set_register32(Register::EBP as usize, value);
        self.eip += 1;
    }
}

pub fn init_instructions() -> Vec<Option<fn(&mut Emulator)>> {
    let mut instructions: Vec<Option<fn(&mut Emulator)>> = (0..256).map(|_| None).collect();

    instructions[0x01] = Some(Emulator::add_rm32_r32);
    for i in 0..8 {
        instructions[0x50 + i] = Some(Emulator::push_r32);
    }
    for i in 0..8 {
        instructions[0x58 + i] = Some(Emulator::pop_r32);
    }
    instructions[0x68] = Some(Emulator::push_imm32);
    instructions[0x6A] = Some(Emulator::push_imm8);
    instructions[0x83] = Some(Emulator::code_83);
    instructions[0x89] = Some(Emulator::mov_rm32_r32);
    instructions[0x8B] = Some(Emulator::mov_r32_rm32);
    for i in 0..8 {
        instructions[0xB8 + i] = Some(Emulator::mov_r32_imm32);
    }
    instructions[0xC3] = Some(Emulator::ret);
    instructions[0xC7] = Some(Emulator::mov_rm32_imm32);
    instructions[0xC9] = Some(Emulator::leave);
    instructions[0xE8] = Some(Emulator::call_rel32);
    instructions[0xE9] = Some(Emulator::near_jump);
    instructions[0xEB] = Some(Emulator::short_jump);
    instructions[0xFF] = Some(Emulator::code_ff);

    instructions
}