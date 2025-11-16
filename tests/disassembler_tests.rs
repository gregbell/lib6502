//! Integration tests for the 6502 disassembler

use lib6502::addressing::AddressingMode;
use lib6502::disassembler::formatter::format_instruction;
use lib6502::disassembler::{disassemble, DisassemblyOptions};

// T015: Integration test for single instruction disassembly (LDA #$42)
#[test]
fn test_single_instruction_disassembly() {
    let bytes = &[0xA9, 0x42]; // LDA #$42

    let instructions = disassemble(bytes, DisassemblyOptions::default());

    assert_eq!(instructions.len(), 1);

    let instr = &instructions[0];
    assert_eq!(instr.address, 0x0000);
    assert_eq!(instr.opcode, 0xA9);
    assert_eq!(instr.mnemonic, "LDA");
    assert_eq!(instr.addressing_mode, AddressingMode::Immediate);
    assert_eq!(instr.operand_bytes, vec![0x42]);
    assert_eq!(instr.size_bytes, 2);

    assert_eq!(format_instruction(instr), "LDA #$42");
}

// T016: Integration test for multi-instruction disassembly
#[test]
fn test_multi_instruction_disassembly() {
    let bytes = &[
        0xA9, 0x42, // LDA #$42
        0x8D, 0x00, 0x80, // STA $8000
        0x4C, 0x00, 0x80, // JMP $8000
    ];

    let options = DisassemblyOptions {
        start_address: 0x8000,
        hex_dump: false,
        show_offsets: false,
    };

    let instructions = disassemble(bytes, options);

    assert_eq!(instructions.len(), 3);

    // LDA #$42
    assert_eq!(instructions[0].address, 0x8000);
    assert_eq!(instructions[0].mnemonic, "LDA");
    assert_eq!(format_instruction(&instructions[0]), "LDA #$42");

    // STA $8000
    assert_eq!(instructions[1].address, 0x8002);
    assert_eq!(instructions[1].mnemonic, "STA");
    assert_eq!(format_instruction(&instructions[1]), "STA $8000");

    // JMP $8000
    assert_eq!(instructions[2].address, 0x8005);
    assert_eq!(instructions[2].mnemonic, "JMP");
    assert_eq!(format_instruction(&instructions[2]), "JMP $8000");
}

// T017: Integration test for illegal opcode handling (".byte $XX")
#[test]
fn test_illegal_opcode_handling() {
    let bytes = &[
        0xA9, 0x42, // LDA #$42 (valid)
        0xFF, // Illegal opcode
        0xEA, // NOP (valid)
    ];

    let instructions = disassemble(bytes, DisassemblyOptions::default());

    assert_eq!(instructions.len(), 3);

    // LDA #$42 is valid
    assert_eq!(instructions[0].mnemonic, "LDA");

    // Illegal opcode should be represented as .byte
    assert_eq!(instructions[1].address, 0x0002);
    assert_eq!(instructions[1].mnemonic, ".byte");
    assert_eq!(instructions[1].opcode, 0xFF);
    assert_eq!(format_instruction(&instructions[1]), ".byte $FF");

    // NOP is valid
    assert_eq!(instructions[2].address, 0x0003);
    assert_eq!(instructions[2].mnemonic, "NOP");
}

// T018: Integration test for starting address offset
#[test]
fn test_starting_address_offset() {
    let bytes = &[
        0xA9, 0x42, // LDA #$42
        0x8D, 0x00, 0x80, // STA $8000
    ];

    let options = DisassemblyOptions {
        start_address: 0xC000,
        hex_dump: false,
        show_offsets: false,
    };

    let instructions = disassemble(bytes, options);

    assert_eq!(instructions.len(), 2);

    // Addresses should start at 0xC000
    assert_eq!(instructions[0].address, 0xC000);
    assert_eq!(instructions[1].address, 0xC002);
}

#[test]
fn test_empty_disassembly() {
    let bytes = &[];
    let instructions = disassemble(bytes, DisassemblyOptions::default());
    assert_eq!(instructions.len(), 0);
}

// ========== Phase 7: User Story 4 - Hex Dump Formatting ==========

// T081: Integration test for hex dump format with single instruction
#[test]
fn test_hex_dump_single_instruction() {
    use lib6502::disassembler::formatter::format_hex_dump;

    let bytes = &[0xA9, 0x42]; // LDA #$42

    let options = DisassemblyOptions {
        start_address: 0x8000,
        hex_dump: true,
        show_offsets: false,
    };

    let instructions = disassemble(bytes, options);
    let output = format_hex_dump(&instructions);

    // Expected format: "8000: A9 42     LDA #$42"
    assert!(output.contains("8000:"), "Should contain address");
    assert!(output.contains("A9 42"), "Should contain hex bytes");
    assert!(output.contains("LDA #$42"), "Should contain assembly");
}

// T082: Integration test for hex dump with varying instruction byte lengths
#[test]
fn test_hex_dump_varying_lengths() {
    use lib6502::disassembler::formatter::format_hex_dump;

    let bytes = &[
        0xEA, // NOP (1 byte)
        0xA9, 0x42, // LDA #$42 (2 bytes)
        0x8D, 0x00, 0x80, // STA $8000 (3 bytes)
    ];

    let options = DisassemblyOptions {
        start_address: 0xC000,
        hex_dump: true,
        show_offsets: false,
    };

    let instructions = disassemble(bytes, options);
    let output = format_hex_dump(&instructions);

    // Should have proper alignment despite different instruction lengths
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 3, "Should have 3 lines of output");

    // Check first line (1-byte instruction)
    assert!(
        lines[0].contains("C000:"),
        "Line 1 should have address C000: {}",
        lines[0]
    );
    assert!(
        lines[0].contains("EA"),
        "Line 1 should have NOP opcode: {}",
        lines[0]
    );
    assert!(
        lines[0].contains("NOP"),
        "Line 1 should have NOP mnemonic: {}",
        lines[0]
    );

    // Check second line (2-byte instruction)
    assert!(
        lines[1].contains("C001:"),
        "Line 2 should have address C001: {}",
        lines[1]
    );
    assert!(
        lines[1].contains("A9 42"),
        "Line 2 should have LDA bytes: {}",
        lines[1]
    );
    assert!(
        lines[1].contains("LDA #$42"),
        "Line 2 should have LDA mnemonic: {}",
        lines[1]
    );

    // Check third line (3-byte instruction)
    // Address should be C003 (C000 + 1 byte NOP + 2 byte LDA = C003), not C004!
    assert!(
        lines[2].contains("C003:"),
        "Line 3 should have address C003: {}",
        lines[2]
    );
    assert!(
        lines[2].contains("8D 00 80"),
        "Line 3 should have STA bytes: {}",
        lines[2]
    );
    assert!(
        lines[2].contains("STA $8000"),
        "Line 3 should have STA mnemonic: {}",
        lines[2]
    );
}

// T083: Integration test for hex dump with multi-line output and address increments
#[test]
fn test_hex_dump_multiline_addresses() {
    use lib6502::disassembler::formatter::format_hex_dump;

    let bytes = &[
        0xA9, 0x01, // LDA #$01 at 0x0000
        0xA9, 0x02, // LDA #$02 at 0x0002
        0xA9, 0x03, // LDA #$03 at 0x0004
        0xA9, 0x04, // LDA #$04 at 0x0006
    ];

    let options = DisassemblyOptions {
        start_address: 0x0000,
        hex_dump: true,
        show_offsets: false,
    };

    let instructions = disassemble(bytes, options);
    let output = format_hex_dump(&instructions);

    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 4, "Should have 4 lines");

    // Verify addresses increment correctly
    assert!(lines[0].contains("0000:"), "First instruction at 0000");
    assert!(lines[1].contains("0002:"), "Second instruction at 0002");
    assert!(lines[2].contains("0004:"), "Third instruction at 0004");
    assert!(lines[3].contains("0006:"), "Fourth instruction at 0006");

    // Verify each line has the expected format
    for line in &lines {
        assert!(line.contains(":"), "Should have address separator");
        assert!(line.contains("A9"), "Should have opcode bytes");
        assert!(line.contains("LDA"), "Should have mnemonic");
    }
}
