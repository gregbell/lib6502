//! Comprehensive tests for the ROR (Rotate Right) instruction.
//!
//! Tests cover:
//! - All 5 addressing modes (Accumulator, Zero Page, Zero Page,X, Absolute, Absolute,X)
//! - Flag updates (C, Z, N)
//! - Various operand values (0, positive, negative, edge cases)
//! - Cycle counts
//! - Memory write-back for non-accumulator modes
//! - Carry flag rotation behavior

use lib6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic ROR Operation Tests ==========

#[test]
fn test_ror_accumulator_basic() {
    let mut cpu = setup_cpu();

    // ROR A (0x6A)
    cpu.memory_mut().write(0x8000, 0x6A);

    cpu.set_a(0x02); // 0b00000010
    cpu.set_flag_c(false); // Carry = 0

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x01); // 0b00000001 (rotated right, bit 7 = old carry = 0)
    assert!(!cpu.flag_c()); // Bit 0 was 0
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.cycles(), 2);
}

#[test]
fn test_ror_accumulator_with_carry_in() {
    let mut cpu = setup_cpu();

    // ROR A (0x6A)
    cpu.memory_mut().write(0x8000, 0x6A);

    cpu.set_a(0x02); // 0b00000010
    cpu.set_flag_c(true); // Carry = 1

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x81); // 0b10000001 (rotated right, bit 7 = old carry = 1)
    assert!(!cpu.flag_c()); // Bit 0 was 0
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is now set
}

// ========== Carry Flag Tests ==========

#[test]
fn test_ror_carry_flag_set() {
    let mut cpu = setup_cpu();

    // ROR A (0x6A)
    cpu.memory_mut().write(0x8000, 0x6A);

    cpu.set_a(0x01); // 0b00000001 - bit 0 is set
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00); // 0b00000000 (rotated right, bit 7 = old carry = 0)
    assert!(cpu.flag_c()); // Old bit 0 was 1
    assert!(cpu.flag_z()); // Result is zero
    assert!(!cpu.flag_n());
}

#[test]
fn test_ror_carry_flag_clear() {
    let mut cpu = setup_cpu();

    // ROR A (0x6A)
    cpu.memory_mut().write(0x8000, 0x6A);

    cpu.set_a(0xFE); // 0b11111110 - bit 0 is 0
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x7F); // 0b01111111
    assert!(!cpu.flag_c()); // Old bit 0 was 0
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n()); // Bit 7 is 0
}

#[test]
fn test_ror_rotate_carry_through() {
    let mut cpu = setup_cpu();

    // ROR A (0x6A)
    cpu.memory_mut().write(0x8000, 0x6A);

    cpu.set_a(0x01); // 0b00000001
    cpu.set_flag_c(true); // Carry = 1

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x80); // 0b10000000 (rotated right, bit 7 = old carry = 1)
    assert!(cpu.flag_c()); // Old bit 0 was 1
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is now set
}

#[test]
fn test_ror_all_bits_set() {
    let mut cpu = setup_cpu();

    // ROR A (0x6A)
    cpu.memory_mut().write(0x8000, 0x6A);

    cpu.set_a(0xFF); // 0b11111111
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF); // 0b11111111 (all bits stay set)
    assert!(cpu.flag_c()); // Old bit 0 was 1
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is 1
}

// ========== Zero Flag Tests ==========

#[test]
fn test_ror_zero_flag_set() {
    let mut cpu = setup_cpu();

    // ROR A (0x6A)
    cpu.memory_mut().write(0x8000, 0x6A);

    cpu.set_a(0x00);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00);
    assert!(cpu.flag_z()); // Result is zero
    assert!(!cpu.flag_c()); // Bit 0 was 0
    assert!(!cpu.flag_n());
}

#[test]
fn test_ror_zero_flag_from_rotate() {
    let mut cpu = setup_cpu();

    // ROR A (0x6A)
    cpu.memory_mut().write(0x8000, 0x6A);

    cpu.set_a(0x01); // 0b00000001
    cpu.set_flag_c(false); // Carry = 0

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00); // Rotated to zero
    assert!(cpu.flag_z()); // Result is zero
    assert!(cpu.flag_c()); // Old bit 0 was 1
}

#[test]
fn test_ror_zero_with_carry_produces_large_value() {
    let mut cpu = setup_cpu();

    // ROR A (0x6A)
    cpu.memory_mut().write(0x8000, 0x6A);

    cpu.set_a(0x00);
    cpu.set_flag_c(true); // Carry = 1

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x80); // 0b10000000 (bit 7 = old carry = 1)
    assert!(!cpu.flag_z()); // Result is not zero
    assert!(!cpu.flag_c()); // Old bit 0 was 0
    assert!(cpu.flag_n()); // Bit 7 is set
}

// ========== Negative Flag Tests ==========

#[test]
fn test_ror_negative_flag_set() {
    let mut cpu = setup_cpu();

    // ROR A (0x6A)
    cpu.memory_mut().write(0x8000, 0x6A);

    cpu.set_a(0x00); // 0b00000000
    cpu.set_flag_c(true); // Carry = 1

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x80); // 0b10000000
    assert!(cpu.flag_n()); // Bit 7 is now set
    assert!(!cpu.flag_c()); // Old bit 0 was 0
    assert!(!cpu.flag_z());
}

#[test]
fn test_ror_negative_flag_clear() {
    let mut cpu = setup_cpu();

    // ROR A (0x6A)
    cpu.memory_mut().write(0x8000, 0x6A);

    cpu.set_a(0x40); // 0b01000000
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x20); // 0b00100000
    assert!(!cpu.flag_n()); // Bit 7 is 0
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
}

// ========== Addressing Mode Tests ==========

#[test]
fn test_ror_zero_page() {
    let mut cpu = setup_cpu();

    // ROR $42 (0x66 0x42)
    cpu.memory_mut().write(0x8000, 0x66);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0xAA); // 0b10101010

    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0x55); // 0b01010101
    assert!(!cpu.flag_c()); // Old bit 0 was 0
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n()); // Bit 7 is 0
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 5);
}

#[test]
fn test_ror_zero_page_with_carry() {
    let mut cpu = setup_cpu();

    // ROR $10 (0x66 0x10)
    cpu.memory_mut().write(0x8000, 0x66);
    cpu.memory_mut().write(0x8001, 0x10);
    cpu.memory_mut().write(0x0010, 0x03); // 0b00000011

    cpu.set_flag_c(true); // Carry = 1

    cpu.step().unwrap();

    // Verify memory was updated: 0b10000001 (bit 7 set from carry)
    assert_eq!(cpu.memory_mut().read(0x0010), 0x81);
    assert!(cpu.flag_c()); // Old bit 0 was 1
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is set
}

#[test]
fn test_ror_zero_page_x() {
    let mut cpu = setup_cpu();

    // ROR $40,X (0x76 0x40)
    cpu.memory_mut().write(0x8000, 0x76);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0045, 0x1E); // Value at 0x40 + 0x05

    cpu.set_x(0x05);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0045), 0x0F); // 0x1E >> 1 = 0x0F
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_ror_zero_page_x_wrap() {
    let mut cpu = setup_cpu();

    // ROR $FF,X with X=2 should wrap to 0x01 within zero page
    cpu.memory_mut().write(0x8000, 0x76);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x0001, 0x04); // Value at (0xFF + 0x02) % 256 = 0x01

    cpu.set_x(0x02);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0001), 0x02); // 4 >> 1 = 2
    assert!(!cpu.flag_c());
}

#[test]
fn test_ror_absolute() {
    let mut cpu = setup_cpu();

    // ROR $1234 (0x6E 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0x6E);
    cpu.memory_mut().write(0x8001, 0x34); // Low byte
    cpu.memory_mut().write(0x8002, 0x12); // High byte
    cpu.memory_mut().write(0x1234, 0x66);

    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x1234), 0x33); // 0x66 >> 1 = 0x33
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_ror_absolute_x() {
    let mut cpu = setup_cpu();

    // ROR $1200,X (0x7E 0x00 0x12)
    cpu.memory_mut().write(0x8000, 0x7E);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1205, 0x22); // Value at 0x1200 + 0x05

    cpu.set_x(0x05);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x1205), 0x11); // 0x22 >> 1 = 0x11
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 7);
}

#[test]
fn test_ror_absolute_x_with_page_cross() {
    let mut cpu = setup_cpu();

    // ROR $12FF,X with X=2 crosses page boundary (0x12FF -> 0x1301)
    cpu.memory_mut().write(0x8000, 0x7E);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1301, 0x10); // Value at 0x12FF + 0x02

    cpu.set_x(0x02);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x1301), 0x08); // 0x10 >> 1 = 0x08
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.cycles(), 7); // No extra cycle for page cross on write instructions
}

// ========== Edge Case Tests ==========

#[test]
fn test_ror_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    // ROR A (0x6A)
    cpu.memory_mut().write(0x8000, 0x6A);

    cpu.set_a(0x02);
    cpu.set_flag_c(false);
    cpu.set_flag_v(true); // Set overflow flag

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x01);
    assert!(cpu.flag_v()); // Overflow flag should be unchanged
}

// ========== Multiple Rotations Test ==========

#[test]
fn test_ror_sequence() {
    let mut cpu = setup_cpu();

    // First ROR: 0b10000000 -> 0b01000000
    cpu.memory_mut().write(0x8000, 0x6A);

    // Second ROR: 0b01000000 -> 0b00100000
    cpu.memory_mut().write(0x8001, 0x6A);

    // Third ROR: 0b00100000 -> 0b00010000
    cpu.memory_mut().write(0x8002, 0x6A);

    cpu.set_a(0x80);
    cpu.set_flag_c(false);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x40);
    assert_eq!(cpu.pc(), 0x8001);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x20);
    assert_eq!(cpu.pc(), 0x8002);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x10);
    assert_eq!(cpu.pc(), 0x8003);
}

#[test]
fn test_ror_progressive_rotations() {
    let mut cpu = setup_cpu();

    // Rotate 0x80 right 8 times to come full circle
    for i in 0..8 {
        cpu.memory_mut().write(0x8000 + i, 0x6A);
    }

    cpu.set_a(0x80);
    cpu.set_flag_c(false);

    let expected_values = [0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01, 0x00];
    let expected_carry = [false, false, false, false, false, false, false, true];

    for (i, (&expected_val, &expected_c)) in expected_values
        .iter()
        .zip(expected_carry.iter())
        .enumerate()
    {
        cpu.step().unwrap();
        assert_eq!(cpu.a(), expected_val, "Failed at iteration {}", i);
        assert_eq!(
            cpu.flag_c(),
            expected_c,
            "Carry flag failed at iteration {}",
            i
        );
    }

    // After 8 rotations with carry involved, value cycles back
    assert!(cpu.flag_z()); // Final result is zero
}

#[test]
fn test_ror_carry_propagation() {
    let mut cpu = setup_cpu();

    // ROR twice: carry should propagate through
    cpu.memory_mut().write(0x8000, 0x6A); // First ROR
    cpu.memory_mut().write(0x8001, 0x6A); // Second ROR

    cpu.set_a(0x01); // 0b00000001
    cpu.set_flag_c(false);

    // First ROR: 0b00000001 -> 0b00000000, carry = 1
    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x00);
    assert!(cpu.flag_c());

    // Second ROR: 0b00000000 with carry=1 -> 0b10000000, carry = 0
    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x80);
    assert!(!cpu.flag_c());
}

#[test]
fn test_ror_full_rotation_with_carry() {
    let mut cpu = setup_cpu();

    // Rotate 0xFF with carry set - all bits should rotate through
    for i in 0..9 {
        cpu.memory_mut().write(0x8000 + i, 0x6A);
    }

    cpu.set_a(0xFF);
    cpu.set_flag_c(true);

    // After 9 rotations, all bits including carry should cycle back
    for _ in 0..9 {
        cpu.step().unwrap();
    }

    assert_eq!(cpu.a(), 0xFF); // All bits rotated back
    assert!(cpu.flag_c()); // Carry also rotated back
}
