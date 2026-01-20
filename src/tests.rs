#[cfg(test)]
mod tests {
    use crate::CPU;
    use crate::registers::FlagsRegister;
    use crate::InstructionSet;
    use crate::execute_opcode;
    use serde::Deserialize;
    use std::fs;
    use std::fs::File;
    use std::io::BufReader;

    #[derive(Debug, Deserialize)]
    struct TestCase {
        name: String,
        initial: CpuState,
        #[serde(rename = "final")]
        final_state: CpuState,
    }

    #[derive(Debug, Deserialize)]
    struct CpuState {
        pc: u16,
        sp: u16,
        a: u8,
        b: u8,
        c: u8,
        d: u8,
        e: u8,
        f: u8,
        h: u8,
        l: u8,
        ime: u8,
        #[serde(default)]
        ie: u8,
        ram: Vec<[u16; 2]>,
    }

    fn setup_cpu(state: &CpuState) -> CPU {
        let mut cpu = CPU::new();
        cpu.pc = state.pc;
        cpu.sp = state.sp;
        cpu.registers.a = state.a;
        cpu.registers.b = state.b;
        cpu.registers.c = state.c;
        cpu.registers.d = state.d;
        cpu.registers.e = state.e;
        cpu.registers.f = state.f;
        cpu.registers.h = state.h;
        cpu.registers.l = state.l;
        cpu.flags_register = FlagsRegister::from(state.f);

        for ram_entry in &state.ram {
            let address = ram_entry[0] as usize;
            let value = ram_entry[1] as u8;
            cpu.ram[address] = value;
        }

        cpu
    }

    fn verify_cpu_state(cpu: &CPU, expected: &CpuState, test_name: &str) {
        assert_eq!(cpu.pc, expected.pc, "{}: PC mismatch", test_name);
        assert_eq!(cpu.sp, expected.sp, "{}: SP mismatch", test_name);
        assert_eq!(cpu.registers.a, expected.a, "{}: A register mismatch", test_name);
        assert_eq!(cpu.registers.b, expected.b, "{}: B register mismatch", test_name);
        assert_eq!(cpu.registers.c, expected.c, "{}: C register mismatch", test_name);
        assert_eq!(cpu.registers.d, expected.d, "{}: D register mismatch", test_name);
        assert_eq!(cpu.registers.e, expected.e, "{}: E register mismatch", test_name);
        assert_eq!(cpu.registers.f, expected.f, "{}: F register mismatch", test_name);
        assert_eq!(cpu.registers.h, expected.h, "{}: H register mismatch", test_name);
        assert_eq!(cpu.registers.l, expected.l, "{}: L register mismatch", test_name);

        for ram_entry in &expected.ram {
            let address = ram_entry[0] as usize;
            let expected_value = ram_entry[1] as u8;
            assert_eq!(
                cpu.ram[address], expected_value,
                "{}: RAM mismatch at address {:#06X}",
                test_name, address
            );
        }
    }

    #[test]
    fn test_nop_00() {
        let json_content = fs::read_to_string("fixtures/00.json")
            .expect("Failed to read fixtures/00.json");
        let test_cases: Vec<TestCase> = serde_json::from_str(&json_content)
            .expect("Failed to parse fixtures/00.json");

        let opcodes_json = fs::read_to_string("opcodes.json")
            .expect("Failed to parse opcodes.json");
        let instruction_set: InstructionSet = serde_json::from_str(&opcodes_json)
            .expect("Failed to parse opcodes.json");

        for test_case in test_cases {
            let mut cpu = setup_cpu(&test_case.initial);
            execute_opcode(&instruction_set, &mut cpu);
            verify_cpu_state(&cpu, &test_case.final_state, &test_case.name);
        }
    }

    #[test]
    fn test_jp_nz_c2() {
        let json_content = fs::read_to_string("fixtures/c2.json")
            .expect("Failed to read fixtures/c2.json");
        let test_cases: Vec<TestCase> = serde_json::from_str(&json_content)
            .expect("Failed to parse fixtures/c2.json");

        let opcodes_json = fs::read_to_string("opcodes.json")
            .expect("Failed to parse opcodes.json");
        let instruction_set: InstructionSet = serde_json::from_str(&opcodes_json)
            .expect("Failed to parse opcodes.json");

        for test_case in test_cases {
            let mut cpu = setup_cpu(&test_case.initial);
            execute_opcode(&instruction_set, &mut cpu);
            verify_cpu_state(&cpu, &test_case.final_state, &test_case.name);
        }
    }

    #[test]
    fn test_jp_c3() {
        let json_content = fs::read_to_string("fixtures/c3.json")
            .expect("Failed to read fixtures/c3.json");
        let test_cases: Vec<TestCase> = serde_json::from_str(&json_content)
            .expect("Failed to parse fixtures/c3.json");

        let opcodes_json = fs::read_to_string("opcodes.json")
            .expect("Failed to parse opcodes.json");
        let instruction_set: InstructionSet = serde_json::from_str(&opcodes_json)
            .expect("Failed to parse opcodes.json");

        for test_case in test_cases {
            let mut cpu = setup_cpu(&test_case.initial);
            execute_opcode(&instruction_set, &mut cpu);
            verify_cpu_state(&cpu, &test_case.final_state, &test_case.name);
        }
    }

    #[test]
    fn test_jp_z_ca() {
        let json_content = fs::read_to_string("fixtures/ca.json")
            .expect("Failed to read fixtures/ca.json");
        let test_cases: Vec<TestCase> = serde_json::from_str(&json_content)
            .expect("Failed to parse fixtures/ca.json");

        let opcodes_json = fs::read_to_string("opcodes.json")
            .expect("Failed to parse opcodes.json");
        let instruction_set: InstructionSet = serde_json::from_str(&opcodes_json)
            .expect("Failed to parse opcodes.json");

        for test_case in test_cases {
            let mut cpu = setup_cpu(&test_case.initial);
            execute_opcode(&instruction_set, &mut cpu);
            verify_cpu_state(&cpu, &test_case.final_state, &test_case.name);
        }
    }

    #[test]
    fn test_jp_nc_d2() {
        let json_content = fs::read_to_string("fixtures/d2.json")
            .expect("Failed to read fixtures/d2.json");
        let test_cases: Vec<TestCase> = serde_json::from_str(&json_content)
            .expect("Failed to parse fixtures/d2.json");

        let opcodes_json = fs::read_to_string("opcodes.json")
            .expect("Failed to parse opcodes.json");
        let instruction_set: InstructionSet = serde_json::from_str(&opcodes_json)
            .expect("Failed to parse opcodes.json");

        for test_case in test_cases {
            let mut cpu = setup_cpu(&test_case.initial);
            execute_opcode(&instruction_set, &mut cpu);
            verify_cpu_state(&cpu, &test_case.final_state, &test_case.name);
        }
    }

    #[test]
    fn test_jp_c_da() {
        let json_content = fs::read_to_string("fixtures/da.json")
            .expect("Failed to read fixtures/da.json");
        let test_cases: Vec<TestCase> = serde_json::from_str(&json_content)
            .expect("Failed to parse fixtures/da.json");

        let opcodes_json = fs::read_to_string("opcodes.json")
            .expect("Failed to parse opcodes.json");
        let instruction_set: InstructionSet = serde_json::from_str(&opcodes_json)
            .expect("Failed to parse opcodes.json");

        for test_case in test_cases {
            let mut cpu = setup_cpu(&test_case.initial);
            execute_opcode(&instruction_set, &mut cpu);
            verify_cpu_state(&cpu, &test_case.final_state, &test_case.name);
        }
    }

    #[test]
    fn test_jp_hl_e9() {
        let json_content = fs::read_to_string("fixtures/e9.json")
            .expect("Failed to read fixtures/e9.json");
        let test_cases: Vec<TestCase> = serde_json::from_str(&json_content)
            .expect("Failed to parse fixtures/e9.json");

        let opcodes_json = fs::read_to_string("opcodes.json")
            .expect("Failed to parse opcodes.json");
        let instruction_set: InstructionSet = serde_json::from_str(&opcodes_json)
            .expect("Failed to parse opcodes.json");

        for test_case in test_cases {
            let mut cpu = setup_cpu(&test_case.initial);
            execute_opcode(&instruction_set, &mut cpu);
            verify_cpu_state(&cpu, &test_case.final_state, &test_case.name);
        }
    }
}
