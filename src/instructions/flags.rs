//! # Status Flag Manipulation Instructions
//!
//! This module implements instructions that directly modify processor status flags:
//! - CLC: Clear Carry Flag
//! - SEC: Set Carry Flag
//! - CLI: Clear Interrupt Disable
//! - SEI: Set Interrupt Disable
//! - CLV: Clear Overflow Flag
//! - CLD: Clear Decimal Mode
//! - SED: Set Decimal Mode
//!
//! These instructions use implied addressing mode and execute in 2 cycles.

use crate::{ExecutionError, MemoryBus, CPU, OPCODE_TABLE};

/// Executes the CLC (Clear Carry Flag) instruction.
///
/// Sets the carry flag to 0.
///
/// Addressing Mode: Implied
/// Opcode: 0x18
/// Bytes: 1
/// Cycles: 2
///
/// Flags affected:
/// - C: Set to 0
/// - All other flags: Unchanged
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this CLC instruction (0x18)
///
/// # Examples
///
/// ```
/// use cpu6502::{CPU, FlatMemory, MemoryBus};
///
/// let mut memory = FlatMemory::new();
/// memory.write(0xFFFC, 0x00);
/// memory.write(0xFFFD, 0x80);
/// memory.write(0x8000, 0x18); // CLC
///
/// let mut cpu = CPU::new(memory);
/// cpu.set_flag_c(true); // Set carry flag
///
/// cpu.step().unwrap();
///
/// assert_eq!(cpu.flag_c(), false); // Carry flag cleared
/// assert_eq!(cpu.pc(), 0x8001);
/// assert_eq!(cpu.cycles(), 2);
/// ```
pub(crate) fn execute_clc<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Clear the carry flag
    cpu.flag_c = false;

    // Advance PC by instruction size (1 byte for implicit addressing)
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    // Add cycles (2 cycles for CLC)
    cpu.cycles += metadata.base_cycles as u64;

    Ok(())
}
