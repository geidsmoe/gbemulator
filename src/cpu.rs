use crate::registers::*;

pub fn two_bytes_to_u16(lsb: u8, msb: u8) -> u16 {
    (((msb as u16) << 8) | (lsb as u16)).into()
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

    pub fn read_u16_at_pc(&self) -> u16 {
        self.read_u16_at(self.pc)
    }

    pub fn add_8bit(&mut self, value: u8) {
        let mut flags_register = FlagsRegister::from(self.registers.f);
        let (result, carry) = self.registers.a.overflowing_add(value);
        let half_carry = (self.registers.a & 0x0f).checked_add(value | 0xf0).is_none();
        flags_register.carry = carry;
        flags_register.zero = result == 0;
        flags_register.half_carry = half_carry;
        flags_register.subtract = false;
        self.registers.f = u8::from(flags_register);
        self.registers.a = result;
    }

    pub fn addc_8bit(&mut self, value: u8) {
        let mut flags_register = FlagsRegister::from(self.registers.f);
        let carry_value: u8 = if flags_register.carry { 1 } else { 0 };
        let result = self.registers.a.wrapping_add(value).wrapping_add(carry_value);
        flags_register.zero = result == 0;
        flags_register.subtract = false;
        flags_register.half_carry = (self.registers.a & 0xf) + (value & 0xf) + carry_value > 0xf;
        flags_register.carry = self.registers.a as u16 + value as u16 + carry_value as u16 > 0xff;
        self.registers.f = u8::from(flags_register);
        self.registers.a = result;
    }

    pub fn sub(&mut self, value: u8, carry: bool) -> u8 {
        let mut flags_register = FlagsRegister::from(self.registers.f);
        let cy = if carry && flags_register.carry { 1 } else { 0 };
        let result = self.registers.a.wrapping_sub(value).wrapping_sub(cy);
        flags_register.zero = result == 0;
        flags_register.subtract = true;
        flags_register.half_carry =
          (self.registers.a & 0xf)
            .wrapping_sub(value & 0xf)
            .wrapping_sub(cy)
            & (0xf + 1) != 0;
        flags_register.carry = (self.registers.a as u16) < (value as u16) + (cy as u16);
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
        flags_register.half_carry = (value & 0x0f) == 0x0f;
        // carry flag not affected
        self.registers.f = u8::from(flags_register);
        result
      }

      pub fn dec_8bit(&mut self, value: u8) -> u8 {
        let mut flags_register = FlagsRegister::from(self.registers.f);
        let result = value.wrapping_sub(1);
        flags_register.zero = result == 0;
        flags_register.subtract = true;
        flags_register.half_carry = (value & 0x0f) == 0x00;
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
}