#[cfg(test)]
mod tests {
    use crate::CPU;
    use crate::registers::FlagsRegister;
    use crate::instructions::InstructionSet;
    use crate::execute_opcode;
    use serde::Deserialize;
    use std::fs;
    use std::os::linux::raw::stat;

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
        cpu.ime = state.ime;
        cpu.ie = state.ie;
        cpu.registers.a = state.a;
        cpu.registers.b = state.b;
        cpu.registers.c = state.c;
        cpu.registers.d = state.d;
        cpu.registers.e = state.e;
        cpu.registers.f = state.f;
        cpu.registers.h = state.h;
        cpu.registers.l = state.l;

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
        assert_eq!(cpu.registers.f, expected.f, "{}: F register mismatch {:#04X} {:#04X}", test_name, cpu.registers.f, expected.f);
        assert_eq!(cpu.registers.h, expected.h, "{}: H register mismatch", test_name);
        assert_eq!(cpu.registers.l, expected.l, "{}: L register mismatch", test_name);

        for ram_entry in &expected.ram {
            let address = ram_entry[0] as usize;
            let expected_value = ram_entry[1] as u8;
            let actual_value = cpu.ram[address];
            assert_eq!(
                actual_value, expected_value,
                "{}: RAM mismatch at address {:#06X}",
                test_name, address
            );
        }
    }

    fn run_test_for_opcode(fixture: &str) {
        let json_content = fs::read_to_string(format!("fixtures/{}.json", fixture))
            .expect(&format!("Failed to read fixtures/{}.json", fixture));
        let test_cases: Vec<TestCase> = serde_json::from_str(&json_content)
            .expect(&format!("Failed to parse fixtures/{}.json", fixture));

        let opcodes_json = fs::read_to_string("opcodes.json")
            .expect("Failed to read opcodes.json");
        let instruction_set: InstructionSet = serde_json::from_str(&opcodes_json)
            .expect("Failed to parse opcodes.json");

        for test_case in test_cases {
            let mut cpu = setup_cpu(&test_case.initial);
            execute_opcode(&instruction_set, &mut cpu);
            verify_cpu_state(&cpu, &test_case.final_state, &test_case.name);
        }
    }

    // Macro for generating opcode tests
    macro_rules! opcode_test {
        ($name:ident, $fixture:expr) => {
            #[test]
            fn $name() {
                run_test_for_opcode($fixture);
            }
        };
        ($name:ident, $fixture:expr, ignore) => {
            #[test]
            #[ignore]
            fn $name() {
                run_test_for_opcode($fixture);
            }
        };
    }

    // ==================== IMPLEMENTED OPCODES ====================
    opcode_test!(test_00_nop, "00");
    opcode_test!(test_01_ld_bc_d16, "01");
    opcode_test!(test_02_ld_bc_a, "02");
    opcode_test!(test_12_ld_de_a, "12");
    opcode_test!(test_22_ld_hli_a, "22");
    opcode_test!(test_06_ld_b_d8, "06");
    opcode_test!(test_16_ld_d_d8, "16");
    opcode_test!(test_26_ld_h_d8, "26");
    opcode_test!(test_36_ld_hl_d8, "36");
    opcode_test!(test_c0_ret_nz, "c0");
    opcode_test!(test_c2_jp_nz_a16, "c2");
    opcode_test!(test_c3_jp_a16, "c3");
    opcode_test!(test_c4_call_nz_a16, "c4");
    opcode_test!(test_c8_ret_z, "c8");
    opcode_test!(test_c9_ret, "c9");
    opcode_test!(test_ca_jp_z_a16, "ca");
    opcode_test!(test_cc_call_z_a16, "cc");
    opcode_test!(test_cd_call_a16, "cd");
    opcode_test!(test_d0_ret_nc, "d0");
    opcode_test!(test_d2_jp_nc_a16, "d2");
    opcode_test!(test_d4_call_nc_a16, "d4");
    opcode_test!(test_d8_ret_c, "d8");
    opcode_test!(test_da_jp_c_a16, "da");
    opcode_test!(test_dc_call_c_a16, "dc");
    opcode_test!(test_e9_jp_hl, "e9");
    opcode_test!(test_08_ld_a16_sp, "08");
    opcode_test!(test_0a_ld_a_bc, "0a");
    opcode_test!(test_1a_ld_a_de, "1a");
    opcode_test!(test_2a_ld_a_hli, "2a");
    opcode_test!(test_3a_ld_a_hld, "3a");
    opcode_test!(test_11_ld_de_d16, "11");
    opcode_test!(test_21_ld_hl_d16, "21");
    opcode_test!(test_31_ld_sp_d16, "31");
    opcode_test!(test_0e_ld_c_d8, "0e");
    opcode_test!(test_1e_ld_e_d8, "1e");
    opcode_test!(test_2e_ld_l_d8, "2e");
    opcode_test!(test_3e_ld_a_d8, "3e");
    opcode_test!(test_32_ld_hld_a, "32");
    // 0x40 - 0x4F (LD B/C, r)
    opcode_test!(test_40_ld_b_b, "40");
    opcode_test!(test_41_ld_b_c, "41");
    opcode_test!(test_42_ld_b_d, "42");
    opcode_test!(test_43_ld_b_e, "43");
    opcode_test!(test_44_ld_b_h, "44");
    opcode_test!(test_45_ld_b_l, "45");
    opcode_test!(test_46_ld_b_hl, "46");
    opcode_test!(test_47_ld_b_a, "47");
    opcode_test!(test_48_ld_c_b, "48");
    opcode_test!(test_49_ld_c_c, "49");
    opcode_test!(test_4a_ld_c_d, "4a");
    opcode_test!(test_4b_ld_c_e, "4b");
    opcode_test!(test_4c_ld_c_h, "4c");
    opcode_test!(test_4d_ld_c_l, "4d");
    opcode_test!(test_4e_ld_c_hl, "4e");
    opcode_test!(test_4f_ld_c_a, "4f");
    // 0x50 - 0x5F (LD D/E, r)
    opcode_test!(test_50_ld_d_b, "50");
    opcode_test!(test_51_ld_d_c, "51");
    opcode_test!(test_52_ld_d_d, "52");
    opcode_test!(test_53_ld_d_e, "53");
    opcode_test!(test_54_ld_d_h, "54");
    opcode_test!(test_55_ld_d_l, "55");
    opcode_test!(test_56_ld_d_hl, "56");
    opcode_test!(test_57_ld_d_a, "57");
    opcode_test!(test_58_ld_e_b, "58");
    opcode_test!(test_59_ld_e_c, "59");
    opcode_test!(test_5a_ld_e_d, "5a");
    opcode_test!(test_5b_ld_e_e, "5b");
    opcode_test!(test_5c_ld_e_h, "5c");
    opcode_test!(test_5d_ld_e_l, "5d");
    opcode_test!(test_5e_ld_e_hl, "5e");
    opcode_test!(test_5f_ld_e_a, "5f");
    // 0x60 - 0x6F (LD H/L, r)
    opcode_test!(test_60_ld_h_b, "60");
    opcode_test!(test_61_ld_h_c, "61");
    opcode_test!(test_62_ld_h_d, "62");
    opcode_test!(test_63_ld_h_e, "63");
    opcode_test!(test_64_ld_h_h, "64");
    opcode_test!(test_65_ld_h_l, "65");
    opcode_test!(test_66_ld_h_hl, "66");
    opcode_test!(test_67_ld_h_a, "67");
    opcode_test!(test_68_ld_l_b, "68");
    opcode_test!(test_69_ld_l_c, "69");
    opcode_test!(test_6a_ld_l_d, "6a");
    opcode_test!(test_6b_ld_l_e, "6b");
    opcode_test!(test_6c_ld_l_h, "6c");
    opcode_test!(test_6d_ld_l_l, "6d");
    opcode_test!(test_6e_ld_l_hl, "6e");
    opcode_test!(test_6f_ld_l_a, "6f");
    // 0x70 - 0x7F (LD (HL)/A, r)
    opcode_test!(test_70_ld_hl_b, "70");
    opcode_test!(test_71_ld_hl_c, "71");
    opcode_test!(test_72_ld_hl_d, "72");
    opcode_test!(test_73_ld_hl_e, "73");
    opcode_test!(test_74_ld_hl_h, "74");
    opcode_test!(test_75_ld_hl_l, "75");
    opcode_test!(test_76_halt, "76");
    opcode_test!(test_77_ld_hl_a, "77");
    opcode_test!(test_78_ld_a_b, "78");
    opcode_test!(test_79_ld_a_c, "79");
    opcode_test!(test_7a_ld_a_d, "7a");
    opcode_test!(test_7b_ld_a_e, "7b");
    opcode_test!(test_7c_ld_a_h, "7c");
    opcode_test!(test_7d_ld_a_l, "7d");
    opcode_test!(test_7e_ld_a_hl, "7e");
    opcode_test!(test_7f_ld_a_a, "7f");
    // 0x03 - 0x0F
    opcode_test!(test_03_inc_bc, "03");
    opcode_test!(test_04_inc_b, "04");
    opcode_test!(test_05_dec_b, "05");
    opcode_test!(test_07_rlca, "07");
    opcode_test!(test_09_add_hl_bc, "09");
    opcode_test!(test_0b_dec_bc, "0b");
    opcode_test!(test_0c_inc_c, "0c");
    opcode_test!(test_0d_dec_c, "0d");
    opcode_test!(test_0f_rrca, "0f");
    // 0x10 - 0x1F
    opcode_test!(test_10_stop, "10", ignore);
    opcode_test!(test_13_inc_de, "13");
    opcode_test!(test_14_inc_d, "14");
    opcode_test!(test_15_dec_d, "15");
    opcode_test!(test_17_rla, "17");
    opcode_test!(test_18_jr_r8, "18");
    opcode_test!(test_19_add_hl_de, "19");
    opcode_test!(test_1b_dec_de, "1b");
    opcode_test!(test_1c_inc_e, "1c");
    opcode_test!(test_1d_dec_e, "1d");
    opcode_test!(test_1f_rra, "1f");

    // 0x20 - 0x2F
    opcode_test!(test_20_jr_nz_r8, "20");
    opcode_test!(test_23_inc_hl, "23");
    opcode_test!(test_24_inc_h, "24");
    opcode_test!(test_25_dec_h, "25");
    opcode_test!(test_27_daa, "27");
    opcode_test!(test_28_jr_z_r8, "28");
    opcode_test!(test_29_add_hl_hl, "29");
    opcode_test!(test_2b_dec_hl, "2b");
    opcode_test!(test_2c_inc_l, "2c");
    opcode_test!(test_2d_dec_l, "2d");
    opcode_test!(test_2f_cpl, "2f");

    // 0x30 - 0x3F
    opcode_test!(test_30_jr_nc_r8, "30");
    opcode_test!(test_33_inc_sp, "33");
    opcode_test!(test_34_inc_hl_ind, "34");
    opcode_test!(test_35_dec_hl_ind, "35");
    opcode_test!(test_37_scf, "37");
    opcode_test!(test_38_jr_c_r8, "38");
    opcode_test!(test_39_add_hl_sp, "39");
    opcode_test!(test_3b_dec_sp, "3b");
    opcode_test!(test_3c_inc_a, "3c");
    opcode_test!(test_3d_dec_a, "3d");
    opcode_test!(test_3f_ccf, "3f");

    // 0x80 - 0x8F (ADD/ADC A, r)
    opcode_test!(test_80_add_a_b, "80");
    opcode_test!(test_81_add_a_c, "81");
    opcode_test!(test_82_add_a_d, "82");
    opcode_test!(test_83_add_a_e, "83");
    opcode_test!(test_84_add_a_h, "84");
    opcode_test!(test_85_add_a_l, "85");
    opcode_test!(test_86_add_a_hl, "86");
    opcode_test!(test_87_add_a_a, "87");
    opcode_test!(test_88_adc_a_b, "88");
    opcode_test!(test_89_adc_a_c, "89");
    opcode_test!(test_8a_adc_a_d, "8a");
    opcode_test!(test_8b_adc_a_e, "8b");
    opcode_test!(test_8c_adc_a_h, "8c");
    opcode_test!(test_8d_adc_a_l, "8d");
    opcode_test!(test_8e_adc_a_hl, "8e");
    opcode_test!(test_8f_adc_a_a, "8f");

    // 0x90 - 0x9F (SUB/SBC A, r)
    opcode_test!(test_90_sub_b, "90");
    opcode_test!(test_91_sub_c, "91");
    opcode_test!(test_92_sub_d, "92");
    opcode_test!(test_93_sub_e, "93");
    opcode_test!(test_94_sub_h, "94");
    opcode_test!(test_95_sub_l, "95");
    opcode_test!(test_96_sub_hl, "96");
    opcode_test!(test_97_sub_a, "97");
    opcode_test!(test_98_sbc_a_b, "98");
    opcode_test!(test_99_sbc_a_c, "99");
    opcode_test!(test_9a_sbc_a_d, "9a");
    opcode_test!(test_9b_sbc_a_e, "9b");
    opcode_test!(test_9c_sbc_a_h, "9c");
    opcode_test!(test_9d_sbc_a_l, "9d");
    opcode_test!(test_9e_sbc_a_hl, "9e");
    opcode_test!(test_9f_sbc_a_a, "9f");

    // 0xA0 - 0xAF (AND/XOR A, r)
    opcode_test!(test_a0_and_b, "a0");
    opcode_test!(test_a1_and_c, "a1");
    opcode_test!(test_a2_and_d, "a2");
    opcode_test!(test_a3_and_e, "a3");
    opcode_test!(test_a4_and_h, "a4");
    opcode_test!(test_a5_and_l, "a5");
    opcode_test!(test_a6_and_hl, "a6");
    opcode_test!(test_a7_and_a, "a7");
    opcode_test!(test_a8_xor_b, "a8");
    opcode_test!(test_a9_xor_c, "a9");
    opcode_test!(test_aa_xor_d, "aa");
    opcode_test!(test_ab_xor_e, "ab");
    opcode_test!(test_ac_xor_h, "ac");
    opcode_test!(test_ad_xor_l, "ad");
    opcode_test!(test_ae_xor_hl, "ae");
    opcode_test!(test_af_xor_a, "af");

    // 0xB0 - 0xBF (OR/CP A, r)
    opcode_test!(test_b0_or_b, "b0");
    opcode_test!(test_b1_or_c, "b1");
    opcode_test!(test_b2_or_d, "b2");
    opcode_test!(test_b3_or_e, "b3");
    opcode_test!(test_b4_or_h, "b4");
    opcode_test!(test_b5_or_l, "b5");
    opcode_test!(test_b6_or_hl, "b6");
    opcode_test!(test_b7_or_a, "b7");
    opcode_test!(test_b8_cp_b, "b8");
    opcode_test!(test_b9_cp_c, "b9");
    opcode_test!(test_ba_cp_d, "ba");
    opcode_test!(test_bb_cp_e, "bb");
    opcode_test!(test_bc_cp_h, "bc");
    opcode_test!(test_bd_cp_l, "bd");
    opcode_test!(test_be_cp_hl, "be");
    opcode_test!(test_bf_cp_a, "bf");

    // 0xC1, 0xC5-0xC7, 0xCE-0xCF
    opcode_test!(test_c1_pop_bc, "c1");
    opcode_test!(test_c5_push_bc, "c5");
    opcode_test!(test_c6_add_a_d8, "c6");
    opcode_test!(test_c7_rst_00h, "c7");
    opcode_test!(test_ce_adc_a_d8, "ce");
    opcode_test!(test_cf_rst_08h, "cf");

    // 0xD1, 0xD5-0xD7, 0xD9, 0xDE-0xDF
    opcode_test!(test_d1_pop_de, "d1");
    opcode_test!(test_d5_push_de, "d5");
    opcode_test!(test_d6_sub_d8, "d6");
    opcode_test!(test_d7_rst_10h, "d7");
    opcode_test!(test_d9_reti, "d9", ignore);
    opcode_test!(test_de_sbc_a_d8, "de");
    opcode_test!(test_df_rst_18h, "df");

    // 0xE0-0xE2, 0xE5-0xE8, 0xEA, 0xEE-0xEF
    opcode_test!(test_e0_ldh_a8_a, "e0");
    opcode_test!(test_e1_pop_hl, "e1");
    opcode_test!(test_e2_ld_c_a, "e2");
    opcode_test!(test_e5_push_hl, "e5");
    opcode_test!(test_e6_and_d8, "e6");
    opcode_test!(test_e7_rst_20h, "e7");
    opcode_test!(test_e8_add_sp_r8, "e8");
    opcode_test!(test_ea_ld_a16_a, "ea");
    opcode_test!(test_ee_xor_d8, "ee");
    opcode_test!(test_ef_rst_28h, "ef");

    // 0xF0-0xF3, 0xF5-0xFB, 0xFE-0xFF
    opcode_test!(test_f0_ldh_a_a8, "f0");
    opcode_test!(test_f1_pop_af, "f1");
    opcode_test!(test_f2_ld_a_c, "f2");
    opcode_test!(test_f3_di, "f3", ignore);
    opcode_test!(test_f5_push_af, "f5");
    opcode_test!(test_f6_or_d8, "f6");
    opcode_test!(test_f7_rst_30h, "f7");
    opcode_test!(test_f8_ld_hl_sp_r8, "f8");
    opcode_test!(test_f9_ld_sp_hl, "f9");
    opcode_test!(test_fa_ld_a_a16, "fa");
    opcode_test!(test_fb_ei, "fb", ignore);
    opcode_test!(test_fe_cp_d8, "fe");
    opcode_test!(test_ff_rst_38h, "ff");

    // ==================== CB PREFIXED OPCODES ====================
    // 0x00-0x07: RLC r (rotate left with carry)
    opcode_test!(test_cb_00_rlc_b, "cb 00");
    opcode_test!(test_cb_01_rlc_c, "cb 01");
    opcode_test!(test_cb_02_rlc_d, "cb 02");
    opcode_test!(test_cb_03_rlc_e, "cb 03");
    opcode_test!(test_cb_04_rlc_h, "cb 04");
    opcode_test!(test_cb_05_rlc_l, "cb 05");
    opcode_test!(test_cb_06_rlc_hl, "cb 06");
    opcode_test!(test_cb_07_rlc_a, "cb 07");

    // 0x08-0x0F: RRC r (rotate right with carry)
    opcode_test!(test_cb_08_rrc_b, "cb 08");
    opcode_test!(test_cb_09_rrc_c, "cb 09");
    opcode_test!(test_cb_0a_rrc_d, "cb 0a");
    opcode_test!(test_cb_0b_rrc_e, "cb 0b");
    opcode_test!(test_cb_0c_rrc_h, "cb 0c");
    opcode_test!(test_cb_0d_rrc_l, "cb 0d");
    opcode_test!(test_cb_0e_rrc_hl, "cb 0e");
    opcode_test!(test_cb_0f_rrc_a, "cb 0f");

    // 0x10-0x17: RL r (rotate left through carry)
    opcode_test!(test_cb_10_rl_b, "cb 10");
    opcode_test!(test_cb_11_rl_c, "cb 11");
    opcode_test!(test_cb_12_rl_d, "cb 12");
    opcode_test!(test_cb_13_rl_e, "cb 13");
    opcode_test!(test_cb_14_rl_h, "cb 14");
    opcode_test!(test_cb_15_rl_l, "cb 15");
    opcode_test!(test_cb_16_rl_hl, "cb 16");
    opcode_test!(test_cb_17_rl_a, "cb 17");

    // 0x18-0x1F: RR r (rotate right through carry)
    opcode_test!(test_cb_18_rr_b, "cb 18");
    opcode_test!(test_cb_19_rr_c, "cb 19");
    opcode_test!(test_cb_1a_rr_d, "cb 1a");
    opcode_test!(test_cb_1b_rr_e, "cb 1b");
    opcode_test!(test_cb_1c_rr_h, "cb 1c");
    opcode_test!(test_cb_1d_rr_l, "cb 1d");
    opcode_test!(test_cb_1e_rr_hl, "cb 1e");
    opcode_test!(test_cb_1f_rr_a, "cb 1f");

    // 0x20-0x27: SLA r (shift left arithmetic)
    opcode_test!(test_cb_20_sla_b, "cb 20");
    opcode_test!(test_cb_21_sla_c, "cb 21");
    opcode_test!(test_cb_22_sla_d, "cb 22");
    opcode_test!(test_cb_23_sla_e, "cb 23");
    opcode_test!(test_cb_24_sla_h, "cb 24");
    opcode_test!(test_cb_25_sla_l, "cb 25");
    opcode_test!(test_cb_26_sla_hl, "cb 26");
    opcode_test!(test_cb_27_sla_a, "cb 27");

    // 0x28-0x2F: SRA r (shift right arithmetic)
    opcode_test!(test_cb_28_sra_b, "cb 28");
    opcode_test!(test_cb_29_sra_c, "cb 29");
    opcode_test!(test_cb_2a_sra_d, "cb 2a");
    opcode_test!(test_cb_2b_sra_e, "cb 2b");
    opcode_test!(test_cb_2c_sra_h, "cb 2c");
    opcode_test!(test_cb_2d_sra_l, "cb 2d");
    opcode_test!(test_cb_2e_sra_hl, "cb 2e");
    opcode_test!(test_cb_2f_sra_a, "cb 2f");

    // 0x30-0x37: SWAP r (swap nibbles)
    opcode_test!(test_cb_30_swap_b, "cb 30");
    opcode_test!(test_cb_31_swap_c, "cb 31");
    opcode_test!(test_cb_32_swap_d, "cb 32");
    opcode_test!(test_cb_33_swap_e, "cb 33");
    opcode_test!(test_cb_34_swap_h, "cb 34");
    opcode_test!(test_cb_35_swap_l, "cb 35");
    opcode_test!(test_cb_36_swap_hl, "cb 36");
    opcode_test!(test_cb_37_swap_a, "cb 37");

    // 0x38-0x3F: SRL r (shift right logical)
    opcode_test!(test_cb_38_srl_b, "cb 38");
    opcode_test!(test_cb_39_srl_c, "cb 39");
    opcode_test!(test_cb_3a_srl_d, "cb 3a");
    opcode_test!(test_cb_3b_srl_e, "cb 3b");
    opcode_test!(test_cb_3c_srl_h, "cb 3c");
    opcode_test!(test_cb_3d_srl_l, "cb 3d");
    opcode_test!(test_cb_3e_srl_hl, "cb 3e");
    opcode_test!(test_cb_3f_srl_a, "cb 3f");

    // 0x40-0x47: BIT 0,r
    opcode_test!(test_cb_40_bit_0_b, "cb 40");
    opcode_test!(test_cb_41_bit_0_c, "cb 41");
    opcode_test!(test_cb_42_bit_0_d, "cb 42");
    opcode_test!(test_cb_43_bit_0_e, "cb 43");
    opcode_test!(test_cb_44_bit_0_h, "cb 44");
    opcode_test!(test_cb_45_bit_0_l, "cb 45");
    opcode_test!(test_cb_46_bit_0_hl, "cb 46");
    opcode_test!(test_cb_47_bit_0_a, "cb 47");

    // 0x48-0x4F: BIT 1,r
    opcode_test!(test_cb_48_bit_1_b, "cb 48");
    opcode_test!(test_cb_49_bit_1_c, "cb 49");
    opcode_test!(test_cb_4a_bit_1_d, "cb 4a");
    opcode_test!(test_cb_4b_bit_1_e, "cb 4b");
    opcode_test!(test_cb_4c_bit_1_h, "cb 4c");
    opcode_test!(test_cb_4d_bit_1_l, "cb 4d");
    opcode_test!(test_cb_4e_bit_1_hl, "cb 4e");
    opcode_test!(test_cb_4f_bit_1_a, "cb 4f");

    // 0x50-0x57: BIT 2,r
    opcode_test!(test_cb_50_bit_2_b, "cb 50");
    opcode_test!(test_cb_51_bit_2_c, "cb 51");
    opcode_test!(test_cb_52_bit_2_d, "cb 52");
    opcode_test!(test_cb_53_bit_2_e, "cb 53");
    opcode_test!(test_cb_54_bit_2_h, "cb 54");
    opcode_test!(test_cb_55_bit_2_l, "cb 55");
    opcode_test!(test_cb_56_bit_2_hl, "cb 56");
    opcode_test!(test_cb_57_bit_2_a, "cb 57");

    // 0x58-0x5F: BIT 3,r
    opcode_test!(test_cb_58_bit_3_b, "cb 58");
    opcode_test!(test_cb_59_bit_3_c, "cb 59");
    opcode_test!(test_cb_5a_bit_3_d, "cb 5a");
    opcode_test!(test_cb_5b_bit_3_e, "cb 5b");
    opcode_test!(test_cb_5c_bit_3_h, "cb 5c");
    opcode_test!(test_cb_5d_bit_3_l, "cb 5d");
    opcode_test!(test_cb_5e_bit_3_hl, "cb 5e");
    opcode_test!(test_cb_5f_bit_3_a, "cb 5f");

    // 0x60-0x67: BIT 4,r
    opcode_test!(test_cb_60_bit_4_b, "cb 60");
    opcode_test!(test_cb_61_bit_4_c, "cb 61");
    opcode_test!(test_cb_62_bit_4_d, "cb 62");
    opcode_test!(test_cb_63_bit_4_e, "cb 63");
    opcode_test!(test_cb_64_bit_4_h, "cb 64");
    opcode_test!(test_cb_65_bit_4_l, "cb 65");
    opcode_test!(test_cb_66_bit_4_hl, "cb 66");
    opcode_test!(test_cb_67_bit_4_a, "cb 67");

    // 0x68-0x6F: BIT 5,r
    opcode_test!(test_cb_68_bit_5_b, "cb 68");
    opcode_test!(test_cb_69_bit_5_c, "cb 69");
    opcode_test!(test_cb_6a_bit_5_d, "cb 6a");
    opcode_test!(test_cb_6b_bit_5_e, "cb 6b");
    opcode_test!(test_cb_6c_bit_5_h, "cb 6c");
    opcode_test!(test_cb_6d_bit_5_l, "cb 6d");
    opcode_test!(test_cb_6e_bit_5_hl, "cb 6e");
    opcode_test!(test_cb_6f_bit_5_a, "cb 6f");

    // 0x70-0x77: BIT 6,r
    opcode_test!(test_cb_70_bit_6_b, "cb 70");
    opcode_test!(test_cb_71_bit_6_c, "cb 71");
    opcode_test!(test_cb_72_bit_6_d, "cb 72");
    opcode_test!(test_cb_73_bit_6_e, "cb 73");
    opcode_test!(test_cb_74_bit_6_h, "cb 74");
    opcode_test!(test_cb_75_bit_6_l, "cb 75");
    opcode_test!(test_cb_76_bit_6_hl, "cb 76");
    opcode_test!(test_cb_77_bit_6_a, "cb 77");

    // 0x78-0x7F: BIT 7,r
    opcode_test!(test_cb_78_bit_7_b, "cb 78");
    opcode_test!(test_cb_79_bit_7_c, "cb 79");
    opcode_test!(test_cb_7a_bit_7_d, "cb 7a");
    opcode_test!(test_cb_7b_bit_7_e, "cb 7b");
    opcode_test!(test_cb_7c_bit_7_h, "cb 7c");
    opcode_test!(test_cb_7d_bit_7_l, "cb 7d");
    opcode_test!(test_cb_7e_bit_7_hl, "cb 7e");
    opcode_test!(test_cb_7f_bit_7_a, "cb 7f");

    // 0x80-0x87: RES 0,r
    opcode_test!(test_cb_80_res_0_b, "cb 80");
    opcode_test!(test_cb_81_res_0_c, "cb 81");
    opcode_test!(test_cb_82_res_0_d, "cb 82");
    opcode_test!(test_cb_83_res_0_e, "cb 83");
    opcode_test!(test_cb_84_res_0_h, "cb 84");
    opcode_test!(test_cb_85_res_0_l, "cb 85");
    opcode_test!(test_cb_86_res_0_hl, "cb 86");
    opcode_test!(test_cb_87_res_0_a, "cb 87");

    // 0x88-0x8F: RES 1,r
    opcode_test!(test_cb_88_res_1_b, "cb 88");
    opcode_test!(test_cb_89_res_1_c, "cb 89");
    opcode_test!(test_cb_8a_res_1_d, "cb 8a");
    opcode_test!(test_cb_8b_res_1_e, "cb 8b");
    opcode_test!(test_cb_8c_res_1_h, "cb 8c");
    opcode_test!(test_cb_8d_res_1_l, "cb 8d");
    opcode_test!(test_cb_8e_res_1_hl, "cb 8e");
    opcode_test!(test_cb_8f_res_1_a, "cb 8f");

    // 0x90-0x97: RES 2,r
    opcode_test!(test_cb_90_res_2_b, "cb 90");
    opcode_test!(test_cb_91_res_2_c, "cb 91");
    opcode_test!(test_cb_92_res_2_d, "cb 92");
    opcode_test!(test_cb_93_res_2_e, "cb 93");
    opcode_test!(test_cb_94_res_2_h, "cb 94");
    opcode_test!(test_cb_95_res_2_l, "cb 95");
    opcode_test!(test_cb_96_res_2_hl, "cb 96");
    opcode_test!(test_cb_97_res_2_a, "cb 97");

    // 0x98-0x9F: RES 3,r
    opcode_test!(test_cb_98_res_3_b, "cb 98");
    opcode_test!(test_cb_99_res_3_c, "cb 99");
    opcode_test!(test_cb_9a_res_3_d, "cb 9a");
    opcode_test!(test_cb_9b_res_3_e, "cb 9b");
    opcode_test!(test_cb_9c_res_3_h, "cb 9c");
    opcode_test!(test_cb_9d_res_3_l, "cb 9d");
    opcode_test!(test_cb_9e_res_3_hl, "cb 9e");
    opcode_test!(test_cb_9f_res_3_a, "cb 9f");

    // 0xA0-0xA7: RES 4,r
    opcode_test!(test_cb_a0_res_4_b, "cb a0");
    opcode_test!(test_cb_a1_res_4_c, "cb a1");
    opcode_test!(test_cb_a2_res_4_d, "cb a2");
    opcode_test!(test_cb_a3_res_4_e, "cb a3");
    opcode_test!(test_cb_a4_res_4_h, "cb a4");
    opcode_test!(test_cb_a5_res_4_l, "cb a5");
    opcode_test!(test_cb_a6_res_4_hl, "cb a6");
    opcode_test!(test_cb_a7_res_4_a, "cb a7");

    // 0xA8-0xAF: RES 5,r
    opcode_test!(test_cb_a8_res_5_b, "cb a8");
    opcode_test!(test_cb_a9_res_5_c, "cb a9");
    opcode_test!(test_cb_aa_res_5_d, "cb aa");
    opcode_test!(test_cb_ab_res_5_e, "cb ab");
    opcode_test!(test_cb_ac_res_5_h, "cb ac");
    opcode_test!(test_cb_ad_res_5_l, "cb ad");
    opcode_test!(test_cb_ae_res_5_hl, "cb ae");
    opcode_test!(test_cb_af_res_5_a, "cb af");

    // 0xB0-0xB7: RES 6,r
    opcode_test!(test_cb_b0_res_6_b, "cb b0");
    opcode_test!(test_cb_b1_res_6_c, "cb b1");
    opcode_test!(test_cb_b2_res_6_d, "cb b2");
    opcode_test!(test_cb_b3_res_6_e, "cb b3");
    opcode_test!(test_cb_b4_res_6_h, "cb b4");
    opcode_test!(test_cb_b5_res_6_l, "cb b5");
    opcode_test!(test_cb_b6_res_6_hl, "cb b6");
    opcode_test!(test_cb_b7_res_6_a, "cb b7");

    // 0xB8-0xBF: RES 7,r
    opcode_test!(test_cb_b8_res_7_b, "cb b8");
    opcode_test!(test_cb_b9_res_7_c, "cb b9");
    opcode_test!(test_cb_ba_res_7_d, "cb ba");
    opcode_test!(test_cb_bb_res_7_e, "cb bb");
    opcode_test!(test_cb_bc_res_7_h, "cb bc");
    opcode_test!(test_cb_bd_res_7_l, "cb bd");
    opcode_test!(test_cb_be_res_7_hl, "cb be");
    opcode_test!(test_cb_bf_res_7_a, "cb bf");

    // 0xC0-0xC7: SET 0,r
    opcode_test!(test_cb_c0_set_0_b, "cb c0", ignore);
    opcode_test!(test_cb_c1_set_0_c, "cb c1", ignore);
    opcode_test!(test_cb_c2_set_0_d, "cb c2", ignore);
    opcode_test!(test_cb_c3_set_0_e, "cb c3", ignore);
    opcode_test!(test_cb_c4_set_0_h, "cb c4", ignore);
    opcode_test!(test_cb_c5_set_0_l, "cb c5", ignore);
    opcode_test!(test_cb_c6_set_0_hl, "cb c6", ignore);
    opcode_test!(test_cb_c7_set_0_a, "cb c7", ignore);

    // 0xC8-0xCF: SET 1,r
    opcode_test!(test_cb_c8_set_1_b, "cb c8", ignore);
    opcode_test!(test_cb_c9_set_1_c, "cb c9", ignore);
    opcode_test!(test_cb_ca_set_1_d, "cb ca", ignore);
    opcode_test!(test_cb_cb_set_1_e, "cb cb", ignore);
    opcode_test!(test_cb_cc_set_1_h, "cb cc", ignore);
    opcode_test!(test_cb_cd_set_1_l, "cb cd", ignore);
    opcode_test!(test_cb_ce_set_1_hl, "cb ce", ignore);
    opcode_test!(test_cb_cf_set_1_a, "cb cf", ignore);

    // 0xD0-0xD7: SET 2,r
    opcode_test!(test_cb_d0_set_2_b, "cb d0", ignore);
    opcode_test!(test_cb_d1_set_2_c, "cb d1", ignore);
    opcode_test!(test_cb_d2_set_2_d, "cb d2", ignore);
    opcode_test!(test_cb_d3_set_2_e, "cb d3", ignore);
    opcode_test!(test_cb_d4_set_2_h, "cb d4", ignore);
    opcode_test!(test_cb_d5_set_2_l, "cb d5", ignore);
    opcode_test!(test_cb_d6_set_2_hl, "cb d6", ignore);
    opcode_test!(test_cb_d7_set_2_a, "cb d7", ignore);

    // 0xD8-0xDF: SET 3,r
    opcode_test!(test_cb_d8_set_3_b, "cb d8", ignore);
    opcode_test!(test_cb_d9_set_3_c, "cb d9", ignore);
    opcode_test!(test_cb_da_set_3_d, "cb da", ignore);
    opcode_test!(test_cb_db_set_3_e, "cb db", ignore);
    opcode_test!(test_cb_dc_set_3_h, "cb dc", ignore);
    opcode_test!(test_cb_dd_set_3_l, "cb dd", ignore);
    opcode_test!(test_cb_de_set_3_hl, "cb de", ignore);
    opcode_test!(test_cb_df_set_3_a, "cb df", ignore);

    // 0xE0-0xE7: SET 4,r
    opcode_test!(test_cb_e0_set_4_b, "cb e0", ignore);
    opcode_test!(test_cb_e1_set_4_c, "cb e1", ignore);
    opcode_test!(test_cb_e2_set_4_d, "cb e2", ignore);
    opcode_test!(test_cb_e3_set_4_e, "cb e3", ignore);
    opcode_test!(test_cb_e4_set_4_h, "cb e4", ignore);
    opcode_test!(test_cb_e5_set_4_l, "cb e5", ignore);
    opcode_test!(test_cb_e6_set_4_hl, "cb e6", ignore);
    opcode_test!(test_cb_e7_set_4_a, "cb e7", ignore);

    // 0xE8-0xEF: SET 5,r
    opcode_test!(test_cb_e8_set_5_b, "cb e8", ignore);
    opcode_test!(test_cb_e9_set_5_c, "cb e9", ignore);
    opcode_test!(test_cb_ea_set_5_d, "cb ea", ignore);
    opcode_test!(test_cb_eb_set_5_e, "cb eb", ignore);
    opcode_test!(test_cb_ec_set_5_h, "cb ec", ignore);
    opcode_test!(test_cb_ed_set_5_l, "cb ed", ignore);
    opcode_test!(test_cb_ee_set_5_hl, "cb ee", ignore);
    opcode_test!(test_cb_ef_set_5_a, "cb ef", ignore);

    // 0xF0-0xF7: SET 6,r
    opcode_test!(test_cb_f0_set_6_b, "cb f0", ignore);
    opcode_test!(test_cb_f1_set_6_c, "cb f1", ignore);
    opcode_test!(test_cb_f2_set_6_d, "cb f2", ignore);
    opcode_test!(test_cb_f3_set_6_e, "cb f3", ignore);
    opcode_test!(test_cb_f4_set_6_h, "cb f4", ignore);
    opcode_test!(test_cb_f5_set_6_l, "cb f5", ignore);
    opcode_test!(test_cb_f6_set_6_hl, "cb f6", ignore);
    opcode_test!(test_cb_f7_set_6_a, "cb f7", ignore);

    // 0xF8-0xFF: SET 7,r
    opcode_test!(test_cb_f8_set_7_b, "cb f8", ignore);
    opcode_test!(test_cb_f9_set_7_c, "cb f9", ignore);
    opcode_test!(test_cb_fa_set_7_d, "cb fa", ignore);
    opcode_test!(test_cb_fb_set_7_e, "cb fb", ignore);
    opcode_test!(test_cb_fc_set_7_h, "cb fc", ignore);
    opcode_test!(test_cb_fd_set_7_l, "cb fd", ignore);
    opcode_test!(test_cb_fe_set_7_hl, "cb fe", ignore);
    opcode_test!(test_cb_ff_set_7_a, "cb ff", ignore);
}
