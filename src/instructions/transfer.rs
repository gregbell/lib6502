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

/// Executes the TAY (Transfer Accumulator to Y) instruction.
///
/// Copies the current contents of the accumulator into the Y register
/// and sets the zero and negative flags as appropriate.
/// Updates Z and N flags.
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this TAY instruction
pub(crate) fn execute_tay<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Transfer accumulator to Y register
    cpu.y = cpu.a;

    // Update Z and N flags based on result
    cpu.flag_z = cpu.y == 0;
    cpu.flag_n = (cpu.y & 0x80) != 0;

    // Update cycle count
    cpu.cycles += metadata.base_cycles as u64;

    // Advance PC
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    Ok(())
}

/// Executes the TSX (Transfer Stack Pointer to X) instruction.
///
/// Copies the current contents of the stack pointer into the X register
/// and sets the zero and negative flags as appropriate.
/// Updates Z and N flags.
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this TSX instruction
pub(crate) fn execute_tsx<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Transfer stack pointer to X register
    cpu.x = cpu.sp;

    // Update Z and N flags based on result
    cpu.flag_z = cpu.x == 0;
    cpu.flag_n = (cpu.x & 0x80) != 0;

    // Update cycle count
    cpu.cycles += metadata.base_cycles as u64;

    // Advance PC
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    Ok(())
}

/// Executes the TXA (Transfer X to Accumulator) instruction.
///
/// Copies the current contents of the X register into the accumulator
/// and sets the zero and negative flags as appropriate.
/// Updates Z and N flags.
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this TXA instruction
pub(crate) fn execute_txa<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Transfer X register to accumulator
    cpu.a = cpu.x;

    // Update Z and N flags based on result
    cpu.flag_z = cpu.a == 0;
    cpu.flag_n = (cpu.a & 0x80) != 0;

    // Update cycle count
    cpu.cycles += metadata.base_cycles as u64;

    // Advance PC
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    Ok(())
}
