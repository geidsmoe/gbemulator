extern crate sdl3;

use sdl3::Sdl;
use sdl3::pixels::Color;
use sdl3::render::Canvas;
use sdl3::video::Window;
use std::time::Duration;

const CLOCK_SPEED: i32 = 4194304;
const FRAME_RATE: i32 = 60;

//const CYCLES_PER_FRAME: f32 = CLOCK_SPEED / FRAME_RATE;

pub const WIDTH: u32 = 160;
pub const HEIGHT: u32 = 144;

pub struct PPU {
    pub display: [[u8; WIDTH as usize]; HEIGHT as usize],
    pub scanline: u16,
    pub viewport_x: u16,
    pub viewport_y: u16
}

/* Tile data is stored in VRAM in the memory area at $8000-$97FF; 
with each tile taking 16 bytes, this area defines data for 384 tiles */

impl PPU {
    pub fn new() -> PPU {
        return PPU { display: [[0; WIDTH as usize]; HEIGHT as usize], scanline: 0, viewport_x: 0, viewport_y: 0 }
    }

    pub fn render(&mut self, canvas: &mut Canvas<Window>, sdl_context: &Sdl) {
        canvas.set_draw_color(Color::RGB(0, 255, 255));
        canvas.clear();
        canvas.present();

        // The rest of the game loop goes here...

        //canvas.present();
        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
