use crate::registers::*;

pub fn two_bytes_to_u16(lsb: u8, msb: u8) -> u16 {
    (((msb as u16) << 8) | (lsb as u16)).into()
}

// 8-bit carry helpers
pub fn half_carry_add_8bit(a: u8, b: u8, carry: u8) -> bool {
    (a & 0x0f) + (b & 0x0f) + carry > 0x0f
}

pub fn carry_add_8bit(a: u8, b: u8, carry: u8) -> bool {
    (a as u16) + (b as u16) + (carry as u16) > 0xff
}

pub fn half_borrow_sub_8bit(a: u8, b: u8, carry: u8) -> bool {
    (a & 0x0f) < (b & 0x0f) + carry
}

pub fn borrow_sub_8bit(a: u8, b: u8, carry: u8) -> bool {
    (a as u16) < (b as u16) + (carry as u16)
}

// 16-bit carry helpers (half_carry at bit 11)
pub fn half_carry_add_16bit(a: u16, b: u16) -> bool {
    (a & 0x0fff) + (b & 0x0fff) > 0x0fff
}

pub fn carry_add_16bit(a: u16, b: u16) -> bool {
    (a as u32) + (b as u32) > 0xffff
}
  
pub struct CPU {
  pub pc: u16,
  pub sp: u16,
  pub registers: Registers8,
  pub ram: [u8; 0x10000],
  pub ime: u8,
  pub ie: u8,
  pub interrupt_flag_request: u8,
  pub div_cycles: u32,
  pub tcycles: u32,
  pub temp_cycles: u32,
  pub halted: bool,
  pub ppu_mode: u8,
  pub dpad: u8,
  pub buttons: u8,
}

impl CPU {
    pub fn new() -> CPU {
        let mut cpu = CPU { pc: 0x0100, sp: 0xFFFE, registers: Registers8::gb_doctor_values(), ram: [0; 0x10000], ime: 0, ie: 0, interrupt_flag_request: 0, div_cycles: 0, tcycles: 0, halted: false, temp_cycles: 0, ppu_mode: 2, dpad: 0, buttons: 0 };
        // Post-boot IO register values (DMG)
        cpu.ram[0xFF00] = 0xCF;
        cpu.dpad = 0xCF;
        cpu.buttons = 0xCF;
        cpu.ram[0xFF01] = 0x00;
        cpu.ram[0xFF02] = 0x7E;
        cpu.ram[0xFF04] = 0xAB;
        cpu.ram[0xFF05] = 0x00; // TIMA
        cpu.ram[0xFF06] = 0x00; // TMA
        cpu.ram[0xFF07] = 0xF8; // TAC
        cpu.ram[0xFF0F] = 0xE1; // IF

        cpu.ram[0xFF10] = 0x80; // NR10
        cpu.ram[0xFF11] = 0xBF; // NR11
        cpu.ram[0xFF12] = 0xF3; // NR12
        cpu.ram[0xFF13] = 0xFF; // NR13
        cpu.ram[0xFF14] = 0xBF; // NR14

        cpu.ram[0xFF16] = 0x3F; // NR21
        cpu.ram[0xFF17] = 0x00; // NR22
        cpu.ram[0xFF18] = 0xFF; // NR23
        cpu.ram[0xFF19] = 0xBF; // NR24

        cpu.ram[0xFF1A] = 0x7F; // NR30
        cpu.ram[0xFF1B] = 0xFF; // NR31
        cpu.ram[0xFF1C] = 0x9F; // NR32
        cpu.ram[0xFF1D] = 0xFF; // NR33
        cpu.ram[0xFF1E] = 0xBF; // NR34

        cpu.ram[0xFF20] = 0xFF; // NR41
        cpu.ram[0xFF21] = 0x00; // NR42
        cpu.ram[0xFF22] = 0x00; // NR43
        cpu.ram[0xFF23] = 0xBF; // NR44

        cpu.ram[0xFF24] = 0x77; // NR50
        cpu.ram[0xFF25] = 0xF3; // NR51
        cpu.ram[0xFF26] = 0xF1; // NR52

        cpu.ram[0xFF40] = 0x91; // LCDC
        cpu.ram[0xFF41] = 0x85; // STAT
        cpu.ram[0xFF42] = 0x00; // SCY
        cpu.ram[0xFF43] = 0x00; // SCX
        cpu.ram[0xFF44] = 0x00; // LY
        cpu.ram[0xFF45] = 0x00; // LYC
        cpu.ram[0xFF46] = 0xFF; // DMA
        cpu.ram[0xFF47] = 0xFC; // BGP
        cpu.ram[0xFF48] = 0xFF; // OBP0
        cpu.ram[0xFF49] = 0xFF; // OBP1
        cpu.ram[0xFF4A] = 0x00; // WY
        cpu.ram[0xFF4B] = 0x00; // WX
        cpu
    }

    pub fn gb_doctor_cpu() -> CPU {
      let mut gb_doctor_cpu = CPU { pc: 0x0100, sp: 0xFFFE, registers: Registers8::gb_doctor_values(), ram: [0; 0x10000], ime: 0, ie: 0, interrupt_flag_request: 0, div_cycles: 0, tcycles: 0, halted: false, temp_cycles: 0, ppu_mode: 2, dpad: 0, buttons: 0 };
      gb_doctor_cpu.ram[0xFF44] = 0x90;
      gb_doctor_cpu
    }

    pub fn pop(&mut self) -> u16 {
        let lsb = self.ram[self.sp as usize];
        let msb = self.ram[(self.sp.wrapping_add(1)) as usize];
        self.sp = self.sp.wrapping_add(2);
        return two_bytes_to_u16(lsb, msb);
    }

    pub fn push(&mut self, next_address: u16) {
        let lsb: u8 =  (next_address & 0xFF) as u8;
        let msb: u8 = ((next_address & 0xFF00) >> 8) as u8;
        self.ram[(self.sp.wrapping_sub(1)) as usize] = msb;
        self.ram[(self.sp.wrapping_sub(2)) as usize] = lsb;
        self.sp = self.sp.wrapping_sub(2);
    }

    pub fn read_u16_at(&self, addr: u16) -> u16 {
        let lsb = self.read(addr as usize);
        let msb = self.read((addr.wrapping_add(1)) as usize);
        two_bytes_to_u16(lsb, msb)
    }

    pub fn read_i8_at(&self, addr: u16) -> i8 {
      self.read(addr as usize) as i8
    }

    pub fn read_i8_at_pc(&self) -> i8 {
      self.read_i8_at(self.pc)
    }

    pub fn read_u16_at_pc(&self) -> u16 {
        self.read_u16_at(self.pc)
    }

    pub fn add_8bit(&mut self, value: u8) {
        let mut flags_register = FlagsRegister::from(self.registers.f);
        let result = self.registers.a.wrapping_add(value);
        flags_register.carry = carry_add_8bit(self.registers.a, value, 0);
        flags_register.zero = result == 0;
        flags_register.half_carry = half_carry_add_8bit(self.registers.a, value, 0);
        flags_register.subtract = false;
        self.registers.f = u8::from(flags_register);
        self.registers.a = result;
    }

    pub fn add_hl_u16(&mut self, operand: u16) {
      let hl = self.registers.get_hl();
      let result = hl.wrapping_add(operand);
      let mut flags_register = FlagsRegister::from(self.registers.f);
      flags_register.carry = carry_add_16bit(hl, operand);
      flags_register.subtract = false;
      flags_register.half_carry = half_carry_add_16bit(hl, operand);
      self.registers.f = u8::from(flags_register);
      self.registers.set_hl(result);
    }

    pub fn add_hl_16bit(&mut self, reg: RegisterNames16) {
      self.add_hl_u16(self.registers.get_16_bit_reg(reg));
    }

    pub fn addc_8bit(&mut self, value: u8) {
        let mut flags_register = FlagsRegister::from(self.registers.f);
        let carry_value: u8 = if flags_register.carry { 1 } else { 0 };
        let result = self.registers.a.wrapping_add(value).wrapping_add(carry_value);
        flags_register.zero = result == 0;
        flags_register.subtract = false;
        flags_register.half_carry = half_carry_add_8bit(self.registers.a, value, carry_value);
        flags_register.carry = carry_add_8bit(self.registers.a, value, carry_value);
        self.registers.f = u8::from(flags_register);
        self.registers.a = result;
    }

    pub fn sub(&mut self, value: u8, carry: bool) -> u8 {
        let mut flags_register = FlagsRegister::from(self.registers.f);
        let cy = if carry && flags_register.carry { 1 } else { 0 };
        let result = self.registers.a.wrapping_sub(value).wrapping_sub(cy);
        flags_register.zero = result == 0;
        flags_register.subtract = true;
        flags_register.half_carry = half_borrow_sub_8bit(self.registers.a, value, cy);
        flags_register.carry = borrow_sub_8bit(self.registers.a, value, cy);
        self.registers.f = u8::from(flags_register);
        result
      }

      pub fn and(&mut self, value: u8) -> u8 {
        let mut flags_register = FlagsRegister::from(self.registers.f);
        let result = self.registers.a & value;
        flags_register.carry = false;
        flags_register.half_carry = true;
        flags_register.zero = result == 0;
        flags_register.subtract = false;
        self.registers.f = u8::from(flags_register);
        result
      }

      pub fn xor(&mut self, value: u8) -> u8 {
        let mut flags_register = FlagsRegister::from(self.registers.f);
        let result = self.registers.a ^ value;
        flags_register.carry = false;
        flags_register.half_carry = false;
        flags_register.zero = result == 0;
        flags_register.subtract = false;
        self.registers.f = u8::from(flags_register);
        result
      }

      pub fn or(&mut self, value: u8) -> u8 {
        let mut flags_register = FlagsRegister::from(self.registers.f);
        let result = self.registers.a | value;
        flags_register.carry = false;
        flags_register.half_carry = false;
        flags_register.zero = result == 0;
        flags_register.subtract = false;
        self.registers.f = u8::from(flags_register);
        result
      }

      pub fn cp(&mut self, value: u8) {
        // CP is like SUB but doesn't store the result
        self.sub(value, false);
      }

      pub fn inc_r16(&mut self, reg: RegisterNames16) {
        let inc = self.registers.get_16_bit_reg(reg).wrapping_add(1);
        self.registers.set_16_bit_reg(reg, inc);
      }

      pub fn dec_r16(&mut self, reg: RegisterNames16) {
        let dec = self.registers.get_16_bit_reg(reg).wrapping_sub(1);
        self.registers.set_16_bit_reg(reg, dec);
      }

      pub fn inc_8bit(&mut self, value: u8) -> u8 {
        let mut flags_register = FlagsRegister::from(self.registers.f);
        let result = value.wrapping_add(1);
        flags_register.zero = result == 0;
        flags_register.subtract = false;
        flags_register.half_carry = half_carry_add_8bit(value, 1, 0);
        // carry flag not affected
        self.registers.f = u8::from(flags_register);
        result
      }

      pub fn dec_8bit(&mut self, value: u8) -> u8 {
        let mut flags_register = FlagsRegister::from(self.registers.f);
        let result = value.wrapping_sub(1);
        flags_register.zero = result == 0;
        flags_register.subtract = true;
        flags_register.half_carry = half_borrow_sub_8bit(value, 1, 0);
        // carry flag not affected
        self.registers.f = u8::from(flags_register);
        result
      }

      pub fn rotate_left_with_carry(&mut self, value: u8, is_cb_prefixed: bool) -> u8 {
        let mut flags_register = FlagsRegister::from(self.registers.f);
        let result = value.rotate_left(1);
        if is_cb_prefixed { flags_register.zero = result == 0 } else { flags_register.zero = false };
        flags_register.carry = (value & 0x80) != 0;
        flags_register.half_carry = false;
        flags_register.subtract = false;
        self.registers.f = u8::from(flags_register);
        result
      }

      pub fn rotate_left_through_carry(&mut self, value: u8, is_cb_prefixed: bool) -> u8 {
        let mut flags_register = FlagsRegister::from(self.registers.f);
        let cy = if flags_register.carry { 1 } else { 0 };
        let result = (value << 1) | cy;
        if is_cb_prefixed { flags_register.zero = result == 0 } else { flags_register.zero = false };
        flags_register.carry = (value & 0x80) != 0;
        flags_register.half_carry = false;
        flags_register.subtract = false;
        self.registers.f = u8::from(flags_register);
        result
      }

      pub fn rotate_right_with_carry(&mut self, value: u8, is_cb_prefixed: bool) -> u8 {
        let mut flags_register = FlagsRegister::from(self.registers.f);
        let result = value.rotate_right(1);
        if is_cb_prefixed { flags_register.zero = result == 0 } else { flags_register.zero = false };
        flags_register.carry = (value & 0x01) != 0;
        flags_register.half_carry = false;
        flags_register.subtract = false;
        self.registers.f = u8::from(flags_register);
        result
      }

      pub fn rotate_right_through_carry(&mut self, value: u8, is_cb_prefixed: bool) -> u8 {
        let mut flags_register = FlagsRegister::from(self.registers.f);
        let cy = if flags_register.carry { 1 } else { 0 };
        let result = (value >> 1) | (cy << 7);
        if is_cb_prefixed { flags_register.zero = result == 0 } else { flags_register.zero = false };
        flags_register.carry = (value & 0x01) != 0;
        flags_register.half_carry = false;
        flags_register.subtract = false;
        self.registers.f = u8::from(flags_register);
        result
      }

      pub fn shift_left_arithmetic(&mut self, value: u8) -> u8 {
        let mut flags_register = FlagsRegister::from(self.registers.f);
        let result = value << 1;
        flags_register.zero = result == 0;
        flags_register.carry = (value & 0x80) != 0;
        flags_register.half_carry = false;
        flags_register.subtract = false;
        self.registers.f = u8::from(flags_register);
        result
      }

      pub fn shift_right_arithmetic(&mut self, value: u8) -> u8 {
        let mut flags_register = FlagsRegister::from(self.registers.f);
        let result = (value >> 1) | (value & 0x80); // preserve bit 7
        flags_register.zero = result == 0;
        flags_register.carry = (value & 0x01) != 0;
        flags_register.half_carry = false;
        flags_register.subtract = false;
        self.registers.f = u8::from(flags_register);
        result
      }

      pub fn shift_right_logical(&mut self, value: u8) -> u8 {
        let mut flags_register = FlagsRegister::from(self.registers.f);
        let result = value >> 1;
        flags_register.zero = result == 0;
        flags_register.carry = (value & 0x01) != 0;
        flags_register.half_carry = false;
        flags_register.subtract = false;
        self.registers.f = u8::from(flags_register);
        result
      }

      pub fn swap_nibbles(&mut self, value: u8) -> u8 {
        let lsn = value & 0x0F;
        let msn = value >> 4;
        let result = (lsn << 4) | msn;
        let mut flags_register = FlagsRegister::from(self.registers.f);
        flags_register.zero = result == 0;
        flags_register.subtract = false;
        flags_register.half_carry = false;
        flags_register.carry = false;
        self.registers.f = u8::from(flags_register);
        result
      }

      pub fn copy_bit_n_to_zero_flag(&mut self, value: u8, n: u8) {
        let bit: u8 = value & (1 << n);
        let mut flags_register = FlagsRegister::from(self.registers.f);
        flags_register.zero = bit == 0;
        flags_register.subtract = false;
        flags_register.half_carry = true;
        self.registers.f = u8::from(flags_register);
      }

      pub fn reset_bit(&self, value: u8, n: u8) -> u8 {
        value & !(1 << n)
      }

      pub fn set_bit(&self, value: u8, n: u8) -> u8 {
        value | (1 << n)
      }

      pub fn call(&mut self) {
        let next_address = self.read_u16_at_pc();
        // push current PC onto the stack
        self.push(self.pc.wrapping_add(2));
        // set the PC to be A16
        self.pc = next_address;
      }

      pub fn read(&self, address: usize) -> u8 {
        let lcd_enabled = (self.get_lcdc() & 0x80) != 0;
        if lcd_enabled {
          // VRAM is inaccessble during PPU mode 3 (drawing pixels)
          if 0x8000 <= address && address <= 0x9FFF && self.ppu_mode == 3 {
            return 0xFF;
          }
          // OAM is inaccessible during PPU modes 2 and 3 (OAM scan and drawing pixels)
          if 0xFE00 <= address && address <= 0xFE9F && self.ppu_mode > 1 {
            return 0xFF;
          }
        }
        
        if address == 0xFF00 {
          let joypad_flags = (self.ram[0xFF00] >> 4) & 0x3;
          // neither dpad or buttons are selected
          if joypad_flags == 0 || joypad_flags == 3 {
            return self.ram[0xFF00] | 0x0F;
          }
          else if joypad_flags == 1 {
            return self.buttons
          }
          else if joypad_flags == 2 {
            return self.dpad
          }
        }
        
        self.ram[address]
      }

      pub fn write(&mut self, address: usize, mut value: u8) {
        let lcd_enabled = (self.get_lcdc() & 0x80) != 0;
        if lcd_enabled {
          // VRAM is inaccessble during PPU mode 3 (drawing pixels)
          if 0x8000 <= address && address <= 0x9FFF && self.ppu_mode == 3 {
            return;
          }
          // OAM is inaccessible during PPU modes 2 and 3 (OAM scan and drawing pixels)
          if 0xFE00 <= address && address <= 0xFE9F && self.ppu_mode > 1 {
            return;
          }
        }
        
        match address {
          0xFF00 => {
            // neither buttons nor dpad selected, no buttons pressed
            if value == 0x30 {
              value |= 0xF;
              self.ram[address] = value;
              self.dpad = value;
              self.buttons = value;
            } else if value == 0x20 {
              value |= self.dpad & 0x0F;
              self.dpad = value;
            } else if value == 0x10 {
              value |= self.buttons & 0x0F;
              self.buttons = value;
            }
            self.ram[address] = value; 
          }
          0xFF04 => { 
            self.ram[address] = 0;
            self.div_cycles = 0;
          }
          0xFF40 => {
            if (self.ram[0xFF40] & 0x80) != 0 && (value & 0x80) == 0 {
              self.set_stat_ppu_mode(0);
            }
            self.ram[address] = value;
          }
          0xFF43 => {
            self.ram[address] = value;
          }
          0xFF42 => {
            self.ram[address] = value;
          }
          0xFF44 => {
            self.set_ly(value);
          }
          0xFF46 => {
            // OAM DMA: copy 160 bytes from (value << 8) into OAM (0xFE00-0xFE9F)
            let src = (value as usize) << 8;
            for i in 0..0xA0 {
              self.ram[0xFE00_u16.wrapping_add(i) as usize] = self.ram[src.wrapping_add(i as usize)];
            }
            self.ram[address] = value;
          }
          0xFF0F => {
            self.interrupt_flag_request = value;
            self.ram[address] = value;
          }
          0xFFFF => {
            self.ie = value;
            self.ram[address] = value;
          }
          _ => { self.ram[address] = value; }
        }
      }

      pub fn handle_interrupts(&mut self) -> u32 {
        // Interrupt Master Enable && Interrupt enable bit flags (IE) & Interrupt request flags (IF)
        let interrupt_flags_allowed = self.ram[0xFFFF] & self.ram[0xFF0F];
        if self.ime == 1 && interrupt_flags_allowed > 0 {
          self.push(self.pc);
          self.ime = 0;
          if interrupt_flags_allowed & 1 == 1 { // VBlank
            self.ram[0xFF0F] = self.reset_bit(self.ram[0xFF0F], 0);
            self.pc = 0x40;
          } else if interrupt_flags_allowed & 2 == 2 { // LCD
            self.ram[0xFF0F] = self.reset_bit(self.ram[0xFF0F], 1);
            self.pc = 0x48;
          } else if interrupt_flags_allowed & 4 == 4 { // Timer
            self.ram[0xFF0F] = self.reset_bit(self.ram[0xFF0F], 2);
            self.pc = 0x50;
          } else if interrupt_flags_allowed & 8 == 8 { // Serial
            self.ram[0xFF0F] = self.reset_bit(self.ram[0xFF0F], 3);
            self.pc = 0x58;
          } else if interrupt_flags_allowed & 16 == 16 { // Joypad
            self.ram[0xFF0F] = self.reset_bit(self.ram[0xFF0F], 4);
            self.pc = 0x60;
          }
          self.halted = false;
          return 20;
        } else if self.halted && interrupt_flags_allowed > 0 {
          self.halted = false;
        }
        return 0;
      }

      pub fn update_timer(&mut self, new_tcycles: u32) {
        let tima_inc_enable_bit = ((self.ram[0xFF07] >> 2) & 1) == 1;
        let tac_clock_select = self.ram[0xFF07] & 3;
        let tma = self.ram[0xFF06];
        let tima = self.ram[0xFF05];
        let mut tima_mcycle_increment = 256;
        if tac_clock_select == 1 {
          tima_mcycle_increment = 4;
        } else if tac_clock_select == 2 {
          tima_mcycle_increment = 16;
        } else if tac_clock_select == 3 {
          tima_mcycle_increment = 64;
        }
        let tima_tcycle_increment = tima_mcycle_increment * 4;
        // if TIMA inc is enabled AND the increment amount will cause another `tima_tycle_increment` to complete
        if tima_inc_enable_bit && (self.tcycles % tima_tcycle_increment) + new_tcycles >= tima_tcycle_increment  {
          if tima == 0xFF {
            self.write(0xFF05, tma);
            // trying to emulate TIMA overflow delay - won't work quite right because during this "cycle" TIMA won't read as 0, it will read as TMA
            self.tcycles = self.tcycles.wrapping_add(4);
            self.request_timer_interrupt();
          } else {
            let increments: u8 = ((new_tcycles + (self.tcycles % tima_tcycle_increment)) / tima_tcycle_increment) as u8;
            self.write(0xFF05, tima.wrapping_add(increments));
          }
        }
        self.tcycles = self.tcycles.wrapping_add(new_tcycles);

        if (self.div_cycles % 256) + new_tcycles >= 256 {
          self.ram[0xFF04] = self.ram[0xFF04].wrapping_add(1);
        }
        self.div_cycles = self.div_cycles.wrapping_add(new_tcycles);
      }

      pub fn set_ly(&mut self, value: u8) {
        self.ram[0xFF44] = value;
        // if LY == LYC
        if self.ram[0xFF44] == self.ram[0xFF45] {
          let stat = self.get_stat() | (1 << 2);
          self.write(0xFF41, stat);
          if stat & (1 << 6) != 0 { // game wants LYC=LY interrupt?
              self.request_lcd_interrupt();
          }
        }
      }

      pub fn get_ly(&self) -> u8 {
        self.ram[0xFF44]
      }

      pub fn set_stat_ppu_mode(&mut self, mode: u8) {
        self.ppu_mode = mode;
        let stat = self.get_stat() & !3;
        let new_value = stat | (mode & 0x03);
        self.write(0xFF41, new_value);
      }

      pub fn get_stat(&self) -> u8 {
        self.ram[0xFF41]
      }

      pub fn request_vblank_interrupt(&mut self) {
        self.ram[0xFF0F] |= 0x01;
        self.interrupt_flag_request |= 0x01;
      }

      pub fn request_lcd_interrupt(&mut self) {
        self.ram[0xFF0F] |= 0x02;
        self.interrupt_flag_request |= 0x02;
      }

      pub fn request_timer_interrupt(&mut self) {
        self.ram[0xFF0F] |= 0x04;
        self.interrupt_flag_request |= 0x04;
      }

      pub fn request_serial_interrupt(&mut self) {
        self.ram[0xFF0F] |= 0x08;
        self.interrupt_flag_request |= 0x08;
      }

      pub fn request_joypad_interrupt(&mut self) {
        self.ram[0xFF0F] |= 0x10;
        self.interrupt_flag_request |= 0x10;
      }

      pub fn get_lcdc(&self) -> u8 {
        self.ram[0xFF40]
      }

      pub fn get_scroll_y(&self) -> u8 {
        self.ram[0xFF42]
      }

      pub fn get_scroll_x(&self) -> u8 {
        self.ram[0xFF43]
      }
}