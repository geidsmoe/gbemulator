mod registers;  

use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub struct CPU {
  pub pc: u16,
  pub registers: registers::Registers,
  pub ram: [u8; 0x10000],
}

impl CPU {
  pub fn new() -> CPU {
    return CPU { pc: 0x100, registers: registers::Registers::new(), ram: [0; 0x10000], }
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let file = File::open("opcodes.json")?;
  let reader = BufReader::new(file);
  let instruction_set: InstructionSet = serde_json::from_reader(reader)?; 

  let mut cpu = CPU::new();
  let mut stack: Vec<u16> = Vec::new();
  

  let file_path = "Tetris.gb"; // "gb-test-roms-master/cpu_instrs/individual/01-special.gb";
  let bytes: Vec<u8> = fs::read(Path::new(&file_path))?;

  let rom_start_in_ram = 0x100; 
  cpu.ram[rom_start_in_ram..rom_start_in_ram + bytes.len()].copy_from_slice(&bytes);

  loop {
    let mut opcode = cpu.ram[cpu.pc as usize];
    cpu.pc += 1;
    match opcode  {
      0x00 => {
        //println!("{:#04X}: NOP", opcode);
      }
      0xCB => {
        opcode = cpu.ram[cpu.pc as usize];
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
        println!("{:#04X}: {} {:#?} Unprefixed opcode not implemented", opcode, instruction.mnemonic, instruction.operands);
      }
    }
    if cpu.pc >= 65535 {
      println!("Preventing PC overflow");
      break;
    }
  }

  Ok(())
}