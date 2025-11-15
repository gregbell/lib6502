//! Comprehensive tests for the INC (Increment Memory) instruction.
//!
//! Tests cover:
//! - All 4 addressing modes (Zero Page, Zero Page,X, Absolute, Absolute,X)
//! - Flag updates (Z, N)
//! - Various operand values (0, positive, negative, edge cases)
//! - Cycle counts
//! - Memory write-back

use cpu6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic INC Operation Tests ==========

#[test]
fn test_inc_basic() {
    let mut cpu = setup_cpu();

    // INC $42 (0xE6 0x42)
    cpu.memory_mut().write(0x8000, 0xE6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x06); // 5 + 1 = 6
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 5);
}

#[test]
fn test_inc_increments_by_one() {
    let mut cpu = setup_cpu();

    // INC $10 (0xE6 0x10)
    cpu.memory_mut().write(0x8000, 0xE6);
    cpu.memory_mut().write(0x8001, 0x10);
    cpu.memory_mut().write(0x0010, 0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0010), 0x43); // 66 + 1 = 67
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

// ========== Zero Flag Tests ==========

#[test]
fn test_inc_zero_flag_set() {
    let mut cpu = setup_cpu();

    // INC $42 (0xE6 0x42)
    cpu.memory_mut().write(0x8000, 0xE6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0xFF); // 255 + 1 = 0

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x00);
    assert!(cpu.flag_z()); // Result is zero
    assert!(!cpu.flag_n());
}

#[test]
fn test_inc_zero_flag_clear() {
    let mut cpu = setup_cpu();

    // INC $42 (0xE6 0x42)
    cpu.memory_mut().write(0x8000, 0xE6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x02);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x03);
    assert!(!cpu.flag_z()); // Result is not zero
    assert!(!cpu.flag_n());
}

// ========== Negative Flag Tests ==========

#[test]
fn test_inc_negative_flag_set() {
    let mut cpu = setup_cpu();

    // INC $42 (0xE6 0x42)
    cpu.memory_mut().write(0x8000, 0xE6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x7F); // 0b01111111

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x80); // 0b10000000
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is set
}

#[test]
fn test_inc_negative_flag_clear() {
    let mut cpu = setup_cpu();

    // INC $42 (0xE6 0x42)
    cpu.memory_mut().write(0x8000, 0xE6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x7E); // 0b01111110

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x7F); // 0b01111111
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n()); // Bit 7 is still 0
}

// ========== Overflow/Wrap Tests ==========

#[test]
fn test_inc_wraps_from_ff_to_zero() {
    let mut cpu = setup_cpu();

    // INC $42 (0xE6 0x42)
    cpu.memory_mut().write(0x8000, 0xE6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0xFF); // 255 + 1 wraps to 0

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x00); // Wrapped to 0
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n()); // 0x00 has bit 7 clear
}

#[test]
fn test_inc_from_0x7f() {
    let mut cpu = setup_cpu();

    // INC $42 (0xE6 0x42)
    cpu.memory_mut().write(0x8000, 0xE6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x7F); // 0b01111111

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x80); // 0b10000000
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 set
}

// ========== Addressing Mode Tests ==========

#[test]
fn test_inc_zero_page() {
    let mut cpu = setup_cpu();

    // INC $42 (0xE6 0x42)
    cpu.memory_mut().write(0x8000, 0xE6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x55);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x56);
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 5);
}

#[test]
fn test_inc_zero_page_x() {
    let mut cpu = setup_cpu();

    // INC $40,X (0xF6 0x40)
    cpu.memory_mut().write(0x8000, 0xF6);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0045, 0x0F); // Value at 0x40 + 0x05

    cpu.set_x(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0045), 0x10); // 0x0F + 1 = 0x10
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_inc_zero_page_x_wrap() {
    let mut cpu = setup_cpu();

    // INC $FF,X with X=2 should wrap to 0x01 within zero page
    cpu.memory_mut().write(0x8000, 0xF6);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x0001, 0x10); // Value at (0xFF + 0x02) % 256 = 0x01

    cpu.set_x(0x02);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0001), 0x11); // 16 + 1 = 17
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_inc_absolute() {
    let mut cpu = setup_cpu();

    // INC $1234 (0xEE 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0xEE);
    cpu.memory_mut().write(0x8001, 0x34); // Low byte
    cpu.memory_mut().write(0x8002, 0x12); // High byte
    cpu.memory_mut().write(0x1234, 0x64);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x1234), 0x65); // 100 + 1 = 101
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_inc_absolute_x() {
    let mut cpu = setup_cpu();

    // INC $1200,X (0xFE 0x00 0x12)
    cpu.memory_mut().write(0x8000, 0xFE);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1205, 0x22); // Value at 0x1200 + 0x05

    cpu.set_x(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x1205), 0x23); // 0x22 + 1 = 0x23
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 7);
}

#[test]
fn test_inc_absolute_x_with_page_cross() {
    let mut cpu = setup_cpu();

    // INC $12FF,X with X=2 crosses page boundary (0x12FF -> 0x1301)
    cpu.memory_mut().write(0x8000, 0xFE);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1301, 0x10); // Value at 0x12FF + 0x02

    cpu.set_x(0x02);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x1301), 0x11); // 0x10 + 1 = 0x11
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.cycles(), 7); // No extra cycle for page cross on write instructions
}

// ========== Edge Case Tests ==========

#[test]
fn test_inc_all_bits_set() {
    let mut cpu = setup_cpu();

    // INC $42 (0xE6 0x42)
    cpu.memory_mut().write(0x8000, 0xE6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0xFE); // 0b11111110

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0xFF); // 0b11111111
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is still set
}

#[test]
fn test_inc_preserves_carry_flag() {
    let mut cpu = setup_cpu();

    // INC $42 (0xE6 0x42)
    cpu.memory_mut().write(0x8000, 0xE6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x05);

    cpu.set_flag_c(true); // Set carry flag

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x06);
    assert!(cpu.flag_c()); // Carry flag should be unchanged
}

#[test]
fn test_inc_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    // INC $42 (0xE6 0x42)
    cpu.memory_mut().write(0x8000, 0xE6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x05);

    cpu.set_flag_v(true); // Set overflow flag

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x06);
    assert!(cpu.flag_v()); // Overflow flag should be unchanged
}

// ========== Multiple Increments Test ==========

#[test]
fn test_inc_sequence() {
    let mut cpu = setup_cpu();

    // First INC: 0x00 + 1 = 0x01
    cpu.memory_mut().write(0x8000, 0xE6);
    cpu.memory_mut().write(0x8001, 0x42);

    // Second INC: 0x01 + 1 = 0x02
    cpu.memory_mut().write(0x8002, 0xE6);
    cpu.memory_mut().write(0x8003, 0x42);

    // Third INC: 0x02 + 1 = 0x03
    cpu.memory_mut().write(0x8004, 0xE6);
    cpu.memory_mut().write(0x8005, 0x42);

    cpu.memory_mut().write(0x0042, 0x00);

    cpu.step().unwrap();
    assert_eq!(cpu.memory_mut().read(0x0042), 0x01);
    assert_eq!(cpu.pc(), 0x8002);
    assert!(!cpu.flag_z());

    cpu.step().unwrap();
    assert_eq!(cpu.memory_mut().read(0x0042), 0x02);
    assert_eq!(cpu.pc(), 0x8004);
    assert!(!cpu.flag_z());

    cpu.step().unwrap();
    assert_eq!(cpu.memory_mut().read(0x0042), 0x03);
    assert_eq!(cpu.pc(), 0x8006);
    assert!(!cpu.flag_z());
}

#[test]
fn test_inc_countup_from_zero() {
    let mut cpu = setup_cpu();

    // Set up 10 INC instructions
    for i in 0..10 {
        cpu.memory_mut().write(0x8000 + (i * 2), 0xE6);
        cpu.memory_mut().write(0x8000 + (i * 2) + 1, 0x42);
    }

    cpu.memory_mut().write(0x0042, 0);

    for i in 1..=10 {
        cpu.step().unwrap();
        assert_eq!(cpu.memory_mut().read(0x0042), i);
    }

    // Last value should not have Z flag set
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_inc_overflow_sequence() {
    let mut cpu = setup_cpu();

    // INC from 0xFE to 0xFF
    cpu.memory_mut().write(0x8000, 0xE6);
    cpu.memory_mut().write(0x8001, 0x42);

    // INC from 0xFF to 0x00
    cpu.memory_mut().write(0x8002, 0xE6);
    cpu.memory_mut().write(0x8003, 0x42);

    cpu.memory_mut().write(0x0042, 0xFE);

    cpu.step().unwrap();
    assert_eq!(cpu.memory_mut().read(0x0042), 0xFF);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());

    cpu.step().unwrap();
    assert_eq!(cpu.memory_mut().read(0x0042), 0x00); // Wrapped to 0
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
}
