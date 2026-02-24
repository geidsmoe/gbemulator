extern crate sdl3;

mod registers;
mod cpu;
mod instructions;
pub mod tests;
pub mod ppu;

use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;

use sdl3::event::Event;
use sdl3::keyboard::Keycode;

use crate::cpu::CPU;
use crate::instructions::{InstructionSet, execute_opcode};
use crate::ppu::{PPU, WIDTH, HEIGHT, MULTIPLIER};

pub fn gameboy_doctor_cpu_log(cpu: &CPU) {
  //prints A:00 F:11 B:22 C:33 D:44 E:55 H:66 L:77 SP:8888 PC:9999 PCMEM:AA,BB,CC,DD
  println!("A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}",
              cpu.registers.a, cpu.registers.f, cpu.registers.b, cpu.registers.c, cpu.registers.d, cpu.registers.e, cpu.registers.h, cpu.registers.l, cpu.sp, cpu.pc, cpu.ram[cpu.pc as usize], cpu.ram[(cpu.pc+1) as usize], cpu.ram[(cpu.pc+2) as usize], cpu.ram[(cpu.pc+3) as usize]);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let file = File::open("opcodes.json")?;
  let reader = BufReader::new(file);
  let instruction_set: InstructionSet = serde_json::from_reader(reader)?; 

  let mut cpu = CPU::new();
  let mut ppu = PPU::new();

  let file_path = "gb-test-roms-master/instr_timing/instr_timing.gb"; //"mealybug-tearoom-tests/m3_scx_low_3_bits.gb";
  let bytes: Vec<u8> = fs::read(Path::new(&file_path))?;

  cpu.ram[..bytes.len()].copy_from_slice(&bytes);
  //cpu.ram[0xFF44] = 0x90; // only for gameboy doctor

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

  //gameboy_doctor_cpu_log(&cpu);

  let sdl_context = sdl3::init().unwrap();
  let video_subsystem = sdl_context.video().unwrap();

  let mut screen_buffer: [[u8; WIDTH]; HEIGHT] = [[0; WIDTH]; HEIGHT];

  let window = video_subsystem.window("rust-sdl3 demo", (WIDTH as u32) * MULTIPLIER, (HEIGHT as u32) * MULTIPLIER)
      .position_centered()
      .build()
      .unwrap();

  let mut canvas = window.into_canvas();
  let mut event_pump = sdl_context.event_pump().unwrap();
  let texture_creator = canvas.texture_creator();
  let mut texture = texture_creator
    .create_texture_streaming(sdl3::pixels::PixelFormat::RGB24, WIDTH as u32, HEIGHT as u32)
    .unwrap();
  texture.set_scale_mode(sdl3::render::ScaleMode::Nearest);

  'running: loop {
    for scanline in 0..154 {
      cpu.set_ly(scanline);
      
      while cpu.temp_cycles < 456 {
        let scroll_x = cpu.get_scroll_x() as usize;
        let mode3_drawing_length: u32 = (HEIGHT as u32) + 12 + (scroll_x as u32 % 8);
        let hblank_length = 376 - mode3_drawing_length;
        if cpu.temp_cycles < 80 && cpu.ppu_mode != 2 { // OAM
          cpu.set_stat_ppu_mode(2);
          if cpu.get_stat() & (1 << 5) != 0 { // game wants OAM interrupt?
              cpu.request_lcd_interrupt();
          }
        } else if cpu.temp_cycles < mode3_drawing_length + 80 && cpu.ppu_mode != 3 { // Drawing
          cpu.set_stat_ppu_mode(3);
        } else if cpu.temp_cycles < 80 + mode3_drawing_length + hblank_length && cpu.ppu_mode != 0 { // Hblank
          cpu.set_stat_ppu_mode(0);
          if cpu.get_stat() & (1 << 3) != 0 { // game wants HBlank interrupt?
              cpu.request_lcd_interrupt();
          }
        }

        let interrupt_cycles = cpu.handle_interrupts();
        cpu.temp_cycles += interrupt_cycles;
        cpu.update_timer(interrupt_cycles);
        if cpu.halted {
          cpu.temp_cycles += 4;
          cpu.update_timer(4);
        } else {
          let cycles = execute_opcode(&instruction_set, &mut cpu);
          cpu.temp_cycles += cycles;
          cpu.update_timer(cycles);
          //gameboy_doctor_cpu_log(&cpu);
        }
      }
      cpu.temp_cycles -= 456;
      if (scanline as usize) < HEIGHT {
        //ppu.update(&mut cpu, &mut screen_buffer, scanline);
        ppu.build_background(&mut cpu);
        ppu.update_whole_buffer(&mut cpu, &mut screen_buffer);
      } else if (scanline as usize) == HEIGHT {
        cpu.request_vblank_interrupt();
        cpu.set_stat_ppu_mode(1); // set STAT to VBlank
      }
    }
    
    /*for i in 0..64 {
      let y = ((i / 8) * 8) as usize;
      let x = ((i % 8) * 8) as usize;
      ppu.copy_tile_to_screen_buffer(&mut cpu, &mut screen_buffer, i, y, x);
    }*/
    ppu.render_sdl_window(&mut canvas, &mut texture, &mut screen_buffer);
    
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit {..} |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                break 'running
            },
            _ => {}
        }
    }
    ::std::thread::sleep(Duration::from_millis(16)); // ~60fps - VSync in present() handles pacing
  }

  Ok(())
}