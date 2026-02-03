

const CLOCK_SPEED: i32 = 4194304;
const FRAME_RATE: i32 = 60;

//const CYCLES_PER_FRAME: f32 = CLOCK_SPEED / FRAME_RATE;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

pub struct ppu {
    pub display: [[u8; WIDTH]; HEIGHT],
    pub scanline: u16,
    pub viewport_x: u16,
    pub viewport_y: u16
}

