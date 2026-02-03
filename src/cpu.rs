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
  pub div_cycles: u32,
  pub timer_cycles: u32,
}

impl CPU {
    pub fn new() -> CPU {
        return CPU { pc: 0x0100, sp: 0xFFFE, registers: Registers8::new(), ram: [0; 0x10000], ime: 0, ie: 0, div_cycles: 0, timer_cycles: 0 }
    }

    pub fn gb_doctor_cpu() -> CPU {
      let mut gb_doctor_cpu = CPU { pc: 0x0100, sp: 0xFFFE, registers: Registers8::gb_doctor_values(), ram: [0; 0x10000], ime: 0, ie: 0, div_cycles: 0, timer_cycles: 0 };
      gb_doctor_cpu.ram[0xFF44] = 0x90;
      gb_doctor_cpu
    }

    pub fn pop(&mut self) -> u16 {
        let lsb = self.ram[self.sp as usize];
        let msb = self.ram[(self.sp + 1) as usize];
        self.sp += 2;
        return two_bytes_to_u16(lsb, msb);
    }

    pub fn push(&mut self, next_address: u16) {
        let lsb: u8 =  (next_address & 0xFF) as u8;
        let msb: u8 = ((next_address & 0xFF00) >> 8) as u8;
        self.ram[(self.sp - 1) as usize] = msb;
        self.ram[(self.sp - 2) as usize] = lsb;
        self.sp -= 2;
    }

    pub fn read_u16_at(&self, addr: u16) -> u16 {
        let lsb = self.ram[addr as usize];
        let msb = self.ram[(addr + 1) as usize];
        two_bytes_to_u16(lsb, msb)
    }

    pub fn read_i8_at(&self, addr: u16) -> i8 {
      self.ram[addr as usize] as i8
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
        self.push(self.pc + 2);
        // set the PC to be A16
        self.pc = next_address;
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
          return 20;
        }
        return 0;
      }
}