//! # Increment and Decrement Instructions
//!
//! This module implements increment and decrement operations:
//! - DEC: Decrement Memory
//! - DEX: Decrement X Register
//! - DEY: Decrement Y Register
//! - (Future: INC, INX, INY)

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

/// Executes the DEX (Decrement X Register) instruction.
///
/// Subtracts one from the X register, setting the zero and negative flags as appropriate.
/// Updates Z and N flags.
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this DEX instruction
pub(crate) fn execute_dex<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Decrement the X register (wrapping on underflow)
    cpu.x = cpu.x.wrapping_sub(1);

    // Update Z and N flags based on result
    cpu.flag_z = cpu.x == 0;
    cpu.flag_n = (cpu.x & 0x80) != 0;

    // Update cycle count
    cpu.cycles += metadata.base_cycles as u64;

    // Advance PC
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    Ok(())
}

/// Executes the DEY (Decrement Y Register) instruction.
///
/// Subtracts one from the Y register, setting the zero and negative flags as appropriate.
/// Updates Z and N flags.
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this DEY instruction
pub(crate) fn execute_dey<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Decrement the Y register (wrapping on underflow)
    cpu.y = cpu.y.wrapping_sub(1);

    // Update Z and N flags based on result
    cpu.flag_z = cpu.y == 0;
    cpu.flag_n = (cpu.y & 0x80) != 0;

    // Update cycle count
    cpu.cycles += metadata.base_cycles as u64;

    // Advance PC
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    Ok(())
}
