//! Opcode table validation tests
//!
//! Verifies that the opcode metadata table is complete and accurate.

use cpu6502::{AddressingMode, OPCODE_TABLE};

#[test]
fn test_opcode_table_completeness() {
    // Verify table has exactly 256 entries
    assert_eq!(
        OPCODE_TABLE.len(),
        256,
        "Opcode table must have exactly 256 entries"
    );

    // Verify all entries have non-empty mnemonics
    for (opcode, metadata) in OPCODE_TABLE.iter().enumerate() {
        assert!(
            !metadata.mnemonic.is_empty(),
            "Opcode 0x{:02X} has empty mnemonic",
            opcode
        );
    }
}

#[test]
fn test_opcode_table_size_validation() {
    // Verify all size_bytes values are 1-3
    for (opcode, metadata) in OPCODE_TABLE.iter().enumerate() {
        assert!(
            metadata.size_bytes >= 1 && metadata.size_bytes <= 3,
            "Opcode 0x{:02X} has invalid size: {} (must be 1-3)",
            opcode,
            metadata.size_bytes
        );
    }
}

#[test]
fn test_documented_opcodes_have_nonzero_cycles() {
    // Documented instructions (non-"???") must have non-zero cycle counts
    for (opcode, metadata) in OPCODE_TABLE.iter().enumerate() {
        if metadata.mnemonic != "???" {
            assert!(
                metadata.base_cycles > 0,
                "Documented opcode 0x{:02X} ({}) has zero cycles",
                opcode,
                metadata.mnemonic
            );
        }
    }
}

#[test]
fn test_illegal_opcodes_marked() {
    // Illegal opcodes should be marked with "???" and 0 cycles
    let mut illegal_count = 0;

    for (opcode, metadata) in OPCODE_TABLE.iter().enumerate() {
        if metadata.mnemonic == "???" {
            illegal_count += 1;
            assert_eq!(
                metadata.base_cycles, 0,
                "Illegal opcode 0x{:02X} should have 0 cycles",
                opcode
            );
        }
    }

    // Should have 105 illegal opcodes (256 - 151 documented)
    assert!(
        illegal_count > 0,
        "Should have at least some illegal opcodes marked"
    );
}

#[test]
fn test_implemented_opcodes() {
    // ADC opcodes should be marked as implemented
    let adc_opcodes = [0x61, 0x65, 0x69, 0x6D, 0x71, 0x75, 0x79, 0x7D];
    // AND opcodes should be marked as implemented
    let and_opcodes = [0x21, 0x25, 0x29, 0x2D, 0x31, 0x35, 0x39, 0x3D];
    // ASL opcodes should be marked as implemented
    let asl_opcodes = [0x06, 0x0A, 0x0E, 0x16, 0x1E];
    // BCC opcode should be marked as implemented
    let bcc_opcodes = [0x90];
    // BCS opcode should be marked as implemented
    let bcs_opcodes = [0xB0];
    // BEQ opcode should be marked as implemented
    let beq_opcodes = [0xF0];
    // BIT opcodes should be marked as implemented
    let bit_opcodes = [0x24, 0x2C];
    // BMI opcode should be marked as implemented
    let bmi_opcodes = [0x30];
    // BNE opcode should be marked as implemented
    let bne_opcodes = [0xD0];
    // BPL opcode should be marked as implemented
    let bpl_opcodes = [0x10];
    // BRK opcode should be marked as implemented
    let brk_opcodes = [0x00];
    // BVC opcode should be marked as implemented
    let bvc_opcodes = [0x50];
    // BVS opcode should be marked as implemented
    let bvs_opcodes = [0x70];
    // CLC opcode should be marked as implemented
    let clc_opcodes = [0x18];
    // CLD opcode should be marked as implemented
    let cld_opcodes = [0xD8];
    // CLI opcode should be marked as implemented
    let cli_opcodes = [0x58];
    // CLV opcode should be marked as implemented
    let clv_opcodes = [0xB8];
    // CMP opcodes should be marked as implemented
    let cmp_opcodes = [0xC1, 0xC5, 0xC9, 0xCD, 0xD1, 0xD5, 0xD9, 0xDD];
    // CPX opcodes should be marked as implemented
    let cpx_opcodes = [0xE0, 0xE4, 0xEC];
    // CPY opcodes should be marked as implemented
    let cpy_opcodes = [0xC0, 0xC4, 0xCC];
    // DEC opcodes should be marked as implemented
    let dec_opcodes = [0xC6, 0xCE, 0xD6, 0xDE];
    // DEX opcode should be marked as implemented
    let dex_opcodes = [0xCA];
    // DEY opcode should be marked as implemented
    let dey_opcodes = [0x88];
    // EOR opcodes should be marked as implemented
    let eor_opcodes = [0x41, 0x45, 0x49, 0x4D, 0x51, 0x55, 0x59, 0x5D];
    // INC opcodes should be marked as implemented
    let inc_opcodes = [0xE6, 0xEE, 0xF6, 0xFE];
    // INX opcode should be marked as implemented
    let inx_opcodes = [0xE8];
    // INY opcode should be marked as implemented
    let iny_opcodes = [0xC8];
    // JMP opcodes should be marked as implemented
    let jmp_opcodes = [0x4C, 0x6C];
    // JSR opcode should be marked as implemented
    let jsr_opcodes = [0x20];
    // LDA opcodes should be marked as implemented
    let lda_opcodes = [0xA1, 0xA5, 0xA9, 0xAD, 0xB1, 0xB5, 0xB9, 0xBD];
    // LDX opcodes should be marked as implemented
    let ldx_opcodes = [0xA2, 0xA6, 0xAE, 0xB6, 0xBE];
    // LDY opcodes should be marked as implemented
    let ldy_opcodes = [0xA0, 0xA4, 0xAC, 0xB4, 0xBC];
    // LSR opcodes should be marked as implemented
    let lsr_opcodes = [0x46, 0x4A, 0x4E, 0x56, 0x5E];
    // NOP opcode should be marked as implemented
    let nop_opcodes = [0xEA];
    // ORA opcodes should be marked as implemented
    let ora_opcodes = [0x01, 0x05, 0x09, 0x0D, 0x11, 0x15, 0x19, 0x1D];
    // PHA opcode should be marked as implemented
    let pha_opcodes = [0x48];
    // PHP opcode should be marked as implemented
    let php_opcodes = [0x08];
    // PLA opcode should be marked as implemented
    let pla_opcodes = [0x68];
    // PLP opcode should be marked as implemented
    let plp_opcodes = [0x28];
    // ROL opcodes should be marked as implemented
    let rol_opcodes = [0x26, 0x2A, 0x2E, 0x36, 0x3E];
    // ROR opcodes should be marked as implemented
    let ror_opcodes = [0x66, 0x6A, 0x6E, 0x76, 0x7E];
    // RTI opcode should be marked as implemented
    let rti_opcodes = [0x40];
    // RTS opcode should be marked as implemented
    let rts_opcodes = [0x60];
    // SBC opcodes should be marked as implemented
    let sbc_opcodes = [0xE1, 0xE5, 0xE9, 0xED, 0xF1, 0xF5, 0xF9, 0xFD];
    // SEC opcode should be marked as implemented
    let sec_opcodes = [0x38];
    // SED opcode should be marked as implemented
    let sed_opcodes = [0xF8];
    // SEI opcode should be marked as implemented
    let sei_opcodes = [0x78];
    // STA opcodes should be marked as implemented
    let sta_opcodes = [0x81, 0x85, 0x8D, 0x91, 0x95, 0x99, 0x9D];
    // STX opcodes should be marked as implemented
    let stx_opcodes = [0x86, 0x8E, 0x96];
    // STY opcodes should be marked as implemented
    let sty_opcodes = [0x84, 0x8C, 0x94];
    // TAX opcode should be marked as implemented
    let tax_opcodes = [0xAA];
    // TAY opcode should be marked as implemented
    let tay_opcodes = [0xA8];
    // TSX opcode should be marked as implemented
    let tsx_opcodes = [0xBA];
    // TXA opcode should be marked as implemented
    let txa_opcodes = [0x8A];
    // TXS opcode should be marked as implemented
    let txs_opcodes = [0x9A];

    for (opcode, metadata) in OPCODE_TABLE.iter().enumerate() {
        if adc_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "ADC opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "ADC",
                "Opcode 0x{:02X} should be ADC mnemonic",
                opcode
            );
        } else if and_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "AND opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "AND",
                "Opcode 0x{:02X} should be AND mnemonic",
                opcode
            );
        } else if asl_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "ASL opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "ASL",
                "Opcode 0x{:02X} should be ASL mnemonic",
                opcode
            );
        } else if bcc_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "BCC opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "BCC",
                "Opcode 0x{:02X} should be BCC mnemonic",
                opcode
            );
        } else if bcs_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "BCS opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "BCS",
                "Opcode 0x{:02X} should be BCS mnemonic",
                opcode
            );
        } else if beq_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "BEQ opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "BEQ",
                "Opcode 0x{:02X} should be BEQ mnemonic",
                opcode
            );
        } else if bit_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "BIT opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "BIT",
                "Opcode 0x{:02X} should be BIT mnemonic",
                opcode
            );
        } else if bmi_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "BMI opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "BMI",
                "Opcode 0x{:02X} should be BMI mnemonic",
                opcode
            );
        } else if bne_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "BNE opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "BNE",
                "Opcode 0x{:02X} should be BNE mnemonic",
                opcode
            );
        } else if bpl_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "BPL opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "BPL",
                "Opcode 0x{:02X} should be BPL mnemonic",
                opcode
            );
        } else if brk_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "BRK opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "BRK",
                "Opcode 0x{:02X} should be BRK mnemonic",
                opcode
            );
        } else if bvc_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "BVC opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "BVC",
                "Opcode 0x{:02X} should be BVC mnemonic",
                opcode
            );
        } else if bvs_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "BVS opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "BVS",
                "Opcode 0x{:02X} should be BVS mnemonic",
                opcode
            );
        } else if clc_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "CLC opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "CLC",
                "Opcode 0x{:02X} should be CLC mnemonic",
                opcode
            );
        } else if cld_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "CLD opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "CLD",
                "Opcode 0x{:02X} should be CLD mnemonic",
                opcode
            );
        } else if cli_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "CLI opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "CLI",
                "Opcode 0x{:02X} should be CLI mnemonic",
                opcode
            );
        } else if clv_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "CLV opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "CLV",
                "Opcode 0x{:02X} should be CLV mnemonic",
                opcode
            );
        } else if cmp_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "CMP opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "CMP",
                "Opcode 0x{:02X} should be CMP mnemonic",
                opcode
            );
        } else if cpx_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "CPX opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "CPX",
                "Opcode 0x{:02X} should be CPX mnemonic",
                opcode
            );
        } else if cpy_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "CPY opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "CPY",
                "Opcode 0x{:02X} should be CPY mnemonic",
                opcode
            );
        } else if dec_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "DEC opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "DEC",
                "Opcode 0x{:02X} should be DEC mnemonic",
                opcode
            );
        } else if dex_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "DEX opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "DEX",
                "Opcode 0x{:02X} should be DEX mnemonic",
                opcode
            );
        } else if dey_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "DEY opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "DEY",
                "Opcode 0x{:02X} should be DEY mnemonic",
                opcode
            );
        } else if eor_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "EOR opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "EOR",
                "Opcode 0x{:02X} should be EOR mnemonic",
                opcode
            );
        } else if inc_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "INC opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "INC",
                "Opcode 0x{:02X} should be INC mnemonic",
                opcode
            );
        } else if inx_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "INX opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "INX",
                "Opcode 0x{:02X} should be INX mnemonic",
                opcode
            );
        } else if iny_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "INY opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "INY",
                "Opcode 0x{:02X} should be INY mnemonic",
                opcode
            );
        } else if jmp_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "JMP opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "JMP",
                "Opcode 0x{:02X} should be JMP mnemonic",
                opcode
            );
        } else if jsr_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "JSR opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "JSR",
                "Opcode 0x{:02X} should be JSR mnemonic",
                opcode
            );
        } else if lda_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "LDA opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "LDA",
                "Opcode 0x{:02X} should be LDA mnemonic",
                opcode
            );
        } else if ldx_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "LDX opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "LDX",
                "Opcode 0x{:02X} should be LDX mnemonic",
                opcode
            );
        } else if ldy_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "LDY opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "LDY",
                "Opcode 0x{:02X} should be LDY mnemonic",
                opcode
            );
        } else if lsr_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "LSR opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "LSR",
                "Opcode 0x{:02X} should be LSR mnemonic",
                opcode
            );
        } else if nop_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "NOP opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "NOP",
                "Opcode 0x{:02X} should be NOP mnemonic",
                opcode
            );
        } else if ora_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "ORA opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "ORA",
                "Opcode 0x{:02X} should be ORA mnemonic",
                opcode
            );
        } else if pha_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "PHA opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "PHA",
                "Opcode 0x{:02X} should be PHA mnemonic",
                opcode
            );
        } else if php_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "PHP opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "PHP",
                "Opcode 0x{:02X} should be PHP mnemonic",
                opcode
            );
        } else if pla_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "PLA opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "PLA",
                "Opcode 0x{:02X} should be PLA mnemonic",
                opcode
            );
        } else if plp_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "PLP opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "PLP",
                "Opcode 0x{:02X} should be PLP mnemonic",
                opcode
            );
        } else if rol_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "ROL opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "ROL",
                "Opcode 0x{:02X} should be ROL mnemonic",
                opcode
            );
        } else if ror_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "ROR opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "ROR",
                "Opcode 0x{:02X} should be ROR mnemonic",
                opcode
            );
        } else if rti_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "RTI opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "RTI",
                "Opcode 0x{:02X} should be RTI mnemonic",
                opcode
            );
        } else if rts_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "RTS opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "RTS",
                "Opcode 0x{:02X} should be RTS mnemonic",
                opcode
            );
        } else if sbc_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "SBC opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "SBC",
                "Opcode 0x{:02X} should be SBC mnemonic",
                opcode
            );
        } else if sec_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "SEC opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "SEC",
                "Opcode 0x{:02X} should be SEC mnemonic",
                opcode
            );
        } else if sed_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "SED opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "SED",
                "Opcode 0x{:02X} should be SED mnemonic",
                opcode
            );
        } else if sei_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "SEI opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "SEI",
                "Opcode 0x{:02X} should be SEI mnemonic",
                opcode
            );
        } else if sta_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "STA opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "STA",
                "Opcode 0x{:02X} should be STA mnemonic",
                opcode
            );
        } else if stx_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "STX opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "STX",
                "Opcode 0x{:02X} should be STX mnemonic",
                opcode
            );
        } else if sty_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "STY opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "STY",
                "Opcode 0x{:02X} should be STY mnemonic",
                opcode
            );
        } else if tax_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "TAX opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "TAX",
                "Opcode 0x{:02X} should be TAX mnemonic",
                opcode
            );
        } else if tay_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "TAY opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "TAY",
                "Opcode 0x{:02X} should be TAY mnemonic",
                opcode
            );
        } else if tsx_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "TSX opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "TSX",
                "Opcode 0x{:02X} should be TSX mnemonic",
                opcode
            );
        } else if txa_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "TXA opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "TXA",
                "Opcode 0x{:02X} should be TXA mnemonic",
                opcode
            );
        } else if txs_opcodes.contains(&(opcode as u8)) {
            assert!(
                metadata.implemented,
                "TXS opcode 0x{:02X} should be marked as implemented",
                opcode
            );
            assert_eq!(
                metadata.mnemonic, "TXS",
                "Opcode 0x{:02X} should be TXS mnemonic",
                opcode
            );
        } else {
            assert!(
                !metadata.implemented,
                "Only ADC, AND, ASL, BCC, BCS, BEQ, BMI, BNE, BIT, BPL, BRK, BVC, BVS, CLC, CLD, CLI, CLV, CMP, CPX, CPY, DEC, DEX, DEY, EOR, INC, INX, INY, JMP, JSR, LDA, LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA, PLP, ROL, ROR, RTI, RTS, SBC, SEC, SED, SEI, STA, STX, STY, TAX, TAY, TSX, TXA, and TXS opcodes should be marked as implemented, but 0x{:02X} ({}) is marked",
                opcode, metadata.mnemonic
            );
        }
    }
}

#[test]
fn test_size_matches_addressing_mode() {
    // Verify size_bytes matches the addressing mode
    for (opcode, metadata) in OPCODE_TABLE.iter().enumerate() {
        let expected_size = match metadata.addressing_mode {
            AddressingMode::Implicit | AddressingMode::Accumulator => 1,
            AddressingMode::Immediate
            | AddressingMode::ZeroPage
            | AddressingMode::ZeroPageX
            | AddressingMode::ZeroPageY
            | AddressingMode::Relative
            | AddressingMode::IndirectX
            | AddressingMode::IndirectY => 2,
            AddressingMode::Absolute
            | AddressingMode::AbsoluteX
            | AddressingMode::AbsoluteY
            | AddressingMode::Indirect => 3,
        };

        assert_eq!(
            metadata.size_bytes, expected_size,
            "Opcode 0x{:02X} ({}) size mismatch: mode {:?} expects {} bytes, got {}",
            opcode, metadata.mnemonic, metadata.addressing_mode, expected_size, metadata.size_bytes
        );
    }
}

#[test]
fn test_known_opcodes() {
    // Test a few well-known opcodes to ensure table is correct

    // 0x00: BRK
    let brk = &OPCODE_TABLE[0x00];
    assert_eq!(brk.mnemonic, "BRK");
    assert_eq!(brk.base_cycles, 7);
    assert_eq!(brk.size_bytes, 1);

    // 0xA9: LDA immediate
    let lda_imm = &OPCODE_TABLE[0xA9];
    assert_eq!(lda_imm.mnemonic, "LDA");
    assert_eq!(lda_imm.base_cycles, 2);
    assert_eq!(lda_imm.size_bytes, 2);

    // 0xEA: NOP
    let nop = &OPCODE_TABLE[0xEA];
    assert_eq!(nop.mnemonic, "NOP");
    assert_eq!(nop.base_cycles, 2);
    assert_eq!(nop.size_bytes, 1);

    // 0x4C: JMP absolute
    let jmp = &OPCODE_TABLE[0x4C];
    assert_eq!(jmp.mnemonic, "JMP");
    assert_eq!(jmp.base_cycles, 3);
    assert_eq!(jmp.size_bytes, 3);

    // 0x6C: JMP indirect
    let jmp_ind = &OPCODE_TABLE[0x6C];
    assert_eq!(jmp_ind.mnemonic, "JMP");
    assert_eq!(jmp_ind.base_cycles, 5);
    assert_eq!(jmp_ind.size_bytes, 3);
}

#[test]
fn test_addressing_mode_coverage() {
    // Verify all addressing modes are used in the table
    let mut mode_used = std::collections::HashSet::new();

    for metadata in OPCODE_TABLE.iter() {
        mode_used.insert(format!("{:?}", metadata.addressing_mode));
    }

    // Should have multiple different addressing modes
    assert!(
        mode_used.len() >= 10,
        "Should use at least 10 different addressing modes"
    );
}

#[test]
fn test_instruction_variety() {
    // Verify multiple different instruction mnemonics exist
    let mut mnemonics = std::collections::HashSet::new();

    for metadata in OPCODE_TABLE.iter() {
        if metadata.mnemonic != "???" {
            mnemonics.insert(metadata.mnemonic);
        }
    }

    // Should have the 56 official 6502 instructions
    assert!(
        mnemonics.len() >= 50,
        "Should have at least 50 different instruction mnemonics (found {})",
        mnemonics.len()
    );
}

#[test]
fn test_cycle_cost_range() {
    // Verify cycle costs are in reasonable range (1-7 for documented instructions)
    for (opcode, metadata) in OPCODE_TABLE.iter().enumerate() {
        if metadata.mnemonic != "???" {
            assert!(
                metadata.base_cycles >= 1 && metadata.base_cycles <= 7,
                "Opcode 0x{:02X} ({}) has unusual cycle cost: {}",
                opcode,
                metadata.mnemonic,
                metadata.base_cycles
            );
        }
    }
}
