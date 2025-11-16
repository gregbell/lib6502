//! Comprehensive tests for the DEX (Decrement X Register) instruction.
//!
//! Tests cover:
//! - Basic DEX operation
//! - Flag updates (Z, N)
//! - Various X register values (0, positive, negative, edge cases)
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

// ========== Basic DEX Operation Tests ==========

#[test]
fn test_dex_basic() {
    let mut cpu = setup_cpu();

    // DEX (0xCA)
    cpu.memory_mut().write(0x8000, 0xCA);
    cpu.set_x(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x04); // 5 - 1 = 4
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.cycles(), 2);
}

#[test]
fn test_dex_decrements_by_one() {
    let mut cpu = setup_cpu();

    // DEX (0xCA)
    cpu.memory_mut().write(0x8000, 0xCA);
    cpu.set_x(0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x41); // 66 - 1 = 65
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

// ========== Zero Flag Tests ==========

#[test]
fn test_dex_zero_flag_set() {
    let mut cpu = setup_cpu();

    // DEX (0xCA)
    cpu.memory_mut().write(0x8000, 0xCA);
    cpu.set_x(0x01); // 1 - 1 = 0

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x00);
    assert!(cpu.flag_z()); // Result is zero
    assert!(!cpu.flag_n());
}

#[test]
fn test_dex_zero_flag_clear() {
    let mut cpu = setup_cpu();

    // DEX (0xCA)
    cpu.memory_mut().write(0x8000, 0xCA);
    cpu.set_x(0x02);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x01);
    assert!(!cpu.flag_z()); // Result is not zero
    assert!(!cpu.flag_n());
}

// ========== Negative Flag Tests ==========

#[test]
fn test_dex_negative_flag_set() {
    let mut cpu = setup_cpu();

    // DEX (0xCA)
    cpu.memory_mut().write(0x8000, 0xCA);
    cpu.set_x(0x81); // 0b10000001

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x80); // 0b10000000
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is set
}

#[test]
fn test_dex_negative_flag_clear() {
    let mut cpu = setup_cpu();

    // DEX (0xCA)
    cpu.memory_mut().write(0x8000, 0xCA);
    cpu.set_x(0x7F); // 0b01111111

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x7E); // 0b01111110
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n()); // Bit 7 is still 0
}

// ========== Underflow/Wrap Tests ==========

#[test]
fn test_dex_wraps_from_zero_to_ff() {
    let mut cpu = setup_cpu();

    // DEX (0xCA)
    cpu.memory_mut().write(0x8000, 0xCA);
    cpu.set_x(0x00); // 0 - 1 wraps to 255

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0xFF); // Wrapped to 255
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // 0xFF has bit 7 set
}

#[test]
fn test_dex_from_0x80() {
    let mut cpu = setup_cpu();

    // DEX (0xCA)
    cpu.memory_mut().write(0x8000, 0xCA);
    cpu.set_x(0x80); // 0b10000000

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x7F); // 0b01111111
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n()); // Bit 7 cleared
}

// ========== Edge Case Tests ==========

#[test]
fn test_dex_all_bits_set() {
    let mut cpu = setup_cpu();

    // DEX (0xCA)
    cpu.memory_mut().write(0x8000, 0xCA);
    cpu.set_x(0xFF); // 0b11111111

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0xFE); // 0b11111110
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is still set
}

#[test]
fn test_dex_preserves_carry_flag() {
    let mut cpu = setup_cpu();

    // DEX (0xCA)
    cpu.memory_mut().write(0x8000, 0xCA);
    cpu.set_x(0x05);
    cpu.set_flag_c(true); // Set carry flag

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x04);
    assert!(cpu.flag_c()); // Carry flag should be unchanged
}

#[test]
fn test_dex_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    // DEX (0xCA)
    cpu.memory_mut().write(0x8000, 0xCA);
    cpu.set_x(0x05);
    cpu.set_flag_v(true); // Set overflow flag

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x04);
    assert!(cpu.flag_v()); // Overflow flag should be unchanged
}

#[test]
fn test_dex_preserves_accumulator() {
    let mut cpu = setup_cpu();

    // DEX (0xCA)
    cpu.memory_mut().write(0x8000, 0xCA);
    cpu.set_x(0x05);
    cpu.set_a(0x42); // Set accumulator

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x04);
    assert_eq!(cpu.a(), 0x42); // Accumulator should be unchanged
}

#[test]
fn test_dex_preserves_y_register() {
    let mut cpu = setup_cpu();

    // DEX (0xCA)
    cpu.memory_mut().write(0x8000, 0xCA);
    cpu.set_x(0x05);
    cpu.set_y(0x99); // Set Y register

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x04);
    assert_eq!(cpu.y(), 0x99); // Y register should be unchanged
}

// ========== Multiple Decrements Test ==========

#[test]
fn test_dex_sequence() {
    let mut cpu = setup_cpu();

    // Set up three DEX instructions
    cpu.memory_mut().write(0x8000, 0xCA);
    cpu.memory_mut().write(0x8001, 0xCA);
    cpu.memory_mut().write(0x8002, 0xCA);

    cpu.set_x(0x03);

    cpu.step().unwrap();
    assert_eq!(cpu.x(), 0x02);
    assert_eq!(cpu.pc(), 0x8001);
    assert!(!cpu.flag_z());

    cpu.step().unwrap();
    assert_eq!(cpu.x(), 0x01);
    assert_eq!(cpu.pc(), 0x8002);
    assert!(!cpu.flag_z());

    cpu.step().unwrap();
    assert_eq!(cpu.x(), 0x00);
    assert_eq!(cpu.pc(), 0x8003);
    assert!(cpu.flag_z()); // Result is zero
}

#[test]
fn test_dex_countdown_to_zero() {
    let mut cpu = setup_cpu();

    // Set up 10 DEX instructions
    for i in 0..10 {
        cpu.memory_mut().write(0x8000 + i, 0xCA);
    }

    cpu.set_x(10);

    for i in (1..=10).rev() {
        cpu.step().unwrap();
        assert_eq!(cpu.x(), i - 1);
    }

    // Last value should have Z flag set
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_dex_underflow_sequence() {
    let mut cpu = setup_cpu();

    // DEX from 0 to 0xFF
    cpu.memory_mut().write(0x8000, 0xCA);
    // DEX from 0xFF to 0xFE
    cpu.memory_mut().write(0x8001, 0xCA);

    cpu.set_x(0x00);

    cpu.step().unwrap();
    assert_eq!(cpu.x(), 0xFF); // Wrapped to 255
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());

    cpu.step().unwrap();
    assert_eq!(cpu.x(), 0xFE);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
}

// ========== Cycle Count Tests ==========

#[test]
fn test_dex_cycle_count() {
    let mut cpu = setup_cpu();

    // DEX (0xCA)
    cpu.memory_mut().write(0x8000, 0xCA);
    cpu.set_x(0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.cycles(), 2); // DEX takes 2 cycles
}

#[test]
fn test_dex_multiple_cycle_count() {
    let mut cpu = setup_cpu();

    // Set up 5 DEX instructions
    for i in 0..5 {
        cpu.memory_mut().write(0x8000 + i, 0xCA);
    }

    cpu.set_x(0x10);

    for i in 1..=5 {
        cpu.step().unwrap();
        assert_eq!(cpu.cycles(), (i * 2) as u64); // Each DEX takes 2 cycles
    }
}

// ========== Program Counter Tests ==========

#[test]
fn test_dex_program_counter_advance() {
    let mut cpu = setup_cpu();

    // DEX (0xCA) - single byte instruction
    cpu.memory_mut().write(0x8000, 0xCA);
    cpu.set_x(0x10);

    let initial_pc = cpu.pc();
    cpu.step().unwrap();

    assert_eq!(cpu.pc(), initial_pc + 1); // PC advances by 1 byte
}
