//! # ALU (Arithmetic Logic Unit) Instructions
//!
//! This module implements arithmetic and logical operations:
//! - ADC: Add with Carry
//! - AND: Logical AND
//! - (Future: SBC, ORA, EOR, CMP, CPX, CPY, BIT)

use crate::{ExecutionError, MemoryBus, OPCODE_TABLE, CPU};

/// Executes the ADC (Add with Carry) instruction.
///
/// Adds the value at the effective address (determined by addressing mode)
/// plus the carry flag to the accumulator. Updates all relevant flags.
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this ADC instruction
pub(crate) fn execute_adc<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Get the operand value and check for page crossing
    let (value, page_crossed) = cpu.get_operand_value(metadata.addressing_mode)?;

    // Perform the ADC operation
    let a = cpu.a;
    let carry_in = if cpu.flag_c { 1 } else { 0 };

    // Perform addition with carry
    let result16 = a as u16 + value as u16 + carry_in as u16;
    let result = result16 as u8;

    // Update flags

    // Carry flag: Set if result > 255
    cpu.flag_c = result16 > 0xFF;

    // Zero flag: Set if result is 0
    cpu.flag_z = result == 0;

    // Negative flag: Set if bit 7 of result is set
    cpu.flag_n = (result & 0x80) != 0;

    // Overflow flag: Set if sign bit is incorrect
    // Overflow occurs when:
    // - Adding two positive numbers yields a negative result, or
    // - Adding two negative numbers yields a positive result
    // Formula: V = (A^result) & (M^result) & 0x80
    // This checks if both operands had same sign but result has different sign
    let overflow = ((a ^ result) & (value ^ result) & 0x80) != 0;
    cpu.flag_v = overflow;

    // Store result in accumulator
    cpu.a = result;

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

/// Executes the AND (Logical AND) instruction.
///
/// Performs a bitwise AND operation between the accumulator and the value at
/// the effective address (determined by addressing mode). Updates Z and N flags.
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this AND instruction
pub(crate) fn execute_and<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Get the operand value and check for page crossing
    let (value, page_crossed) = cpu.get_operand_value(metadata.addressing_mode)?;

    // Perform the AND operation
    let result = cpu.a & value;

    // Update flags

    // Zero flag: Set if result is 0
    cpu.flag_z = result == 0;

    // Negative flag: Set if bit 7 of result is set
    cpu.flag_n = (result & 0x80) != 0;

    // Store result in accumulator
    cpu.a = result;

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
