extern crate sdl3;

use sdl3::Sdl;
use sdl3::pixels::Color;
use sdl3::rect::Rect;
use sdl3::render::Canvas;
use sdl3::video::Window;
use std::ops::Div;
use std::time::Duration;

use crate::cpu::CPU;

const CLOCK_SPEED: i32 = 4194304;
const FRAME_RATE: i32 = 60;

//const CYCLES_PER_FRAME: f32 = CLOCK_SPEED / FRAME_RATE;

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;
pub const MULTIPLIER: u32 = 5;

pub const OBJECT_TILE_DATA_START: usize = 0x8000;

pub struct PPU {
    pub display: [[u8; WIDTH]; HEIGHT],
    pub scanline: u16,
    pub viewport_x: u16,
    pub viewport_y: u16
}

/* Tile data is stored in VRAM in the memory area at $8000-$97FF; 
with each tile taking 16 bytes, this area defines data for 384 tiles */

impl PPU {
    pub fn new() -> PPU {
        return PPU { display: [[0; WIDTH]; HEIGHT], scanline: 0, viewport_x: 0, viewport_y: 0 }
    }

    pub fn render_sdl_window(&mut self, canvas: &mut Canvas<Window>, cpu: &mut CPU, screen_buffer: &[[u8; WIDTH]; HEIGHT]) {
        canvas.clear();
        // goes through scanline 0-144
        for (y, row) in screen_buffer.iter().enumerate() {       
            for (x, &pixel) in row.iter().enumerate() {
                let rect = Rect::new(
                    (x as u32 * MULTIPLIER) as i32,
                    (y as u32 * MULTIPLIER) as i32,
                    MULTIPLIER,
                    MULTIPLIER,
                );
                let mut color = Color::RGB(0xFF, 0xFF, 0xFF);
                //let blah = if scanline % 2 == 0 { 1 } else { 3 };
                match pixel {
                    1 => { color = Color::RGB(0xAA, 0xAA, 0xAA) }
                    2 => { color = Color::RGB(0x44, 0x44, 0x44) }
                    3 => { color = Color::RGB(0, 0, 0) }
                    _ => { /* 0 is already set by default above, all other values should never happen */ }
                }
                canvas.set_draw_color(color);
                canvas.fill_rect(rect).unwrap();
            }
        }
        canvas.present();
    }

    pub fn copy_tile_to_screen_buffer(&mut self, cpu: &mut CPU, screen_buffer: &mut [[u8; WIDTH]; HEIGHT], tile_num: u16) {
        let tile_start_address = OBJECT_TILE_DATA_START + (16 * tile_num) as usize;
        let tile = &cpu.ram[tile_start_address..tile_start_address+16];
        let tile_xmod: usize = ((tile_num % 8) * 8) as usize;
        let tile_ymod: usize = (tile_num.div(8) * 8) as usize;
        for i in 0..8 {
            let lsb = tile[2*i as usize];
            let msb = tile[2*i+1 as usize];
            //let mut row: [u8; 8] = [0; 8];
            for j in 0..8 {
                let color_lbit = (lsb >> (7 - j)) & 1;
                let color_mbit = (msb >> (7 - j)) & 1;
                let color = color_mbit << 1 | color_lbit;
                screen_buffer[i + tile_xmod][j + tile_ymod] = color;
            }
        }
    }
}
