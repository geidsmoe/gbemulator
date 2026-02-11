extern crate sdl3;

use sdl3::Sdl;
use sdl3::render::{Canvas, Texture};
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

    pub fn render_sdl_window(&mut self, canvas: &mut Canvas<Window>, texture: &mut Texture, screen_buffer: &[[u8; WIDTH]; HEIGHT]) {
        //canvas.clear();
        let palette: [[u8; 3]; 4] = [
            [0xFF, 0xFF, 0xFF],
            [0xAA, 0xAA, 0xAA],
            [0x44, 0x44, 0x44],
            [0x00, 0x00, 0x00],
        ];
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for (y, row) in screen_buffer.iter().enumerate() {
                for (x, &pixel) in row.iter().enumerate() {
                    let idx = y * pitch + x * 3;
                    let color = &palette[(pixel & 3) as usize];
                    buffer[idx] = color[0];
                    buffer[idx + 1] = color[1];
                    buffer[idx + 2] = color[2];
                }
            }
        }).unwrap();
        canvas.copy(texture, None, None).unwrap();
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
                screen_buffer[i + tile_ymod][j + tile_xmod] = color;
            }
        }
    }
}
