//! # ALU (Arithmetic Logic Unit) Instructions
//!
//! This module implements arithmetic and logical operations:
//! - ADC: Add with Carry
//! - AND: Logical AND
//! - BIT: Bit Test
//! - CMP: Compare Accumulator
//! - CPX: Compare X Register
//! - CPY: Compare Y Register
//! - EOR: Exclusive OR
//! - ORA: Logical Inclusive OR
//! - (Future: SBC)

use crate::{ExecutionError, MemoryBus, CPU, OPCODE_TABLE};

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

/// Executes the EOR (Exclusive OR) instruction.
///
/// Performs a bitwise exclusive OR operation between the accumulator and the value at
/// the effective address (determined by addressing mode). Updates Z and N flags.
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this EOR instruction
pub(crate) fn execute_eor<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Get the operand value and check for page crossing
    let (value, page_crossed) = cpu.get_operand_value(metadata.addressing_mode)?;

    // Perform the EOR operation
    let result = cpu.a ^ value;

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

/// Executes the ORA (Logical Inclusive OR) instruction.
///
/// Performs a bitwise inclusive OR operation between the accumulator and the value at
/// the effective address (determined by addressing mode). Updates Z and N flags.
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this ORA instruction
pub(crate) fn execute_ora<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Get the operand value and check for page crossing
    let (value, page_crossed) = cpu.get_operand_value(metadata.addressing_mode)?;

    // Perform the ORA operation
    let result = cpu.a | value;

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

/// Executes the BIT (Bit Test) instruction.
///
/// Tests bits in memory with the accumulator. The result of A & M is used to
/// set the Z flag, but the result is not stored. Bits 7 and 6 of the memory
/// value are copied directly into the N and V flags respectively.
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this BIT instruction
pub(crate) fn execute_bit<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Get the memory value (BIT doesn't have page crossing penalties)
    let (value, _page_crossed) = cpu.get_operand_value(metadata.addressing_mode)?;

    // Perform the AND operation for the Z flag (but don't store result)
    let result = cpu.a & value;

    // Update flags

    // Zero flag: Set if (A & M) is 0
    cpu.flag_z = result == 0;

    // Negative flag: Set to bit 7 of memory value
    cpu.flag_n = (value & 0x80) != 0;

    // Overflow flag: Set to bit 6 of memory value
    cpu.flag_v = (value & 0x40) != 0;

    // Note: A is NOT modified - result is discarded

    // Update cycle count (BIT has no page crossing penalty)
    cpu.cycles += metadata.base_cycles as u64;

    // Advance PC
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    Ok(())
}

/// Executes the CMP (Compare Accumulator) instruction.
///
/// Compares the accumulator with the value at the effective address by performing
/// a subtraction (A - M) and setting flags based on the result. The accumulator
/// is NOT modified.
///
/// # Flag Behavior
///
/// - Carry (C): Set if A >= M (no borrow needed)
/// - Zero (Z): Set if A == M (result is zero)
/// - Negative (N): Set if bit 7 of the result is set
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this CMP instruction
pub(crate) fn execute_cmp<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Get the operand value and check for page crossing
    let (value, page_crossed) = cpu.get_operand_value(metadata.addressing_mode)?;

    // Perform the comparison (A - M)
    // The subtraction is: A - M, which is equivalent to A + (!M) + 1
    let a = cpu.a;
    let result = a.wrapping_sub(value);

    // Update flags

    // Carry flag: Set if A >= M (no borrow needed)
    // In subtraction, carry is set when no borrow occurs
    cpu.flag_c = a >= value;

    // Zero flag: Set if A == M (result is zero)
    cpu.flag_z = result == 0;

    // Negative flag: Set if bit 7 of result is set
    cpu.flag_n = (result & 0x80) != 0;

    // Note: Accumulator is NOT modified - this is a comparison only

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

/// Executes the CPX (Compare X Register) instruction.
///
/// Compares the X register with the value at the effective address by performing
/// a subtraction (X - M) and setting flags based on the result. The X register
/// is NOT modified.
///
/// # Flag Behavior
///
/// - Carry (C): Set if X >= M (no borrow needed)
/// - Zero (Z): Set if X == M (result is zero)
/// - Negative (N): Set if bit 7 of the result is set
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this CPX instruction
pub(crate) fn execute_cpx<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Get the operand value (CPX doesn't have page crossing penalties)
    let (value, _page_crossed) = cpu.get_operand_value(metadata.addressing_mode)?;

    // Perform the comparison (X - M)
    let x = cpu.x;
    let result = x.wrapping_sub(value);

    // Update flags

    // Carry flag: Set if X >= M (no borrow needed)
    // In subtraction, carry is set when no borrow occurs
    cpu.flag_c = x >= value;

    // Zero flag: Set if X == M (result is zero)
    cpu.flag_z = result == 0;

    // Negative flag: Set if bit 7 of result is set
    cpu.flag_n = (result & 0x80) != 0;

    // Note: X register is NOT modified - this is a comparison only

    // Update cycle count (CPX has no page crossing penalty)
    cpu.cycles += metadata.base_cycles as u64;

    // Advance PC
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    Ok(())
}

/// Executes the CPY (Compare Y Register) instruction.
///
/// Compares the Y register with the value at the effective address by performing
/// a subtraction (Y - M) and setting flags based on the result. The Y register
/// is NOT modified.
///
/// # Flag Behavior
///
/// - Carry (C): Set if Y >= M (no borrow needed)
/// - Zero (Z): Set if Y == M (result is zero)
/// - Negative (N): Set if bit 7 of the result is set
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this CPY instruction
pub(crate) fn execute_cpy<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Get the operand value (CPY doesn't have page crossing penalties)
    let (value, _page_crossed) = cpu.get_operand_value(metadata.addressing_mode)?;

    // Perform the comparison (Y - M)
    let y = cpu.y;
    let result = y.wrapping_sub(value);

    // Update flags

    // Carry flag: Set if Y >= M (no borrow needed)
    // In subtraction, carry is set when no borrow occurs
    cpu.flag_c = y >= value;

    // Zero flag: Set if Y == M (result is zero)
    cpu.flag_z = result == 0;

    // Negative flag: Set if bit 7 of result is set
    cpu.flag_n = (result & 0x80) != 0;

    // Note: Y register is NOT modified - this is a comparison only

    // Update cycle count (CPY has no page crossing penalty)
    cpu.cycles += metadata.base_cycles as u64;

    // Advance PC
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    Ok(())
}
