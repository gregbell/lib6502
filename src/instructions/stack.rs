//! # Stack Operations
//!
//! This module implements stack manipulation instructions:
//! - PHA: Push Accumulator on Stack
//! - PHP: Push Processor Status on Stack
//! - (Future: PLA, PLP)
//!
//! The 6502 stack is located at memory addresses 0x0100-0x01FF and grows downward.
//! The stack pointer (SP) is an 8-bit register that serves as an offset into this
//! page. The full stack address is calculated as 0x0100 | SP.

use crate::{ExecutionError, MemoryBus, CPU, OPCODE_TABLE};

/// Executes the PHA (Push Accumulator) instruction.
///
/// PHA pushes a copy of the accumulator onto the stack and decrements the
/// stack pointer.
///
/// Stack operation:
/// 1. Write accumulator value to 0x0100 | SP
/// 2. Decrement SP (wraps from 0x00 to 0xFF)
///
/// Addressing Mode: Implicit (opcode 0x48)
/// Bytes: 1
/// Cycles: 3
///
/// Flags affected: None
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this PHA instruction (0x48)
///
/// # Examples
///
/// ```
/// use cpu6502::{CPU, FlatMemory, MemoryBus};
///
/// let mut memory = FlatMemory::new();
/// memory.write(0xFFFC, 0x00);
/// memory.write(0xFFFD, 0x80);
/// memory.write(0x8000, 0x48); // PHA
///
/// let mut cpu = CPU::new(memory);
/// cpu.set_a(0x42);
///
/// cpu.step().unwrap();
///
/// // Stack should contain the accumulator value
/// assert_eq!(cpu.memory_mut().read(0x01FD), 0x42);
/// assert_eq!(cpu.sp(), 0xFC);
/// assert_eq!(cpu.pc(), 0x8001);
/// assert_eq!(cpu.cycles(), 3);
/// ```
pub(crate) fn execute_pha<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Push accumulator to stack
    let stack_addr = 0x0100 | (cpu.sp as u16);
    cpu.memory.write(stack_addr, cpu.a);
    cpu.sp = cpu.sp.wrapping_sub(1);

    // Advance PC by instruction size (1 byte for implicit addressing)
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    // Add cycles (3 cycles for PHA)
    cpu.cycles += metadata.base_cycles as u64;

    Ok(())
}

/// Executes the PHP (Push Processor Status) instruction.
///
/// PHP pushes a copy of the processor status flags onto the stack and decrements
/// the stack pointer.
///
/// **Important**: The pushed value has bits 4 (B) and 5 (unused) set to 1, regardless
/// of the actual state of the B flag in the CPU. This is a hardware quirk of the 6502.
///
/// Stack operation:
/// 1. Get status byte with B flag (bit 4) set to 1
/// 2. Write status to 0x0100 | SP
/// 3. Decrement SP (wraps from 0x00 to 0xFF)
///
/// Addressing Mode: Implicit (opcode 0x08)
/// Bytes: 1
/// Cycles: 3
///
/// Flags affected: None (the CPU flags are not modified, only pushed)
///
/// # Arguments
///
/// * `cpu` - Mutable reference to the CPU
/// * `opcode` - The opcode byte for this PHP instruction (0x08)
///
/// # Examples
///
/// ```
/// use cpu6502::{CPU, FlatMemory, MemoryBus};
///
/// let mut memory = FlatMemory::new();
/// memory.write(0xFFFC, 0x00);
/// memory.write(0xFFFD, 0x80);
/// memory.write(0x8000, 0x08); // PHP
///
/// let mut cpu = CPU::new(memory);
/// cpu.set_flag_c(true);
/// cpu.set_flag_z(true);
///
/// cpu.step().unwrap();
///
/// // Stack should contain the status byte
/// let status = cpu.memory_mut().read(0x01FD);
/// assert_eq!(status & 0b00000001, 0b00000001); // Carry set
/// assert_eq!(status & 0b00000010, 0b00000010); // Zero set
/// assert_eq!(status & 0b00110000, 0b00110000); // Bits 4 and 5 set
/// assert_eq!(cpu.sp(), 0xFC);
/// assert_eq!(cpu.pc(), 0x8001);
/// assert_eq!(cpu.cycles(), 3);
/// ```
pub(crate) fn execute_php<M: MemoryBus>(
    cpu: &mut CPU<M>,
    opcode: u8,
) -> Result<(), ExecutionError> {
    let metadata = &OPCODE_TABLE[opcode as usize];

    // Get status byte and set bit 4 (B flag) to 1
    // Bit 5 is already set to 1 by the status() method
    let status = cpu.status() | 0b00010000;

    // Push status to stack
    let stack_addr = 0x0100 | (cpu.sp as u16);
    cpu.memory.write(stack_addr, status);
    cpu.sp = cpu.sp.wrapping_sub(1);

    // Advance PC by instruction size (1 byte for implicit addressing)
    cpu.pc = cpu.pc.wrapping_add(metadata.size_bytes as u16);

    // Add cycles (3 cycles for PHP)
    cpu.cycles += metadata.base_cycles as u64;

    Ok(())
}
