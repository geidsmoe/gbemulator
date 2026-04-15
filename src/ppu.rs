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

#[derive(Clone, Copy)]
pub struct ScreenBufferPixel {
    pub value: u8,
    pub priority: u8
}

#[derive(Clone, Copy, PartialEq)]
pub struct ObjectAttributes {
    pub y: u8,
    pub x: u8,
    pub tile_index: u8,
    pub attributes: u8
}

impl ObjectAttributes {
    pub fn new(y: u8, x: u8, tile_index: u8, attributes: u8) -> ObjectAttributes {
        return ObjectAttributes { y, x, tile_index, attributes }
    }

    // Priority: 0 = No, 1 = BG and Window color indices 1–3 are drawn over this OBJ
    pub fn get_priority(&self) -> u8 {
        (self.attributes >> 7) & 1
    }
    // Y flip: 0 = Normal, 1 = Entire OBJ is vertically mirrored
    pub fn get_yflip(&self) -> u8 {
        (self.attributes >> 6) & 1
    }
    // X flip: 0 = Normal, 1 = Entire OBJ is horizontally mirrored
    pub fn get_xflip(&self) -> u8 {
        (self.attributes >> 5) & 1
    }
    // DMG palette [Non CGB Mode only]: 0 = OBP0, 1 = OBP1
    pub fn get_dmg_palette(&self) -> u8 {
        (self.attributes >> 4) & 1
    }
}

pub struct PPU {
    pub display: [[u8; WIDTH]; HEIGHT],
    pub scanline: u16,
    pub viewport_x: u16,
    pub viewport_y: u16,
    pub background: [[u8; 256]; 256],
}

/* Tile data is stored in VRAM in the memory area at $8000-$97FF; 
with each tile taking 16 bytes, this area defines data for 384 tiles */

impl PPU {
    pub fn new() -> PPU {
        return PPU { display: [[0; WIDTH]; HEIGHT], scanline: 0, viewport_x: 0, viewport_y: 0, background: [[0; 256]; 256] }
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

    pub fn copy_tile_to_screen_buffer(&mut self, cpu: &mut CPU, screen_buffer: &mut [[u8; WIDTH]; HEIGHT], tile_num: u16, y_top: usize, x_left: usize) {
        let tile_start_address = 0x8000 + (16 * tile_num) as usize;
        let tile = &cpu.ram[tile_start_address..tile_start_address+16];
        // let tile_xmod: usize = ((tile_num % 8) * 8) as usize;
        // let tile_ymod: usize = (tile_num.div(8) * 8) as usize;

        for i in 0..8 {
            let lsb = tile[2*i as usize];
            let msb = tile[2*i+1 as usize];
            //let mut row: [u8; 8] = [0; 8];
            for j in 0..8 {
                let color_lbit = (lsb >> (7 - j)) & 1;
                let color_mbit = (msb >> (7 - j)) & 1;
                let color = color_mbit << 1 | color_lbit;
                screen_buffer[i + y_top][j + x_left] = color;
            }
        }
    }

    pub fn build_background(&mut self, cpu: &mut CPU) {
        let tilemap: &[u8];
        let lcdc_bit3_tile_map_toggle = (cpu.get_lcdc() & 8) == 8;
        let lcdc_bit4_tile_data_area = (cpu.get_lcdc() & 16) == 16;

        if lcdc_bit3_tile_map_toggle { // BG uses tilemap $9C00 
            tilemap = &cpu.ram[0x9C00..0xA000];
        } else { // BG uses tilemap 9800
            tilemap = &cpu.ram[0x9800..0x9C00];
        }

        // update BG
        for y in 0..32 {
            for x in 0..32 {
                let background_offset = y*32 + x;
                let tile_start_address: usize;
                if lcdc_bit4_tile_data_area { // use [$8000-$8FFF], unsigned indices 0-255
                    let bg_tile_data_start: usize = 0x8000;
                    let tile_index = tilemap[background_offset];
                    tile_start_address = (bg_tile_data_start + (16 * tile_index as usize));
                } else { // use [[$8800-$97FF]], signed indices [-128, 127]
                    let bg_tile_data_start:i32 = 0x9000;
                    let tile_index:i8 = tilemap[background_offset] as i8;
                    tile_start_address = (bg_tile_data_start + (16 * tile_index as i32)) as usize;
                }

                let tile = &cpu.ram[tile_start_address..tile_start_address+16];
                for i in 0..8 {
                    let lsb = tile[2*i as usize];
                    let msb = tile[2*i+1 as usize];
                    for j in 0..8 {
                        let color_lbit = (lsb >> (7 - j)) & 1;
                        let color_mbit = (msb >> (7 - j)) & 1;
                        let color = color_mbit << 1 | color_lbit;
                        let background_y = (y * 8) as usize;
                        let background_x = (x * 8) as usize; 
                        self.background[i + background_y][j + background_x] = color;
                    }
                }
            }
        }
    }

    pub fn update_whole_buffer(&mut self, cpu: &mut CPU, screen_buffer: &mut [[u8; WIDTH]; HEIGHT]) {
        let lcdc_bit0_bg_enable = (cpu.get_lcdc() & 1) == 1;
        
        // IFF lcdc.0 == 1: copy BG to screen buffer accounting for current values of SCY and SCX
        let scy = cpu.get_scroll_y() as usize;
        let scx = cpu.get_scroll_x() as usize;
        let bottom = scy + 144;
        let right = scx + 160;
        if lcdc_bit0_bg_enable {
            for screen_y in 0..HEIGHT {
                for screen_x in 0..WIDTH {
                    let bg_y = (scy + screen_y as usize) % 256;
                    let bg_x = (scx + screen_x) % 256;
                    screen_buffer[screen_y][screen_x] = self.background[bg_y][bg_x];
                }
            }
        }   
    }

    pub fn update_new(&mut self, cpu: &mut CPU, screen_buffer: &mut [[u8; WIDTH]; HEIGHT], scanline: u8) -> u32 {
        let lcdc = cpu.get_lcdc();
        let bg_win_enable  = (lcdc & 0x01) != 0; // BG & Window master enable (DMG)
        let obj_enable     = (lcdc & 0x02) != 0;
        let obj_tall       = (lcdc & 0x04) != 0; // false=8×8, true=8×16
        let bg_tilemap_hi  = (lcdc & 0x08) != 0; // false=$9800, true=$9C00
        let tile_data_hi   = (lcdc & 0x10) != 0; // false=$8800 signed, true=$8000 unsigned
        let win_enable     = (lcdc & 0x20) != 0;
        let win_tilemap_hi = (lcdc & 0x40) != 0;

        let bgp  = cpu.ram[0xFF47];
        let obp0 = cpu.ram[0xFF48];
        let obp1 = cpu.ram[0xFF49];
        let scy  = cpu.get_scroll_y() as usize;
        let scx  = cpu.get_scroll_x() as usize;
        let wy   = cpu.ram[0xFF4A] as usize;
        let wx   = cpu.ram[0xFF4B] as usize;

        let obj_height: usize = if obj_tall { 16 } else { 8 };
        let sl = scanline as usize;

        // ── Collect sprites for this scanline (max 10, OAM order) ───────
        let mut sprites: Vec<ObjectAttributes> = Vec::new();
        for i in 0..40usize {
            if sprites.len() == 10 { break; }
            let base = 0xFE00 + i * 4;
            let obj_y = cpu.ram[base] as usize;
            // A sprite is on this scanline when: obj_y <= scanline+16 < obj_y+obj_height
            if obj_y <= sl + 16 && sl + 16 < obj_y + obj_height {
                sprites.push(ObjectAttributes::new(
                    cpu.ram[base],
                    cpu.ram[base + 1],
                    cpu.ram[base + 2],
                    cpu.ram[base + 3],
                ));
            }
        }

        // ── OBJ mode-3 dot penalty ───────────────────────────────────────
        // Each sprite costs 6 base dots plus up to 5 variable dots.
        // The variable portion is the number of BG fetch pipeline dots already
        // spent on the current tile when the sprite is encountered:
        //   extra = (obj.x + scx) % 8, capped at 5.
        // Sprites at x=0 are off-screen but still penalise the full 11 dots.
        let oam_dot_penalty: u32 = sprites.iter().map(|obj| {
            let extra = if obj.x == 0 {
                5
            } else {
                ((obj.x as usize + scx) % 8).min(5) as u32
            };
            6 + extra
        }).sum();

        let bg_tilemap_base:  usize = if bg_tilemap_hi  { 0x9C00 } else { 0x9800 };
        let win_tilemap_base: usize = if win_tilemap_hi { 0x9C00 } else { 0x9800 };

        // Is the window active on this scanline?
        let win_y_active = win_enable && bg_win_enable && sl >= wy;
        // wx=7 → window left edge = screen x=0; wx<7 is undefined, saturating_sub handles it.
        let win_left = wx.saturating_sub(7);

        let bg_y = (scy + sl) % 256;

        // ── Per-pixel rendering ─────────────────────────────────────────
        for screen_x in 0..WIDTH {
            let mut bg_color_id: u8 = 0;

            if bg_win_enable {
                let in_window = win_y_active && screen_x >= win_left;

                if in_window {
                    // Window pixel — window has its own tile coordinates
                    let win_px    = screen_x - win_left;
                    let win_py    = sl - wy;
                    let tile_col  = win_px / 8;
                    let tile_row  = win_py / 8;
                    let pixel_col = win_px % 8;
                    let pixel_row = win_py % 8;

                    let tile_start = if tile_data_hi {
                        0x8000 + 16 * cpu.ram[win_tilemap_base + tile_row * 32 + tile_col] as usize
                    } else {
                        (0x9000i32 + 16 * cpu.ram[win_tilemap_base + tile_row * 32 + tile_col] as i8 as i32) as usize
                    };
                    let lsb = cpu.ram[tile_start + 2 * pixel_row];
                    let msb = cpu.ram[tile_start + 2 * pixel_row + 1];
                    bg_color_id = ((msb >> (7 - pixel_col)) & 1) << 1
                                | ((lsb >> (7 - pixel_col)) & 1);
                } else {
                    // Background pixel
                    let bg_x      = (scx + screen_x) % 256;
                    let tile_col  = bg_x / 8;
                    let tile_row  = bg_y / 8;
                    let pixel_col = bg_x % 8;
                    let pixel_row = bg_y % 8;

                    let tile_start = if tile_data_hi {
                        0x8000 + 16 * cpu.ram[bg_tilemap_base + tile_row * 32 + tile_col] as usize
                    } else {
                        (0x9000i32 + 16 * cpu.ram[bg_tilemap_base + tile_row * 32 + tile_col] as i8 as i32) as usize
                    };
                    let lsb = cpu.ram[tile_start + 2 * pixel_row];
                    let msb = cpu.ram[tile_start + 2 * pixel_row + 1];
                    bg_color_id = ((msb >> (7 - pixel_col)) & 1) << 1
                                | ((lsb >> (7 - pixel_col)) & 1);
                }
            }

            screen_buffer[sl][screen_x] = (bgp >> (bg_color_id * 2)) & 0x03;

            // ── Sprite overlay ──────────────────────────────────────────
            if obj_enable {
                for obj in &sprites {
                    // x=0 means the sprite is shifted fully off the left edge — skip
                    if obj.x == 0 { continue; }
                    let obj_left = obj.x.wrapping_sub(8) as usize;
                    if screen_x < obj_left || screen_x >= obj_left.wrapping_add(8) { continue; }

                    let mut tile_px_col = screen_x.wrapping_sub(obj_left);
                    if obj.get_xflip() == 1 { tile_px_col = 7usize.wrapping_sub(tile_px_col); }

                    let mut tile_px_row = (sl + 16) - obj.y as usize;
                    if obj.get_yflip() == 1 { tile_px_row = obj_height.wrapping_sub(1).wrapping_sub(tile_px_row); }

                    // In 8×16 mode bit 0 of tile index selects top/bottom tile
                    let (tile_idx, row_in_tile) = if obj_tall {
                        let base = obj.tile_index & 0xFE;
                        if tile_px_row < 8 { (base, tile_px_row) }
                        else               { (base.wrapping_add(1), tile_px_row.wrapping_sub(8)) }
                    } else {
                        (obj.tile_index, tile_px_row)
                    };

                    let tile_start = 0x8000 + 16 * tile_idx as usize;
                    let lsb = cpu.ram[tile_start + 2 * row_in_tile];
                    let msb = cpu.ram[tile_start + 2 * row_in_tile + 1];
                    let color_id = ((msb >> (7 - tile_px_col)) & 1) << 1
                                 | ((lsb >> (7 - tile_px_col)) & 1);

                    if color_id == 0 { continue; } // color 0 is always transparent

                    // BG-over-OBJ priority: BG/Win colors 1-3 win over this sprite
                    if obj.get_priority() == 1 && bg_color_id != 0 { break; }

                    let palette = if obj.get_dmg_palette() == 1 { obp1 } else { obp0 };
                    screen_buffer[sl][screen_x] = (palette >> (color_id * 2)) & 0x03;
                    break; // first non-transparent sprite pixel wins
                }
            }
        }

        oam_dot_penalty
    }

    pub fn update(&mut self, cpu: &mut CPU, screen_buffer: &mut [[u8; WIDTH]; HEIGHT], scanline: u8) -> u32 {
        let mut oam_dot_penalty = 0;
        
        let lcdc_bit0_bg_enable = (cpu.get_lcdc() & 1) == 1;
        let lcdc_bit1_obj_enable = (cpu.get_lcdc() & 2) == 2; 
        let lcdc_bit2_obj_size = (cpu.get_lcdc() & 4) == 4; // 0 = 8×8; 1 = 8×16
        let lcdc_bit3_tile_map_toggle = (cpu.get_lcdc() & 8) == 8;
        let lcdc_bit4_tile_data_area = (cpu.get_lcdc() & 16) == 16;
        let bgp = cpu.ram[0xFF47];
        let obp0 = cpu.ram[0xFF48];
        let obp1 = cpu.ram[0xFF49];

        let obj_height = if lcdc_bit2_obj_size { 16 } else { 8 };
        let obj_width = 8; // this doesn't vary but it will make later code less painful to read
        let obj_tileblock_base: usize = 0x8000;
        let bg_tilemap_base: usize = if lcdc_bit3_tile_map_toggle { 0x9C00 } else { 0x9800 };

        let scy = cpu.get_scroll_y() as usize;
        let scx = cpu.get_scroll_x() as usize;

        // The row in the 256x256 background that this scanline maps to
        let bg_y = (scy + scanline as usize) % 256;
        let tile_row = bg_y / 8;
        let pixel_row = bg_y % 8;

        if lcdc_bit0_bg_enable {
            // render background
            for screen_x in 0..WIDTH {
                let bg_x = (scx + screen_x) % 256;
                let tile_col = bg_x / 8;
                let pixel_col = bg_x % 8;

                let tilemap_index = tile_row * 32 + tile_col;
                let tile_start_address: usize = if lcdc_bit4_tile_data_area {
                    0x8000 + 16 * cpu.ram[bg_tilemap_base + tilemap_index] as usize
                } else {
                    (0x9000i32 + 16 * cpu.ram[bg_tilemap_base + tilemap_index] as i8 as i32) as usize
                };

                let lsb = cpu.ram[tile_start_address + 2 * pixel_row];
                let msb = cpu.ram[tile_start_address + 2 * pixel_row + 1];
                let color_lbit = (lsb >> (7 - pixel_col)) & 1;
                let color_mbit = (msb >> (7 - pixel_col)) & 1;
                let color_id = color_mbit << 1 | color_lbit;

                // Remap through BGP palette register
                let shade = (bgp >> (color_id * 2)) & 0x03;
                screen_buffer[scanline as usize][screen_x] = shade;
            }
        }

        let mut oam_memory: u16 = 0xFE00;
        if lcdc_bit1_obj_enable {
            let mut objects_to_render: Vec<ObjectAttributes> = Vec::new();
            // OAM is from FE00-FE9F
            while oam_memory <= 0xFE9F {
                // obj_attributes.y includes 16 pixels on either side of the screen 
                let obj_y = cpu.ram[oam_memory as usize];
                // object is visible on screen, 10 object max per scanline isn't hit yet, 
                // and scanline is within the bounds of the object
                if obj_y.wrapping_add(obj_height) > 16 && obj_y < (HEIGHT as u8 + 16) && objects_to_render.len() < 10 
                    && obj_y <= (scanline + 16) && (scanline + 16) < (obj_y.wrapping_add(obj_height)) {
                    let obj = ObjectAttributes::new(
                        cpu.ram[oam_memory as usize],
                        cpu.ram[(oam_memory + 1) as usize], 
                        cpu.ram[(oam_memory + 2) as usize],
                        cpu.ram[(oam_memory + 3) as usize]
                    );
                    objects_to_render.push(obj);
                }
                oam_memory += 4;
            }
            /*  Potential TODO: the DMG considers objects in order of X attribute, not order in OAM. 
                The CGP considers objects in OAM address order though, so this *should* be okay to get games running */ 

            let mut objects_added_to_penalty: Vec<ObjectAttributes> = Vec::new();
            let mut bg_tiles_added_to_penalty = Vec::new();
            
            for screen_x in 0..WIDTH {
                for obj_attrs in &objects_to_render {
                    if !objects_added_to_penalty.contains(obj_attrs) {
                        // Incur a flat, 6-dot penalty (from fetching the OBJ’s tile).
                        let mut current_obj_penalty = 6;
                        
                        let bg_y = (scy + scanline as usize) % 256;
                        let bg_tile_row = bg_y / 8;
                        // find where in the background the object's leftmost pixel is
                        let obj_x_bg_x = (obj_attrs.x as i16 - 8 + scx as i16).rem_euclid(256);
                        let bg_tile_col = obj_x_bg_x / 8;
                        let bg_pixel_col = obj_x_bg_x % 8;

                        let tilemap_index = bg_tile_row * 32 + bg_tile_col as usize;
                        if !bg_tiles_added_to_penalty.contains(&tilemap_index) {
                            if obj_attrs.x == 0 {
                                current_obj_penalty = 11;
                            } else {
                                let mut bg_tile_pixels_to_right = 7 - bg_pixel_col;
                                bg_tile_pixels_to_right -= 2;
                                if bg_tile_pixels_to_right > 0 {
                                    current_obj_penalty += bg_tile_pixels_to_right;
                                }
                            bg_tiles_added_to_penalty.push(tilemap_index);
                            }
                            objects_added_to_penalty.push(*obj_attrs);
                            oam_dot_penalty += current_obj_penalty;
                        }
                    }

                    // object is on screen and current pixel on scanline is in this object
                    if obj_attrs.x > 0 && obj_attrs.x < (WIDTH as u8 + 8) && 
                        (screen_x + 8) < (obj_attrs.x.wrapping_add(obj_width)) as usize && (screen_x + 8) >= obj_attrs.x as usize {
                        let mut tile_pixel_row = (scanline + 16 - obj_attrs.y) as usize;
                        if obj_attrs.get_yflip() == 1 {
                            tile_pixel_row = tile_pixel_row.abs_diff(obj_height as usize - 1);
                        }
                        let mut tile_pixel_col = screen_x + 8 - obj_attrs.x as usize;
                        if obj_attrs.get_xflip() == 1 {
                            tile_pixel_col = tile_pixel_col.abs_diff(7);
                        }

                        let tile_start_address: usize = obj_tileblock_base + 16 * obj_attrs.tile_index as usize;
                        let lsb = cpu.ram[tile_start_address + 2 * tile_pixel_row];
                        let msb = cpu.ram[tile_start_address + 2 * tile_pixel_row + 1];
                        let color_lbit = (lsb >> (7 - tile_pixel_col)) & 1;
                        let color_mbit = (msb >> (7 - tile_pixel_col)) & 1;
                        let color_id = color_mbit << 1 | color_lbit;

                        // Remap through OBP0 or OBP1 palette register
                        let object_palette = if obj_attrs.get_dmg_palette() == 1 { obp1 } else { obp0 };
                        let shade = (object_palette >> (color_id * 2)) & 0x03;
                        
                        if color_id != 0 && (obj_attrs.get_priority() == 0 || screen_buffer[scanline as usize][screen_x] == 0) {
                            screen_buffer[scanline as usize][screen_x] = shade;
                        }  
                        break;
                    } 
                }
            }
        }
        
        
        return oam_dot_penalty as u32;
    }

    
}
