//! # Shift and Rotate Instructions
//!
//! This module implements bit shift and rotate operations:
//! - ASL: Arithmetic Shift Left
//! - LSR: Logical Shift Right
//! - ROL: Rotate Left
//! - ROR: Rotate Right

use crate::{AddressingMode, ExecutionError, MemoryBus, CPU, OPCODE_TABLE};

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

/// Executes the LSR (Logical Shift Right) instruction.
///
/// Shifts all bits of the accumulator or memory contents one bit right.
/// Bit 7 is set to 0 and bit 0 is placed in the carry flag.
/// Updates C, Z, and N flags.
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this LSR instruction
pub(crate) fn execute_lsr<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    let result = if metadata.addressing_mode == AddressingMode::Accumulator {
        // Accumulator mode: shift the accumulator
        let value = cpu.a;

        // Carry flag gets old bit 0
        cpu.flag_c = (value & 0x01) != 0;

        // Shift right by 1 (bit 7 becomes 0)
        let result = value >> 1;

        // Update accumulator
        cpu.a = result;

        result
    } else {
        // Memory mode: read, shift, write back
        let addr = cpu.get_effective_address(metadata.addressing_mode)?;
        let value = cpu.memory.read(addr);

        // Carry flag gets old bit 0
        cpu.flag_c = (value & 0x01) != 0;

        // Shift right by 1
        let result = value >> 1;

        // Write back to memory
        cpu.memory.write(addr, result);

        result
    };

    // Update Z and N flags based on result
    cpu.flag_z = result == 0;
    cpu.flag_n = (result & 0x80) != 0;

    // Update cycle count (no page crossing penalties for LSR)
    cpu.cycles += metadata.base_cycles as u64;

    // Advance PC
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    Ok(())
}

/// Executes the ROL (Rotate Left) instruction.
///
/// Rotates all bits of the accumulator or memory contents one bit left.
/// Bit 0 is filled with the current carry flag value, and bit 7 is placed in the carry flag.
/// Updates C, Z, and N flags.
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this ROL instruction
pub(crate) fn execute_rol<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    let result = if metadata.addressing_mode == AddressingMode::Accumulator {
        // Accumulator mode: rotate the accumulator
        let value = cpu.a;

        // Save old bit 7
        let old_bit_7 = (value & 0x80) != 0;

        // Shift left by 1
        let mut result = value << 1;

        // Bit 0 gets current carry flag
        if cpu.flag_c {
            result |= 0x01;
        }

        // Carry flag gets old bit 7
        cpu.flag_c = old_bit_7;

        // Update accumulator
        cpu.a = result;

        result
    } else {
        // Memory mode: read, rotate, write back
        let addr = cpu.get_effective_address(metadata.addressing_mode)?;
        let value = cpu.memory.read(addr);

        // Save old bit 7
        let old_bit_7 = (value & 0x80) != 0;

        // Shift left by 1
        let mut result = value << 1;

        // Bit 0 gets current carry flag
        if cpu.flag_c {
            result |= 0x01;
        }

        // Carry flag gets old bit 7
        cpu.flag_c = old_bit_7;

        // Write back to memory
        cpu.memory.write(addr, result);

        result
    };

    // Update Z and N flags based on result
    cpu.flag_z = result == 0;
    cpu.flag_n = (result & 0x80) != 0;

    // Update cycle count (no page crossing penalties for ROL)
    cpu.cycles += metadata.base_cycles as u64;

    // Advance PC
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    Ok(())
}

/// Executes the ROR (Rotate Right) instruction.
///
/// Rotates all bits of the accumulator or memory contents one bit right.
/// Bit 7 is filled with the current carry flag value, and bit 0 is placed in the carry flag.
/// Updates C, Z, and N flags.
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this ROR instruction
pub(crate) fn execute_ror<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    let result = if metadata.addressing_mode == AddressingMode::Accumulator {
        // Accumulator mode: rotate the accumulator
        let value = cpu.a;

        // Save old bit 0
        let old_bit_0 = (value & 0x01) != 0;

        // Shift right by 1
        let mut result = value >> 1;

        // Bit 7 gets current carry flag
        if cpu.flag_c {
            result |= 0x80;
        }

        // Carry flag gets old bit 0
        cpu.flag_c = old_bit_0;

        // Update accumulator
        cpu.a = result;

        result
    } else {
        // Memory mode: read, rotate, write back
        let addr = cpu.get_effective_address(metadata.addressing_mode)?;
        let value = cpu.memory.read(addr);

        // Save old bit 0
        let old_bit_0 = (value & 0x01) != 0;

        // Shift right by 1
        let mut result = value >> 1;

        // Bit 7 gets current carry flag
        if cpu.flag_c {
            result |= 0x80;
        }

        // Carry flag gets old bit 0
        cpu.flag_c = old_bit_0;

        // Write back to memory
        cpu.memory.write(addr, result);

        result
    };

    // Update Z and N flags based on result
    cpu.flag_z = result == 0;
    cpu.flag_n = (result & 0x80) != 0;

    // Update cycle count (no page crossing penalties for ROR)
    cpu.cycles += metadata.base_cycles as u64;

    // Advance PC
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    Ok(())
}
