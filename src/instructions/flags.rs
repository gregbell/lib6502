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

/// Executes the SEC (Set Carry Flag) instruction.
///
/// Sets the carry flag to 1.
///
/// Addressing Mode: Implied
/// Opcode: 0x38
/// Bytes: 1
/// Cycles: 2
///
/// Flags affected:
/// - C: Set to 1
/// - All other flags: Unchanged
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this SEC instruction (0x38)
///
/// # Examples
///
/// ```
/// use cpu6502::{CPU, FlatMemory, MemoryBus};
///
/// let mut memory = FlatMemory::new();
/// memory.write(0xFFFC, 0x00);
/// memory.write(0xFFFD, 0x80);
/// memory.write(0x8000, 0x38); // SEC
///
/// let mut cpu = CPU::new(memory);
/// cpu.set_flag_c(false); // Clear carry flag
///
/// cpu.step().unwrap();
///
/// assert_eq!(cpu.flag_c(), true); // Carry flag set
/// assert_eq!(cpu.pc(), 0x8001);
/// assert_eq!(cpu.cycles(), 2);
/// ```
pub(crate) fn execute_sec<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Set the carry flag
    cpu.flag_c = true;

    // Advance PC by instruction size (1 byte for implicit addressing)
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    // Add cycles (2 cycles for SEC)
    cpu.cycles += metadata.base_cycles as u64;

    Ok(())
}

/// Executes the CLI (Clear Interrupt Disable) instruction.
///
/// Sets the interrupt disable flag to 0, allowing normal interrupt requests to be serviced.
///
/// Addressing Mode: Implied
/// Opcode: 0x58
/// Bytes: 1
/// Cycles: 2
///
/// Flags affected:
/// - I: Set to 0
/// - All other flags: Unchanged
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this CLI instruction (0x58)
///
/// # Examples
///
/// ```
/// use cpu6502::{CPU, FlatMemory, MemoryBus};
///
/// let mut memory = FlatMemory::new();
/// memory.write(0xFFFC, 0x00);
/// memory.write(0xFFFD, 0x80);
/// memory.write(0x8000, 0x58); // CLI
///
/// let mut cpu = CPU::new(memory);
/// cpu.set_flag_i(true); // Set interrupt disable flag
///
/// cpu.step().unwrap();
///
/// assert_eq!(cpu.flag_i(), false); // Interrupt disable flag cleared
/// assert_eq!(cpu.pc(), 0x8001);
/// assert_eq!(cpu.cycles(), 2);
/// ```
pub(crate) fn execute_cli<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Clear the interrupt disable flag
    cpu.flag_i = false;

    // Advance PC by instruction size (1 byte for implicit addressing)
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    // Add cycles (2 cycles for CLI)
    cpu.cycles += metadata.base_cycles as u64;

    Ok(())
}

/// Executes the SEI (Set Interrupt Disable) instruction.
///
/// Sets the interrupt disable flag to 1, preventing the CPU from responding to IRQ interrupts.
///
/// Addressing Mode: Implied
/// Opcode: 0x78
/// Bytes: 1
/// Cycles: 2
///
/// Flags affected:
/// - I: Set to 1
/// - All other flags: Unchanged
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this SEI instruction (0x78)
///
/// # Examples
///
/// ```
/// use cpu6502::{CPU, FlatMemory, MemoryBus};
///
/// let mut memory = FlatMemory::new();
/// memory.write(0xFFFC, 0x00);
/// memory.write(0xFFFD, 0x80);
/// memory.write(0x8000, 0x78); // SEI
///
/// let mut cpu = CPU::new(memory);
/// cpu.set_flag_i(false); // Clear interrupt disable flag
///
/// cpu.step().unwrap();
///
/// assert_eq!(cpu.flag_i(), true); // Interrupt disable flag set
/// assert_eq!(cpu.pc(), 0x8001);
/// assert_eq!(cpu.cycles(), 2);
/// ```
pub(crate) fn execute_sei<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Set the interrupt disable flag
    cpu.flag_i = true;

    // Advance PC by instruction size (1 byte for implicit addressing)
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    // Add cycles (2 cycles for SEI)
    cpu.cycles += metadata.base_cycles as u64;

    Ok(())
}

/// Executes the CLD (Clear Decimal Mode) instruction.
///
/// Sets the decimal mode flag to 0.
///
/// Addressing Mode: Implied
/// Opcode: 0xD8
/// Bytes: 1
/// Cycles: 2
///
/// Flags affected:
/// - D: Set to 0
/// - All other flags: Unchanged
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this CLD instruction (0xD8)
///
/// # Examples
///
/// ```
/// use cpu6502::{CPU, FlatMemory, MemoryBus};
///
/// let mut memory = FlatMemory::new();
/// memory.write(0xFFFC, 0x00);
/// memory.write(0xFFFD, 0x80);
/// memory.write(0x8000, 0xD8); // CLD
///
/// let mut cpu = CPU::new(memory);
/// cpu.set_flag_d(true); // Set decimal mode flag
///
/// cpu.step().unwrap();
///
/// assert_eq!(cpu.flag_d(), false); // Decimal mode flag cleared
/// assert_eq!(cpu.pc(), 0x8001);
/// assert_eq!(cpu.cycles(), 2);
/// ```
pub(crate) fn execute_cld<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Clear the decimal mode flag
    cpu.flag_d = false;

    // Advance PC by instruction size (1 byte for implicit addressing)
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    // Add cycles (2 cycles for CLD)
    cpu.cycles += metadata.base_cycles as u64;

    Ok(())
}

/// Executes the SED (Set Decimal Mode) instruction.
///
/// Sets the decimal mode flag to 1.
///
/// Addressing Mode: Implied
/// Opcode: 0xF8
/// Bytes: 1
/// Cycles: 2
///
/// Flags affected:
/// - D: Set to 1
/// - All other flags: Unchanged
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this SED instruction (0xF8)
///
/// # Examples
///
/// ```
/// use cpu6502::{CPU, FlatMemory, MemoryBus};
///
/// let mut memory = FlatMemory::new();
/// memory.write(0xFFFC, 0x00);
/// memory.write(0xFFFD, 0x80);
/// memory.write(0x8000, 0xF8); // SED
///
/// let mut cpu = CPU::new(memory);
/// cpu.set_flag_d(false); // Clear decimal mode flag
///
/// cpu.step().unwrap();
///
/// assert_eq!(cpu.flag_d(), true); // Decimal mode flag set
/// assert_eq!(cpu.pc(), 0x8001);
/// assert_eq!(cpu.cycles(), 2);
/// ```
pub(crate) fn execute_sed<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Set the decimal mode flag
    cpu.flag_d = true;

    // Advance PC by instruction size (1 byte for implicit addressing)
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    // Add cycles (2 cycles for SED)
    cpu.cycles += metadata.base_cycles as u64;

    Ok(())
}

/// Executes the CLV (Clear Overflow Flag) instruction.
///
/// Sets the overflow flag to 0.
///
/// Addressing Mode: Implied
/// Opcode: 0xB8
/// Bytes: 1
/// Cycles: 2
///
/// Flags affected:
/// - V: Set to 0
/// - All other flags: Unchanged
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this CLV instruction (0xB8)
///
/// # Examples
///
/// ```
/// use cpu6502::{CPU, FlatMemory, MemoryBus};
///
/// let mut memory = FlatMemory::new();
/// memory.write(0xFFFC, 0x00);
/// memory.write(0xFFFD, 0x80);
/// memory.write(0x8000, 0xB8); // CLV
///
/// let mut cpu = CPU::new(memory);
/// cpu.set_flag_v(true); // Set overflow flag
///
/// cpu.step().unwrap();
///
/// assert_eq!(cpu.flag_v(), false); // Overflow flag cleared
/// assert_eq!(cpu.pc(), 0x8001);
/// assert_eq!(cpu.cycles(), 2);
/// ```
pub(crate) fn execute_clv<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Clear the overflow flag
    cpu.flag_v = false;

    // Advance PC by instruction size (1 byte for implicit addressing)
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    // Add cycles (2 cycles for CLV)
    cpu.cycles += metadata.base_cycles as u64;

    Ok(())
}
