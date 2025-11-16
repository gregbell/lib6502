//! Basic disassembler usage example

use cpu6502::disassembler::formatter::format_instruction;
use cpu6502::disassembler::{disassemble, DisassemblyOptions};

fn main() {
    // Example machine code bytes
    let code = &[
        0xA9, 0x42, // LDA #$42
        0x8D, 0x00, 0x80, // STA $8000
        0x4C, 0x00, 0x80, // JMP $8000
    ];

    // Disassemble with default options
    let options = DisassemblyOptions {
        start_address: 0x8000,
        hex_dump: false,
        show_offsets: false,
    };

    let instructions = disassemble(code, options);

    // Print instructions
    println!("Disassembled code:");
    for instr in instructions {
        println!("{:04X}: {}", instr.address, format_instruction(&instr));
    }
}
