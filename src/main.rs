mod registers;
pub mod tests;  

use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;


fn two_bytes_to_u16(lsb: u8, msb: u8) -> u16 {
  (((msb as u16) << 8) | (lsb as u16)).into()
}

pub struct CPU {
  pub pc: u16,
  pub sp: u16,
  pub registers: registers::Registers,
  pub flags_register: registers::FlagsRegister,
  pub ram: [u8; 0x10000],
}

impl CPU {
  pub fn new() -> CPU {
    return CPU { pc: 0x100, sp: 0xFFFF, registers: registers::Registers::new(), flags_register: registers::FlagsRegister::new(), ram: [0; 0x10000], }
  }

  pub fn pop(&mut self) -> u16 {
    let lsb = self.ram[self.sp as usize];
    let msb = self.ram[(self.sp + 1) as usize];
    self.sp += 2;
    return two_bytes_to_u16(lsb, msb);
  }

  pub fn push(&mut self, next_address: u16) {
    let lsb: u8 =  (next_address & 0xFF) as u8;
    let msb: u8 = ((next_address & 0xFF00) >> 8) as u8;
    self.ram[(self.sp - 1) as usize] = msb;
    self.ram[(self.sp - 2) as usize] = lsb;
    self.sp -= 2;
  }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Instruction {
    mnemonic: String,
    bytes: u8,
    cycles: Vec<u32>,
    operands: Vec<Operand>,
    immediate: bool,
    flags: Flags,
}

#[derive(Debug, Deserialize, Serialize)]
struct Operand {
    name: String,
    #[serde(default)]
    bytes: Option<u8>,
    immediate: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct Flags {
    Z: String,
    N: String,
    H: String,
    C: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstructionSet {
    unprefixed: HashMap<String, Instruction>,
    cbprefixed: HashMap<String, Instruction>
}

fn jump(opcode: u8, instruction_set: &InstructionSet, cpu: &CPU) -> u16 {
  let instruction = &instruction_set.unprefixed[&format!("{:#04X}", opcode)];
  println!("{:#04X} {:#04X}: {} {:#?} JP A16", cpu.pc, opcode, instruction.mnemonic, instruction.operands);
  let lsb = cpu.ram[cpu.pc as usize];
  let msb = cpu.ram[(cpu.pc + 1) as usize];
  let next_address: u16 = two_bytes_to_u16(lsb, msb);
  println!("{:#04X} = {:#04X} | {:#04X}", next_address, msb, lsb);
  return next_address;
}

pub fn execute_opcode(instruction_set: &InstructionSet, cpu: &mut CPU) {
  let opcode = cpu.ram[cpu.pc as usize];
    cpu.pc += 1;
    match opcode  {
      0x00 => {
        //println!("{:#04X}: NOP", opcode);
      }
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
      /* START JUMP OPCODES */
      0xC2 => {
        if !cpu.flags_register.zero {
          cpu.pc = jump(opcode, &instruction_set, &cpu);
        } else {
          cpu.pc += 2;
        }
      }
      0xC3 => {
        cpu.pc = jump(opcode, &instruction_set, &cpu);
      }
      0xCA => {
        if cpu.flags_register.zero {
          cpu.pc = jump(opcode, &instruction_set, &cpu);
        } else {
          cpu.pc += 2;
        }
      }
      0xD2 => {
        if !cpu.flags_register.carry {
          cpu.pc = jump(opcode, &instruction_set, &cpu);
        } else {
          cpu.pc += 2
        }
      }
      0xDA => {
        if cpu.flags_register.carry {
          cpu.pc = jump(opcode, &instruction_set, &cpu);
        } else {
          cpu.pc += 2
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
        if !cpu.flags_register.zero {
          cpu.pc = cpu.pop();
        }
      }
      0xC8 => {
        if cpu.flags_register.zero {
          cpu.pc = cpu.pop();
        }
      }
      0xD0 => {
        if !cpu.flags_register.carry {
          cpu.pc = cpu.pop();
        }
      }
      0xD8 => {
        if cpu.flags_register.carry {
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
        if cpu.flags_register.zero {
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
        if !cpu.flags_register.zero {
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
        if !cpu.flags_register.carry {
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
        if cpu.flags_register.carry {
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