//! Comprehensive tests for the ROL (Rotate Left) instruction.
//!
//! Tests cover:
//! - All 5 addressing modes (Accumulator, Zero Page, Zero Page,X, Absolute, Absolute,X)
//! - Flag updates (C, Z, N)
//! - Various operand values (0, positive, negative, edge cases)
//! - Cycle counts
//! - Memory write-back for non-accumulator modes
//! - Carry flag rotation behavior

use cpu6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic ROL Operation Tests ==========

#[test]
fn test_rol_accumulator_basic() {
    let mut cpu = setup_cpu();

    // ROL A (0x2A)
    cpu.memory_mut().write(0x8000, 0x2A);

    cpu.set_a(0x01); // 0b00000001
    cpu.set_flag_c(false); // Carry = 0

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x02); // 0b00000010 (rotated left, bit 0 = old carry = 0)
    assert!(!cpu.flag_c()); // Bit 7 was 0
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.cycles(), 2);
}

#[test]
fn test_rol_accumulator_with_carry_in() {
    let mut cpu = setup_cpu();

    // ROL A (0x2A)
    cpu.memory_mut().write(0x8000, 0x2A);

    cpu.set_a(0x01); // 0b00000001
    cpu.set_flag_c(true); // Carry = 1

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x03); // 0b00000011 (rotated left, bit 0 = old carry = 1)
    assert!(!cpu.flag_c()); // Bit 7 was 0
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

// ========== Carry Flag Tests ==========

#[test]
fn test_rol_carry_flag_set() {
    let mut cpu = setup_cpu();

    // ROL A (0x2A)
    cpu.memory_mut().write(0x8000, 0x2A);

    cpu.set_a(0x80); // 0b10000000 - bit 7 is set
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00); // 0b00000000 (rotated left, bit 0 = old carry = 0)
    assert!(cpu.flag_c()); // Old bit 7 was 1
    assert!(cpu.flag_z()); // Result is zero
    assert!(!cpu.flag_n());
}

#[test]
fn test_rol_carry_flag_clear() {
    let mut cpu = setup_cpu();

    // ROL A (0x2A)
    cpu.memory_mut().write(0x8000, 0x2A);

    cpu.set_a(0x7F); // 0b01111111 - bit 7 is 0
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFE); // 0b11111110
    assert!(!cpu.flag_c()); // Old bit 7 was 0
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // New bit 7 is 1
}

#[test]
fn test_rol_rotate_carry_through() {
    let mut cpu = setup_cpu();

    // ROL A (0x2A)
    cpu.memory_mut().write(0x8000, 0x2A);

    cpu.set_a(0x80); // 0b10000000
    cpu.set_flag_c(true); // Carry = 1

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x01); // 0b00000001 (rotated left, bit 0 = old carry = 1)
    assert!(cpu.flag_c()); // Old bit 7 was 1
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_rol_all_bits_set() {
    let mut cpu = setup_cpu();

    // ROL A (0x2A)
    cpu.memory_mut().write(0x8000, 0x2A);

    cpu.set_a(0xFF); // 0b11111111
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF); // 0b11111111 (all bits stay set)
    assert!(cpu.flag_c()); // Old bit 7 was 1
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is 1
}

// ========== Zero Flag Tests ==========

#[test]
fn test_rol_zero_flag_set() {
    let mut cpu = setup_cpu();

    // ROL A (0x2A)
    cpu.memory_mut().write(0x8000, 0x2A);

    cpu.set_a(0x00);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00);
    assert!(cpu.flag_z()); // Result is zero
    assert!(!cpu.flag_c()); // Bit 7 was 0
    assert!(!cpu.flag_n());
}

#[test]
fn test_rol_zero_flag_from_rotate() {
    let mut cpu = setup_cpu();

    // ROL A (0x2A)
    cpu.memory_mut().write(0x8000, 0x2A);

    cpu.set_a(0x80); // 0b10000000
    cpu.set_flag_c(false); // Carry = 0

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00); // Rotated to zero
    assert!(cpu.flag_z()); // Result is zero
    assert!(cpu.flag_c()); // Old bit 7 was 1
}

#[test]
fn test_rol_zero_with_carry_produces_one() {
    let mut cpu = setup_cpu();

    // ROL A (0x2A)
    cpu.memory_mut().write(0x8000, 0x2A);

    cpu.set_a(0x00);
    cpu.set_flag_c(true); // Carry = 1

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x01); // 0b00000001 (bit 0 = old carry = 1)
    assert!(!cpu.flag_z()); // Result is not zero
    assert!(!cpu.flag_c()); // Old bit 7 was 0
    assert!(!cpu.flag_n());
}

// ========== Negative Flag Tests ==========

#[test]
fn test_rol_negative_flag_set() {
    let mut cpu = setup_cpu();

    // ROL A (0x2A)
    cpu.memory_mut().write(0x8000, 0x2A);

    cpu.set_a(0x40); // 0b01000000
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x80); // 0b10000000
    assert!(cpu.flag_n()); // Bit 7 is now set
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
}

#[test]
fn test_rol_negative_flag_clear() {
    let mut cpu = setup_cpu();

    // ROL A (0x2A)
    cpu.memory_mut().write(0x8000, 0x2A);

    cpu.set_a(0x20); // 0b00100000
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x40); // 0b01000000
    assert!(!cpu.flag_n()); // Bit 7 is 0
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
}

// ========== Addressing Mode Tests ==========

#[test]
fn test_rol_zero_page() {
    let mut cpu = setup_cpu();

    // ROL $42 (0x26 0x42)
    cpu.memory_mut().write(0x8000, 0x26);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x55); // 0b01010101

    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0042), 0xAA); // 0b10101010
    assert!(!cpu.flag_c()); // Old bit 7 was 0
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // New bit 7 is 1
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 5);
}

#[test]
fn test_rol_zero_page_with_carry() {
    let mut cpu = setup_cpu();

    // ROL $10 (0x26 0x10)
    cpu.memory_mut().write(0x8000, 0x26);
    cpu.memory_mut().write(0x8001, 0x10);
    cpu.memory_mut().write(0x0010, 0x03); // 0b00000011

    cpu.set_flag_c(true); // Carry = 1

    cpu.step().unwrap();

    // Verify memory was updated: 0b00000111 (bit 0 set from carry)
    assert_eq!(cpu.memory_mut().read(0x0010), 0x07);
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
}

#[test]
fn test_rol_zero_page_x() {
    let mut cpu = setup_cpu();

    // ROL $40,X (0x36 0x40)
    cpu.memory_mut().write(0x8000, 0x36);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0045, 0x0F); // Value at 0x40 + 0x05

    cpu.set_x(0x05);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0045), 0x1E); // 0x0F << 1 = 0x1E
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_rol_zero_page_x_wrap() {
    let mut cpu = setup_cpu();

    // ROL $FF,X with X=2 should wrap to 0x01 within zero page
    cpu.memory_mut().write(0x8000, 0x36);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x0001, 0x02); // Value at (0xFF + 0x02) % 256 = 0x01

    cpu.set_x(0x02);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x0001), 0x04); // 2 << 1 = 4
    assert!(!cpu.flag_c());
}

#[test]
fn test_rol_absolute() {
    let mut cpu = setup_cpu();

    // ROL $1234 (0x2E 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0x2E);
    cpu.memory_mut().write(0x8001, 0x34); // Low byte
    cpu.memory_mut().write(0x8002, 0x12); // High byte
    cpu.memory_mut().write(0x1234, 0x33);

    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x1234), 0x66); // 0x33 << 1 = 0x66
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_rol_absolute_x() {
    let mut cpu = setup_cpu();

    // ROL $1200,X (0x3E 0x00 0x12)
    cpu.memory_mut().write(0x8000, 0x3E);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1205, 0x11); // Value at 0x1200 + 0x05

    cpu.set_x(0x05);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x1205), 0x22); // 0x11 << 1 = 0x22
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 7);
}

#[test]
fn test_rol_absolute_x_with_page_cross() {
    let mut cpu = setup_cpu();

    // ROL $12FF,X with X=2 crosses page boundary (0x12FF -> 0x1301)
    cpu.memory_mut().write(0x8000, 0x3E);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1301, 0x08); // Value at 0x12FF + 0x02

    cpu.set_x(0x02);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x1301), 0x10); // 0x08 << 1 = 0x10
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.cycles(), 7); // No extra cycle for page cross on write instructions
}

// ========== Edge Case Tests ==========

#[test]
fn test_rol_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    // ROL A (0x2A)
    cpu.memory_mut().write(0x8000, 0x2A);

    cpu.set_a(0x01);
    cpu.set_flag_c(false);
    cpu.set_flag_v(true); // Set overflow flag

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x02);
    assert!(cpu.flag_v()); // Overflow flag should be unchanged
}

// ========== Multiple Rotations Test ==========

#[test]
fn test_rol_sequence() {
    let mut cpu = setup_cpu();

    // First ROL: 0b00000001 -> 0b00000010
    cpu.memory_mut().write(0x8000, 0x2A);

    // Second ROL: 0b00000010 -> 0b00000100
    cpu.memory_mut().write(0x8001, 0x2A);

    // Third ROL: 0b00000100 -> 0b00001000
    cpu.memory_mut().write(0x8002, 0x2A);

    cpu.set_a(0x01);
    cpu.set_flag_c(false);

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
fn test_rol_progressive_rotations() {
    let mut cpu = setup_cpu();

    // Rotate 0x01 left 8 times to come full circle
    for i in 0..8 {
        cpu.memory_mut().write(0x8000 + i, 0x2A);
    }

    cpu.set_a(0x01);
    cpu.set_flag_c(false);

    let expected_values = [0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x00];
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
fn test_rol_carry_propagation() {
    let mut cpu = setup_cpu();

    // ROL twice: carry should propagate through
    cpu.memory_mut().write(0x8000, 0x2A); // First ROL
    cpu.memory_mut().write(0x8001, 0x2A); // Second ROL

    cpu.set_a(0x80); // 0b10000000
    cpu.set_flag_c(false);

    // First ROL: 0b10000000 -> 0b00000000, carry = 1
    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x00);
    assert!(cpu.flag_c());

    // Second ROL: 0b00000000 with carry=1 -> 0b00000001, carry = 0
    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x01);
    assert!(!cpu.flag_c());
}

#[test]
fn test_rol_full_rotation_with_carry() {
    let mut cpu = setup_cpu();

    // Rotate 0xFF with carry set - all bits should rotate through
    for i in 0..9 {
        cpu.memory_mut().write(0x8000 + i, 0x2A);
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
