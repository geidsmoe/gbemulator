use crate::registers;

pub fn two_bytes_to_u16(lsb: u8, msb: u8) -> u16 {
    (((msb as u16) << 8) | (lsb as u16)).into()
}
  
pub struct CPU {
pub pc: u16,
pub sp: u16,
pub registers: registers::Registers,
pub ram: [u8; 0x10000],
}

impl CPU {
pub fn new() -> CPU {
    return CPU { pc: 0x100, sp: 0xFFFF, registers: registers::Registers::new(), ram: [0; 0x10000], }
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
}