use std::collections::HashMap;
use std::fmt;
use serde::{Deserialize, Serialize};

use std::ops::Not;

use crate::cpu::{CPU, two_bytes_to_u16, half_carry_add_8bit, carry_add_8bit};
use crate::registers::{FlagsRegister, RegisterNames16};

#[derive(Debug, Deserialize, Serialize)]
pub struct Instruction {
    pub mnemonic: String,
    pub bytes: u16,
    pub cycles: Vec<u32>,
    pub operands: Vec<Operand>,
    pub immediate: bool,
    pub flags: Flags,
}

pub fn gameboy_doctor_cpu_log(cpu: &CPU) {
  //prints A:00 F:11 B:22 C:33 D:44 E:55 H:66 L:77 SP:8888 PC:9999 PCMEM:AA,BB,CC,DD
  println!("A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}",
              cpu.registers.a, cpu.registers.f, cpu.registers.b, cpu.registers.c, cpu.registers.d, cpu.registers.e, cpu.registers.h, cpu.registers.l, cpu.sp, cpu.pc,
              cpu.read(cpu.pc as usize),
              cpu.read(cpu.pc.wrapping_add(1) as usize),
              cpu.read(cpu.pc.wrapping_add(2) as usize),
              cpu.read(cpu.pc.wrapping_add(3) as usize));
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut operand_string = String::from("");
        for operand in self.operands.iter() {
            operand_string.push_str(&operand.name);
            operand_string.push(' ');
        }
        write!(f, "{} {}", self.mnemonic, operand_string)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Operand {
    pub name: String,
    #[serde(default)]
    pub bytes: Option<u8>,
    pub immediate: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Flags {
    pub Z: String,
    pub N: String,
    pub H: String,
    pub C: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstructionSet {
    pub unprefixed: HashMap<String, Instruction>,
    pub cbprefixed: HashMap<String, Instruction>
}

pub fn execute_cb_prefixed_opcode(cpu: &mut CPU, instruction: &Instruction, opcode: u8, show_debug_messages: bool) -> u32 {
  let mut cycles: u32 = 0;
  if show_debug_messages {
    println!("{:#04X} {:#04X}: {}", cpu.pc, opcode, instruction);
    gameboy_doctor_cpu_log(&cpu);
  }
  match opcode {
    // 0x00-0x07: RLC r
    0x00 => { cpu.registers.b = cpu.rotate_left_with_carry(cpu.registers.b, true); }
    0x01 => { cpu.registers.c = cpu.rotate_left_with_carry(cpu.registers.c, true); }
    0x02 => { cpu.registers.d = cpu.rotate_left_with_carry(cpu.registers.d, true); }
    0x03 => { cpu.registers.e = cpu.rotate_left_with_carry(cpu.registers.e, true); }
    0x04 => { cpu.registers.h = cpu.rotate_left_with_carry(cpu.registers.h, true); }
    0x05 => { cpu.registers.l = cpu.rotate_left_with_carry(cpu.registers.l, true); }
    0x06 => {
      let addr = cpu.registers.get_hl() as usize;
      let rlc_value = cpu.rotate_left_with_carry(cpu.read(addr), true);
      cpu.write(addr, rlc_value);
    }
    0x07 => { cpu.registers.a = cpu.rotate_left_with_carry(cpu.registers.a, true); }

    // 0x08-0x0F: RRC r
    0x08 => { cpu.registers.b = cpu.rotate_right_with_carry(cpu.registers.b, true); }
    0x09 => { cpu.registers.c = cpu.rotate_right_with_carry(cpu.registers.c, true); }
    0x0A => { cpu.registers.d = cpu.rotate_right_with_carry(cpu.registers.d, true); }
    0x0B => { cpu.registers.e = cpu.rotate_right_with_carry(cpu.registers.e, true); }
    0x0C => { cpu.registers.h = cpu.rotate_right_with_carry(cpu.registers.h, true); }
    0x0D => { cpu.registers.l = cpu.rotate_right_with_carry(cpu.registers.l, true); }
    0x0E => {
      let addr = cpu.registers.get_hl() as usize;
      let rrc_value = cpu.rotate_right_with_carry(cpu.read(addr), true);
      cpu.write(addr, rrc_value);
    }
    0x0F => { cpu.registers.a = cpu.rotate_right_with_carry(cpu.registers.a, true); }

    // 0x10-0x17: RL r
    0x10 => { cpu.registers.b = cpu.rotate_left_through_carry(cpu.registers.b, true); }
    0x11 => { cpu.registers.c = cpu.rotate_left_through_carry(cpu.registers.c, true); }
    0x12 => { cpu.registers.d = cpu.rotate_left_through_carry(cpu.registers.d, true); }
    0x13 => { cpu.registers.e = cpu.rotate_left_through_carry(cpu.registers.e, true); }
    0x14 => { cpu.registers.h = cpu.rotate_left_through_carry(cpu.registers.h, true); }
    0x15 => { cpu.registers.l = cpu.rotate_left_through_carry(cpu.registers.l, true); }
    0x16 => {
      let addr = cpu.registers.get_hl() as usize;
      let rl_value = cpu.rotate_left_through_carry(cpu.read(addr), true);
      cpu.write(addr, rl_value);
    }
    0x17 => { cpu.registers.a = cpu.rotate_left_through_carry(cpu.registers.a, true); }

    // 0x18-0x1F: RR r
    0x18 => { cpu.registers.b = cpu.rotate_right_through_carry(cpu.registers.b, true); }
    0x19 => { cpu.registers.c = cpu.rotate_right_through_carry(cpu.registers.c, true); }
    0x1A => { cpu.registers.d = cpu.rotate_right_through_carry(cpu.registers.d, true); }
    0x1B => { cpu.registers.e = cpu.rotate_right_through_carry(cpu.registers.e, true); }
    0x1C => { cpu.registers.h = cpu.rotate_right_through_carry(cpu.registers.h, true); }
    0x1D => { cpu.registers.l = cpu.rotate_right_through_carry(cpu.registers.l, true); }
    0x1E => {
      let addr = cpu.registers.get_hl() as usize;
      let rr_value = cpu.rotate_right_through_carry(cpu.read(addr), true);
      cpu.write(addr, rr_value);
    }
    0x1F => { cpu.registers.a = cpu.rotate_right_through_carry(cpu.registers.a, true); }

    // 0x20-0x27: SLA r
    0x20 => { cpu.registers.b = cpu.shift_left_arithmetic(cpu.registers.b); }
    0x21 => { cpu.registers.c = cpu.shift_left_arithmetic(cpu.registers.c); }
    0x22 => { cpu.registers.d = cpu.shift_left_arithmetic(cpu.registers.d); }
    0x23 => { cpu.registers.e = cpu.shift_left_arithmetic(cpu.registers.e); }
    0x24 => { cpu.registers.h = cpu.shift_left_arithmetic(cpu.registers.h); }
    0x25 => { cpu.registers.l = cpu.shift_left_arithmetic(cpu.registers.l); }
    0x26 => {
      let addr = cpu.registers.get_hl() as usize;
      let sla_value = cpu.shift_left_arithmetic(cpu.read(addr));
      cpu.write(addr, sla_value);
    }
    0x27 => { cpu.registers.a = cpu.shift_left_arithmetic(cpu.registers.a); }

    // 0x28-0x2F: SRA r
    0x28 => { cpu.registers.b = cpu.shift_right_arithmetic(cpu.registers.b); }
    0x29 => { cpu.registers.c = cpu.shift_right_arithmetic(cpu.registers.c); }
    0x2A => { cpu.registers.d = cpu.shift_right_arithmetic(cpu.registers.d); }
    0x2B => { cpu.registers.e = cpu.shift_right_arithmetic(cpu.registers.e); }
    0x2C => { cpu.registers.h = cpu.shift_right_arithmetic(cpu.registers.h); }
    0x2D => { cpu.registers.l = cpu.shift_right_arithmetic(cpu.registers.l); }
    0x2E => {
      let addr = cpu.registers.get_hl() as usize;
      let sra_value = cpu.shift_right_arithmetic(cpu.read(addr));
      cpu.write(addr, sra_value);
    }
    0x2F => { cpu.registers.a = cpu.shift_right_arithmetic(cpu.registers.a); }

    // 0x30-0x37: SWAP r
    0x30 => { cpu.registers.b = cpu.swap_nibbles(cpu.registers.b); }
    0x31 => { cpu.registers.c = cpu.swap_nibbles(cpu.registers.c); }
    0x32 => { cpu.registers.d = cpu.swap_nibbles(cpu.registers.d); }
    0x33 => { cpu.registers.e = cpu.swap_nibbles(cpu.registers.e); }
    0x34 => { cpu.registers.h = cpu.swap_nibbles(cpu.registers.h); }
    0x35 => { cpu.registers.l = cpu.swap_nibbles(cpu.registers.l); }
    0x36 => {
      let addr = cpu.registers.get_hl() as usize;
      let nibble_swap_value = cpu.swap_nibbles(cpu.read(addr));
      cpu.write(addr, nibble_swap_value);
    }
    0x37 => { cpu.registers.a = cpu.swap_nibbles(cpu.registers.a); }

    // 0x38-0x3F: SRL r
    0x38 => { cpu.registers.b = cpu.shift_right_logical(cpu.registers.b); }
    0x39 => { cpu.registers.c = cpu.shift_right_logical(cpu.registers.c); }
    0x3A => { cpu.registers.d = cpu.shift_right_logical(cpu.registers.d); }
    0x3B => { cpu.registers.e = cpu.shift_right_logical(cpu.registers.e); }
    0x3C => { cpu.registers.h = cpu.shift_right_logical(cpu.registers.h); }
    0x3D => { cpu.registers.l = cpu.shift_right_logical(cpu.registers.l); }
    0x3E => {
      let addr = cpu.registers.get_hl() as usize;
      let srl_value = cpu.shift_right_logical(cpu.read(addr));
      cpu.write(addr, srl_value);
    }
    0x3F => { cpu.registers.a = cpu.shift_right_logical(cpu.registers.a); }

    // 0x40-0x47: BIT 0,r
    0x40 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.b, 0); }
    0x41 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.c, 0); }
    0x42 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.d, 0); }
    0x43 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.e, 0); }
    0x44 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.h, 0); }
    0x45 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.l, 0); }
    0x46 => { cpu.copy_bit_n_to_zero_flag(cpu.read(cpu.registers.get_hl() as usize), 0); }
    0x47 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.a, 0); }

    // 0x48-0x4F: BIT 1,r
    0x48 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.b, 1); }
    0x49 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.c, 1); }
    0x4A => { cpu.copy_bit_n_to_zero_flag(cpu.registers.d, 1); }
    0x4B => { cpu.copy_bit_n_to_zero_flag(cpu.registers.e, 1); }
    0x4C => { cpu.copy_bit_n_to_zero_flag(cpu.registers.h, 1); }
    0x4D => { cpu.copy_bit_n_to_zero_flag(cpu.registers.l, 1); }
    0x4E => { cpu.copy_bit_n_to_zero_flag(cpu.read(cpu.registers.get_hl() as usize), 1); }
    0x4F => { cpu.copy_bit_n_to_zero_flag(cpu.registers.a, 1); }

    // 0x50-0x57: BIT 2,r
    0x50 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.b, 2); }
    0x51 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.c, 2); }
    0x52 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.d, 2); }
    0x53 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.e, 2); }
    0x54 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.h, 2); }
    0x55 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.l, 2); }
    0x56 => { cpu.copy_bit_n_to_zero_flag(cpu.read(cpu.registers.get_hl() as usize), 2); }
    0x57 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.a, 2); }

    // 0x58-0x5F: BIT 3,r
    0x58 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.b, 3); }
    0x59 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.c, 3); }
    0x5A => { cpu.copy_bit_n_to_zero_flag(cpu.registers.d, 3); }
    0x5B => { cpu.copy_bit_n_to_zero_flag(cpu.registers.e, 3); }
    0x5C => { cpu.copy_bit_n_to_zero_flag(cpu.registers.h, 3); }
    0x5D => { cpu.copy_bit_n_to_zero_flag(cpu.registers.l, 3); }
    0x5E => { cpu.copy_bit_n_to_zero_flag(cpu.read(cpu.registers.get_hl() as usize), 3); }
    0x5F => { cpu.copy_bit_n_to_zero_flag(cpu.registers.a, 3); }

    // 0x60-0x67: BIT 4,r
    0x60 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.b, 4); }
    0x61 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.c, 4); }
    0x62 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.d, 4); }
    0x63 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.e, 4); }
    0x64 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.h, 4); }
    0x65 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.l, 4); }
    0x66 => { cpu.copy_bit_n_to_zero_flag(cpu.read(cpu.registers.get_hl() as usize), 4); }
    0x67 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.a, 4); }

    // 0x68-0x6F: BIT 5,r
    0x68 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.b, 5); }
    0x69 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.c, 5); }
    0x6A => { cpu.copy_bit_n_to_zero_flag(cpu.registers.d, 5); }
    0x6B => { cpu.copy_bit_n_to_zero_flag(cpu.registers.e, 5); }
    0x6C => { cpu.copy_bit_n_to_zero_flag(cpu.registers.h, 5); }
    0x6D => { cpu.copy_bit_n_to_zero_flag(cpu.registers.l, 5); }
    0x6E => { cpu.copy_bit_n_to_zero_flag(cpu.read(cpu.registers.get_hl() as usize), 5); }
    0x6F => { cpu.copy_bit_n_to_zero_flag(cpu.registers.a, 5); }

    // 0x70-0x77: BIT 6,r
    0x70 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.b, 6); }
    0x71 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.c, 6); }
    0x72 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.d, 6); }
    0x73 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.e, 6); }
    0x74 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.h, 6); }
    0x75 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.l, 6); }
    0x76 => { cpu.copy_bit_n_to_zero_flag(cpu.read(cpu.registers.get_hl() as usize), 6); }
    0x77 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.a, 6); }

    // 0x78-0x7F: BIT 7,r
    0x78 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.b, 7); }
    0x79 => { cpu.copy_bit_n_to_zero_flag(cpu.registers.c, 7); }
    0x7A => { cpu.copy_bit_n_to_zero_flag(cpu.registers.d, 7); }
    0x7B => { cpu.copy_bit_n_to_zero_flag(cpu.registers.e, 7); }
    0x7C => { cpu.copy_bit_n_to_zero_flag(cpu.registers.h, 7); }
    0x7D => { cpu.copy_bit_n_to_zero_flag(cpu.registers.l, 7); }
    0x7E => { cpu.copy_bit_n_to_zero_flag(cpu.read(cpu.registers.get_hl() as usize), 7); }
    0x7F => { cpu.copy_bit_n_to_zero_flag(cpu.registers.a, 7); }

    // 0x80-0x87: RES 0,r
    0x80 => { cpu.registers.b = cpu.reset_bit(cpu.registers.b, 0); }
    0x81 => { cpu.registers.c = cpu.reset_bit(cpu.registers.c, 0); }
    0x82 => { cpu.registers.d = cpu.reset_bit(cpu.registers.d, 0); }
    0x83 => { cpu.registers.e = cpu.reset_bit(cpu.registers.e, 0); }
    0x84 => { cpu.registers.h = cpu.reset_bit(cpu.registers.h, 0); }
    0x85 => { cpu.registers.l = cpu.reset_bit(cpu.registers.l, 0); }
    0x86 => {
      let addr = cpu.registers.get_hl() as usize;
      cpu.write(addr, cpu.reset_bit(cpu.read(addr), 0));
    }
    0x87 => { cpu.registers.a = cpu.reset_bit(cpu.registers.a, 0); }

    // 0x88-0x8F: RES 1,r
    0x88 => { cpu.registers.b = cpu.reset_bit(cpu.registers.b, 1); }
    0x89 => { cpu.registers.c = cpu.reset_bit(cpu.registers.c, 1); }
    0x8A => { cpu.registers.d = cpu.reset_bit(cpu.registers.d, 1); }
    0x8B => { cpu.registers.e = cpu.reset_bit(cpu.registers.e, 1); }
    0x8C => { cpu.registers.h = cpu.reset_bit(cpu.registers.h, 1); }
    0x8D => { cpu.registers.l = cpu.reset_bit(cpu.registers.l, 1); }
    0x8E => {
      let addr = cpu.registers.get_hl() as usize;
      cpu.write(addr, cpu.reset_bit(cpu.read(addr), 1));
    }
    0x8F => { cpu.registers.a = cpu.reset_bit(cpu.registers.a, 1); }

    // 0x90-0x97: RES 2,r
    0x90 => { cpu.registers.b = cpu.reset_bit(cpu.registers.b, 2); }
    0x91 => { cpu.registers.c = cpu.reset_bit(cpu.registers.c, 2); }
    0x92 => { cpu.registers.d = cpu.reset_bit(cpu.registers.d, 2); }
    0x93 => { cpu.registers.e = cpu.reset_bit(cpu.registers.e, 2); }
    0x94 => { cpu.registers.h = cpu.reset_bit(cpu.registers.h, 2); }
    0x95 => { cpu.registers.l = cpu.reset_bit(cpu.registers.l, 2); }
    0x96 => {
      let addr = cpu.registers.get_hl() as usize;
      cpu.write(addr, cpu.reset_bit(cpu.read(addr), 2));
    }
    0x97 => { cpu.registers.a = cpu.reset_bit(cpu.registers.a, 2); }

    // 0x98-0x9F: RES 3,r
    0x98 => { cpu.registers.b = cpu.reset_bit(cpu.registers.b, 3); }
    0x99 => { cpu.registers.c = cpu.reset_bit(cpu.registers.c, 3); }
    0x9A => { cpu.registers.d = cpu.reset_bit(cpu.registers.d, 3); }
    0x9B => { cpu.registers.e = cpu.reset_bit(cpu.registers.e, 3); }
    0x9C => { cpu.registers.h = cpu.reset_bit(cpu.registers.h, 3); }
    0x9D => { cpu.registers.l = cpu.reset_bit(cpu.registers.l, 3); }
    0x9E => {
      let addr = cpu.registers.get_hl() as usize;
      cpu.write(addr, cpu.reset_bit(cpu.read(addr), 3));
    }
    0x9F => { cpu.registers.a = cpu.reset_bit(cpu.registers.a, 3); }

    // 0xA0-0xA7: RES 4,r
    0xA0 => { cpu.registers.b = cpu.reset_bit(cpu.registers.b, 4); }
    0xA1 => { cpu.registers.c = cpu.reset_bit(cpu.registers.c, 4); }
    0xA2 => { cpu.registers.d = cpu.reset_bit(cpu.registers.d, 4); }
    0xA3 => { cpu.registers.e = cpu.reset_bit(cpu.registers.e, 4); }
    0xA4 => { cpu.registers.h = cpu.reset_bit(cpu.registers.h, 4); }
    0xA5 => { cpu.registers.l = cpu.reset_bit(cpu.registers.l, 4); }
    0xA6 => {
      let addr = cpu.registers.get_hl() as usize;
      cpu.write(addr, cpu.reset_bit(cpu.read(addr), 4));
    }
    0xA7 => { cpu.registers.a = cpu.reset_bit(cpu.registers.a, 4); }

    // 0xA8-0xAF: RES 5,r
    0xA8 => { cpu.registers.b = cpu.reset_bit(cpu.registers.b, 5); }
    0xA9 => { cpu.registers.c = cpu.reset_bit(cpu.registers.c, 5); }
    0xAA => { cpu.registers.d = cpu.reset_bit(cpu.registers.d, 5); }
    0xAB => { cpu.registers.e = cpu.reset_bit(cpu.registers.e, 5); }
    0xAC => { cpu.registers.h = cpu.reset_bit(cpu.registers.h, 5); }
    0xAD => { cpu.registers.l = cpu.reset_bit(cpu.registers.l, 5); }
    0xAE => {
      let addr = cpu.registers.get_hl() as usize;
      cpu.write(addr, cpu.reset_bit(cpu.read(addr), 5));
    }
    0xAF => { cpu.registers.a = cpu.reset_bit(cpu.registers.a, 5); }

    // 0xB0-0xB7: RES 6,r
    0xB0 => { cpu.registers.b = cpu.reset_bit(cpu.registers.b, 6); }
    0xB1 => { cpu.registers.c = cpu.reset_bit(cpu.registers.c, 6); }
    0xB2 => { cpu.registers.d = cpu.reset_bit(cpu.registers.d, 6); }
    0xB3 => { cpu.registers.e = cpu.reset_bit(cpu.registers.e, 6); }
    0xB4 => { cpu.registers.h = cpu.reset_bit(cpu.registers.h, 6); }
    0xB5 => { cpu.registers.l = cpu.reset_bit(cpu.registers.l, 6); }
    0xB6 => {
      let addr = cpu.registers.get_hl() as usize;
      cpu.write(addr, cpu.reset_bit(cpu.read(addr), 6));
    }
    0xB7 => { cpu.registers.a = cpu.reset_bit(cpu.registers.a, 6); }

    // 0xB8-0xBF: RES 7,r
    0xB8 => { cpu.registers.b = cpu.reset_bit(cpu.registers.b, 7); }
    0xB9 => { cpu.registers.c = cpu.reset_bit(cpu.registers.c, 7); }
    0xBA => { cpu.registers.d = cpu.reset_bit(cpu.registers.d, 7); }
    0xBB => { cpu.registers.e = cpu.reset_bit(cpu.registers.e, 7); }
    0xBC => { cpu.registers.h = cpu.reset_bit(cpu.registers.h, 7); }
    0xBD => { cpu.registers.l = cpu.reset_bit(cpu.registers.l, 7); }
    0xBE => {
      let addr = cpu.registers.get_hl() as usize;
      cpu.write(addr, cpu.reset_bit(cpu.read(addr), 7));
    }
    0xBF => { cpu.registers.a = cpu.reset_bit(cpu.registers.a, 7); }

    // 0xC0-0xC7: SET 0,r
    0xC0 => { cpu.registers.b = cpu.set_bit(cpu.registers.b, 0); }
    0xC1 => { cpu.registers.c = cpu.set_bit(cpu.registers.c, 0); }
    0xC2 => { cpu.registers.d = cpu.set_bit(cpu.registers.d, 0); }
    0xC3 => { cpu.registers.e = cpu.set_bit(cpu.registers.e, 0); }
    0xC4 => { cpu.registers.h = cpu.set_bit(cpu.registers.h, 0); }
    0xC5 => { cpu.registers.l = cpu.set_bit(cpu.registers.l, 0); }
    0xC6 => {
      let addr = cpu.registers.get_hl() as usize;
      cpu.write(addr, cpu.set_bit(cpu.read(addr), 0));
    }
    0xC7 => { cpu.registers.a = cpu.set_bit(cpu.registers.a, 0); }

    // 0xC8-0xCF: SET 1,r
    0xC8 => { cpu.registers.b = cpu.set_bit(cpu.registers.b, 1); }
    0xC9 => { cpu.registers.c = cpu.set_bit(cpu.registers.c, 1); }
    0xCA => { cpu.registers.d = cpu.set_bit(cpu.registers.d, 1); }
    0xCB => { cpu.registers.e = cpu.set_bit(cpu.registers.e, 1); }
    0xCC => { cpu.registers.h = cpu.set_bit(cpu.registers.h, 1); }
    0xCD => { cpu.registers.l = cpu.set_bit(cpu.registers.l, 1); }
    0xCE => {
      let addr = cpu.registers.get_hl() as usize;
      cpu.write(addr, cpu.set_bit(cpu.read(addr), 1));
    }
    0xCF => { cpu.registers.a = cpu.set_bit(cpu.registers.a, 1); }

    // 0xD0-0xD7: SET 2,r
    0xD0 => { cpu.registers.b = cpu.set_bit(cpu.registers.b, 2); }
    0xD1 => { cpu.registers.c = cpu.set_bit(cpu.registers.c, 2); }
    0xD2 => { cpu.registers.d = cpu.set_bit(cpu.registers.d, 2); }
    0xD3 => { cpu.registers.e = cpu.set_bit(cpu.registers.e, 2); }
    0xD4 => { cpu.registers.h = cpu.set_bit(cpu.registers.h, 2); }
    0xD5 => { cpu.registers.l = cpu.set_bit(cpu.registers.l, 2); }
    0xD6 => {
      let addr = cpu.registers.get_hl() as usize;
      cpu.write(addr, cpu.set_bit(cpu.read(addr), 2));
    }
    0xD7 => { cpu.registers.a = cpu.set_bit(cpu.registers.a, 2); }

    // 0xD8-0xDF: SET 3,r
    0xD8 => { cpu.registers.b = cpu.set_bit(cpu.registers.b, 3); }
    0xD9 => { cpu.registers.c = cpu.set_bit(cpu.registers.c, 3); }
    0xDA => { cpu.registers.d = cpu.set_bit(cpu.registers.d, 3); }
    0xDB => { cpu.registers.e = cpu.set_bit(cpu.registers.e, 3); }
    0xDC => { cpu.registers.h = cpu.set_bit(cpu.registers.h, 3); }
    0xDD => { cpu.registers.l = cpu.set_bit(cpu.registers.l, 3); }
    0xDE => {
      let addr = cpu.registers.get_hl() as usize;
      cpu.write(addr, cpu.set_bit(cpu.read(addr), 3));
    }
    0xDF => { cpu.registers.a = cpu.set_bit(cpu.registers.a, 3); }

    // 0xE0-0xE7: SET 4,r
    0xE0 => { cpu.registers.b = cpu.set_bit(cpu.registers.b, 4); }
    0xE1 => { cpu.registers.c = cpu.set_bit(cpu.registers.c, 4); }
    0xE2 => { cpu.registers.d = cpu.set_bit(cpu.registers.d, 4); }
    0xE3 => { cpu.registers.e = cpu.set_bit(cpu.registers.e, 4); }
    0xE4 => { cpu.registers.h = cpu.set_bit(cpu.registers.h, 4); }
    0xE5 => { cpu.registers.l = cpu.set_bit(cpu.registers.l, 4); }
    0xE6 => {
      let addr = cpu.registers.get_hl() as usize;
      cpu.write(addr, cpu.set_bit(cpu.read(addr), 4));
    }
    0xE7 => { cpu.registers.a = cpu.set_bit(cpu.registers.a, 4); }

    // 0xE8-0xEF: SET 5,r
    0xE8 => { cpu.registers.b = cpu.set_bit(cpu.registers.b, 5); }
    0xE9 => { cpu.registers.c = cpu.set_bit(cpu.registers.c, 5); }
    0xEA => { cpu.registers.d = cpu.set_bit(cpu.registers.d, 5); }
    0xEB => { cpu.registers.e = cpu.set_bit(cpu.registers.e, 5); }
    0xEC => { cpu.registers.h = cpu.set_bit(cpu.registers.h, 5); }
    0xED => { cpu.registers.l = cpu.set_bit(cpu.registers.l, 5); }
    0xEE => {
      let addr = cpu.registers.get_hl() as usize;
      cpu.write(addr, cpu.set_bit(cpu.read(addr), 5));
    }
    0xEF => { cpu.registers.a = cpu.set_bit(cpu.registers.a, 5); }

    // 0xF0-0xF7: SET 6,r
    0xF0 => { cpu.registers.b = cpu.set_bit(cpu.registers.b, 6); }
    0xF1 => { cpu.registers.c = cpu.set_bit(cpu.registers.c, 6); }
    0xF2 => { cpu.registers.d = cpu.set_bit(cpu.registers.d, 6); }
    0xF3 => { cpu.registers.e = cpu.set_bit(cpu.registers.e, 6); }
    0xF4 => { cpu.registers.h = cpu.set_bit(cpu.registers.h, 6); }
    0xF5 => { cpu.registers.l = cpu.set_bit(cpu.registers.l, 6); }
    0xF6 => {
      let addr = cpu.registers.get_hl() as usize;
      cpu.write(addr, cpu.set_bit(cpu.read(addr), 6));
    }
    0xF7 => { cpu.registers.a = cpu.set_bit(cpu.registers.a, 6); }

    // 0xF8-0xFF: SET 7,r
    0xF8 => { cpu.registers.b = cpu.set_bit(cpu.registers.b, 7); }
    0xF9 => { cpu.registers.c = cpu.set_bit(cpu.registers.c, 7); }
    0xFA => { cpu.registers.d = cpu.set_bit(cpu.registers.d, 7); }
    0xFB => { cpu.registers.e = cpu.set_bit(cpu.registers.e, 7); }
    0xFC => { cpu.registers.h = cpu.set_bit(cpu.registers.h, 7); }
    0xFD => { cpu.registers.l = cpu.set_bit(cpu.registers.l, 7); }
    0xFE => {
      let addr = cpu.registers.get_hl() as usize;
      cpu.write(addr, cpu.set_bit(cpu.read(addr), 7));
    }
    0xFF => { cpu.registers.a = cpu.set_bit(cpu.registers.a, 7); }
  }
  
  // if cycles haven't been set already set them now - this will handle the simple case of ops that take a set number of T-states
  if cycles == 0 {
    cycles = *instruction.cycles.first().unwrap();
  }
  cycles
}

pub fn execute_opcode(instruction_set: &InstructionSet, cpu: &mut CPU, show_debug_messages: bool) -> u32 {
  let opcode = cpu.read(cpu.pc as usize);
  let instruction = &instruction_set.unprefixed[&format!("{:#04X}", opcode)];
  if show_debug_messages {
    println!("{:#04X} {:#04X}: {}", cpu.pc, opcode, instruction);
    gameboy_doctor_cpu_log(&cpu);
  }
  let mut cycles: u32 = 0;
  cpu.pc = cpu.pc.wrapping_add(1); // increment PC past the opcode
  match opcode  {
    0x00 => { /* NO OP */ }
    0x10 => {
      cpu.write(0xFF04, 0);
    }
    0xF3 => {
      cpu.ime = 0;
    }
    0xFB => {
      cpu.ime = 1;
    }
    0x07 => { cpu.registers.a = cpu.rotate_left_with_carry(cpu.registers.a, false) }
    0x17 => { cpu.registers.a = cpu.rotate_left_through_carry(cpu.registers.a, false) }
    0x0F => { cpu.registers.a = cpu.rotate_right_with_carry(cpu.registers.a, false) }
    0x1F => { cpu.registers.a = cpu.rotate_right_through_carry(cpu.registers.a, false) }
    0x2F => { 
      let mut flags_register = FlagsRegister::from(cpu.registers.f);
      flags_register.subtract = true;
      flags_register.half_carry = true;
      cpu.registers.f = u8::from(flags_register);
      cpu.registers.a = cpu.registers.a.not(); 
    }
    0x3F => {
      let mut flags_register = FlagsRegister::from(cpu.registers.f);
      flags_register.subtract = false;
      flags_register.half_carry = false;
      flags_register.carry = !flags_register.carry;
      cpu.registers.f = u8::from(flags_register);
    }
    0x27 => { // DAA
      let mut flags_register = FlagsRegister::from(cpu.registers.f);
      let mut adjustment: u8 = 0;
      if flags_register.subtract {
        if flags_register.half_carry {
          adjustment |= 0x06;
        }
        if flags_register.carry {
          adjustment |= 0x60;
        }
        cpu.registers.a = cpu.registers.a.wrapping_sub(adjustment);
      } else {
        if flags_register.half_carry || cpu.registers.a & 0xF > 9 {
          adjustment |= 0x06;
        }
        let mut should_carry = false;
        if flags_register.carry || cpu.registers.a > 0x99 {
          adjustment |= 0x60;
          should_carry = true;
        }
        cpu.registers.a  = cpu.registers.a.wrapping_add(adjustment);
        flags_register.carry = should_carry;
      }
      flags_register.half_carry = false;
      flags_register.zero = cpu.registers.a == 0;
      cpu.registers.f = u8::from(flags_register);
    }
    0x37 => {
      let mut flags_register = FlagsRegister::from(cpu.registers.f);
      flags_register.carry = true;
      flags_register.subtract = false;
      flags_register.half_carry = false;
      cpu.registers.f = u8::from(flags_register);
    }
    /* START JR OPCODES */
    0x18 => {
      cpu.pc = cpu.pc.wrapping_add(cpu.read_i8_at_pc() as u16);
      cpu.pc = cpu.pc.wrapping_add(1);
    }
    0x20 => {
      if !FlagsRegister::from(cpu.registers.f).zero {
        cpu.pc = cpu.pc.wrapping_add(cpu.read_i8_at_pc() as u16);
        cycles = *instruction.cycles.first().unwrap();
      } else {
        cycles = *instruction.cycles.get(1).unwrap();
      }
      cpu.pc = cpu.pc.wrapping_add(1);
    }
    0x28 => {
      if FlagsRegister::from(cpu.registers.f).zero {
        cpu.pc = cpu.pc.wrapping_add(cpu.read_i8_at_pc() as u16);
        cycles = *instruction.cycles.first().unwrap();
      } else {
        cycles = *instruction.cycles.get(1).unwrap();
      }
      cpu.pc = cpu.pc.wrapping_add(1);
    }
    0x30 => {
      if !FlagsRegister::from(cpu.registers.f).carry {
        cpu.pc = cpu.pc.wrapping_add(cpu.read_i8_at_pc() as u16);
        cycles = *instruction.cycles.first().unwrap();
      } else {
        cycles = *instruction.cycles.get(1).unwrap();
      }
      cpu.pc = cpu.pc.wrapping_add(1);
    }
    0x38 => {
      if FlagsRegister::from(cpu.registers.f).carry {
        cpu.pc = cpu.pc.wrapping_add(cpu.read_i8_at_pc() as u16);
        cycles = *instruction.cycles.first().unwrap();
      } else {
        cycles = *instruction.cycles.get(1).unwrap();
      }
      cpu.pc = cpu.pc.wrapping_add(1);
    }
    /* END JR OPCODES */
    /* START LOAD OPCODES */
    /* Start direct loads */
    0x01 => {
      cpu.registers.set_bc(cpu.read_u16_at_pc());
      cpu.pc = cpu.pc.wrapping_add(2);
    }
    0x11 => {
      cpu.registers.set_de(cpu.read_u16_at_pc());
      cpu.pc = cpu.pc.wrapping_add(2);
    }
    0x21 => {
      cpu.registers.set_hl(cpu.read_u16_at_pc());
      cpu.pc = cpu.pc.wrapping_add(2);
    }
    0x31 => {
      cpu.sp = cpu.read_u16_at_pc();
      cpu.pc = cpu.pc.wrapping_add(2)
    }
    /* End direct loads */
    0x02 => {
      cpu.write(cpu.registers.get_bc() as usize, cpu.registers.a);
    }
    0x12 => {
      cpu.write(cpu.registers.get_de() as usize, cpu.registers.a);
    }
    0x22 => {
      let hl_value = cpu.registers.get_hl();
      cpu.write(hl_value as usize, cpu.registers.a);
      cpu.registers.set_hl(hl_value + 1);
    }
    0x32 => {
      let hl_value = cpu.registers.get_hl();
      cpu.write(hl_value as usize, cpu.registers.a);
      cpu.registers.set_hl(hl_value - 1);
    }
    0x06 => {
      cpu.registers.b = cpu.read(cpu.pc as usize);
      cpu.pc = cpu.pc.wrapping_add(1);
    }
    0x16 => {
      cpu.registers.d = cpu.read(cpu.pc as usize);
      cpu.pc = cpu.pc.wrapping_add(1);
    }
    0x26 => {
      cpu.registers.h = cpu.read(cpu.pc as usize);
      cpu.pc = cpu.pc.wrapping_add(1);
    }
    0x36 => {
      let value = cpu.read(cpu.pc as usize);
      cpu.write(cpu.registers.get_hl() as usize, value);
      cpu.pc = cpu.pc.wrapping_add(1);
    }
    0x08 => {
      let addr = cpu.read_u16_at_pc();
      cpu.write(addr as usize, (cpu.sp & 0xFF) as u8);
      cpu.write((addr + 1) as usize, (cpu.sp >> 8) as u8);
      cpu.pc = cpu.pc.wrapping_add(2);
    }
    0x09 => { cpu.add_hl_16bit(RegisterNames16::BC); }
    0x19 => { cpu.add_hl_16bit(RegisterNames16::DE); }
    0x29 => { cpu.add_hl_16bit(RegisterNames16::HL); }
    0x39 => { cpu.add_hl_u16(cpu.sp); }
    0xE8 => {
      let operand: i8 = cpu.read_i8_at_pc();
      let result = cpu.sp.wrapping_add(operand as u16);
      let mut flags_register = FlagsRegister::from(cpu.registers.f);
      flags_register.zero = false;
      flags_register.subtract = false;
      flags_register.half_carry = half_carry_add_8bit(cpu.sp as u8, operand as u8, 0);
      flags_register.carry = carry_add_8bit(cpu.sp as u8, operand as u8, 0);
      cpu.registers.f = u8::from(flags_register);
      cpu.sp = result;
      cpu.pc = cpu.pc.wrapping_add(1);
    }
    
    0x0A => { cpu.registers.a = cpu.read(cpu.registers.get_bc() as usize) }
    0x1A => { cpu.registers.a = cpu.read(cpu.registers.get_de() as usize) }
    0x2A => { 
      let hl_value = cpu.registers.get_hl();
      cpu.registers.a = cpu.read(hl_value as usize);
      cpu.registers.set_hl(hl_value + 1);
    }
    0x3A => { 
      let hl_value = cpu.registers.get_hl();
      cpu.registers.a = cpu.read(hl_value as usize);
      cpu.registers.set_hl(hl_value - 1);
    }
    0x0E => { 
      cpu.registers.c = cpu.read(cpu.pc as usize);
      cpu.pc = cpu.pc.wrapping_add(1);
    }
    0x1E => { 
      cpu.registers.e = cpu.read(cpu.pc as usize);
      cpu.pc = cpu.pc.wrapping_add(1);
    }
    0x2E => { 
      cpu.registers.l = cpu.read(cpu.pc as usize);
      cpu.pc = cpu.pc.wrapping_add(1);
    }
    0x3E => {
      cpu.registers.a = cpu.read(cpu.pc as usize);
      cpu.pc = cpu.pc.wrapping_add(1);
    }
    /* LD B, r */
    0x40 => { /* LD reg into itself is no op */ }
    0x41 => { cpu.registers.b = cpu.registers.c; }
    0x42 => { cpu.registers.b = cpu.registers.d; }
    0x43 => { cpu.registers.b = cpu.registers.e; }
    0x44 => { cpu.registers.b = cpu.registers.h; }
    0x45 => { cpu.registers.b = cpu.registers.l; }
    0x46 => { cpu.registers.b = cpu.read(cpu.registers.get_hl() as usize); }
    0x47 => { cpu.registers.b = cpu.registers.a; }
    /* LD C, r */
    0x48 => { cpu.registers.c = cpu.registers.b; }
    0x49 => { /* LD reg into itself is no op */ }
    0x4A => { cpu.registers.c = cpu.registers.d; }
    0x4B => { cpu.registers.c = cpu.registers.e; }
    0x4C => { cpu.registers.c = cpu.registers.h; }
    0x4D => { cpu.registers.c = cpu.registers.l; }
    0x4E => { cpu.registers.c = cpu.read(cpu.registers.get_hl() as usize); }
    0x4F => { cpu.registers.c = cpu.registers.a; }
    /* LD D, r */
    0x50 => { cpu.registers.d = cpu.registers.b; }
    0x51 => { cpu.registers.d = cpu.registers.c; }
    0x52 => { /* LD reg into itself is no op */ }
    0x53 => { cpu.registers.d = cpu.registers.e; }
    0x54 => { cpu.registers.d = cpu.registers.h; }
    0x55 => { cpu.registers.d = cpu.registers.l; }
    0x56 => { cpu.registers.d = cpu.read(cpu.registers.get_hl() as usize); }
    0x57 => { cpu.registers.d = cpu.registers.a; }
    /* LD E, r */
    0x58 => { cpu.registers.e = cpu.registers.b; }
    0x59 => { cpu.registers.e = cpu.registers.c; }
    0x5A => { cpu.registers.e = cpu.registers.d; }
    0x5B => { /* LD reg into itself is no op */ }
    0x5C => { cpu.registers.e = cpu.registers.h; }
    0x5D => { cpu.registers.e = cpu.registers.l; }
    0x5E => { cpu.registers.e = cpu.read(cpu.registers.get_hl() as usize); }
    0x5F => { cpu.registers.e = cpu.registers.a; }
    /* LD H, r */
    0x60 => { cpu.registers.h = cpu.registers.b; }
    0x61 => { cpu.registers.h = cpu.registers.c; }
    0x62 => { cpu.registers.h = cpu.registers.d; }
    0x63 => { cpu.registers.h = cpu.registers.e; }
    0x64 => { /* LD reg into itself is no op */ }
    0x65 => { cpu.registers.h = cpu.registers.l; }
    0x66 => { cpu.registers.h = cpu.read(cpu.registers.get_hl() as usize); }
    0x67 => { cpu.registers.h = cpu.registers.a; }
    /* LD L, r */
    0x68 => { cpu.registers.l = cpu.registers.b; }
    0x69 => { cpu.registers.l = cpu.registers.c; }
    0x6A => { cpu.registers.l = cpu.registers.d; }
    0x6B => { cpu.registers.l = cpu.registers.e; }
    0x6C => { cpu.registers.l = cpu.registers.h; }
    0x6D => { /* LD reg into itself is no op */ }
    0x6E => { cpu.registers.l = cpu.read(cpu.registers.get_hl() as usize); }
    0x6F => { cpu.registers.l = cpu.registers.a; }
    /* LD (HL), r */
    0x70 => { cpu.write(cpu.registers.get_hl() as usize, cpu.registers.b); }
    0x71 => { cpu.write(cpu.registers.get_hl() as usize, cpu.registers.c); }
    0x72 => { cpu.write(cpu.registers.get_hl() as usize, cpu.registers.d); }
    0x73 => { cpu.write(cpu.registers.get_hl() as usize, cpu.registers.e); }
    0x74 => { cpu.write(cpu.registers.get_hl() as usize, cpu.registers.h); }
    0x75 => { cpu.write(cpu.registers.get_hl() as usize, cpu.registers.l); }
    0x76 => { /* HALT - TODO: implement properly */ 
      cpu.halted = true;

    }
    0x77 => { cpu.write(cpu.registers.get_hl() as usize, cpu.registers.a); }
    /* LD A, r */
    0x78 => { cpu.registers.a = cpu.registers.b; }
    0x79 => { cpu.registers.a = cpu.registers.c; }
    0x7A => { cpu.registers.a = cpu.registers.d; }
    0x7B => { cpu.registers.a = cpu.registers.e; }
    0x7C => { cpu.registers.a = cpu.registers.h; }
    0x7D => { cpu.registers.a = cpu.registers.l; }
    0x7E => { cpu.registers.a = cpu.read(cpu.registers.get_hl() as usize); }
    0x7F => { /* LD reg into itself is no op */ }
    0xE0 => {
      let addr_lsb = cpu.read(cpu.pc as usize);
      let addr = 0xFF00 | (addr_lsb as u16);
      cpu.write(addr as usize, cpu.registers.a);
      cpu.pc = cpu.pc.wrapping_add(1);
    }
    0xF0 => {
      let addr_lsb = cpu.read(cpu.pc as usize);
      let addr = 0xFF00 | (addr_lsb as u16);
      cpu.registers.a = cpu.read(addr as usize);
      cpu.pc = cpu.pc.wrapping_add(1);
    }
    0xE2 => {
      let addr = 0xFF00 | (cpu.registers.c as u16);
      cpu.write(addr as usize, cpu.registers.a);
    }
    0xF2 => {
      let addr = 0xFF00 | (cpu.registers.c as u16);
      cpu.registers.a = cpu.read(addr as usize);
    }
    0xEA => {
      let addr = cpu.read_u16_at_pc();
      cpu.write(addr as usize, cpu.registers.a);
      cpu.pc = cpu.pc.wrapping_add(2);
    }
    0xFA => {
      let addr = cpu.read_u16_at_pc();
      cpu.registers.a = cpu.read(addr as usize);
      cpu.pc = cpu.pc.wrapping_add(2);
    }
    0xF8 => {
      let operand: i8 = cpu.read_i8_at_pc();
      let result = cpu.sp.wrapping_add(operand as u16);
      let mut flags_register = FlagsRegister::from(cpu.registers.f);
      flags_register.zero = false;
      flags_register.subtract = false;
      flags_register.half_carry = half_carry_add_8bit(cpu.sp as u8, operand as u8, 0);
      flags_register.carry = carry_add_8bit(cpu.sp as u8, operand as u8, 0);
      cpu.registers.f = u8::from(flags_register);
      cpu.registers.set_hl(result);
      cpu.pc = cpu.pc.wrapping_add(1);
    }
    0xF9 => {
      cpu.sp = cpu.registers.get_hl();
    }
    /* END LOAD OPCODES */
    /* START 16-BIT INC */
    0x03 => { cpu.inc_r16(RegisterNames16::BC); }
    0x13 => { cpu.inc_r16(RegisterNames16::DE); }
    0x23 => { cpu.inc_r16(RegisterNames16::HL); }
    0x33 => { cpu.sp = cpu.sp.wrapping_add(1) }
    /* END 16-BIT INC */
    /* START 16-BIT DEC */
    0x0B => { cpu.dec_r16(RegisterNames16::BC); }
    0x1B => { cpu.dec_r16(RegisterNames16::DE); }
    0x2B => { cpu.dec_r16(RegisterNames16::HL); }
    0x3B => { cpu.sp = cpu.sp.wrapping_sub(1) }
    /* END 16-BIT DEC */
    /* START 8-BIT INC */
    0x04 => { cpu.registers.b = cpu.inc_8bit(cpu.registers.b); }
    0x0C => { cpu.registers.c = cpu.inc_8bit(cpu.registers.c); }
    0x14 => { cpu.registers.d = cpu.inc_8bit(cpu.registers.d); }
    0x1C => { cpu.registers.e = cpu.inc_8bit(cpu.registers.e); }
    0x24 => { cpu.registers.h = cpu.inc_8bit(cpu.registers.h); }
    0x2C => { cpu.registers.l = cpu.inc_8bit(cpu.registers.l); }
    0x34 => {
      let addr = cpu.registers.get_hl() as usize;
      let inc_value = cpu.inc_8bit(cpu.read(addr));
      cpu.write(addr, inc_value);
    }
    0x3C => { cpu.registers.a = cpu.inc_8bit(cpu.registers.a); }
    /* END 8-BIT INC */
    /* START 8-BIT DEC */
    0x05 => { cpu.registers.b = cpu.dec_8bit(cpu.registers.b); }
    0x0D => { cpu.registers.c = cpu.dec_8bit(cpu.registers.c); }
    0x15 => { cpu.registers.d = cpu.dec_8bit(cpu.registers.d); }
    0x1D => { cpu.registers.e = cpu.dec_8bit(cpu.registers.e); }
    0x25 => { cpu.registers.h = cpu.dec_8bit(cpu.registers.h); }
    0x2D => { cpu.registers.l = cpu.dec_8bit(cpu.registers.l); }
    0x35 => {
      let addr = cpu.registers.get_hl() as usize;
      let dec_value = cpu.dec_8bit(cpu.read(addr));
      cpu.write(addr, dec_value);
    }
    0x3D => { cpu.registers.a = cpu.dec_8bit(cpu.registers.a); }
    /* END 8-BIT DEC */
    /* START 8BIT ADD */
    0x80 => { cpu.add_8bit(cpu.registers.b); }
    0x81 => { cpu.add_8bit(cpu.registers.c); }
    0x82 => { cpu.add_8bit(cpu.registers.d); }
    0x83 => { cpu.add_8bit(cpu.registers.e); }
    0x84 => { cpu.add_8bit(cpu.registers.h); }
    0x85 => { cpu.add_8bit(cpu.registers.l); }
    0x86 => { cpu.add_8bit(cpu.read(cpu.registers.get_hl() as usize)); }
    0x87 => { cpu.add_8bit(cpu.registers.a); }
    /* END 8BIT ADD */
    /* START 8BIT ADDC */
    0x88 => { cpu.addc_8bit(cpu.registers.b); }
    0x89 => { cpu.addc_8bit(cpu.registers.c); }
    0x8A => { cpu.addc_8bit(cpu.registers.d); }
    0x8B => { cpu.addc_8bit(cpu.registers.e); }
    0x8C => { cpu.addc_8bit(cpu.registers.h); }
    0x8D => { cpu.addc_8bit(cpu.registers.l); }
    0x8E => { cpu.addc_8bit(cpu.read(cpu.registers.get_hl() as usize)); }
    0x8F => { cpu.addc_8bit(cpu.registers.a); }
    /* END 8BIT ADDC */
    /* START 8BIT SUB */
    0x90 => { cpu.registers.a = cpu.sub(cpu.registers.b, false) }
    0x91 => { cpu.registers.a = cpu.sub(cpu.registers.c, false) }
    0x92 => { cpu.registers.a = cpu.sub(cpu.registers.d, false) }
    0x93 => { cpu.registers.a = cpu.sub(cpu.registers.e, false) }
    0x94 => { cpu.registers.a = cpu.sub(cpu.registers.h, false) }
    0x95 => { cpu.registers.a = cpu.sub(cpu.registers.l, false) }
    0x96 => { cpu.registers.a = cpu.sub(cpu.read(cpu.registers.get_hl() as usize), false) }
    0x97 => { cpu.registers.a = cpu.sub(cpu.registers.a, false) }
    /* END 8BIT SUB */
    /* START 8BIT SBC */
    0x98 => { cpu.registers.a = cpu.sub(cpu.registers.b, true) }
    0x99 => { cpu.registers.a = cpu.sub(cpu.registers.c, true) }
    0x9A => { cpu.registers.a = cpu.sub(cpu.registers.d, true) }
    0x9B => { cpu.registers.a = cpu.sub(cpu.registers.e, true) }
    0x9C => { cpu.registers.a = cpu.sub(cpu.registers.h, true) }
    0x9D => { cpu.registers.a = cpu.sub(cpu.registers.l, true) }
    0x9E => { cpu.registers.a = cpu.sub(cpu.read(cpu.registers.get_hl() as usize), true) }
    0x9F => { cpu.registers.a = cpu.sub(cpu.registers.a, true) }
    /* END 8BIT SBC */
    /* START 8BIT AND */
    0xA0 => { cpu.registers.a = cpu.and(cpu.registers.b) }
    0xA1 => { cpu.registers.a = cpu.and(cpu.registers.c) }
    0xA2 => { cpu.registers.a = cpu.and(cpu.registers.d) }
    0xA3 => { cpu.registers.a = cpu.and(cpu.registers.e) }
    0xA4 => { cpu.registers.a = cpu.and(cpu.registers.h) }
    0xA5 => { cpu.registers.a = cpu.and(cpu.registers.l) }
    0xA6 => { cpu.registers.a = cpu.and(cpu.read(cpu.registers.get_hl() as usize)) }
    0xA7 => { cpu.registers.a = cpu.and(cpu.registers.a) }
    /* END 8BIT AND */
    /* START 8BIT XOR */
    0xA8 => { cpu.registers.a = cpu.xor(cpu.registers.b) }
    0xA9 => { cpu.registers.a = cpu.xor(cpu.registers.c) }
    0xAA => { cpu.registers.a = cpu.xor(cpu.registers.d) }
    0xAB => { cpu.registers.a = cpu.xor(cpu.registers.e) }
    0xAC => { cpu.registers.a = cpu.xor(cpu.registers.h) }
    0xAD => { cpu.registers.a = cpu.xor(cpu.registers.l) }
    0xAE => { cpu.registers.a = cpu.xor(cpu.read(cpu.registers.get_hl() as usize)) }
    0xAF => { cpu.registers.a = cpu.xor(cpu.registers.a) }
    /* END 8BIT XOR */
    /* START 8BIT OR */
    0xB0 => { cpu.registers.a = cpu.or(cpu.registers.b) }
    0xB1 => { cpu.registers.a = cpu.or(cpu.registers.c) }
    0xB2 => { cpu.registers.a = cpu.or(cpu.registers.d) }
    0xB3 => { cpu.registers.a = cpu.or(cpu.registers.e) }
    0xB4 => { cpu.registers.a = cpu.or(cpu.registers.h) }
    0xB5 => { cpu.registers.a = cpu.or(cpu.registers.l) }
    0xB6 => { cpu.registers.a = cpu.or(cpu.read(cpu.registers.get_hl() as usize)) }
    0xB7 => { cpu.registers.a = cpu.or(cpu.registers.a) }
    /* END 8BIT OR */
    /* START 8BIT CP */
    0xB8 => { cpu.cp(cpu.registers.b) }
    0xB9 => { cpu.cp(cpu.registers.c) }
    0xBA => { cpu.cp(cpu.registers.d) }
    0xBB => { cpu.cp(cpu.registers.e) }
    0xBC => { cpu.cp(cpu.registers.h) }
    0xBD => { cpu.cp(cpu.registers.l) }
    0xBE => { cpu.cp(cpu.read(cpu.registers.get_hl() as usize)) }
    0xBF => { cpu.cp(cpu.registers.a) }
    /* END 8BIT CP */
    /* START 8BIT IMMEDIATE ALU */
    0xC6 => {
      let value = cpu.read(cpu.pc as usize);
      cpu.pc = cpu.pc.wrapping_add(1);
      cpu.add_8bit(value);
    }
    0xCE => {
      let value = cpu.read(cpu.pc as usize);
      cpu.pc = cpu.pc.wrapping_add(1);
      cpu.addc_8bit(value);
    }
    0xD6 => {
      let value = cpu.read(cpu.pc as usize);
      cpu.pc = cpu.pc.wrapping_add(1);
      cpu.registers.a = cpu.sub(value, false);
    }
    0xDE => {
      let value = cpu.read(cpu.pc as usize);
      cpu.pc = cpu.pc.wrapping_add(1);
      cpu.registers.a = cpu.sub(value, true);
    }
    0xE6 => {
      let value = cpu.read(cpu.pc as usize);
      cpu.pc = cpu.pc.wrapping_add(1);
      cpu.registers.a = cpu.and(value);
    }
    0xEE => {
      let value = cpu.read(cpu.pc as usize);
      cpu.pc = cpu.pc.wrapping_add(1);
      cpu.registers.a = cpu.xor(value);
    }
    0xF6 => {
      let value = cpu.read(cpu.pc as usize);
      cpu.pc = cpu.pc.wrapping_add(1);
      cpu.registers.a = cpu.or(value);
    }
    0xFE => {
      let value = cpu.read(cpu.pc as usize);
      cpu.pc = cpu.pc.wrapping_add(1);
      cpu.cp(value);
    }
    /* END 8BIT IMMEDIATE ALU */
    /* START RST OPCODES */
    0xC7 => {
      cpu.push(cpu.pc);
      cpu.pc = two_bytes_to_u16(0x00, 0x00);
    }
    0xD7 => {
      cpu.push(cpu.pc);
      cpu.pc = two_bytes_to_u16(0x10, 0x00);
    }
    0xE7 => {
      cpu.push(cpu.pc);
      cpu.pc = two_bytes_to_u16(0x20, 0x00);
    }
    0xF7 => {
      cpu.push(cpu.pc);
      cpu.pc = two_bytes_to_u16(0x30, 0x00);
    }
    0xCF => {
      cpu.push(cpu.pc);
      cpu.pc = two_bytes_to_u16(0x08, 0x00);
    }
    0xDF => {
      cpu.push(cpu.pc);
      cpu.pc = two_bytes_to_u16(0x18, 0x00);
    }
    0xEF => {
      cpu.push(cpu.pc);
      cpu.pc = two_bytes_to_u16(0x28, 0x00);
    }
    0xFF => {
      cpu.push(cpu.pc);
      cpu.pc = two_bytes_to_u16(0x38, 0x00);
    }
    /* END RST OPCODES */
    /* START JUMP OPCODES */
    0xC2 => {
      if !FlagsRegister::from(cpu.registers.f).zero {
        cpu.pc = cpu.read_u16_at_pc();
        cycles = *instruction.cycles.first().unwrap();
      } else {
        cpu.pc = cpu.pc.wrapping_add(2);
        cycles = *instruction.cycles.get(1).unwrap();
      }
    }
    0xC3 => {
      cpu.pc = cpu.read_u16_at_pc();
    }
    0xCA => {
      if FlagsRegister::from(cpu.registers.f).zero {
        cpu.pc = cpu.read_u16_at_pc();
        cycles = *instruction.cycles.first().unwrap();
      } else {
        cpu.pc = cpu.pc.wrapping_add(2);
        cycles = *instruction.cycles.get(1).unwrap();
      }
    }
    0xD2 => {
      if !FlagsRegister::from(cpu.registers.f).carry {
        cpu.pc = cpu.read_u16_at_pc();
        cycles = *instruction.cycles.first().unwrap();
      } else {
        cpu.pc = cpu.pc.wrapping_add(2);
        cycles = *instruction.cycles.get(1).unwrap();
      }
    }
    0xDA => {
      if FlagsRegister::from(cpu.registers.f).carry {
        cpu.pc = cpu.read_u16_at_pc();
        cycles = *instruction.cycles.first().unwrap();
      } else {
        cpu.pc = cpu.pc.wrapping_add(2);
        cycles = *instruction.cycles.get(1).unwrap();
      }
    }
    0xE9 => {
      cpu.pc = cpu.registers.get_hl();
    }
    /* END JUMP OPCODES */
    /* START RETURN OPCODES */
    0xC9 => {
      cpu.pc = cpu.pop();
    }
    0xC0 => {
      if !FlagsRegister::from(cpu.registers.f).zero {
        cpu.pc = cpu.pop();
        cycles = *instruction.cycles.first().unwrap();
      } else {
        cycles = *instruction.cycles.get(1).unwrap();
      }
    }
    0xC8 => {
      if FlagsRegister::from(cpu.registers.f).zero {
        cpu.pc = cpu.pop();
        cycles = *instruction.cycles.first().unwrap();
      } else {
        cycles = *instruction.cycles.get(1).unwrap();
      }
    }
    0xD0 => {
      if !FlagsRegister::from(cpu.registers.f).carry {
        cpu.pc = cpu.pop();
        cycles = *instruction.cycles.first().unwrap();
      } else {
        cycles = *instruction.cycles.get(1).unwrap();
      }
    }
    0xD8 => {
      if FlagsRegister::from(cpu.registers.f).carry {
        cpu.pc = cpu.pop();
        cycles = *instruction.cycles.first().unwrap();
      } else {
        cycles = *instruction.cycles.get(1).unwrap();
      }
    }
    0xD9 => {
      // RETI
      cpu.ime = 1;
      cpu.pc = cpu.pop();
    }
    /* END RETURN OPCODES */
    /* START CALL OPCODES */
    0xCD => {
      cpu.call();
    }
    0xCC => {
      if FlagsRegister::from(cpu.registers.f).zero {
        cpu.call();
        cycles = *instruction.cycles.first().unwrap();
      } else {
        cpu.pc += 2;
        cycles = *instruction.cycles.get(1).unwrap();
      }
    }
    0xC4 => {
      if !FlagsRegister::from(cpu.registers.f).zero {
        cpu.call();
        cycles = *instruction.cycles.first().unwrap();
      } else {
        cpu.pc += 2;
        cycles = *instruction.cycles.get(1).unwrap();
      }
    }
    0xD4 => {
      if !FlagsRegister::from(cpu.registers.f).carry {
        cpu.call();
        cycles = *instruction.cycles.first().unwrap();
      } else {
        cpu.pc += 2;
        cycles = *instruction.cycles.get(1).unwrap();
      }
    }
    0xDC => {
      if FlagsRegister::from(cpu.registers.f).carry {
        cpu.call();
        cycles = *instruction.cycles.first().unwrap();
      } else {
        cpu.pc += 2;
        cycles = *instruction.cycles.get(1).unwrap();
      }
    }
    /* END CALL OPCODES */
    /* START POP r16 */
    0xC1 => {
      let sp_memory = cpu.pop();
      cpu.registers.set_bc(sp_memory);
    }
    0xD1 => {
      let sp_memory = cpu.pop();
      cpu.registers.set_de(sp_memory);
    }
    0xE1 => {
      let sp_memory = cpu.pop();
      cpu.registers.set_hl(sp_memory);
    }
    0xF1 => {
      let sp_memory = cpu.pop();
      cpu.registers.set_af(sp_memory);
    }
    /* END POP r16 */
    /* START PUSH r16 */
    0xC5 => { cpu.push(cpu.registers.get_bc()) }
    0xD5 => { cpu.push(cpu.registers.get_de()) }
    0xE5 => { cpu.push(cpu.registers.get_hl()) }
    0xF5 => { cpu.push(cpu.registers.get_af()) }
    /* END PUSH r16 */
    0xCB => {
      let opcode = cpu.read(cpu.pc as usize);
      cpu.pc = cpu.pc.wrapping_add(1);
      let instruction = &instruction_set.cbprefixed[&format!("{:#04X}", opcode)];
      
      cycles = execute_cb_prefixed_opcode(cpu, instruction, opcode, show_debug_messages);
    }
    _ => {
      panic!("{:#04X} {:#04X}: {} {:#?} Undefined opcode - this shouldn't have happened!\n", cpu.pc, opcode, instruction.mnemonic, instruction.operands);
    }
  }
  // if cycles haven't been set already set them now - this will handle the simple case of ops that take a set number of T-states
  if cycles == 0 {
    cycles = *instruction.cycles.first().unwrap();
  }
  cycles
}

