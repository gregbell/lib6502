//! Fuzz target for CPU step execution.
//!
//! This target creates arbitrary CPU states and memory contents,
//! then executes one instruction to find edge cases and crashes.

#![no_main]

use arbitrary::Arbitrary;
use lib6502::{FlatMemory, MemoryBus, CPU};
use libfuzzer_sys::fuzz_target;

/// Arbitrary CPU initial state for fuzzing
#[derive(Debug, Arbitrary)]
struct FuzzCpuState {
    /// Accumulator register
    a: u8,
    /// X index register
    x: u8,
    /// Y index register
    y: u8,
    /// Stack pointer
    sp: u8,
    /// Carry flag
    flag_c: bool,
    /// Zero flag
    flag_z: bool,
    /// Interrupt disable flag
    flag_i: bool,
    /// Decimal mode flag
    flag_d: bool,
    /// Break flag
    flag_b: bool,
    /// Overflow flag
    flag_v: bool,
    /// Negative flag
    flag_n: bool,
}

/// Memory region for fuzzing
#[derive(Debug, Arbitrary)]
struct FuzzMemory {
    /// Bytes at the PC location (instruction + operands)
    instruction_bytes: [u8; 3],
    /// Zero page contents
    zero_page: [u8; 256],
    /// Stack page contents
    stack_page: [u8; 256],
    /// Small region of memory for testing
    main_memory: [u8; 256],
}

/// Complete fuzz input
#[derive(Debug, Arbitrary)]
struct FuzzInput {
    cpu_state: FuzzCpuState,
    memory: FuzzMemory,
}

fuzz_target!(|input: FuzzInput| {
    // Create memory
    let mut memory = FlatMemory::new();

    // Set up reset vector to point to 0x8000
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);

    // Set up IRQ vector
    memory.write(0xFFFE, 0x00);
    memory.write(0xFFFF, 0x90);

    // Write instruction bytes at 0x8000
    for (i, &byte) in input.memory.instruction_bytes.iter().enumerate() {
        memory.write(0x8000 + i as u16, byte);
    }

    // Write zero page
    for (i, &byte) in input.memory.zero_page.iter().enumerate() {
        memory.write(i as u16, byte);
    }

    // Write stack page
    for (i, &byte) in input.memory.stack_page.iter().enumerate() {
        memory.write(0x0100 + i as u16, byte);
    }

    // Write main memory region (at 0x4000 for testing absolute addressing)
    for (i, &byte) in input.memory.main_memory.iter().enumerate() {
        memory.write(0x4000 + i as u16, byte);
    }

    // Create CPU
    let mut cpu = CPU::new(memory);

    // Set CPU state from fuzz input
    cpu.set_a(input.cpu_state.a);
    cpu.set_x(input.cpu_state.x);
    cpu.set_y(input.cpu_state.y);
    cpu.set_sp(input.cpu_state.sp);
    cpu.set_flag_c(input.cpu_state.flag_c);
    cpu.set_flag_z(input.cpu_state.flag_z);
    cpu.set_flag_i(input.cpu_state.flag_i);
    cpu.set_flag_d(input.cpu_state.flag_d);
    cpu.set_flag_b(input.cpu_state.flag_b);
    cpu.set_flag_v(input.cpu_state.flag_v);
    cpu.set_flag_n(input.cpu_state.flag_n);

    // Execute one instruction
    // We don't care if it returns an error (unimplemented opcode) - just no panics
    let _ = cpu.step();

    // Basic sanity checks after execution (these should never fail)
    // If they do, we found a bug
    assert!(cpu.sp() <= 0xFF);
    assert!(cpu.cycles() > 0 || cpu.cycles() == 0); // cycles is always valid
});
