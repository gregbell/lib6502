//! Comprehensive tests for the DEY (Decrement Y Register) instruction.
//!
//! Tests cover:
//! - Basic DEY operation
//! - Flag updates (Z, N)
//! - Various Y register values (0, positive, negative, edge cases)
//! - Cycle counts
//! - Wrapping behavior

use lib6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic DEY Operation Tests ==========

#[test]
fn test_dey_basic() {
    let mut cpu = setup_cpu();

    // DEY (0x88)
    cpu.memory_mut().write(0x8000, 0x88);
    cpu.set_y(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x04); // 5 - 1 = 4
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.cycles(), 2);
}

#[test]
fn test_dey_decrements_by_one() {
    let mut cpu = setup_cpu();

    // DEY (0x88)
    cpu.memory_mut().write(0x8000, 0x88);
    cpu.set_y(0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x41); // 66 - 1 = 65
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

// ========== Zero Flag Tests ==========

#[test]
fn test_dey_zero_flag_set() {
    let mut cpu = setup_cpu();

    // DEY (0x88)
    cpu.memory_mut().write(0x8000, 0x88);
    cpu.set_y(0x01); // 1 - 1 = 0

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x00);
    assert!(cpu.flag_z()); // Result is zero
    assert!(!cpu.flag_n());
}

#[test]
fn test_dey_zero_flag_clear() {
    let mut cpu = setup_cpu();

    // DEY (0x88)
    cpu.memory_mut().write(0x8000, 0x88);
    cpu.set_y(0x02);

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x01);
    assert!(!cpu.flag_z()); // Result is not zero
    assert!(!cpu.flag_n());
}

// ========== Negative Flag Tests ==========

#[test]
fn test_dey_negative_flag_set() {
    let mut cpu = setup_cpu();

    // DEY (0x88)
    cpu.memory_mut().write(0x8000, 0x88);
    cpu.set_y(0x81); // 0b10000001

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x80); // 0b10000000
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is set
}

#[test]
fn test_dey_negative_flag_clear() {
    let mut cpu = setup_cpu();

    // DEY (0x88)
    cpu.memory_mut().write(0x8000, 0x88);
    cpu.set_y(0x7F); // 0b01111111

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x7E); // 0b01111110
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n()); // Bit 7 is still 0
}

// ========== Underflow/Wrap Tests ==========

#[test]
fn test_dey_wraps_from_zero_to_ff() {
    let mut cpu = setup_cpu();

    // DEY (0x88)
    cpu.memory_mut().write(0x8000, 0x88);
    cpu.set_y(0x00); // 0 - 1 wraps to 255

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0xFF); // Wrapped to 255
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // 0xFF has bit 7 set
}

#[test]
fn test_dey_from_0x80() {
    let mut cpu = setup_cpu();

    // DEY (0x88)
    cpu.memory_mut().write(0x8000, 0x88);
    cpu.set_y(0x80); // 0b10000000

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x7F); // 0b01111111
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n()); // Bit 7 cleared
}

// ========== Edge Case Tests ==========

#[test]
fn test_dey_all_bits_set() {
    let mut cpu = setup_cpu();

    // DEY (0x88)
    cpu.memory_mut().write(0x8000, 0x88);
    cpu.set_y(0xFF); // 0b11111111

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0xFE); // 0b11111110
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is still set
}

#[test]
fn test_dey_preserves_carry_flag() {
    let mut cpu = setup_cpu();

    // DEY (0x88)
    cpu.memory_mut().write(0x8000, 0x88);
    cpu.set_y(0x05);
    cpu.set_flag_c(true); // Set carry flag

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x04);
    assert!(cpu.flag_c()); // Carry flag should be unchanged
}

#[test]
fn test_dey_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    // DEY (0x88)
    cpu.memory_mut().write(0x8000, 0x88);
    cpu.set_y(0x05);
    cpu.set_flag_v(true); // Set overflow flag

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x04);
    assert!(cpu.flag_v()); // Overflow flag should be unchanged
}

#[test]
fn test_dey_preserves_accumulator() {
    let mut cpu = setup_cpu();

    // DEY (0x88)
    cpu.memory_mut().write(0x8000, 0x88);
    cpu.set_y(0x05);
    cpu.set_a(0x42); // Set accumulator

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x04);
    assert_eq!(cpu.a(), 0x42); // Accumulator should be unchanged
}

#[test]
fn test_dey_preserves_x_register() {
    let mut cpu = setup_cpu();

    // DEY (0x88)
    cpu.memory_mut().write(0x8000, 0x88);
    cpu.set_y(0x05);
    cpu.set_x(0x99); // Set X register

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x04);
    assert_eq!(cpu.x(), 0x99); // X register should be unchanged
}

// ========== Multiple Decrements Test ==========

#[test]
fn test_dey_sequence() {
    let mut cpu = setup_cpu();

    // Set up three DEY instructions
    cpu.memory_mut().write(0x8000, 0x88);
    cpu.memory_mut().write(0x8001, 0x88);
    cpu.memory_mut().write(0x8002, 0x88);

    cpu.set_y(0x03);

    cpu.step().unwrap();
    assert_eq!(cpu.y(), 0x02);
    assert_eq!(cpu.pc(), 0x8001);
    assert!(!cpu.flag_z());

    cpu.step().unwrap();
    assert_eq!(cpu.y(), 0x01);
    assert_eq!(cpu.pc(), 0x8002);
    assert!(!cpu.flag_z());

    cpu.step().unwrap();
    assert_eq!(cpu.y(), 0x00);
    assert_eq!(cpu.pc(), 0x8003);
    assert!(cpu.flag_z()); // Result is zero
}

#[test]
fn test_dey_countdown_to_zero() {
    let mut cpu = setup_cpu();

    // Set up 10 DEY instructions
    for i in 0..10 {
        cpu.memory_mut().write(0x8000 + i, 0x88);
    }

    cpu.set_y(10);

    for i in (1..=10).rev() {
        cpu.step().unwrap();
        assert_eq!(cpu.y(), i - 1);
    }

    // Last value should have Z flag set
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_dey_underflow_sequence() {
    let mut cpu = setup_cpu();

    // DEY from 0 to 0xFF
    cpu.memory_mut().write(0x8000, 0x88);
    // DEY from 0xFF to 0xFE
    cpu.memory_mut().write(0x8001, 0x88);

    cpu.set_y(0x00);

    cpu.step().unwrap();
    assert_eq!(cpu.y(), 0xFF); // Wrapped to 255
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());

    cpu.step().unwrap();
    assert_eq!(cpu.y(), 0xFE);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
}

// ========== Cycle Count Tests ==========

#[test]
fn test_dey_cycle_count() {
    let mut cpu = setup_cpu();

    // DEY (0x88)
    cpu.memory_mut().write(0x8000, 0x88);
    cpu.set_y(0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.cycles(), 2); // DEY takes 2 cycles
}

#[test]
fn test_dey_multiple_cycle_count() {
    let mut cpu = setup_cpu();

    // Set up 5 DEY instructions
    for i in 0..5 {
        cpu.memory_mut().write(0x8000 + i, 0x88);
    }

    cpu.set_y(0x10);

    for i in 1..=5 {
        cpu.step().unwrap();
        assert_eq!(cpu.cycles(), (i * 2) as u64); // Each DEY takes 2 cycles
    }
}

// ========== Program Counter Tests ==========

#[test]
fn test_dey_program_counter_advance() {
    let mut cpu = setup_cpu();

    // DEY (0x88) - single byte instruction
    cpu.memory_mut().write(0x8000, 0x88);
    cpu.set_y(0x10);

    let initial_pc = cpu.pc();
    cpu.step().unwrap();

    assert_eq!(cpu.pc(), initial_pc + 1); // PC advances by 1 byte
}
