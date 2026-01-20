mod registers;  

use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub struct CPU {
  pub pc: u16,
  pub registers: registers::Registers,
  pub flags_register: registers::FlagsRegister,
  pub ram: [u8; 0x10000],
}

impl CPU {
  pub fn new() -> CPU {
    return CPU { pc: 0x100, registers: registers::Registers::new(), flags_register: registers::FlagsRegister::new(), ram: [0; 0x10000], }
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
struct InstructionSet {
    unprefixed: HashMap<String, Instruction>,
    cbprefixed: HashMap<String, Instruction>
}

fn jump(opcode: u8, instruction_set: &InstructionSet, cpu: &CPU) -> u16 {
  let instruction = &instruction_set.unprefixed[&format!("{:#04X}", opcode)];
  println!("{:#04X} {:#04X}: {} {:#?} JP A16", cpu.pc, opcode, instruction.mnemonic, instruction.operands);
  let lower_order = cpu.ram[cpu.pc as usize];
  let upper_order = cpu.ram[(cpu.pc + 1) as usize];
  let next_address: u16 = (((upper_order as u16) << 8) | (lower_order as u16)).into();
  println!("{:#04X} = {:#04X} | {:#04X}", next_address, upper_order, lower_order);
  return next_address;
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


  let mut num_instructions: i64 = 0;
  loop {
    let opcode = cpu.ram[cpu.pc as usize];
    cpu.pc += 1;
    num_instructions += 1;
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
        println!("{:#04X} {:#04X}: {} {:#?} CB Prefixed opcode not implemented", num_instructions, opcode, instruction.mnemonic, instruction.operands);
      }
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
    if cpu.pc >= 65535 {
      println!("Preventing PC overflow");
      break;
    }
  }

  Ok(())
}