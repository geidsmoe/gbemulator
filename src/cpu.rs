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
}

impl CPU {
    pub fn new() -> CPU {
        return CPU { pc: 0x100, sp: 0xFFFF, registers: Registers8::new(), ram: [0; 0x10000], }
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

    pub fn add_hl_16bit(&mut self, reg: Registers16) {
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

      pub fn inc_r16(&mut self, reg: Registers16) {
        let inc = self.registers.get_16_bit_reg(reg).wrapping_add(1);
        self.registers.set_16_bit_reg(reg, inc);
      }

      pub fn dec_r16(&mut self, reg: Registers16) {
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

      pub fn rotate_left_with_carry(&mut self, value: u8) -> u8 {
        let mut flags_register = FlagsRegister::from(self.registers.f);
        let result = value.rotate_left(1);
        flags_register.zero = false;
        flags_register.carry = (value & 0x80) != 0;
        flags_register.half_carry = false;
        flags_register.subtract = false;
        self.registers.f = u8::from(flags_register);
        result
      }

      pub fn rotate_left_through_carry(&mut self, value: u8) -> u8 {
        let mut flags_register = FlagsRegister::from(self.registers.f);
        let cy = if flags_register.carry { 1 } else { 0 };
        let result = (value << 1) | cy;
        flags_register.zero = false;
        flags_register.carry = (value & 0x80) != 0;
        flags_register.half_carry = false;
        flags_register.subtract = false;
        self.registers.f = u8::from(flags_register);
        result
      }

      pub fn rotate_right_with_carry(&mut self, value: u8) -> u8 {
        let mut flags_register = FlagsRegister::from(self.registers.f);
        let result = value.rotate_right(1);
        flags_register.zero = false;
        flags_register.carry = (value & 0x01) != 0;
        flags_register.half_carry = false;
        flags_register.subtract = false;
        self.registers.f = u8::from(flags_register);
        result
      }

      pub fn rotate_right_through_carry(&mut self, value: u8) -> u8 {
        let mut flags_register = FlagsRegister::from(self.registers.f);
        let cy = if flags_register.carry { 1 } else { 0 };
        let result = (value >> 1) | (cy << 7);
        flags_register.zero = false;
        flags_register.carry = (value & 0x01) != 0;
        flags_register.half_carry = false;
        flags_register.subtract = false;
        self.registers.f = u8::from(flags_register);
        result
      }
}