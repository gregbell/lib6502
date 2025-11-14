//! # Increment and Decrement Instructions
//!
//! This module implements increment and decrement operations:
//! - DEC: Decrement Memory
//! - (Future: INC, INX, INY, DEX, DEY)

use crate::{ExecutionError, MemoryBus, CPU, OPCODE_TABLE};

/// Executes the DEC (Decrement Memory) instruction.
///
/// Subtracts one from the value held at a specified memory location,
/// setting the zero and negative flags as appropriate.
/// Updates Z and N flags.
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this DEC instruction
pub(crate) fn execute_dec<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Get the memory address to decrement
    let addr = cpu.get_effective_address(metadata.addressing_mode)?;
    let value = cpu.memory.read(addr);

    // Decrement the value (wrapping on underflow)
    let result = value.wrapping_sub(1);

    // Write back to memory
    cpu.memory.write(addr, result);

    // Update Z and N flags based on result
    cpu.flag_z = result == 0;
    cpu.flag_n = (result & 0x80) != 0;

    // Update cycle count (no page crossing penalties for DEC)
    cpu.cycles += metadata.base_cycles as u64;

    // Advance PC
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    Ok(())
}
