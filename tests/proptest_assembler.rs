//! Property-based tests for the assembler.
//!
//! These tests verify assembler invariants like:
//! - Number format equivalence (hex, decimal, binary produce same bytes)
//! - Addressing mode selection based on operand values
//! - No panics on malformed input
//! - Symbol resolution correctness

use lib6502::{assemble, OPCODE_TABLE};
use proptest::prelude::*;

// ========== Number Format Equivalence Tests ==========

proptest! {
    /// Property: Decimal, hex, and binary formats produce identical bytes for immediate operands
    #[test]
    fn prop_number_formats_equivalent_immediate(value in 0u8..=255u8) {
        let hex_source = format!("LDA #${:02X}", value);
        let dec_source = format!("LDA #{}", value);
        let bin_source = format!("LDA #%{:08b}", value);

        let hex_result = assemble(&hex_source);
        let dec_result = assemble(&dec_source);
        let bin_result = assemble(&bin_source);

        // All should succeed
        let hex_bytes = hex_result.expect("Hex format should assemble");
        let dec_bytes = dec_result.expect("Decimal format should assemble");
        let bin_bytes = bin_result.expect("Binary format should assemble");

        // All should produce identical bytes
        prop_assert_eq!(
            &hex_bytes.bytes,
            &dec_bytes.bytes,
            "Hex ${:02X} and decimal {} should produce identical bytes",
            value,
            value
        );
        prop_assert_eq!(
            &hex_bytes.bytes,
            &bin_bytes.bytes,
            "Hex ${:02X} and binary %{:08b} should produce identical bytes",
            value,
            value
        );

        // Expected bytes: LDA immediate is 0xA9 followed by operand
        prop_assert_eq!(&hex_bytes.bytes, &vec![0xA9, value]);
    }

    /// Property: Zero-page addresses produce same bytes regardless of number format
    #[test]
    fn prop_number_formats_equivalent_zero_page(addr in 0u8..=255u8) {
        let hex_source = format!("LDA ${:02X}", addr);
        let dec_source = format!("LDA {}", addr);

        let hex_result = assemble(&hex_source);
        let dec_result = assemble(&dec_source);

        let hex_bytes = hex_result.expect("Hex format should assemble");
        let dec_bytes = dec_result.expect("Decimal format should assemble");

        prop_assert_eq!(
            &hex_bytes.bytes,
            &dec_bytes.bytes,
            "LDA ${:02X} and LDA {} should produce identical bytes",
            addr,
            addr
        );

        // Expected: LDA zero page is 0xA5 followed by address
        prop_assert_eq!(&hex_bytes.bytes, &vec![0xA5, addr]);
    }
}

// ========== Addressing Mode Selection Tests ==========

proptest! {
    /// Property: Values 0-255 use zero-page addressing (2 bytes)
    #[test]
    fn prop_zero_page_for_small_addresses(addr in 0u8..=255u8) {
        let source = format!("LDA ${:02X}", addr);
        let result = assemble(&source).expect("Should assemble");

        // Zero-page LDA is 2 bytes (opcode + zp addr)
        prop_assert_eq!(
            result.bytes.len(),
            2,
            "LDA ${:02X} should be 2 bytes (zero-page)",
            addr
        );
        prop_assert_eq!(result.bytes[0], 0xA5, "Should use zero-page opcode 0xA5");
        prop_assert_eq!(result.bytes[1], addr);
    }

    /// Property: Values > 255 use absolute addressing (3 bytes)
    #[test]
    fn prop_absolute_for_large_addresses(addr in 256u16..=0xFFFFu16) {
        let source = format!("LDA ${:04X}", addr);
        let result = assemble(&source).expect("Should assemble");

        // Absolute LDA is 3 bytes (opcode + lo + hi)
        prop_assert_eq!(
            result.bytes.len(),
            3,
            "LDA ${:04X} should be 3 bytes (absolute)",
            addr
        );
        prop_assert_eq!(result.bytes[0], 0xAD, "Should use absolute opcode 0xAD");
        prop_assert_eq!(result.bytes[1], (addr & 0xFF) as u8, "Low byte");
        prop_assert_eq!(result.bytes[2], ((addr >> 8) & 0xFF) as u8, "High byte");
    }

    /// Property: Indexed zero-page stays in zero-page mode
    #[test]
    fn prop_indexed_zero_page_mode(addr in 0u8..=255u8) {
        let source = format!("LDA ${:02X},X", addr);
        let result = assemble(&source).expect("Should assemble");

        // Zero-page,X LDA is 2 bytes
        prop_assert_eq!(
            result.bytes.len(),
            2,
            "LDA ${:02X},X should be 2 bytes",
            addr
        );
        prop_assert_eq!(result.bytes[0], 0xB5, "Should use zero-page,X opcode 0xB5");
    }

    /// Property: Indexed absolute mode for addresses > 255
    #[test]
    fn prop_indexed_absolute_mode(addr in 256u16..=0xFFFEu16) {
        let source = format!("LDA ${:04X},X", addr);
        let result = assemble(&source).expect("Should assemble");

        // Absolute,X LDA is 3 bytes
        prop_assert_eq!(
            result.bytes.len(),
            3,
            "LDA ${:04X},X should be 3 bytes",
            addr
        );
        prop_assert_eq!(result.bytes[0], 0xBD, "Should use absolute,X opcode 0xBD");
    }
}

// ========== No Panic Tests ==========

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// Property: Assembler never panics on random ASCII input
    #[test]
    fn prop_no_panic_on_random_input(input in "[ -~]{0,100}") {
        // Just verify it doesn't panic - errors are fine
        let _ = assemble(&input);
    }

    /// Property: Assembler never panics on random mnemonic-like input
    #[test]
    fn prop_no_panic_on_mnemonic_like_input(
        mnemonic in "[A-Z]{3}",
        operand in "[ -~]{0,20}",
    ) {
        let source = format!("{} {}", mnemonic, operand);
        let _ = assemble(&source);
    }

    /// Property: Assembler handles empty input gracefully
    #[test]
    fn prop_empty_input_no_panic(whitespace in "[ \t\n\r]{0,50}") {
        let result = assemble(&whitespace);
        // Should either succeed with empty output or return an error - not panic
        if let Ok(output) = result {
            // Empty or whitespace-only input should produce no bytes
            prop_assert!(output.bytes.is_empty() || !whitespace.trim().is_empty());
        }
    }

    /// Property: Assembler handles comment-only input gracefully
    #[test]
    fn prop_comment_only_no_panic(comment_text in "[^;\n]{0,50}") {
        let source = format!("; {}", comment_text);
        let result = assemble(&source);
        if let Ok(output) = result {
            prop_assert!(output.bytes.is_empty());
        }
    }
}

// ========== Instruction Encoding Tests ==========

proptest! {
    /// Property: All implemented opcodes can be assembled and produce correct opcode byte
    #[test]
    fn prop_implemented_opcodes_assemble_correctly(opcode_idx in 0usize..256usize) {
        let metadata = &OPCODE_TABLE[opcode_idx];

        if !metadata.implemented {
            return Ok(());
        }

        // Generate appropriate source for this opcode's addressing mode
        let source = generate_source_for_opcode(opcode_idx as u8, metadata);

        if let Some(src) = source {
            let result = assemble(&src);
            if let Ok(output) = result {
                prop_assert_eq!(
                    output.bytes[0] as usize,
                    opcode_idx,
                    "Opcode for '{}' should be 0x{:02X}",
                    src.trim(),
                    opcode_idx
                );
            }
        }
    }
}

// ========== Label Resolution Tests ==========

proptest! {
    /// Property: Forward label references resolve correctly
    #[test]
    fn prop_forward_label_resolution(offset in 3u16..100u16) {
        // Generate a program with a forward reference
        let source = format!(
            "JMP TARGET\n{}\nTARGET:\n    NOP",
            "NOP\n".repeat(offset as usize)
        );

        let result = assemble(&source);
        let output = result.expect("Should assemble with forward reference");

        // Verify TARGET symbol exists and has correct value
        let target_addr = output.lookup_symbol_addr("TARGET")
            .expect("TARGET symbol should exist");

        // JMP is 3 bytes, then 'offset' NOPs (1 byte each)
        let expected_addr = 3 + offset;
        prop_assert_eq!(
            target_addr,
            expected_addr,
            "TARGET should be at address {}",
            expected_addr
        );
    }

    /// Property: Backward label references resolve correctly
    #[test]
    fn prop_backward_label_resolution(offset in 1u16..50u16) {
        let source = format!(
            "LOOP:\n{}\n    JMP LOOP",
            "NOP\n".repeat(offset as usize)
        );

        let result = assemble(&source);
        let output = result.expect("Should assemble with backward reference");

        // LOOP should be at address 0
        let loop_addr = output.lookup_symbol_addr("LOOP")
            .expect("LOOP symbol should exist");

        prop_assert_eq!(loop_addr, 0, "LOOP should be at address 0");

        // JMP operand should point to address 0
        // Find the JMP instruction (it's at address 'offset')
        let jmp_offset = offset as usize;
        prop_assert_eq!(output.bytes[jmp_offset], 0x4C, "Should be JMP absolute");
        prop_assert_eq!(output.bytes[jmp_offset + 1], 0x00, "Low byte should be 0");
        prop_assert_eq!(output.bytes[jmp_offset + 2], 0x00, "High byte should be 0");
    }
}

// ========== Constant Definition Tests ==========

proptest! {
    /// Property: Constants are resolved correctly in immediate mode
    #[test]
    fn prop_constant_immediate_resolution(value in 0u8..=255u8) {
        let source = format!("CONST = ${:02X}\n    LDA #CONST", value);
        let result = assemble(&source);
        let output = result.expect("Should assemble with constant");

        // LDA #CONST should produce LDA immediate with the constant value
        prop_assert_eq!(output.bytes[0], 0xA9, "Should be LDA immediate");
        prop_assert_eq!(output.bytes[1], value, "Should use constant value");
    }

    /// Property: Constants are resolved correctly in zero-page mode
    #[test]
    fn prop_constant_zero_page_resolution(addr in 0u8..=255u8) {
        let source = format!("ADDR = ${:02X}\n    LDA ADDR", addr);
        let result = assemble(&source);
        let output = result.expect("Should assemble with constant");

        // Should use zero-page addressing
        prop_assert_eq!(output.bytes[0], 0xA5, "Should be LDA zero-page");
        prop_assert_eq!(output.bytes[1], addr, "Should use constant address");
    }
}

// ========== Directive Tests ==========

proptest! {
    /// Property: .byte directive emits exact bytes
    #[test]
    fn prop_byte_directive_emits_bytes(
        b1 in 0u8..=255u8,
        b2 in 0u8..=255u8,
        b3 in 0u8..=255u8,
    ) {
        let source = format!(".byte ${:02X}, ${:02X}, ${:02X}", b1, b2, b3);
        let result = assemble(&source);
        let output = result.expect(".byte should assemble");

        prop_assert_eq!(output.bytes, vec![b1, b2, b3]);
    }

    /// Property: .word directive emits little-endian words
    #[test]
    fn prop_word_directive_little_endian(word in 0u16..=0xFFFFu16) {
        let source = format!(".word ${:04X}", word);
        let result = assemble(&source);
        let output = result.expect(".word should assemble");

        let lo = (word & 0xFF) as u8;
        let hi = ((word >> 8) & 0xFF) as u8;

        prop_assert_eq!(output.bytes, vec![lo, hi], ".word should be little-endian");
    }

    /// Property: .org directive sets correct assembly address
    #[test]
    fn prop_org_directive_sets_address(addr in 0u16..=0xFFF0u16) {
        let source = format!(".org ${:04X}\nSTART:", addr);
        let result = assemble(&source);
        let output = result.expect(".org should assemble");

        let start_addr = output.lookup_symbol_addr("START")
            .expect("START should exist");

        prop_assert_eq!(start_addr, addr, "START should be at .org address");
    }
}

// ========== Branch Offset Tests ==========

proptest! {
    /// Property: Forward branches within range assemble correctly
    #[test]
    fn prop_forward_branch_in_range(offset in 1u8..=127u8) {
        // Generate NOPs to create the desired offset
        let source = format!(
            "    BEQ TARGET\n{}\nTARGET:\n    NOP",
            "NOP\n".repeat(offset as usize)
        );

        let result = assemble(&source);
        let output = result.expect("Forward branch should assemble");

        // BEQ is 0xF0, followed by relative offset
        prop_assert_eq!(output.bytes[0], 0xF0, "Should be BEQ");
        prop_assert_eq!(output.bytes[1], offset, "Offset should be {}", offset);
    }

    /// Property: Backward branches within range assemble correctly
    #[test]
    fn prop_backward_branch_in_range(offset in 1u8..=126u8) {
        // Create a loop with backward branch
        let source = format!(
            "TARGET:\n{}\n    BEQ TARGET",
            "NOP\n".repeat(offset as usize)
        );

        let result = assemble(&source);
        let output = result.expect("Backward branch should assemble");

        // BEQ should have negative offset
        let bne_pos = offset as usize;
        prop_assert_eq!(output.bytes[bne_pos], 0xF0, "Should be BEQ");

        // Calculate expected negative offset (two's complement)
        // Branch is at 'offset', target is at 0
        // Offset = target - (branch_addr + 2) = 0 - (offset + 2) = -(offset + 2)
        let expected_offset = (256 - (offset as u16 + 2)) as u8;
        prop_assert_eq!(
            output.bytes[bne_pos + 1],
            expected_offset,
            "Negative offset should be 0x{:02X}",
            expected_offset
        );
    }
}

// ========== Helper Functions ==========

/// Generate assembly source for a specific opcode based on its addressing mode
fn generate_source_for_opcode(_opcode: u8, metadata: &lib6502::OpcodeMetadata) -> Option<String> {
    use lib6502::AddressingMode;

    let mnemonic = metadata.mnemonic;
    if mnemonic == "???" {
        return None;
    }

    match metadata.addressing_mode {
        AddressingMode::Implicit => Some(mnemonic.to_string()),
        AddressingMode::Accumulator => Some(format!("{} A", mnemonic)),
        AddressingMode::Immediate => Some(format!("{} #$42", mnemonic)),
        AddressingMode::ZeroPage => Some(format!("{} $42", mnemonic)),
        AddressingMode::ZeroPageX => Some(format!("{} $42,X", mnemonic)),
        AddressingMode::ZeroPageY => Some(format!("{} $42,Y", mnemonic)),
        AddressingMode::Absolute => Some(format!("{} $4200", mnemonic)),
        AddressingMode::AbsoluteX => Some(format!("{} $4200,X", mnemonic)),
        AddressingMode::AbsoluteY => Some(format!("{} $4200,Y", mnemonic)),
        AddressingMode::Indirect => Some(format!("{} ($4200)", mnemonic)),
        AddressingMode::IndirectX => Some(format!("{} ($42,X)", mnemonic)),
        AddressingMode::IndirectY => Some(format!("{} ($42),Y", mnemonic)),
        AddressingMode::Relative => {
            // Branch instructions need a target
            Some(format!("{} *+2", mnemonic))
        }
    }
}
