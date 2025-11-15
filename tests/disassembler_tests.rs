//! Integration tests for the 6502 disassembler

use cpu6502::addressing::AddressingMode;
use cpu6502::disassembler::{disassemble, DisassemblyOptions};
use cpu6502::disassembler::formatter::format_instruction;

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
        0xA9, 0x42,       // LDA #$42
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
        0xFF,       // Illegal opcode
        0xEA,       // NOP (valid)
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
        0xA9, 0x42,       // LDA #$42
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
