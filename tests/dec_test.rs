//! Comprehensive tests for the DEC (Decrement Memory) instruction.
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

// ========== Basic DEC Operation Tests ==========

#[test]
fn test_dec_basic() {
    let mut cpu = setup_cpu();

    // DEC $42 (0xC6 0x42)
    cpu.memory_mut().write(0x8000, 0xC6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x04); // 5 - 1 = 4
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 5);
}

#[test]
fn test_dec_decrements_by_one() {
    let mut cpu = setup_cpu();

    // DEC $10 (0xC6 0x10)
    cpu.memory_mut().write(0x8000, 0xC6);
    cpu.memory_mut().write(0x8001, 0x10);
    cpu.memory_mut().write(0x0010, 0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0010), 0x41); // 66 - 1 = 65
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

// ========== Zero Flag Tests ==========

#[test]
fn test_dec_zero_flag_set() {
    let mut cpu = setup_cpu();

    // DEC $42 (0xC6 0x42)
    cpu.memory_mut().write(0x8000, 0xC6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x01); // 1 - 1 = 0

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x00);
    assert!(cpu.flag_z()); // Result is zero
    assert!(!cpu.flag_n());
}

#[test]
fn test_dec_zero_flag_clear() {
    let mut cpu = setup_cpu();

    // DEC $42 (0xC6 0x42)
    cpu.memory_mut().write(0x8000, 0xC6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x02);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x01);
    assert!(!cpu.flag_z()); // Result is not zero
    assert!(!cpu.flag_n());
}

// ========== Negative Flag Tests ==========

#[test]
fn test_dec_negative_flag_set() {
    let mut cpu = setup_cpu();

    // DEC $42 (0xC6 0x42)
    cpu.memory_mut().write(0x8000, 0xC6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x81); // 0b10000001

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x80); // 0b10000000
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is set
}

#[test]
fn test_dec_negative_flag_clear() {
    let mut cpu = setup_cpu();

    // DEC $42 (0xC6 0x42)
    cpu.memory_mut().write(0x8000, 0xC6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x7F); // 0b01111111

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x7E); // 0b01111110
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n()); // Bit 7 is still 0
}

// ========== Underflow/Wrap Tests ==========

#[test]
fn test_dec_wraps_from_zero_to_ff() {
    let mut cpu = setup_cpu();

    // DEC $42 (0xC6 0x42)
    cpu.memory_mut().write(0x8000, 0xC6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x00); // 0 - 1 wraps to 255

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0xFF); // Wrapped to 255
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // 0xFF has bit 7 set
}

#[test]
fn test_dec_from_0x80() {
    let mut cpu = setup_cpu();

    // DEC $42 (0xC6 0x42)
    cpu.memory_mut().write(0x8000, 0xC6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x80); // 0b10000000

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x7F); // 0b01111111
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n()); // Bit 7 cleared
}

// ========== Addressing Mode Tests ==========

#[test]
fn test_dec_zero_page() {
    let mut cpu = setup_cpu();

    // DEC $42 (0xC6 0x42)
    cpu.memory_mut().write(0x8000, 0xC6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x55);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x54);
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 5);
}

#[test]
fn test_dec_zero_page_x() {
    let mut cpu = setup_cpu();

    // DEC $40,X (0xD6 0x40)
    cpu.memory_mut().write(0x8000, 0xD6);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0045, 0x0F); // Value at 0x40 + 0x05

    cpu.set_x(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0045), 0x0E); // 0x0F - 1 = 0x0E
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_dec_zero_page_x_wrap() {
    let mut cpu = setup_cpu();

    // DEC $FF,X with X=2 should wrap to 0x01 within zero page
    cpu.memory_mut().write(0x8000, 0xD6);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x0001, 0x10); // Value at (0xFF + 0x02) % 256 = 0x01

    cpu.set_x(0x02);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0001), 0x0F); // 16 - 1 = 15
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_dec_absolute() {
    let mut cpu = setup_cpu();

    // DEC $1234 (0xCE 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0xCE);
    cpu.memory_mut().write(0x8001, 0x34); // Low byte
    cpu.memory_mut().write(0x8002, 0x12); // High byte
    cpu.memory_mut().write(0x1234, 0x64);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x1234), 0x63); // 100 - 1 = 99
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_dec_absolute_x() {
    let mut cpu = setup_cpu();

    // DEC $1200,X (0xDE 0x00 0x12)
    cpu.memory_mut().write(0x8000, 0xDE);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1205, 0x22); // Value at 0x1200 + 0x05

    cpu.set_x(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x1205), 0x21); // 0x22 - 1 = 0x21
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 7);
}

#[test]
fn test_dec_absolute_x_with_page_cross() {
    let mut cpu = setup_cpu();

    // DEC $12FF,X with X=2 crosses page boundary (0x12FF -> 0x1301)
    cpu.memory_mut().write(0x8000, 0xDE);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1301, 0x10); // Value at 0x12FF + 0x02

    cpu.set_x(0x02);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x1301), 0x0F); // 0x10 - 1 = 0x0F
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.cycles(), 7); // No extra cycle for page cross on write instructions
}

// ========== Edge Case Tests ==========

#[test]
fn test_dec_all_bits_set() {
    let mut cpu = setup_cpu();

    // DEC $42 (0xC6 0x42)
    cpu.memory_mut().write(0x8000, 0xC6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0xFF); // 0b11111111

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0xFE); // 0b11111110
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is still set
}

#[test]
fn test_dec_preserves_carry_flag() {
    let mut cpu = setup_cpu();

    // DEC $42 (0xC6 0x42)
    cpu.memory_mut().write(0x8000, 0xC6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x05);

    cpu.set_flag_c(true); // Set carry flag

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x04);
    assert!(cpu.flag_c()); // Carry flag should be unchanged
}

#[test]
fn test_dec_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    // DEC $42 (0xC6 0x42)
    cpu.memory_mut().write(0x8000, 0xC6);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x05);

    cpu.set_flag_v(true); // Set overflow flag

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x04);
    assert!(cpu.flag_v()); // Overflow flag should be unchanged
}

// ========== Multiple Decrements Test ==========

#[test]
fn test_dec_sequence() {
    let mut cpu = setup_cpu();

    // First DEC: 0x03 - 1 = 0x02
    cpu.memory_mut().write(0x8000, 0xC6);
    cpu.memory_mut().write(0x8001, 0x42);

    // Second DEC: 0x02 - 1 = 0x01
    cpu.memory_mut().write(0x8002, 0xC6);
    cpu.memory_mut().write(0x8003, 0x42);

    // Third DEC: 0x01 - 1 = 0x00
    cpu.memory_mut().write(0x8004, 0xC6);
    cpu.memory_mut().write(0x8005, 0x42);

    cpu.memory_mut().write(0x0042, 0x03);

    cpu.step().unwrap();
    assert_eq!(cpu.memory_mut().read(0x0042), 0x02);
    assert_eq!(cpu.pc(), 0x8002);
    assert!(!cpu.flag_z());

    cpu.step().unwrap();
    assert_eq!(cpu.memory_mut().read(0x0042), 0x01);
    assert_eq!(cpu.pc(), 0x8004);
    assert!(!cpu.flag_z());

    cpu.step().unwrap();
    assert_eq!(cpu.memory_mut().read(0x0042), 0x00);
    assert_eq!(cpu.pc(), 0x8006);
    assert!(cpu.flag_z()); // Result is zero
}

#[test]
fn test_dec_countdown_to_zero() {
    let mut cpu = setup_cpu();

    // Set up 10 DEC instructions
    for i in 0..10 {
        cpu.memory_mut().write(0x8000 + (i * 2), 0xC6);
        cpu.memory_mut().write(0x8000 + (i * 2) + 1, 0x42);
    }

    cpu.memory_mut().write(0x0042, 10);

    for i in (1..=10).rev() {
        cpu.step().unwrap();
        assert_eq!(cpu.memory_mut().read(0x0042), i - 1);
    }

    // Last value should have Z flag set
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_dec_underflow_sequence() {
    let mut cpu = setup_cpu();

    // DEC from 0 to 0xFF
    cpu.memory_mut().write(0x8000, 0xC6);
    cpu.memory_mut().write(0x8001, 0x42);

    // DEC from 0xFF to 0xFE
    cpu.memory_mut().write(0x8002, 0xC6);
    cpu.memory_mut().write(0x8003, 0x42);

    cpu.memory_mut().write(0x0042, 0x00);

    cpu.step().unwrap();
    assert_eq!(cpu.memory_mut().read(0x0042), 0xFF); // Wrapped to 255
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());

    cpu.step().unwrap();
    assert_eq!(cpu.memory_mut().read(0x0042), 0xFE);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
}
