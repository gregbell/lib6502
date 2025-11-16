//! # Load and Store Instructions
//!
//! This module implements load and store operations:
//! - LDA: Load Accumulator
//! - LDX: Load X Register
//! - LDY: Load Y Register
//! - STA: Store Accumulator
//! - STX: Store X Register
//! - STY: Store Y Register

use crate::{ExecutionError, MemoryBus, CPU, OPCODE_TABLE};

/// Executes the LDA (Load Accumulator) instruction.
///
/// Loads a byte of memory into the accumulator, setting the zero and negative
/// flags as appropriate.
///
/// # Flag Behavior
///
/// - Zero (Z): Set if A = 0
/// - Negative (N): Set if bit 7 of A is set
/// - Other flags: Not affected
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this LDA instruction
pub(crate) fn execute_lda<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Get the operand value and check for page crossing
    let (value, page_crossed) = cpu.get_operand_value(metadata.addressing_mode)?;

    // Load value into accumulator
    cpu.a = value;

    // Update flags

    // Zero flag: Set if A = 0
    cpu.flag_z = value == 0;

    // Negative flag: Set if bit 7 of A is set
    cpu.flag_n = (value & 0x80) != 0;

    // Update cycle count (add extra cycle for page crossing if applicable)
    let mut cycles = metadata.base_cycles as u64;
    if page_crossed {
        cycles += 1;
    }
    cpu.cycles += cycles;

    // Advance PC
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    Ok(())
}

/// Executes the LDX (Load X Register) instruction.
///
/// Loads a byte of memory into the X register, setting the zero and negative
/// flags as appropriate.
///
/// # Flag Behavior
///
/// - Zero (Z): Set if X = 0
/// - Negative (N): Set if bit 7 of X is set
/// - Other flags: Not affected
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this LDX instruction
pub(crate) fn execute_ldx<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Get the operand value and check for page crossing
    let (value, page_crossed) = cpu.get_operand_value(metadata.addressing_mode)?;

    // Load value into X register
    cpu.x = value;

    // Update flags

    // Zero flag: Set if X = 0
    cpu.flag_z = value == 0;

    // Negative flag: Set if bit 7 of X is set
    cpu.flag_n = (value & 0x80) != 0;

    // Update cycle count (add extra cycle for page crossing if applicable)
    let mut cycles = metadata.base_cycles as u64;
    if page_crossed {
        cycles += 1;
    }
    cpu.cycles += cycles;

    // Advance PC
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    Ok(())
}

/// Executes the LDY (Load Y Register) instruction.
///
/// Loads a byte of memory into the Y register, setting the zero and negative
/// flags as appropriate.
///
/// # Flag Behavior
///
/// - Zero (Z): Set if Y = 0
/// - Negative (N): Set if bit 7 of Y is set
/// - Other flags: Not affected
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this LDY instruction
pub(crate) fn execute_ldy<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Get the operand value and check for page crossing
    let (value, page_crossed) = cpu.get_operand_value(metadata.addressing_mode)?;

    // Load value into Y register
    cpu.y = value;

    // Update flags

    // Zero flag: Set if Y = 0
    cpu.flag_z = value == 0;

    // Negative flag: Set if bit 7 of Y is set
    cpu.flag_n = (value & 0x80) != 0;

    // Update cycle count (add extra cycle for page crossing if applicable)
    let mut cycles = metadata.base_cycles as u64;
    if page_crossed {
        cycles += 1;
    }
    cpu.cycles += cycles;

    // Advance PC
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    Ok(())
}

/// Executes the STA (Store Accumulator) instruction.
///
/// Stores the contents of the accumulator into memory at the address specified
/// by the addressing mode.
///
/// # Flag Behavior
///
/// - No flags affected
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this STA instruction
pub(crate) fn execute_sta<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Get the effective address where we should store the accumulator
    let addr = cpu.get_effective_address(metadata.addressing_mode)?;

    // Store accumulator value at the effective address
    cpu.memory.write(addr, cpu.a);

    // Update cycle count (store instructions do NOT have page crossing penalties)
    cpu.cycles += metadata.base_cycles as u64;

    // Advance PC
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    Ok(())
}
