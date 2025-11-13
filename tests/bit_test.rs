//! Comprehensive tests for the BIT (Bit Test) instruction.
//!
//! Tests cover:
//! - Both addressing modes (Zero Page, Absolute)
//! - Flag updates (Z, N, V)
//! - Various operand values
//! - Correct cycle counts
//! - Accumulator is NOT modified

use cpu6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic BIT Operation Tests ==========

#[test]
fn test_bit_zero_page_basic() {
    let mut cpu = setup_cpu();

    // BIT $42 (0x24 0x42)
    cpu.memory_mut().write(0x8000, 0x24);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0xFF); // Value at zero page address

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    // Accumulator should NOT be modified
    assert_eq!(cpu.a(), 0xFF);

    // 0xFF & 0xFF = 0xFF (non-zero), so Z should be clear
    assert!(!cpu.flag_z());

    // Bit 7 of 0xFF is 1, so N should be set
    assert!(cpu.flag_n());

    // Bit 6 of 0xFF is 1, so V should be set
    assert!(cpu.flag_v());

    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 3);
}

#[test]
fn test_bit_absolute_basic() {
    let mut cpu = setup_cpu();

    // BIT $1234 (0x2C 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0x2C);
    cpu.memory_mut().write(0x8001, 0x34); // Low byte
    cpu.memory_mut().write(0x8002, 0x12); // High byte
    cpu.memory_mut().write(0x1234, 0xFF);

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    // Accumulator should NOT be modified
    assert_eq!(cpu.a(), 0xFF);

    // 0xFF & 0xFF = 0xFF (non-zero), so Z should be clear
    assert!(!cpu.flag_z());

    // Bit 7 of 0xFF is 1, so N should be set
    assert!(cpu.flag_n());

    // Bit 6 of 0xFF is 1, so V should be set
    assert!(cpu.flag_v());

    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 4);
}

// ========== Zero Flag Tests ==========

#[test]
fn test_bit_zero_flag_set() {
    let mut cpu = setup_cpu();

    // BIT $42 with A=0x0F, Memory=0xF0
    // 0x0F & 0xF0 = 0x00, so Z should be set
    cpu.memory_mut().write(0x8000, 0x24);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0xF0); // 0b11110000

    cpu.set_a(0x0F); // 0b00001111

    cpu.step().unwrap();

    // 0x0F & 0xF0 = 0x00, so Z should be set
    assert!(cpu.flag_z());

    // Bit 7 of 0xF0 is 1, so N should be set
    assert!(cpu.flag_n());

    // Bit 6 of 0xF0 is 1, so V should be set
    assert!(cpu.flag_v());

    // Accumulator should NOT be modified
    assert_eq!(cpu.a(), 0x0F);
}

#[test]
fn test_bit_zero_flag_clear() {
    let mut cpu = setup_cpu();

    // BIT $42 with A=0xFF, Memory=0x01
    // 0xFF & 0x01 = 0x01, so Z should be clear
    cpu.memory_mut().write(0x8000, 0x24);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x01);

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    // 0xFF & 0x01 = 0x01 (non-zero), so Z should be clear
    assert!(!cpu.flag_z());

    // Bit 7 of 0x01 is 0, so N should be clear
    assert!(!cpu.flag_n());

    // Bit 6 of 0x01 is 0, so V should be clear
    assert!(!cpu.flag_v());

    // Accumulator should NOT be modified
    assert_eq!(cpu.a(), 0xFF);
}

// ========== Negative Flag Tests ==========

#[test]
fn test_bit_negative_flag_from_bit7() {
    let mut cpu = setup_cpu();

    // Test that N flag is set from bit 7 of memory value, NOT from result
    // BIT $42 with A=0x00, Memory=0x80
    // A & M = 0x00 & 0x80 = 0x00 (result is zero)
    // But N flag should be set because bit 7 of memory is 1
    cpu.memory_mut().write(0x8000, 0x24);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x80); // 0b10000000

    cpu.set_a(0x00);

    cpu.step().unwrap();

    // Z should be set (result is zero)
    assert!(cpu.flag_z());

    // N should be set (bit 7 of memory is 1)
    assert!(cpu.flag_n());

    // V should be clear (bit 6 of memory is 0)
    assert!(!cpu.flag_v());

    // Accumulator should NOT be modified
    assert_eq!(cpu.a(), 0x00);
}

#[test]
fn test_bit_negative_flag_clear() {
    let mut cpu = setup_cpu();

    // BIT with memory value that has bit 7 clear
    cpu.memory_mut().write(0x8000, 0x24);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x7F); // 0b01111111

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    // N should be clear (bit 7 of memory is 0)
    assert!(!cpu.flag_n());

    // V should be set (bit 6 of memory is 1)
    assert!(cpu.flag_v());

    // Z should be clear (0xFF & 0x7F = 0x7F, non-zero)
    assert!(!cpu.flag_z());
}

// ========== Overflow Flag Tests ==========

#[test]
fn test_bit_overflow_flag_from_bit6() {
    let mut cpu = setup_cpu();

    // Test that V flag is set from bit 6 of memory value, NOT from result
    // BIT $42 with A=0x00, Memory=0x40
    // A & M = 0x00 & 0x40 = 0x00 (result is zero)
    // But V flag should be set because bit 6 of memory is 1
    cpu.memory_mut().write(0x8000, 0x24);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x40); // 0b01000000

    cpu.set_a(0x00);

    cpu.step().unwrap();

    // Z should be set (result is zero)
    assert!(cpu.flag_z());

    // N should be clear (bit 7 of memory is 0)
    assert!(!cpu.flag_n());

    // V should be set (bit 6 of memory is 1)
    assert!(cpu.flag_v());

    // Accumulator should NOT be modified
    assert_eq!(cpu.a(), 0x00);
}

#[test]
fn test_bit_overflow_flag_clear() {
    let mut cpu = setup_cpu();

    // BIT with memory value that has bit 6 clear
    cpu.memory_mut().write(0x8000, 0x24);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0xBF); // 0b10111111

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    // N should be set (bit 7 of memory is 1)
    assert!(cpu.flag_n());

    // V should be clear (bit 6 of memory is 0)
    assert!(!cpu.flag_v());

    // Z should be clear (0xFF & 0xBF = 0xBF, non-zero)
    assert!(!cpu.flag_z());
}

// ========== All Flag Combinations ==========

#[test]
fn test_bit_all_flags_set() {
    let mut cpu = setup_cpu();

    // Memory = 0xC0 (0b11000000) - bits 7 and 6 set
    // A = 0xFF (all bits set)
    // Result = 0xC0 (non-zero)
    cpu.memory_mut().write(0x8000, 0x24);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0xC0);

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    // Z should be clear (result is non-zero)
    assert!(!cpu.flag_z());

    // N should be set (bit 7 of memory is 1)
    assert!(cpu.flag_n());

    // V should be set (bit 6 of memory is 1)
    assert!(cpu.flag_v());
}

#[test]
fn test_bit_all_flags_clear() {
    let mut cpu = setup_cpu();

    // Memory = 0x3F (0b00111111) - bits 7 and 6 clear
    // A = 0xFF (all bits set)
    // Result = 0x3F (non-zero)
    cpu.memory_mut().write(0x8000, 0x24);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x3F);

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    // Z should be clear (result is non-zero)
    assert!(!cpu.flag_z());

    // N should be clear (bit 7 of memory is 0)
    assert!(!cpu.flag_n());

    // V should be clear (bit 6 of memory is 0)
    assert!(!cpu.flag_v());
}

#[test]
fn test_bit_z_and_n_set_v_clear() {
    let mut cpu = setup_cpu();

    // Memory = 0x80 (0b10000000) - bit 7 set, bit 6 clear
    // A = 0x00 (no bits set)
    // Result = 0x00 (zero)
    cpu.memory_mut().write(0x8000, 0x24);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x80);

    cpu.set_a(0x00);

    cpu.step().unwrap();

    // Z should be set (result is zero)
    assert!(cpu.flag_z());

    // N should be set (bit 7 of memory is 1)
    assert!(cpu.flag_n());

    // V should be clear (bit 6 of memory is 0)
    assert!(!cpu.flag_v());
}

#[test]
fn test_bit_z_and_v_set_n_clear() {
    let mut cpu = setup_cpu();

    // Memory = 0x40 (0b01000000) - bit 7 clear, bit 6 set
    // A = 0x00 (no bits set)
    // Result = 0x00 (zero)
    cpu.memory_mut().write(0x8000, 0x24);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x40);

    cpu.set_a(0x00);

    cpu.step().unwrap();

    // Z should be set (result is zero)
    assert!(cpu.flag_z());

    // N should be clear (bit 7 of memory is 0)
    assert!(!cpu.flag_n());

    // V should be set (bit 6 of memory is 1)
    assert!(cpu.flag_v());
}

// ========== Accumulator Preservation Tests ==========

#[test]
fn test_bit_does_not_modify_accumulator() {
    let mut cpu = setup_cpu();

    // Test with various values to ensure A is never modified
    cpu.memory_mut().write(0x8000, 0x24);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x00);

    cpu.set_a(0x42);

    cpu.step().unwrap();

    // Accumulator should still be 0x42
    assert_eq!(cpu.a(), 0x42);
}

#[test]
fn test_bit_accumulator_preserved_zero_page() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x24);
    cpu.memory_mut().write(0x8001, 0x50);
    cpu.memory_mut().write(0x0050, 0xAA);

    cpu.set_a(0x55);

    cpu.step().unwrap();

    // Even though 0x55 & 0xAA = 0x00, A should still be 0x55
    assert_eq!(cpu.a(), 0x55);
    assert!(cpu.flag_z()); // Result is zero
}

#[test]
fn test_bit_accumulator_preserved_absolute() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x2C);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x20);
    cpu.memory_mut().write(0x2000, 0xFF);

    cpu.set_a(0x00);

    cpu.step().unwrap();

    // Even though memory is 0xFF, A should still be 0x00
    assert_eq!(cpu.a(), 0x00);
    assert!(cpu.flag_z()); // Result is zero
}

// ========== Other Flags Preservation Tests ==========

#[test]
fn test_bit_preserves_carry_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x24);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0xFF);

    cpu.set_a(0xFF);
    cpu.set_flag_c(true); // Set carry flag

    cpu.step().unwrap();

    // Carry flag should be unchanged
    assert!(cpu.flag_c());
}

#[test]
fn test_bit_preserves_decimal_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x24);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0xFF);

    cpu.set_a(0xFF);
    cpu.set_flag_d(true); // Set decimal flag

    cpu.step().unwrap();

    // Decimal flag should be unchanged
    assert!(cpu.flag_d());
}

// ========== Edge Cases ==========

#[test]
fn test_bit_all_zeros() {
    let mut cpu = setup_cpu();

    // Memory = 0x00, A = 0x00
    cpu.memory_mut().write(0x8000, 0x24);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x00);

    cpu.set_a(0x00);

    cpu.step().unwrap();

    // Z should be set (0x00 & 0x00 = 0x00)
    assert!(cpu.flag_z());

    // N should be clear (bit 7 of memory is 0)
    assert!(!cpu.flag_n());

    // V should be clear (bit 6 of memory is 0)
    assert!(!cpu.flag_v());
}

#[test]
fn test_bit_all_ones() {
    let mut cpu = setup_cpu();

    // Memory = 0xFF, A = 0xFF
    cpu.memory_mut().write(0x8000, 0x24);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0xFF);

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    // Z should be clear (0xFF & 0xFF = 0xFF, non-zero)
    assert!(!cpu.flag_z());

    // N should be set (bit 7 of memory is 1)
    assert!(cpu.flag_n());

    // V should be set (bit 6 of memory is 1)
    assert!(cpu.flag_v());
}

// ========== Multiple Instructions Test ==========

#[test]
fn test_bit_sequence() {
    let mut cpu = setup_cpu();

    // First BIT: test with 0xC0 (bits 7,6 set)
    cpu.memory_mut().write(0x8000, 0x24);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0xC0);

    // Second BIT: test with 0x3F (bits 7,6 clear)
    cpu.memory_mut().write(0x8002, 0x24);
    cpu.memory_mut().write(0x8003, 0x43);
    cpu.memory_mut().write(0x0043, 0x3F);

    cpu.set_a(0xFF);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0xFF); // A unchanged
    assert_eq!(cpu.pc(), 0x8002);
    assert!(cpu.flag_n());
    assert!(cpu.flag_v());
    assert!(!cpu.flag_z());

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0xFF); // A still unchanged
    assert_eq!(cpu.pc(), 0x8004);
    assert!(!cpu.flag_n());
    assert!(!cpu.flag_v());
    assert!(!cpu.flag_z());
}

// ========== Practical Use Cases ==========

#[test]
fn test_bit_check_bit7_status() {
    let mut cpu = setup_cpu();

    // Common use: check if bit 7 is set (e.g., testing sign bit)
    cpu.memory_mut().write(0x8000, 0x24);
    cpu.memory_mut().write(0x8001, 0x50);
    cpu.memory_mut().write(0x0050, 0x80); // Only bit 7 set

    cpu.set_a(0x80); // Test bit 7

    cpu.step().unwrap();

    // Result is non-zero (bit 7 matches)
    assert!(!cpu.flag_z());
    // N flag is set (bit 7 of memory)
    assert!(cpu.flag_n());
    // V flag is clear (bit 6 of memory)
    assert!(!cpu.flag_v());
}

#[test]
fn test_bit_check_bit6_status() {
    let mut cpu = setup_cpu();

    // Common use: check if bit 6 is set (e.g., testing overflow)
    cpu.memory_mut().write(0x8000, 0x24);
    cpu.memory_mut().write(0x8001, 0x50);
    cpu.memory_mut().write(0x0050, 0x40); // Only bit 6 set

    cpu.set_a(0x40); // Test bit 6

    cpu.step().unwrap();

    // Result is non-zero (bit 6 matches)
    assert!(!cpu.flag_z());
    // N flag is clear (bit 7 of memory)
    assert!(!cpu.flag_n());
    // V flag is set (bit 6 of memory)
    assert!(cpu.flag_v());
}
