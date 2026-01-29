use std::collections::HashMap;
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