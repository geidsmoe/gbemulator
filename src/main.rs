mod registers;
mod cpu;
mod instructions;
pub mod tests;
pub mod ppu;

use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::cpu::CPU;
use crate::instructions::{InstructionSet, execute_opcode};

pub fn gameboy_doctor_cpu_log(cpu: &CPU) {
  //prints A:00 F:11 B:22 C:33 D:44 E:55 H:66 L:77 SP:8888 PC:9999 PCMEM:AA,BB,CC,DD
  println!("A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}",
              cpu.registers.a, cpu.registers.f, cpu.registers.b, cpu.registers.c, cpu.registers.d, cpu.registers.e, cpu.registers.h, cpu.registers.l, cpu.sp, cpu.pc, cpu.ram[cpu.pc as usize], cpu.ram[(cpu.pc+1) as usize], cpu.ram[(cpu.pc+2) as usize], cpu.ram[(cpu.pc+3) as usize]);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let file = File::open("opcodes.json")?;
  let reader = BufReader::new(file);
  let instruction_set: InstructionSet = serde_json::from_reader(reader)?; 

  let mut cpu = CPU::gb_doctor_cpu();

  let file_path = "gb-test-roms-master/cpu_instrs/individual/02-interrupts.gb";
  let bytes: Vec<u8> = fs::read(Path::new(&file_path))?;

  cpu.ram[..bytes.len()].copy_from_slice(&bytes);
  cpu.ram[0xFF44] = 0x90;

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

  //let cartridge_metadata_start = 0x100;
  //let cartridge_metadata_end = 0x14F;
  //let cartridge_title = str::from_utf8(&cpu.ram[0x134..0x143])?;
  //println!("Cartridge title: {}", cartridge_title);

  gameboy_doctor_cpu_log(&cpu);

  loop {
    let interrupt_cycles = cpu.handle_interrupts();
    cpu.update_timer(interrupt_cycles);
    if cpu.halted {
      cpu.update_timer(4);
    } else {
      let cycles = execute_opcode(&instruction_set, &mut cpu);
      cpu.update_timer(cycles);
      gameboy_doctor_cpu_log(&cpu);
    }
  }

  Ok(())
}