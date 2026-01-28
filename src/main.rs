mod registers;
mod cpu;
mod instructions;
pub mod tests;

use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::ops::Not;
use std::path::Path;
use cpu::{CPU, two_bytes_to_u16, half_carry_add_8bit, carry_add_8bit};
use crate::registers::{FlagsRegister, Registers16};

use crate::instructions::*;

pub fn execute_opcode(instruction_set: &InstructionSet, cpu: &mut CPU) {
  let opcode = cpu.ram[cpu.pc as usize];
    cpu.pc = cpu.pc.wrapping_add(1); // increment PC past the opcode
    match opcode  {
      0x00 => { /* NO OP */ }
      0x10 => { /* TODO IMPLEMENT STOP */}
      0xF3 => { /* TODO IMPLEMENT DI (Reset the interrupt master enable (IME) flag and prohibit maskable interrupts.) */}
      0x07 => { cpu.registers.a = cpu.rotate_left_with_carry(cpu.registers.a) }
      0x17 => { cpu.registers.a = cpu.rotate_left_through_carry(cpu.registers.a) }
      0x0F => { cpu.registers.a = cpu.rotate_right_with_carry(cpu.registers.a) }
      0x1F => { cpu.registers.a = cpu.rotate_right_through_carry(cpu.registers.a) }
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
      0x27 => { 
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
        }
        cpu.pc = cpu.pc.wrapping_add(1);
      }
      0x28 => {
        if FlagsRegister::from(cpu.registers.f).zero {
          cpu.pc = cpu.pc.wrapping_add(cpu.read_i8_at_pc() as u16);
        }
        cpu.pc = cpu.pc.wrapping_add(1);
      }
      0x30 => {
        if !FlagsRegister::from(cpu.registers.f).carry {
          cpu.pc = cpu.pc.wrapping_add(cpu.read_i8_at_pc() as u16);
        }
        cpu.pc = cpu.pc.wrapping_add(1);
      }
      0x38 => {
        if FlagsRegister::from(cpu.registers.f).carry {
          cpu.pc = cpu.pc.wrapping_add(cpu.read_i8_at_pc() as u16);
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
        cpu.ram[cpu.registers.get_bc() as usize] = cpu.registers.a; 
      }
      0x12 => {
        cpu.ram[cpu.registers.get_de() as usize] = cpu.registers.a;
      }
      0x22 => {
        let hl_value = cpu.registers.get_hl();
        cpu.ram[hl_value as usize] = cpu.registers.a;
        cpu.registers.set_hl(hl_value + 1);
      }
      0x32 => {
        let hl_value = cpu.registers.get_hl();
        cpu.ram[hl_value as usize] = cpu.registers.a;
        cpu.registers.set_hl(hl_value - 1);
      }
      0x06 => {
        cpu.registers.b = cpu.ram[cpu.pc as usize];
        cpu.pc = cpu.pc.wrapping_add(1);
      }
      0x16 => {
        cpu.registers.d = cpu.ram[cpu.pc as usize];
        cpu.pc = cpu.pc.wrapping_add(1);
      }
      0x26 => {
        cpu.registers.h = cpu.ram[cpu.pc as usize];
        cpu.pc = cpu.pc.wrapping_add(1);
      }
      0x36 => {
        cpu.ram[cpu.registers.get_hl() as usize] = cpu.ram[cpu.pc as usize];
        cpu.pc = cpu.pc.wrapping_add(1);
      }
      0x08 => {
        let addr = cpu.read_u16_at_pc();
        cpu.ram[addr as usize] = (cpu.sp & 0xFF) as u8;
        cpu.ram[(addr + 1) as usize] = (cpu.sp >> 8) as u8;
        cpu.pc = cpu.pc.wrapping_add(2);
      }
      0x09 => { cpu.add_hl_16bit(Registers16::BC); }
      0x19 => { cpu.add_hl_16bit(Registers16::DE); }
      0x29 => { cpu.add_hl_16bit(Registers16::HL); }
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
      
      0x0A => { cpu.registers.a = cpu.ram[cpu.registers.get_bc() as usize] }
      0x1A => { cpu.registers.a = cpu.ram[cpu.registers.get_de() as usize] }
      0x2A => { 
        let hl_value = cpu.registers.get_hl();
        cpu.registers.a = cpu.ram[hl_value as usize];
        cpu.registers.set_hl(hl_value + 1);
      }
      0x3A => { 
        let hl_value = cpu.registers.get_hl();
        cpu.registers.a = cpu.ram[hl_value as usize];
        cpu.registers.set_hl(hl_value - 1);
      }
      0x0E => { 
        cpu.registers.c = cpu.ram[cpu.pc as usize];
        cpu.pc = cpu.pc.wrapping_add(1);
      }
      0x1E => { 
        cpu.registers.e = cpu.ram[cpu.pc as usize];
        cpu.pc = cpu.pc.wrapping_add(1);
      }
      0x2E => { 
        cpu.registers.l = cpu.ram[cpu.pc as usize];
        cpu.pc = cpu.pc.wrapping_add(1);
      }
      0x3E => {
        cpu.registers.a = cpu.ram[cpu.pc as usize];
        cpu.pc = cpu.pc.wrapping_add(1);
      }
      /* LD B, r */
      0x40 => { /* LD reg into itself is no op */ }
      0x41 => { cpu.registers.b = cpu.registers.c; }
      0x42 => { cpu.registers.b = cpu.registers.d; }
      0x43 => { cpu.registers.b = cpu.registers.e; }
      0x44 => { cpu.registers.b = cpu.registers.h; }
      0x45 => { cpu.registers.b = cpu.registers.l; }
      0x46 => { cpu.registers.b = cpu.ram[cpu.registers.get_hl() as usize]; }
      0x47 => { cpu.registers.b = cpu.registers.a; }
      /* LD C, r */
      0x48 => { cpu.registers.c = cpu.registers.b; }
      0x49 => { /* LD reg into itself is no op */ }
      0x4A => { cpu.registers.c = cpu.registers.d; }
      0x4B => { cpu.registers.c = cpu.registers.e; }
      0x4C => { cpu.registers.c = cpu.registers.h; }
      0x4D => { cpu.registers.c = cpu.registers.l; }
      0x4E => { cpu.registers.c = cpu.ram[cpu.registers.get_hl() as usize]; }
      0x4F => { cpu.registers.c = cpu.registers.a; }
      /* LD D, r */
      0x50 => { cpu.registers.d = cpu.registers.b; }
      0x51 => { cpu.registers.d = cpu.registers.c; }
      0x52 => { /* LD reg into itself is no op */ }
      0x53 => { cpu.registers.d = cpu.registers.e; }
      0x54 => { cpu.registers.d = cpu.registers.h; }
      0x55 => { cpu.registers.d = cpu.registers.l; }
      0x56 => { cpu.registers.d = cpu.ram[cpu.registers.get_hl() as usize]; }
      0x57 => { cpu.registers.d = cpu.registers.a; }
      /* LD E, r */
      0x58 => { cpu.registers.e = cpu.registers.b; }
      0x59 => { cpu.registers.e = cpu.registers.c; }
      0x5A => { cpu.registers.e = cpu.registers.d; }
      0x5B => { /* LD reg into itself is no op */ }
      0x5C => { cpu.registers.e = cpu.registers.h; }
      0x5D => { cpu.registers.e = cpu.registers.l; }
      0x5E => { cpu.registers.e = cpu.ram[cpu.registers.get_hl() as usize]; }
      0x5F => { cpu.registers.e = cpu.registers.a; }
      /* LD H, r */
      0x60 => { cpu.registers.h = cpu.registers.b; }
      0x61 => { cpu.registers.h = cpu.registers.c; }
      0x62 => { cpu.registers.h = cpu.registers.d; }
      0x63 => { cpu.registers.h = cpu.registers.e; }
      0x64 => { /* LD reg into itself is no op */ }
      0x65 => { cpu.registers.h = cpu.registers.l; }
      0x66 => { cpu.registers.h = cpu.ram[cpu.registers.get_hl() as usize]; }
      0x67 => { cpu.registers.h = cpu.registers.a; }
      /* LD L, r */
      0x68 => { cpu.registers.l = cpu.registers.b; }
      0x69 => { cpu.registers.l = cpu.registers.c; }
      0x6A => { cpu.registers.l = cpu.registers.d; }
      0x6B => { cpu.registers.l = cpu.registers.e; }
      0x6C => { cpu.registers.l = cpu.registers.h; }
      0x6D => { /* LD reg into itself is no op */ }
      0x6E => { cpu.registers.l = cpu.ram[cpu.registers.get_hl() as usize]; }
      0x6F => { cpu.registers.l = cpu.registers.a; }
      /* LD (HL), r */
      0x70 => { cpu.ram[cpu.registers.get_hl() as usize] = cpu.registers.b; }
      0x71 => { cpu.ram[cpu.registers.get_hl() as usize] = cpu.registers.c; }
      0x72 => { cpu.ram[cpu.registers.get_hl() as usize] = cpu.registers.d; }
      0x73 => { cpu.ram[cpu.registers.get_hl() as usize] = cpu.registers.e; }
      0x74 => { cpu.ram[cpu.registers.get_hl() as usize] = cpu.registers.h; }
      0x75 => { cpu.ram[cpu.registers.get_hl() as usize] = cpu.registers.l; }
      0x76 => { /* HALT - TODO: implement properly */ }
      0x77 => { cpu.ram[cpu.registers.get_hl() as usize] = cpu.registers.a; }
      /* LD A, r */
      0x78 => { cpu.registers.a = cpu.registers.b; }
      0x79 => { cpu.registers.a = cpu.registers.c; }
      0x7A => { cpu.registers.a = cpu.registers.d; }
      0x7B => { cpu.registers.a = cpu.registers.e; }
      0x7C => { cpu.registers.a = cpu.registers.h; }
      0x7D => { cpu.registers.a = cpu.registers.l; }
      0x7E => { cpu.registers.a = cpu.ram[cpu.registers.get_hl() as usize]; }
      0x7F => { /* LD reg into itself is no op */ }
      0xE0 => {
        let addr_lsb = cpu.ram[cpu.pc as usize];
        let addr = 0xFF00 | (addr_lsb as u16);
        cpu.ram[addr as usize] = cpu.registers.a;
        cpu.pc = cpu.pc.wrapping_add(1);
      }
      0xF0 => {
        let addr_lsb = cpu.ram[cpu.pc as usize];
        let addr = 0xFF00 | (addr_lsb as u16);
        cpu.registers.a = cpu.ram[addr as usize];
        cpu.pc = cpu.pc.wrapping_add(1);
      }
      0xE2 => {
        let addr = 0xFF00 | (cpu.registers.c as u16);
        cpu.ram[addr as usize] = cpu.registers.a;
      }
      0xF2 => {
        let addr = 0xFF00 | (cpu.registers.c as u16);
        cpu.registers.a = cpu.ram[addr as usize];
      }
      0xEA => {
        let addr = cpu.read_u16_at_pc();
        cpu.ram[addr as usize] = cpu.registers.a; 
        cpu.pc = cpu.pc.wrapping_add(2);
      }
      0xFA => {
        let addr = cpu.read_u16_at_pc();
        cpu.registers.a = cpu.ram[addr as usize];
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
      0x03 => { cpu.inc_r16(Registers16::BC); }
      0x13 => { cpu.inc_r16(Registers16::DE); }
      0x23 => { cpu.inc_r16(Registers16::HL); }
      0x33 => { cpu.sp = cpu.sp.wrapping_add(1) }
      /* END 16-BIT INC */
      /* START 16-BIT DEC */
      0x0B => { cpu.dec_r16(Registers16::BC); }
      0x1B => { cpu.dec_r16(Registers16::DE); }
      0x2B => { cpu.dec_r16(Registers16::HL); }
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
        cpu.ram[addr] = cpu.inc_8bit(cpu.ram[addr]);
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
        cpu.ram[addr] = cpu.dec_8bit(cpu.ram[addr]);
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
      0x86 => { cpu.add_8bit(cpu.ram[cpu.registers.get_hl() as usize]); }
      0x87 => { cpu.add_8bit(cpu.registers.a); }
      /* END 8BIT ADD */
      /* START 8BIT ADDC */
      0x88 => { cpu.addc_8bit(cpu.registers.b); }
      0x89 => { cpu.addc_8bit(cpu.registers.c); }
      0x8A => { cpu.addc_8bit(cpu.registers.d); }
      0x8B => { cpu.addc_8bit(cpu.registers.e); }
      0x8C => { cpu.addc_8bit(cpu.registers.h); }
      0x8D => { cpu.addc_8bit(cpu.registers.l); }
      0x8E => { cpu.addc_8bit(cpu.ram[cpu.registers.get_hl() as usize]); }
      0x8F => { cpu.addc_8bit(cpu.registers.a); }
      /* END 8BIT ADDC */
      /* START 8BIT SUB */
      0x90 => { cpu.registers.a = cpu.sub(cpu.registers.b, false) }
      0x91 => { cpu.registers.a = cpu.sub(cpu.registers.c, false) }
      0x92 => { cpu.registers.a = cpu.sub(cpu.registers.d, false) }
      0x93 => { cpu.registers.a = cpu.sub(cpu.registers.e, false) }
      0x94 => { cpu.registers.a = cpu.sub(cpu.registers.h, false) }
      0x95 => { cpu.registers.a = cpu.sub(cpu.registers.l, false) }
      0x96 => { cpu.registers.a = cpu.sub(cpu.ram[cpu.registers.get_hl() as usize], false) }
      0x97 => { cpu.registers.a = cpu.sub(cpu.registers.a, false) }
      /* END 8BIT SUB */
      /* START 8BIT SBC */
      0x98 => { cpu.registers.a = cpu.sub(cpu.registers.b, true) }
      0x99 => { cpu.registers.a = cpu.sub(cpu.registers.c, true) }
      0x9A => { cpu.registers.a = cpu.sub(cpu.registers.d, true) }
      0x9B => { cpu.registers.a = cpu.sub(cpu.registers.e, true) }
      0x9C => { cpu.registers.a = cpu.sub(cpu.registers.h, true) }
      0x9D => { cpu.registers.a = cpu.sub(cpu.registers.l, true) }
      0x9E => { cpu.registers.a = cpu.sub(cpu.ram[cpu.registers.get_hl() as usize], true) }
      0x9F => { cpu.registers.a = cpu.sub(cpu.registers.a, true) }
      /* END 8BIT SBC */
      /* START 8BIT AND */
      0xA0 => { cpu.registers.a = cpu.and(cpu.registers.b) }
      0xA1 => { cpu.registers.a = cpu.and(cpu.registers.c) }
      0xA2 => { cpu.registers.a = cpu.and(cpu.registers.d) }
      0xA3 => { cpu.registers.a = cpu.and(cpu.registers.e) }
      0xA4 => { cpu.registers.a = cpu.and(cpu.registers.h) }
      0xA5 => { cpu.registers.a = cpu.and(cpu.registers.l) }
      0xA6 => { cpu.registers.a = cpu.and(cpu.ram[cpu.registers.get_hl() as usize]) }
      0xA7 => { cpu.registers.a = cpu.and(cpu.registers.a) }
      /* END 8BIT AND */
      /* START 8BIT XOR */
      0xA8 => { cpu.registers.a = cpu.xor(cpu.registers.b) }
      0xA9 => { cpu.registers.a = cpu.xor(cpu.registers.c) }
      0xAA => { cpu.registers.a = cpu.xor(cpu.registers.d) }
      0xAB => { cpu.registers.a = cpu.xor(cpu.registers.e) }
      0xAC => { cpu.registers.a = cpu.xor(cpu.registers.h) }
      0xAD => { cpu.registers.a = cpu.xor(cpu.registers.l) }
      0xAE => { cpu.registers.a = cpu.xor(cpu.ram[cpu.registers.get_hl() as usize]) }
      0xAF => { cpu.registers.a = cpu.xor(cpu.registers.a) }
      /* END 8BIT XOR */
      /* START 8BIT OR */
      0xB0 => { cpu.registers.a = cpu.or(cpu.registers.b) }
      0xB1 => { cpu.registers.a = cpu.or(cpu.registers.c) }
      0xB2 => { cpu.registers.a = cpu.or(cpu.registers.d) }
      0xB3 => { cpu.registers.a = cpu.or(cpu.registers.e) }
      0xB4 => { cpu.registers.a = cpu.or(cpu.registers.h) }
      0xB5 => { cpu.registers.a = cpu.or(cpu.registers.l) }
      0xB6 => { cpu.registers.a = cpu.or(cpu.ram[cpu.registers.get_hl() as usize]) }
      0xB7 => { cpu.registers.a = cpu.or(cpu.registers.a) }
      /* END 8BIT OR */
      /* START 8BIT CP */
      0xB8 => { cpu.cp(cpu.registers.b) }
      0xB9 => { cpu.cp(cpu.registers.c) }
      0xBA => { cpu.cp(cpu.registers.d) }
      0xBB => { cpu.cp(cpu.registers.e) }
      0xBC => { cpu.cp(cpu.registers.h) }
      0xBD => { cpu.cp(cpu.registers.l) }
      0xBE => { cpu.cp(cpu.ram[cpu.registers.get_hl() as usize]) }
      0xBF => { cpu.cp(cpu.registers.a) }
      /* END 8BIT CP */
      /* START 8BIT IMMEDIATE ALU */
      0xC6 => {
        let value = cpu.ram[cpu.pc as usize];
        cpu.pc = cpu.pc.wrapping_add(1);
        cpu.add_8bit(value);
      }
      0xCE => {
        let value = cpu.ram[cpu.pc as usize];
        cpu.pc = cpu.pc.wrapping_add(1);
        cpu.addc_8bit(value);
      }
      0xD6 => {
        let value = cpu.ram[cpu.pc as usize];
        cpu.pc = cpu.pc.wrapping_add(1);
        cpu.registers.a = cpu.sub(value, false);
      }
      0xDE => {
        let value = cpu.ram[cpu.pc as usize];
        cpu.pc = cpu.pc.wrapping_add(1);
        cpu.registers.a = cpu.sub(value, true);
      }
      0xE6 => {
        let value = cpu.ram[cpu.pc as usize];
        cpu.pc = cpu.pc.wrapping_add(1);
        cpu.registers.a = cpu.and(value);
      }
      0xEE => {
        let value = cpu.ram[cpu.pc as usize];
        cpu.pc = cpu.pc.wrapping_add(1);
        cpu.registers.a = cpu.xor(value);
      }
      0xF6 => {
        let value = cpu.ram[cpu.pc as usize];
        cpu.pc = cpu.pc.wrapping_add(1);
        cpu.registers.a = cpu.or(value);
      }
      0xFE => {
        let value = cpu.ram[cpu.pc as usize];
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
        } else {
          cpu.pc = cpu.pc.wrapping_add(2);
        }
      }
      0xC3 => {
        cpu.pc = cpu.read_u16_at_pc();
      }
      0xCA => {
        if FlagsRegister::from(cpu.registers.f).zero {
          cpu.pc = cpu.read_u16_at_pc();
        } else {
          cpu.pc = cpu.pc.wrapping_add(2);
        }
      }
      0xD2 => {
        if !FlagsRegister::from(cpu.registers.f).carry {
          cpu.pc = cpu.read_u16_at_pc();
        } else {
          cpu.pc = cpu.pc.wrapping_add(2)
        }
      }
      0xDA => {
        if FlagsRegister::from(cpu.registers.f).carry {
          cpu.pc = cpu.read_u16_at_pc();
        } else {
          cpu.pc = cpu.pc.wrapping_add(2)
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
        }
      }
      0xC8 => {
        if FlagsRegister::from(cpu.registers.f).zero {
          cpu.pc = cpu.pop();
        }
      }
      0xD0 => {
        if !FlagsRegister::from(cpu.registers.f).carry {
          cpu.pc = cpu.pop();
        }
      }
      0xD8 => {
        if FlagsRegister::from(cpu.registers.f).carry {
          cpu.pc = cpu.pop();
        }
      }
      0xD9 => {
        // TODO IMPLEMENT EI THEN COME BACK, THIS IS EI + RET
      }
      /* END RETURN OPCODES */
      /* START CALL OPCODES */
      0xCD => {
        let lsb = cpu.ram[cpu.pc as usize];
        let msb = cpu.ram[(cpu.pc + 1) as usize];
        let next_address: u16 = two_bytes_to_u16(lsb, msb);
        
        // push current PC onto the stack
        cpu.push(cpu.pc + 2);
        // set the PC to be A16
        cpu.pc = next_address;
      }
      0xCC => {
        if FlagsRegister::from(cpu.registers.f).zero {
          let lsb = cpu.ram[cpu.pc as usize];
          let msb = cpu.ram[(cpu.pc + 1) as usize];
          let next_address: u16 = two_bytes_to_u16(lsb, msb);
          
          // push current PC onto the stack
          cpu.push(cpu.pc + 2);
          // set the PC to be A16
          cpu.pc = next_address;
        } else {
          cpu.pc += 2;
        }
      }
      0xC4 => {
        if !FlagsRegister::from(cpu.registers.f).zero {
          let lsb = cpu.ram[cpu.pc as usize];
          let msb = cpu.ram[(cpu.pc + 1) as usize];
          let next_address: u16 = two_bytes_to_u16(lsb, msb);
          
          // push current PC onto the stack
          cpu.push(cpu.pc + 2);
          // set the PC to be A16
          cpu.pc = next_address;
        } else {
          cpu.pc += 2;
        }
      }
      0xD4 => {
        if !FlagsRegister::from(cpu.registers.f).carry {
          let lsb = cpu.ram[cpu.pc as usize];
          let msb = cpu.ram[(cpu.pc + 1) as usize];
          let next_address: u16 = two_bytes_to_u16(lsb, msb);
          // push current PC onto the stack
          cpu.push(cpu.pc + 2);
          // set the PC to be A16
          cpu.pc = next_address;
        } else {
          cpu.pc += 2;
        }
      }
      0xDC => {
        if FlagsRegister::from(cpu.registers.f).carry {
          let lsb = cpu.ram[cpu.pc as usize];
          let msb = cpu.ram[(cpu.pc + 1) as usize];
          let next_address: u16 = two_bytes_to_u16(lsb, msb);
          
          // push current PC onto the stack
          cpu.push(cpu.pc + 2);
          // set the PC to be A16
          cpu.pc = next_address;
        } else {
          cpu.pc += 2;
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
        let opcode = cpu.ram[cpu.pc as usize];
        cpu.pc += 1;
        let instruction = &instruction_set.cbprefixed[&format!("{:#04X}", opcode)];
        for operand in &instruction.operands {
          if operand.bytes.is_some() {
            cpu.pc += operand.bytes.unwrap() as u16;
          }
        }
        println!("{:#04X}: {} {:#?} CB Prefixed opcode not implemented", opcode, instruction.mnemonic, instruction.operands);
      }
      _ => {
        let instruction = &instruction_set.unprefixed[&format!("{:#04X}", opcode)];
        for operand in &instruction.operands {
          if operand.bytes.is_some() {
            cpu.pc += operand.bytes.unwrap() as u16;
          }
        }
        println!("{:#04X} {:#04X}: {} {:#?} Unprefixed opcode not implemented", cpu.pc, opcode, instruction.mnemonic, instruction.operands);
      }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let file = File::open("opcodes.json")?;
  let reader = BufReader::new(file);
  let instruction_set: InstructionSet = serde_json::from_reader(reader)?; 

  let mut cpu = CPU::new();
  let mut stack: Vec<u16> = Vec::new();
  

  let file_path = "Tetris.gb"; // "gb-test-roms-master/cpu_instrs/individual/01-special.gb";
  let bytes: Vec<u8> = fs::read(Path::new(&file_path))?;

  cpu.ram[..bytes.len()].copy_from_slice(&bytes);

  /*
  Metadata fields
  (None, 'xxxx'), # 0x100-0x103 (entrypoint)
  (None, '48x'), # 0x104-0x133 (nintendo logo)
  ("title", '15s'), # 0x134-0x142 (cartridge title) (0x143 is shared with the cgb flag)
  ("cgb", 'B'), # 0x143 (cgb flag)
  ("new_licensee_code", 'H'), # 0x144-0x145 (new licensee code)
  ("sgb", 'B'), # 0x146 (sgb `flag)
  ("cartridge_type", 'B'), # 0x147 (cartridge type)
  ("rom_size", 'B'), # 0x148 (ROM size)
  ("ram_size", 'B'), # 0x149 (RAM size)
  ("destination_code", 'B'), # 0x14A (destination code)
  ("old_licensee_code", 'B'), # 0x14B (old licensee code)
  ("mask_rom_version", 'B'), # 0x14C (mask rom version)
  ("header_checksum", 'B'), # 0x14D (header checksum)
  ("global_checksum", 'H'), # 0x14E-0x14F (global checksum)
  
   */

  let cartridge_metadata_start = 0x100;
  let cartridge_metadata_end = 0x14F;
  let cartridge_title = str::from_utf8(&cpu.ram[0x134..0x143])?;
  println!("Cartridge title: {}", cartridge_title);



  loop {
    execute_opcode(&instruction_set, &mut cpu);
    // if cpu.pc >= 65535 {
    //   println!("Preventing PC overflow");
    //   break;
    // }
  }

  Ok(())
}