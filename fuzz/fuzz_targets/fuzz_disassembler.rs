//! Fuzz target for the disassembler.
//!
//! This target feeds arbitrary byte sequences to the disassembler
//! to find edge cases and crashes in instruction decoding.

#![no_main]

use arbitrary::Arbitrary;
use lib6502::{disassemble, DisassemblyOptions};
use libfuzzer_sys::fuzz_target;

/// Disassembly options for fuzzing
#[derive(Debug, Arbitrary)]
struct FuzzOptions {
    start_address: u16,
    hex_dump: bool,
    show_offsets: bool,
}

/// Complete fuzz input
#[derive(Debug, Arbitrary)]
struct FuzzInput {
    bytes: Vec<u8>,
    options: FuzzOptions,
}

fuzz_target!(|input: FuzzInput| {
    // Limit input size to prevent OOM
    if input.bytes.len() > 65536 {
        return;
    }

    let options = DisassemblyOptions {
        start_address: input.options.start_address,
        hex_dump: input.options.hex_dump,
        show_offsets: input.options.show_offsets,
    };

    // Disassemble the bytes
    let instructions = disassemble(&input.bytes, options);

    // Verify invariants
    let mut total_size: usize = 0;
    let mut expected_address = input.options.start_address;

    for instr in &instructions {
        // Each instruction should have correct address
        assert_eq!(instr.address, expected_address);

        // Size should be 1-3 bytes
        assert!(instr.size_bytes >= 1 && instr.size_bytes <= 3);

        // Operand bytes length should be consistent with size
        // For regular instructions: operand_bytes.len() == size_bytes - 1
        // For .byte pseudo-instructions (illegal opcodes): operand_bytes contains the byte value
        // so operand_bytes.len() may equal size_bytes
        assert!(instr.operand_bytes.len() <= instr.size_bytes as usize);

        total_size += instr.size_bytes as usize;
        expected_address = expected_address.wrapping_add(instr.size_bytes as u16);
    }

    // Total size should equal input size
    assert_eq!(total_size, input.bytes.len());
});
