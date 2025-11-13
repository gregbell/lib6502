//! # Shift and Rotate Instructions
//!
//! This module implements bit shift and rotate operations:
//! - ASL: Arithmetic Shift Left
//! - (Future: LSR, ROL, ROR)

use crate::{AddressingMode, ExecutionError, MemoryBus, OPCODE_TABLE, CPU};

/// Executes the ASL (Arithmetic Shift Left) instruction.
///
/// Shifts all bits of the accumulator or memory contents one bit left.
/// Bit 0 is set to 0 and bit 7 is placed in the carry flag.
/// Updates C, Z, and N flags.
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this ASL instruction
pub(crate) fn execute_asl<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    let result = if metadata.addressing_mode == AddressingMode::Accumulator {
        // Accumulator mode: shift the accumulator
        let value = cpu.a;

        // Carry flag gets old bit 7
        cpu.flag_c = (value & 0x80) != 0;

        // Shift left by 1 (bit 0 becomes 0)
        let result = value << 1;

        // Update accumulator
        cpu.a = result;

        result
    } else {
        // Memory mode: read, shift, write back
        let addr = cpu.get_effective_address(metadata.addressing_mode)?;
        let value = cpu.memory.read(addr);

        // Carry flag gets old bit 7
        cpu.flag_c = (value & 0x80) != 0;

        // Shift left by 1
        let result = value << 1;

        // Write back to memory
        cpu.memory.write(addr, result);

        result
    };

    // Update Z and N flags based on result
    cpu.flag_z = result == 0;
    cpu.flag_n = (result & 0x80) != 0;

    // Update cycle count (no page crossing penalties for ASL)
    cpu.cycles += metadata.base_cycles as u64;

    // Advance PC
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    Ok(())
}
