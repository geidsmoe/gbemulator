use std::collections::HashMap;
use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Instruction {
    pub mnemonic: String,
    pub bytes: u16,
    pub cycles: Vec<u32>,
    pub operands: Vec<Operand>,
    pub immediate: bool,
    pub flags: Flags,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut operand_string = String::from("");
        for operand in self.operands.iter() {
            operand_string.push_str(&operand.name);
            operand_string.push(' ');
        }
        write!(f, "{} {}", self.mnemonic, operand_string)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Operand {
    pub name: String,
    #[serde(default)]
    pub bytes: Option<u8>,
    pub immediate: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Flags {
    pub Z: String,
    pub N: String,
    pub H: String,
    pub C: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstructionSet {
    pub unprefixed: HashMap<String, Instruction>,
    pub cbprefixed: HashMap<String, Instruction>
}