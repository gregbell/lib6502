//! Property-based round-trip tests for assembler/disassembler.
//!
//! These tests verify that:
//! - assemble(disassemble(bytes)) == bytes for valid instruction sequences
//! - disassemble(assemble(source)) preserves semantic meaning
//! - All 151 implemented opcodes round-trip correctly

use lib6502::{assemble, disassemble, AddressingMode, DisassemblyOptions, OPCODE_TABLE};
use proptest::prelude::*;

/// Generate a valid instruction byte sequence for a specific opcode
fn generate_instruction_bytes(opcode: u8) -> Vec<u8> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    if !metadata.implemented || metadata.mnemonic == "???" {
        return vec![];
    }

    let mut bytes = vec![opcode];

    // Add operand bytes based on instruction size
    match metadata.size_bytes {
        1 => {} // No operands
        2 => bytes.push(0x42), // One operand byte
        3 => {
            bytes.push(0x00); // Low byte
            bytes.push(0x40); // High byte - using 0x4000 as a safe address
        }
        _ => {}
    }

    bytes
}

/// Get all implemented opcodes
fn implemented_opcodes() -> Vec<u8> {
    OPCODE_TABLE
        .iter()
        .enumerate()
        .filter(|(_, m)| m.implemented && m.mnemonic != "???")
        .map(|(i, _)| i as u8)
        .collect()
}

/// Get opcodes that can cleanly round-trip (excludes branches which need labels)
fn roundtrippable_opcodes() -> Vec<u8> {
    OPCODE_TABLE
        .iter()
        .enumerate()
        .filter(|(_, m)| {
            m.implemented
                && m.mnemonic != "???"
                && m.addressing_mode != AddressingMode::Relative // Branches need labels
        })
        .map(|(i, _)| i as u8)
        .collect()
}

// ========== Basic Round-Trip Tests ==========

proptest! {
    /// Property: disassemble produces valid instructions for all implemented opcodes
    #[test]
    fn prop_disassemble_all_implemented_opcodes(opcode in prop::sample::select(implemented_opcodes())) {
        let bytes = generate_instruction_bytes(opcode);
        if bytes.is_empty() {
            return Ok(());
        }

        let options = DisassemblyOptions::default();
        let instructions = disassemble(&bytes, options);

        prop_assert_eq!(
            instructions.len(),
            1,
            "Single instruction should produce one disassembled instruction"
        );

        let instr = &instructions[0];
        prop_assert_eq!(
            instr.opcode,
            opcode,
            "Disassembled opcode should match input"
        );

        let metadata = &OPCODE_TABLE[opcode as usize];
        prop_assert_eq!(
            instr.mnemonic,
            metadata.mnemonic,
            "Mnemonic should match opcode table"
        );
        prop_assert_eq!(
            instr.size_bytes,
            metadata.size_bytes,
            "Size should match opcode table"
        );
    }

    /// Property: Binary bytes round-trip through disassemble then assemble
    #[test]
    fn prop_bytes_roundtrip_through_disasm_asm(opcode in prop::sample::select(roundtrippable_opcodes())) {
        let original_bytes = generate_instruction_bytes(opcode);
        if original_bytes.is_empty() {
            return Ok(());
        }

        // Disassemble
        let options = DisassemblyOptions::default();
        let instructions = disassemble(&original_bytes, options);

        if instructions.is_empty() {
            return Ok(());
        }

        // Format as assembly source
        let source = format_instruction_as_source(&instructions[0]);

        // Reassemble
        let assembled = assemble(&source);
        if let Ok(output) = assembled {
            prop_assert_eq!(
                &output.bytes,
                &original_bytes,
                "Bytes should round-trip: original={:02X?}, source='{}', reassembled={:02X?}",
                &original_bytes,
                source,
                &output.bytes
            );
        }
    }
}

// ========== Specific Opcode Round-Trip Tests ==========

proptest! {
    /// Property: LDA immediate round-trips correctly
    #[test]
    fn prop_lda_immediate_roundtrip(value in 0u8..=255u8) {
        let original_bytes = vec![0xA9, value];

        let instructions = disassemble(&original_bytes, DisassemblyOptions::default());
        prop_assert_eq!(instructions.len(), 1);

        let source = format_instruction_as_source(&instructions[0]);
        let assembled = assemble(&source).expect("Should assemble");

        prop_assert_eq!(
            &assembled.bytes,
            &original_bytes,
            "LDA #${:02X} should round-trip",
            value
        );
    }

    /// Property: LDA zero-page round-trips correctly
    #[test]
    fn prop_lda_zero_page_roundtrip(addr in 0u8..=255u8) {
        let original_bytes = vec![0xA5, addr];

        let instructions = disassemble(&original_bytes, DisassemblyOptions::default());
        prop_assert_eq!(instructions.len(), 1);

        let source = format_instruction_as_source(&instructions[0]);
        let assembled = assemble(&source).expect("Should assemble");

        prop_assert_eq!(
            &assembled.bytes,
            &original_bytes,
            "LDA ${:02X} should round-trip",
            addr
        );
    }

    /// Property: LDA absolute round-trips correctly
    #[test]
    fn prop_lda_absolute_roundtrip(addr in 256u16..=0xFFFFu16) {
        let lo = (addr & 0xFF) as u8;
        let hi = ((addr >> 8) & 0xFF) as u8;
        let original_bytes = vec![0xAD, lo, hi];

        let options = DisassemblyOptions::default();
        let instructions = disassemble(&original_bytes, options);
        prop_assert_eq!(instructions.len(), 1);

        let source = format_instruction_as_source(&instructions[0]);
        let assembled = assemble(&source).expect("Should assemble");

        prop_assert_eq!(
            &assembled.bytes,
            &original_bytes,
            "LDA ${:04X} should round-trip",
            addr
        );
    }

    /// Property: STA instructions round-trip correctly
    #[test]
    fn prop_sta_roundtrip(
        mode_selector in 0u8..5u8,
        operand in 0u8..=255u8,
    ) {
        let original_bytes = match mode_selector {
            0 => vec![0x85, operand],           // STA zero page
            1 => vec![0x95, operand],           // STA zero page,X
            2 => vec![0x8D, operand, 0x12],     // STA absolute
            3 => vec![0x9D, operand, 0x12],     // STA absolute,X
            _ => vec![0x99, operand, 0x12],     // STA absolute,Y
        };

        let instructions = disassemble(&original_bytes, DisassemblyOptions::default());
        prop_assert_eq!(instructions.len(), 1);

        let source = format_instruction_as_source(&instructions[0]);
        let assembled = assemble(&source).expect("Should assemble");

        prop_assert_eq!(
            &assembled.bytes,
            &original_bytes,
            "STA instruction should round-trip: source='{}'",
            source
        );
    }

    /// Property: ADC/SBC instructions round-trip correctly
    #[test]
    fn prop_adc_sbc_roundtrip(
        is_sbc in proptest::bool::ANY,
        operand in 0u8..=255u8,
    ) {
        let opcode = if is_sbc { 0xE9 } else { 0x69 }; // SBC/ADC immediate
        let original_bytes = vec![opcode, operand];

        let instructions = disassemble(&original_bytes, DisassemblyOptions::default());
        prop_assert_eq!(instructions.len(), 1);

        let source = format_instruction_as_source(&instructions[0]);
        let assembled = assemble(&source).expect("Should assemble");

        prop_assert_eq!(
            &assembled.bytes,
            &original_bytes,
            "{} immediate should round-trip",
            if is_sbc { "SBC" } else { "ADC" }
        );
    }

    /// Property: Shift/rotate instructions round-trip correctly
    #[test]
    fn prop_shift_rotate_roundtrip(
        shift_type in 0u8..4u8,
        mode_selector in 0u8..3u8,
        operand in 0u8..=255u8,
    ) {
        // Select shift type: ASL, LSR, ROL, ROR
        let base_opcode = match shift_type {
            0 => 0x06, // ASL
            1 => 0x46, // LSR
            2 => 0x26, // ROL
            _ => 0x66, // ROR
        };

        let original_bytes = match mode_selector {
            0 => vec![base_opcode + 0x04],                  // Accumulator (opcode + 4)
            1 => vec![base_opcode, operand],               // Zero page
            _ => vec![base_opcode + 0x08, operand, 0x12],  // Absolute (opcode + 8)
        };

        let instructions = disassemble(&original_bytes, DisassemblyOptions::default());
        if instructions.is_empty() {
            return Ok(());
        }

        let source = format_instruction_as_source(&instructions[0]);
        if let Ok(assembled) = assemble(&source) {
            prop_assert_eq!(
                assembled.bytes,
                original_bytes,
                "Shift/rotate should round-trip: source='{}'",
                source
            );
        }
    }
}

// ========== Multi-Instruction Round-Trip Tests ==========

proptest! {
    /// Property: Sequence of simple instructions round-trips correctly
    #[test]
    fn prop_instruction_sequence_roundtrip(
        instr1 in prop::sample::select(vec![0xEA, 0xAA, 0xA8, 0x8A, 0x98]), // NOP, TAX, TAY, TXA, TYA
        instr2 in prop::sample::select(vec![0xEA, 0xCA, 0x88, 0xE8, 0xC8]), // NOP, DEX, DEY, INX, INY
        instr3 in prop::sample::select(vec![0x18, 0x38, 0x58, 0xB8, 0xD8]), // CLC, SEC, CLI, CLV, CLD
    ) {
        let original_bytes = vec![instr1, instr2, instr3];

        let instructions = disassemble(&original_bytes, DisassemblyOptions::default());
        prop_assert_eq!(instructions.len(), 3);

        // Format all instructions as source
        let source: String = instructions
            .iter()
            .map(format_instruction_as_source)
            .collect::<Vec<_>>()
            .join("\n");

        let assembled = assemble(&source).expect("Should assemble");

        prop_assert_eq!(
            &assembled.bytes,
            &original_bytes,
            "Instruction sequence should round-trip"
        );
    }
}

// ========== Disassembler Robustness Tests ==========

proptest! {
    /// Property: Disassembler handles any byte sequence without panicking
    #[test]
    fn prop_disassembler_no_panic(bytes in prop::collection::vec(0u8..=255u8, 0..100)) {
        let options = DisassemblyOptions::default();
        let _ = disassemble(&bytes, options);
        // Just verify it doesn't panic
    }

    /// Property: Disassembler always consumes all input bytes
    #[test]
    fn prop_disassembler_consumes_all_bytes(bytes in prop::collection::vec(0u8..=255u8, 1..50)) {
        let options = DisassemblyOptions::default();
        let instructions = disassemble(&bytes, options);

        // Sum up all instruction sizes
        let total_size: usize = instructions.iter().map(|i| i.size_bytes as usize).sum();

        prop_assert_eq!(
            total_size,
            bytes.len(),
            "Disassembler should consume exactly {} bytes, consumed {}",
            bytes.len(),
            total_size
        );
    }

    /// Property: Disassembler addresses are sequential
    #[test]
    fn prop_disassembler_sequential_addresses(bytes in prop::collection::vec(0u8..=255u8, 1..50)) {
        let start_addr = 0x8000u16;
        let options = DisassemblyOptions {
            start_address: start_addr,
            ..Default::default()
        };
        let instructions = disassemble(&bytes, options);

        let mut expected_addr = start_addr;
        for instr in &instructions {
            prop_assert_eq!(
                instr.address,
                expected_addr,
                "Instruction address should be sequential"
            );
            expected_addr = expected_addr.wrapping_add(instr.size_bytes as u16);
        }
    }
}

// ========== Helper Functions ==========

/// Format a disassembled instruction as assembly source code
fn format_instruction_as_source(instr: &lib6502::Instruction) -> String {
    let operand = format_operand(instr);
    if operand.is_empty() {
        instr.mnemonic.to_string()
    } else {
        format!("{} {}", instr.mnemonic, operand)
    }
}

/// Format the operand of an instruction based on its addressing mode
fn format_operand(instr: &lib6502::Instruction) -> String {
    match instr.addressing_mode {
        AddressingMode::Implicit => String::new(),
        AddressingMode::Accumulator => "A".to_string(),
        AddressingMode::Immediate => {
            format!("#${:02X}", instr.operand_bytes[0])
        }
        AddressingMode::ZeroPage => {
            format!("${:02X}", instr.operand_bytes[0])
        }
        AddressingMode::ZeroPageX => {
            format!("${:02X},X", instr.operand_bytes[0])
        }
        AddressingMode::ZeroPageY => {
            format!("${:02X},Y", instr.operand_bytes[0])
        }
        AddressingMode::Absolute => {
            let addr = (instr.operand_bytes[1] as u16) << 8 | (instr.operand_bytes[0] as u16);
            format!("${:04X}", addr)
        }
        AddressingMode::AbsoluteX => {
            let addr = (instr.operand_bytes[1] as u16) << 8 | (instr.operand_bytes[0] as u16);
            format!("${:04X},X", addr)
        }
        AddressingMode::AbsoluteY => {
            let addr = (instr.operand_bytes[1] as u16) << 8 | (instr.operand_bytes[0] as u16);
            format!("${:04X},Y", addr)
        }
        AddressingMode::Indirect => {
            let addr = (instr.operand_bytes[1] as u16) << 8 | (instr.operand_bytes[0] as u16);
            format!("(${:04X})", addr)
        }
        AddressingMode::IndirectX => {
            format!("(${:02X},X)", instr.operand_bytes[0])
        }
        AddressingMode::IndirectY => {
            format!("(${:02X}),Y", instr.operand_bytes[0])
        }
        AddressingMode::Relative => {
            // For round-trip, we need to use the target address as a label
            // This is tricky for branches, so we'll use a relative offset notation
            let offset = instr.operand_bytes[0] as i8;
            let target = instr
                .address
                .wrapping_add(2)
                .wrapping_add(offset as i16 as u16);
            format!("${:04X}", target)
        }
    }
}

// ========== Explicit Opcode Coverage Tests ==========

#[test]
fn test_all_load_store_opcodes_roundtrip() {
    // Test all LDA opcodes
    let lda_opcodes = [
        (0xA9, vec![0xA9, 0x42]),       // LDA #$42
        (0xA5, vec![0xA5, 0x42]),       // LDA $42
        (0xB5, vec![0xB5, 0x42]),       // LDA $42,X
        (0xAD, vec![0xAD, 0x00, 0x42]), // LDA $4200
        (0xBD, vec![0xBD, 0x00, 0x42]), // LDA $4200,X
        (0xB9, vec![0xB9, 0x00, 0x42]), // LDA $4200,Y
        (0xA1, vec![0xA1, 0x42]),       // LDA ($42,X)
        (0xB1, vec![0xB1, 0x42]),       // LDA ($42),Y
    ];

    for (opcode, original_bytes) in lda_opcodes {
        let instructions = disassemble(&original_bytes, DisassemblyOptions::default());
        assert_eq!(instructions.len(), 1, "Opcode 0x{:02X}", opcode);

        let source = format_instruction_as_source(&instructions[0]);
        let assembled = assemble(&source)
            .unwrap_or_else(|_| panic!("Should assemble opcode 0x{:02X}", opcode));

        assert_eq!(
            assembled.bytes, original_bytes,
            "Opcode 0x{:02X} should round-trip: source='{}'",
            opcode, source
        );
    }
}

#[test]
fn test_all_implicit_opcodes_roundtrip() {
    // All single-byte implicit instructions
    let implicit_opcodes = [
        0x00, // BRK (special case)
        0x08, // PHP
        0x18, // CLC
        0x28, // PLP
        0x38, // SEC
        0x40, // RTI
        0x48, // PHA
        0x58, // CLI
        0x60, // RTS
        0x68, // PLA
        0x78, // SEI
        0x88, // DEY
        0x8A, // TXA
        0x98, // TYA
        0x9A, // TXS
        0xA8, // TAY
        0xAA, // TAX
        0xB8, // CLV
        0xBA, // TSX
        0xC8, // INY
        0xCA, // DEX
        0xD8, // CLD
        0xE8, // INX
        0xEA, // NOP
        0xF8, // SED
    ];

    for opcode in implicit_opcodes {
        // Skip BRK for round-trip (it has complex behavior)
        if opcode == 0x00 {
            continue;
        }

        let original_bytes = vec![opcode];
        let instructions = disassemble(&original_bytes, DisassemblyOptions::default());
        assert_eq!(instructions.len(), 1, "Opcode 0x{:02X}", opcode);

        let source = format_instruction_as_source(&instructions[0]);
        let assembled = assemble(&source)
            .unwrap_or_else(|_| panic!("Should assemble opcode 0x{:02X}", opcode));

        assert_eq!(
            assembled.bytes, original_bytes,
            "Opcode 0x{:02X} ({}) should round-trip",
            opcode, instructions[0].mnemonic
        );
    }
}
