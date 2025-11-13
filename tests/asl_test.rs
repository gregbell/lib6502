//! Comprehensive tests for the ASL (Arithmetic Shift Left) instruction.
//!
//! Tests cover:
//! - All 5 addressing modes (Accumulator, Zero Page, Zero Page,X, Absolute, Absolute,X)
//! - Flag updates (C, Z, N)
//! - Various operand values (0, positive, negative, edge cases)
//! - Cycle counts
//! - Memory write-back for non-accumulator modes

use cpu6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic ASL Operation Tests ==========

#[test]
fn test_asl_accumulator_basic() {
    let mut cpu = setup_cpu();

    // ASL A (0x0A)
    cpu.memory_mut().write(0x8000, 0x0A);

    cpu.set_a(0x01); // 0b00000001

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x02); // 0b00000010 (shifted left)
    assert!(!cpu.flag_c()); // Bit 7 was 0
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.cycles(), 2);
}

#[test]
fn test_asl_multiplies_by_two() {
    let mut cpu = setup_cpu();

    // ASL A (0x0A)
    cpu.memory_mut().write(0x8000, 0x0A);

    cpu.set_a(0x15); // 21 in decimal

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x2A); // 42 in decimal (21 * 2)
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

// ========== Carry Flag Tests ==========

#[test]
fn test_asl_carry_flag_set() {
    let mut cpu = setup_cpu();

    // ASL A (0x0A)
    cpu.memory_mut().write(0x8000, 0x0A);

    cpu.set_a(0x80); // 0b10000000 - bit 7 is set

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00); // 0b00000000 (shifted left, wrapped)
    assert!(cpu.flag_c()); // Old bit 7 was 1
    assert!(cpu.flag_z()); // Result is zero
    assert!(!cpu.flag_n());
}

#[test]
fn test_asl_carry_flag_clear() {
    let mut cpu = setup_cpu();

    // ASL A (0x0A)
    cpu.memory_mut().write(0x8000, 0x0A);

    cpu.set_a(0x7F); // 0b01111111 - bit 7 is 0

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFE); // 0b11111110
    assert!(!cpu.flag_c()); // Old bit 7 was 0
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // New bit 7 is 1
}

#[test]
fn test_asl_carry_with_overflow() {
    let mut cpu = setup_cpu();

    // ASL A (0x0A)
    cpu.memory_mut().write(0x8000, 0x0A);

    cpu.set_a(0xFF); // 0b11111111

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFE); // 0b11111110
    assert!(cpu.flag_c()); // Old bit 7 was 1
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // New bit 7 is 1
}

// ========== Zero Flag Tests ==========

#[test]
fn test_asl_zero_flag_set() {
    let mut cpu = setup_cpu();

    // ASL A (0x0A)
    cpu.memory_mut().write(0x8000, 0x0A);

    cpu.set_a(0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00);
    assert!(cpu.flag_z()); // Result is zero
    assert!(!cpu.flag_c()); // Bit 7 was 0
    assert!(!cpu.flag_n());
}

#[test]
fn test_asl_zero_flag_from_shift() {
    let mut cpu = setup_cpu();

    // ASL A (0x0A)
    cpu.memory_mut().write(0x8000, 0x0A);

    cpu.set_a(0x80); // 0b10000000

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00); // Shifted to zero
    assert!(cpu.flag_z()); // Result is zero
    assert!(cpu.flag_c()); // Old bit 7 was 1
}

// ========== Negative Flag Tests ==========

#[test]
fn test_asl_negative_flag_set() {
    let mut cpu = setup_cpu();

    // ASL A (0x0A)
    cpu.memory_mut().write(0x8000, 0x0A);

    cpu.set_a(0x40); // 0b01000000

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x80); // 0b10000000
    assert!(cpu.flag_n()); // Bit 7 is now set
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
}

#[test]
fn test_asl_negative_flag_clear() {
    let mut cpu = setup_cpu();

    // ASL A (0x0A)
    cpu.memory_mut().write(0x8000, 0x0A);

    cpu.set_a(0x20); // 0b00100000

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x40); // 0b01000000
    assert!(!cpu.flag_n()); // Bit 7 is 0
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
}

// ========== Addressing Mode Tests ==========

#[test]
fn test_asl_zero_page() {
    let mut cpu = setup_cpu();

    // ASL $42 (0x06 0x42)
    cpu.memory_mut().write(0x8000, 0x06);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x55); // 0b01010101

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0xAA); // 0b10101010
    assert!(!cpu.flag_c()); // Old bit 7 was 0
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // New bit 7 is 1
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 5);
}

#[test]
fn test_asl_zero_page_write_back() {
    let mut cpu = setup_cpu();

    // ASL $10 (0x06 0x10)
    cpu.memory_mut().write(0x8000, 0x06);
    cpu.memory_mut().write(0x8001, 0x10);
    cpu.memory_mut().write(0x0010, 0x03);

    cpu.step().unwrap();

    // Verify memory was updated
    assert_eq!(cpu.memory_mut().read(0x0010), 0x06); // 3 * 2 = 6
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
}

#[test]
fn test_asl_zero_page_x() {
    let mut cpu = setup_cpu();

    // ASL $40,X (0x16 0x40)
    cpu.memory_mut().write(0x8000, 0x16);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0045, 0x0F); // Value at 0x40 + 0x05

    cpu.set_x(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0045), 0x1E); // 0x0F * 2 = 0x1E
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_asl_zero_page_x_wrap() {
    let mut cpu = setup_cpu();

    // ASL $FF,X with X=2 should wrap to 0x01 within zero page
    cpu.memory_mut().write(0x8000, 0x16);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x0001, 0x02); // Value at (0xFF + 0x02) % 256 = 0x01

    cpu.set_x(0x02);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0001), 0x04); // 2 * 2 = 4
    assert!(!cpu.flag_c());
}

#[test]
fn test_asl_absolute() {
    let mut cpu = setup_cpu();

    // ASL $1234 (0x0E 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0x0E);
    cpu.memory_mut().write(0x8001, 0x34); // Low byte
    cpu.memory_mut().write(0x8002, 0x12); // High byte
    cpu.memory_mut().write(0x1234, 0x33);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x1234), 0x66); // 0x33 * 2 = 0x66
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_asl_absolute_x() {
    let mut cpu = setup_cpu();

    // ASL $1200,X (0x1E 0x00 0x12)
    cpu.memory_mut().write(0x8000, 0x1E);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1205, 0x11); // Value at 0x1200 + 0x05

    cpu.set_x(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x1205), 0x22); // 0x11 * 2 = 0x22
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 7);
}

#[test]
fn test_asl_absolute_x_with_page_cross() {
    let mut cpu = setup_cpu();

    // ASL $12FF,X with X=2 crosses page boundary (0x12FF -> 0x1301)
    cpu.memory_mut().write(0x8000, 0x1E);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1301, 0x08); // Value at 0x12FF + 0x02

    cpu.set_x(0x02);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x1301), 0x10); // 0x08 * 2 = 0x10
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.cycles(), 7); // No extra cycle for page cross on write instructions
}

// ========== Edge Case Tests ==========

#[test]
fn test_asl_shifts_in_zero() {
    let mut cpu = setup_cpu();

    // ASL A (0x0A)
    cpu.memory_mut().write(0x8000, 0x0A);

    cpu.set_a(0x01); // 0b00000001

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x02); // 0b00000010 - bit 0 is now 0
    assert!(!cpu.flag_c());
}

#[test]
fn test_asl_all_bits_set() {
    let mut cpu = setup_cpu();

    // ASL A (0x0A)
    cpu.memory_mut().write(0x8000, 0x0A);

    cpu.set_a(0xFF); // 0b11111111

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFE); // 0b11111110
    assert!(cpu.flag_c()); // Old bit 7 was 1
    assert!(cpu.flag_n()); // New bit 7 is 1
    assert!(!cpu.flag_z());
}

#[test]
fn test_asl_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    // ASL A (0x0A)
    cpu.memory_mut().write(0x8000, 0x0A);

    cpu.set_a(0x01);
    cpu.set_flag_v(true); // Set overflow flag

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x02);
    assert!(cpu.flag_v()); // Overflow flag should be unchanged
}

#[test]
fn test_asl_does_not_preserve_carry() {
    let mut cpu = setup_cpu();

    // ASL A (0x0A)
    cpu.memory_mut().write(0x8000, 0x0A);

    cpu.set_a(0x01); // Bit 7 is 0
    cpu.set_flag_c(true); // Set carry flag before instruction

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x02);
    assert!(!cpu.flag_c()); // Carry should be cleared (old bit 7 was 0)
}

// ========== Multiple Shifts Test ==========

#[test]
fn test_asl_sequence() {
    let mut cpu = setup_cpu();

    // First ASL: 0x01 << 1 = 0x02
    cpu.memory_mut().write(0x8000, 0x0A);

    // Second ASL: 0x02 << 1 = 0x04
    cpu.memory_mut().write(0x8001, 0x0A);

    // Third ASL: 0x04 << 1 = 0x08
    cpu.memory_mut().write(0x8002, 0x0A);

    cpu.set_a(0x01);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x02);
    assert_eq!(cpu.pc(), 0x8001);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x04);
    assert_eq!(cpu.pc(), 0x8002);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x08);
    assert_eq!(cpu.pc(), 0x8003);
}

#[test]
fn test_asl_progressive_shifts() {
    let mut cpu = setup_cpu();

    // Shift 0x01 left 7 times to reach 0x80
    for i in 0..7 {
        cpu.memory_mut().write(0x8000 + i, 0x0A);
    }

    cpu.set_a(0x01);

    let expected_values = [0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80];

    for (i, &expected) in expected_values.iter().enumerate() {
        cpu.step().unwrap();
        assert_eq!(cpu.a(), expected, "Failed at iteration {}", i);
    }

    // Last value should have N flag set
    assert!(cpu.flag_n());
    assert!(!cpu.flag_c());
}

#[test]
fn test_asl_shift_until_overflow() {
    let mut cpu = setup_cpu();

    // Shift 0x80 left once to trigger carry
    cpu.memory_mut().write(0x8000, 0x0A);

    cpu.set_a(0x80); // 0b10000000

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00); // Wrapped to zero
    assert!(cpu.flag_c()); // Carry set from old bit 7
    assert!(cpu.flag_z()); // Result is zero
    assert!(!cpu.flag_n());
}
