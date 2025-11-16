//! Assembler/Disassembler Round-Trip Validation
//!
//! This module validates the assembler and disassembler by using the Klaus
//! functional test binary as a comprehensive test case.
//!
//! ## Test Strategy
//!
//! 1. Load the Klaus 64KB binary (known-good machine code)
//! 2. Disassemble the entire binary
//! 3. Convert disassembled instructions to assembly source
//! 4. Reassemble the generated source
//! 5. Compare reassembled bytes with original binary
//!
//! ## What This Tests
//!
//! - **Disassembler**: Correctly decodes all 151 NMOS 6502 opcodes
//! - **Assembler**: Correctly encodes all instructions and addressing modes
//! - **Addressing modes**: All 13 addressing modes work correctly
//! - **Operand encoding**: Immediate, zero page, absolute, indexed, indirect modes
//! - **Round-trip fidelity**: disassemble(assemble(x)) == x
//!
//! ## Success Criteria
//!
//! If the reassembled binary matches the original Klaus binary byte-for-byte,
//! both the assembler and disassembler are validated across all 151 opcodes.

use lib6502::assembler::assemble;
use lib6502::disassembler::{disassemble, DisassemblyOptions, Instruction};
use std::fs::File;
use std::io::Read;

/// Load the Klaus test binary
fn load_test_binary(path: &str) -> Vec<u8> {
    let mut file =
        File::open(path).unwrap_or_else(|e| panic!("Failed to open test binary {}: {}", path, e));

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .unwrap_or_else(|e| panic!("Failed to read test binary: {}", e));

    assert_eq!(buffer.len(), 65536, "Test binary must be exactly 64KB");
    buffer
}

/// Convert disassembled instructions to assembly source code
///
/// This generates valid assembly source that can be reassembled.
/// It handles address continuity by inserting .org directives when needed.
fn instructions_to_source(instructions: &[Instruction]) -> String {
    let mut source = String::new();
    let mut current_address: Option<u16> = None;

    for instr in instructions {
        // Insert .org directive if this is the first instruction or if address jumped
        match current_address {
            None => {
                // First instruction - always insert .org
                source.push_str(&format!(".org ${:04X}\n", instr.address));
                current_address = Some(instr.address);
            }
            Some(addr) if addr != instr.address => {
                // Address jumped - insert .org
                source.push_str(&format!(".org ${:04X}\n", instr.address));
                current_address = Some(instr.address);
            }
            _ => {
                // Address is continuous, no .org needed
            }
        }

        // Format the instruction
        let line = format_instruction_as_source(instr);
        source.push_str(&line);
        source.push('\n');

        // Track next expected address
        current_address = Some(
            current_address
                .unwrap()
                .wrapping_add(instr.size_bytes as u16),
        );
    }

    source
}

/// Format a single instruction as assembly source
fn format_instruction_as_source(instr: &Instruction) -> String {
    use lib6502::addressing::AddressingMode;

    // Handle .byte directive for invalid opcodes
    if instr.mnemonic == ".byte" {
        return format!("    .byte ${:02X}", instr.opcode);
    }

    let mnemonic = instr.mnemonic;

    match instr.addressing_mode {
        AddressingMode::Implicit => {
            format!("    {}", mnemonic)
        }
        AddressingMode::Accumulator => {
            format!("    {} A", mnemonic)
        }
        AddressingMode::Immediate => {
            let value = instr.operand_bytes[0];
            format!("    {} #${:02X}", mnemonic, value)
        }
        AddressingMode::ZeroPage => {
            let addr = instr.operand_bytes[0];
            format!("    {} ${:02X}", mnemonic, addr)
        }
        AddressingMode::ZeroPageX => {
            let addr = instr.operand_bytes[0];
            format!("    {} ${:02X},X", mnemonic, addr)
        }
        AddressingMode::ZeroPageY => {
            let addr = instr.operand_bytes[0];
            format!("    {} ${:02X},Y", mnemonic, addr)
        }
        AddressingMode::Absolute => {
            let addr = u16::from_le_bytes([instr.operand_bytes[0], instr.operand_bytes[1]]);
            format!("    {} ${:04X}", mnemonic, addr)
        }
        AddressingMode::AbsoluteX => {
            let addr = u16::from_le_bytes([instr.operand_bytes[0], instr.operand_bytes[1]]);
            format!("    {} ${:04X},X", mnemonic, addr)
        }
        AddressingMode::AbsoluteY => {
            let addr = u16::from_le_bytes([instr.operand_bytes[0], instr.operand_bytes[1]]);
            format!("    {} ${:04X},Y", mnemonic, addr)
        }
        AddressingMode::Indirect => {
            let addr = u16::from_le_bytes([instr.operand_bytes[0], instr.operand_bytes[1]]);
            format!("    {} (${:04X})", mnemonic, addr)
        }
        AddressingMode::IndirectX => {
            let addr = instr.operand_bytes[0];
            format!("    {} (${:02X},X)", mnemonic, addr)
        }
        AddressingMode::IndirectY => {
            let addr = instr.operand_bytes[0];
            format!("    {} (${:02X}),Y", mnemonic, addr)
        }
        AddressingMode::Relative => {
            // For relative addressing, we need to calculate the target address
            // Branch offset is relative to the address of the next instruction
            let offset = instr.operand_bytes[0] as i8;
            let next_instr_addr = instr.address.wrapping_add(2);
            let target_addr = if offset >= 0 {
                next_instr_addr.wrapping_add(offset as u16)
            } else {
                next_instr_addr.wrapping_sub((-offset) as u16)
            };
            format!("    {} ${:04X}", mnemonic, target_addr)
        }
    }
}

/// Compare two byte arrays and report differences
fn compare_binaries(original: &[u8], reassembled: &[u8]) -> Result<(), String> {
    if original.len() != reassembled.len() {
        return Err(format!(
            "Length mismatch: original {} bytes, reassembled {} bytes",
            original.len(),
            reassembled.len()
        ));
    }

    let mut first_mismatch: Option<usize> = None;
    let mut total_mismatches = 0;

    for (i, (&orig, &reasm)) in original.iter().zip(reassembled.iter()).enumerate() {
        if orig != reasm {
            if first_mismatch.is_none() {
                first_mismatch = Some(i);
            }
            total_mismatches += 1;
        }
    }

    if let Some(idx) = first_mismatch {
        let context_start = idx.saturating_sub(8);
        let context_end = (idx + 8).min(original.len());

        let mut error_msg = format!(
            "Byte mismatch: {} total differences, first at offset ${:04X}\n",
            total_mismatches, idx
        );
        error_msg.push_str("\nContext:\n");
        error_msg.push_str("Offset   Original  Reassembled\n");
        error_msg.push_str("------   --------  -----------\n");

        for i in context_start..context_end {
            let marker = if i == idx { " <--" } else { "" };
            error_msg.push_str(&format!(
                "${:04X}:  ${:02X}       ${:02X}         {}\n",
                i, original[i], reassembled[i], marker
            ));
        }

        Err(error_msg)
    } else {
        Ok(())
    }
}

#[test]
#[ignore = "slow comprehensive test (~10 seconds) - run with --ignored or --include-ignored"]
fn klaus_assembler_disassembler_roundtrip() {
    println!("\n=== Klaus Assembler/Disassembler Round-Trip Test ===\n");

    // Step 1: Load the Klaus binary
    println!("Step 1: Loading Klaus test binary...");
    let original_binary = load_test_binary("tests/fixtures/6502_functional_test.bin");
    println!("  Loaded {} bytes", original_binary.len());

    // Step 2: Disassemble the entire binary
    println!("\nStep 2: Disassembling binary...");
    let options = DisassemblyOptions {
        start_address: 0x0000,
        hex_dump: false,
        show_offsets: false,
    };
    let instructions = disassemble(&original_binary, options);
    println!("  Disassembled {} instructions", instructions.len());

    // Count instruction types
    let mut opcode_set = std::collections::HashSet::new();
    let mut invalid_count = 0;
    for instr in &instructions {
        opcode_set.insert(instr.opcode);
        if instr.mnemonic == ".byte" {
            invalid_count += 1;
        }
    }
    println!(
        "  Found {} unique opcodes ({} invalid/data bytes)",
        opcode_set.len(),
        invalid_count
    );

    // Step 3: Convert to assembly source
    println!("\nStep 3: Converting to assembly source...");
    let asm_source = instructions_to_source(&instructions);
    let source_lines = asm_source.lines().count();
    println!("  Generated {} lines of assembly", source_lines);

    // Optional: Save source for debugging
    if std::env::var("SAVE_ROUNDTRIP_SOURCE").is_ok() {
        std::fs::write("target/roundtrip_disassembled.asm", &asm_source)
            .expect("Failed to write debug source");
        println!("  Saved to target/roundtrip_disassembled.asm");
    }

    // Step 4: Reassemble the source
    println!("\nStep 4: Reassembling source...");
    let assembled = match assemble(&asm_source) {
        Ok(output) => output,
        Err(errors) => {
            eprintln!("\n=== ASSEMBLY ERRORS ===");
            for (i, error) in errors.iter().enumerate() {
                eprintln!("Error {}: {}", i + 1, error);
                if i >= 10 {
                    eprintln!("... and {} more errors", errors.len() - 10);
                    break;
                }
            }
            panic!("Assembly failed with {} errors", errors.len());
        }
    };
    println!("  Assembled {} bytes", assembled.bytes.len());

    // Step 5: Compare binaries
    println!("\nStep 5: Comparing binaries...");
    match compare_binaries(&original_binary, &assembled.bytes) {
        Ok(()) => {
            println!("\n✓ SUCCESS: Round-trip test passed!");
            println!("  All {} bytes match perfectly", original_binary.len());
            println!("\nThis validates:");
            println!("  - Disassembler correctly decodes all opcodes");
            println!("  - Assembler correctly encodes all instructions");
            println!(
                "  - All {} unique opcodes round-trip correctly",
                opcode_set.len()
            );
        }
        Err(msg) => {
            eprintln!("\n✗ FAILURE: Round-trip test failed!");
            eprintln!("{}", msg);
            panic!("Round-trip test failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sanity test: verify the test binary exists and loads
    #[test]
    fn test_binary_loads() {
        let binary = load_test_binary("tests/fixtures/6502_functional_test.bin");
        assert_eq!(binary.len(), 65536);

        // Verify it starts with CLD at $0400
        assert_eq!(binary[0x0400], 0xD8); // CLD opcode
    }

    /// Test instruction formatting for common cases
    #[test]
    fn test_format_instruction_implicit() {
        use lib6502::addressing::AddressingMode;

        let instr = Instruction {
            address: 0x1000,
            opcode: 0xEA,
            mnemonic: "NOP",
            addressing_mode: AddressingMode::Implicit,
            operand_bytes: vec![],
            size_bytes: 1,
            base_cycles: 2,
        };

        let formatted = format_instruction_as_source(&instr);
        assert_eq!(formatted, "    NOP");
    }

    #[test]
    fn test_format_instruction_immediate() {
        use lib6502::addressing::AddressingMode;

        let instr = Instruction {
            address: 0x1000,
            opcode: 0xA9,
            mnemonic: "LDA",
            addressing_mode: AddressingMode::Immediate,
            operand_bytes: vec![0x42],
            size_bytes: 2,
            base_cycles: 2,
        };

        let formatted = format_instruction_as_source(&instr);
        assert_eq!(formatted, "    LDA #$42");
    }

    #[test]
    fn test_format_instruction_absolute() {
        use lib6502::addressing::AddressingMode;

        let instr = Instruction {
            address: 0x1000,
            opcode: 0xAD,
            mnemonic: "LDA",
            addressing_mode: AddressingMode::Absolute,
            operand_bytes: vec![0x34, 0x12], // Little-endian: $1234
            size_bytes: 3,
            base_cycles: 4,
        };

        let formatted = format_instruction_as_source(&instr);
        assert_eq!(formatted, "    LDA $1234");
    }

    #[test]
    fn test_format_instruction_relative() {
        use lib6502::addressing::AddressingMode;

        // BEQ at $1000 with offset +5 (jumps to $1007)
        let instr = Instruction {
            address: 0x1000,
            opcode: 0xF0,
            mnemonic: "BEQ",
            addressing_mode: AddressingMode::Relative,
            operand_bytes: vec![0x05],
            size_bytes: 2,
            base_cycles: 2,
        };

        let formatted = format_instruction_as_source(&instr);
        // Next instruction is at $1002, offset +5 = $1007
        assert_eq!(formatted, "    BEQ $1007");
    }

    #[test]
    fn test_format_instruction_relative_backward() {
        use lib6502::addressing::AddressingMode;

        // BNE at $1000 with offset -2 (jumps to $1000)
        // -2 in two's complement is 0xFE
        let instr = Instruction {
            address: 0x1000,
            opcode: 0xD0,
            mnemonic: "BNE",
            addressing_mode: AddressingMode::Relative,
            operand_bytes: vec![0xFE],
            size_bytes: 2,
            base_cycles: 2,
        };

        let formatted = format_instruction_as_source(&instr);
        // Next instruction is at $1002, offset -2 = $1000
        assert_eq!(formatted, "    BNE $1000");
    }

    /// Test that .org directives are inserted when addresses jump
    #[test]
    fn test_instructions_to_source_with_org() {
        use lib6502::addressing::AddressingMode;

        let instructions = vec![
            Instruction {
                address: 0x0000,
                opcode: 0xEA,
                mnemonic: "NOP",
                addressing_mode: AddressingMode::Implicit,
                operand_bytes: vec![],
                size_bytes: 1,
                base_cycles: 2,
            },
            // Address jumps to 0x0400 (not consecutive)
            Instruction {
                address: 0x0400,
                opcode: 0xD8,
                mnemonic: "CLD",
                addressing_mode: AddressingMode::Implicit,
                operand_bytes: vec![],
                size_bytes: 1,
                base_cycles: 2,
            },
        ];

        let source = instructions_to_source(&instructions);
        assert!(source.contains(".org $0000"));
        assert!(source.contains(".org $0400"));
        assert!(source.contains("NOP"));
        assert!(source.contains("CLD"));
    }

    /// Test a simple round-trip with a small code snippet
    #[test]
    fn test_simple_roundtrip() {
        // Create a simple instruction sequence
        let original_bytes = vec![
            0xA9, 0x42, // LDA #$42
            0x8D, 0x00, 0x20, // STA $2000
            0xEA, // NOP
        ];

        // Disassemble
        let options = DisassemblyOptions {
            start_address: 0x0000,
            hex_dump: false,
            show_offsets: false,
        };
        let instructions = disassemble(&original_bytes, options);

        // Convert to source
        let source = instructions_to_source(&instructions);

        // Reassemble
        let assembled = assemble(&source).expect("Assembly should succeed");

        // Compare
        assert_eq!(
            original_bytes, assembled.bytes,
            "Round-trip should produce identical bytes"
        );
    }
}
