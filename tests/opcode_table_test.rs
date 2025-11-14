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
        } else {
            assert!(
                !metadata.implemented,
                "Only ADC, AND, ASL, BCC, BCS, BEQ, BMI, BNE, BIT, and BPL opcodes should be marked as implemented, but 0x{:02X} ({}) is marked",
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
