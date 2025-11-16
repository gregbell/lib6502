//! # Register Transfer Instructions
//!
//! This module implements register transfer operations:
//! - TAX: Transfer Accumulator to X
//! - TAY: Transfer Accumulator to Y
//! - TXA: Transfer X to Accumulator
//! - TYA: Transfer Y to Accumulator
//! - TSX: Transfer Stack Pointer to X
//! - TXS: Transfer X to Stack Pointer

use crate::{ExecutionError, MemoryBus, CPU, OPCODE_TABLE};

/// Executes the TAX (Transfer Accumulator to X) instruction.
///
/// Copies the current contents of the accumulator into the X register
/// and sets the zero and negative flags as appropriate.
/// Updates Z and N flags.
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this TAX instruction
pub(crate) fn execute_tax<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Transfer accumulator to X register
    cpu.x = cpu.a;

    // Update Z and N flags based on result
    cpu.flag_z = cpu.x == 0;
    cpu.flag_n = (cpu.x & 0x80) != 0;

    // Update cycle count
    cpu.cycles += metadata.base_cycles as u64;

    // Advance PC
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    Ok(())
}
