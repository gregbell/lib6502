//! Comprehensive tests for the CPY (Compare Y Register) instruction.
//!
//! Tests cover:
//! - All 3 addressing modes (Immediate, Zero Page, Absolute)
//! - Flag updates (C, Z, N)
//! - Various comparison scenarios (Y > M, Y == M, Y < M)
//! - Edge cases and boundary conditions
//! - Correct cycle counts
//! - Y register is NOT modified

use cpu6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Immediate Mode Tests ==========

#[test]
fn test_cpy_immediate_equal() {
    let mut cpu = setup_cpu();

    // CPY #$42 (0xC0 0x42) - Y = M = 0x42
    cpu.memory_mut().write(0x8000, 0xC0);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_y(0x42);

    cpu.step().unwrap();

    // Y == M, so Z should be set, C should be set (Y >= M)
    assert!(cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(!cpu.flag_n()); // Result is 0, so N is clear

    // Y register should NOT be modified
    assert_eq!(cpu.y(), 0x42);

    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 2);
}

#[test]
fn test_cpy_immediate_greater() {
    let mut cpu = setup_cpu();

    // CPY #$30 (0xC0 0x30) - Y = 0x42, M = 0x30
    cpu.memory_mut().write(0x8000, 0xC0);
    cpu.memory_mut().write(0x8001, 0x30);

    cpu.set_y(0x42);

    cpu.step().unwrap();

    // Y > M, so C should be set, Z should be clear
    assert!(!cpu.flag_z());
    assert!(cpu.flag_c());
    // Result is 0x42 - 0x30 = 0x12, bit 7 is 0, so N is clear
    assert!(!cpu.flag_n());

    // Y register should NOT be modified
    assert_eq!(cpu.y(), 0x42);

    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 2);
}

#[test]
fn test_cpy_immediate_less() {
    let mut cpu = setup_cpu();

    // CPY #$50 (0xC0 0x50) - Y = 0x42, M = 0x50
    cpu.memory_mut().write(0x8000, 0xC0);
    cpu.memory_mut().write(0x8001, 0x50);

    cpu.set_y(0x42);

    cpu.step().unwrap();

    // Y < M, so C should be clear, Z should be clear
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_c());
    // Result is 0x42 - 0x50 = 0xF2 (wrapping), bit 7 is 1, so N is set
    assert!(cpu.flag_n());

    // Y register should NOT be modified
    assert_eq!(cpu.y(), 0x42);

    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 2);
}

// ========== Zero Page Mode Tests ==========

#[test]
fn test_cpy_zero_page() {
    let mut cpu = setup_cpu();

    // CPY $42 (0xC4 0x42)
    cpu.memory_mut().write(0x8000, 0xC4);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x30); // Value at zero page

    cpu.set_y(0x42);

    cpu.step().unwrap();

    // Y > M (0x42 > 0x30)
    assert!(!cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(!cpu.flag_n());

    assert_eq!(cpu.y(), 0x42);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 3);
}

#[test]
fn test_cpy_zero_page_equal() {
    let mut cpu = setup_cpu();

    // CPY $42 (0xC4 0x42)
    cpu.memory_mut().write(0x8000, 0xC4);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x42); // Value at zero page

    cpu.set_y(0x42);

    cpu.step().unwrap();

    // Y == M
    assert!(cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(!cpu.flag_n());

    assert_eq!(cpu.y(), 0x42);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 3);
}

#[test]
fn test_cpy_zero_page_less() {
    let mut cpu = setup_cpu();

    // CPY $42 (0xC4 0x42)
    cpu.memory_mut().write(0x8000, 0xC4);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x50); // Value at zero page

    cpu.set_y(0x30);

    cpu.step().unwrap();

    // Y < M (0x30 < 0x50)
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_c());
    assert!(cpu.flag_n()); // 0x30 - 0x50 = 0xE0 (negative)

    assert_eq!(cpu.y(), 0x30);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 3);
}

// ========== Absolute Mode Tests ==========

#[test]
fn test_cpy_absolute() {
    let mut cpu = setup_cpu();

    // CPY $1234 (0xCC 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0xCC);
    cpu.memory_mut().write(0x8001, 0x34); // Low byte
    cpu.memory_mut().write(0x8002, 0x12); // High byte
    cpu.memory_mut().write(0x1234, 0x42);

    cpu.set_y(0x42);

    cpu.step().unwrap();

    // Y == M
    assert!(cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(!cpu.flag_n());

    assert_eq!(cpu.y(), 0x42);
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_cpy_absolute_greater() {
    let mut cpu = setup_cpu();

    // CPY $1234 (0xCC 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0xCC);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1234, 0x30);

    cpu.set_y(0x42);

    cpu.step().unwrap();

    // Y > M
    assert!(!cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(!cpu.flag_n());

    assert_eq!(cpu.y(), 0x42);
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_cpy_absolute_less() {
    let mut cpu = setup_cpu();

    // CPY $1234 (0xCC 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0xCC);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1234, 0x50);

    cpu.set_y(0x30);

    cpu.step().unwrap();

    // Y < M
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_c());
    assert!(cpu.flag_n()); // 0x30 - 0x50 = 0xE0 (negative)

    assert_eq!(cpu.y(), 0x30);
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 4);
}

// ========== Flag Behavior Tests ==========

#[test]
fn test_cpy_zero_flag() {
    let mut cpu = setup_cpu();

    // Test Z flag when Y == M
    cpu.memory_mut().write(0x8000, 0xC0);
    cpu.memory_mut().write(0x8001, 0x7F);

    cpu.set_y(0x7F);

    cpu.step().unwrap();

    assert!(cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(!cpu.flag_n());
}

#[test]
fn test_cpy_carry_flag_set() {
    let mut cpu = setup_cpu();

    // Test C flag when Y >= M
    cpu.memory_mut().write(0x8000, 0xC0);
    cpu.memory_mut().write(0x8001, 0x50);

    cpu.set_y(0x50);

    cpu.step().unwrap();

    assert!(cpu.flag_c()); // Y == M, so C is set
}

#[test]
fn test_cpy_carry_flag_clear() {
    let mut cpu = setup_cpu();

    // Test C flag when Y < M
    cpu.memory_mut().write(0x8000, 0xC0);
    cpu.memory_mut().write(0x8001, 0x80);

    cpu.set_y(0x50);

    cpu.step().unwrap();

    assert!(!cpu.flag_c()); // Y < M, so C is clear
}

#[test]
fn test_cpy_negative_flag_set() {
    let mut cpu = setup_cpu();

    // Test N flag when result has bit 7 set
    // Y = 0x10, M = 0x20, result = 0x10 - 0x20 = 0xF0 (bit 7 set)
    cpu.memory_mut().write(0x8000, 0xC0);
    cpu.memory_mut().write(0x8001, 0x20);

    cpu.set_y(0x10);

    cpu.step().unwrap();

    assert!(cpu.flag_n()); // Result has bit 7 set
    assert!(!cpu.flag_c()); // Y < M
    assert!(!cpu.flag_z()); // Y != M
}

#[test]
fn test_cpy_negative_flag_clear() {
    let mut cpu = setup_cpu();

    // Test N flag when result has bit 7 clear
    // Y = 0x50, M = 0x30, result = 0x50 - 0x30 = 0x20 (bit 7 clear)
    cpu.memory_mut().write(0x8000, 0xC0);
    cpu.memory_mut().write(0x8001, 0x30);

    cpu.set_y(0x50);

    cpu.step().unwrap();

    assert!(!cpu.flag_n()); // Result has bit 7 clear
    assert!(cpu.flag_c()); // Y > M
    assert!(!cpu.flag_z()); // Y != M
}

// ========== Edge Cases ==========

#[test]
fn test_cpy_zero_vs_zero() {
    let mut cpu = setup_cpu();

    // CPY #$00 with Y = 0x00
    cpu.memory_mut().write(0x8000, 0xC0);
    cpu.memory_mut().write(0x8001, 0x00);

    cpu.set_y(0x00);

    cpu.step().unwrap();

    // 0x00 == 0x00
    assert!(cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(!cpu.flag_n()); // Result is 0x00, bit 7 is 0
}

#[test]
fn test_cpy_max_values() {
    let mut cpu = setup_cpu();

    // CPY #$FF with Y = 0xFF
    cpu.memory_mut().write(0x8000, 0xC0);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_y(0xFF);

    cpu.step().unwrap();

    // 0xFF == 0xFF
    assert!(cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(!cpu.flag_n()); // Result is 0x00, bit 7 is 0
}

#[test]
fn test_cpy_zero_vs_max() {
    let mut cpu = setup_cpu();

    // CPY #$FF with Y = 0x00
    cpu.memory_mut().write(0x8000, 0xC0);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_y(0x00);

    cpu.step().unwrap();

    // 0x00 < 0xFF
    // Result: 0x00 - 0xFF = 0x01 (wrapping)
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_n()); // Result is 0x01, bit 7 is 0
}

#[test]
fn test_cpy_max_vs_zero() {
    let mut cpu = setup_cpu();

    // CPY #$00 with Y = 0xFF
    cpu.memory_mut().write(0x8000, 0xC0);
    cpu.memory_mut().write(0x8001, 0x00);

    cpu.set_y(0xFF);

    cpu.step().unwrap();

    // 0xFF > 0x00
    // Result: 0xFF - 0x00 = 0xFF
    assert!(!cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(cpu.flag_n()); // Result is 0xFF, bit 7 is 1
}

#[test]
fn test_cpy_signed_boundary() {
    let mut cpu = setup_cpu();

    // Test comparison across signed boundary
    // CPY #$7F with Y = 0x80
    cpu.memory_mut().write(0x8000, 0xC0);
    cpu.memory_mut().write(0x8001, 0x7F);

    cpu.set_y(0x80);

    cpu.step().unwrap();

    // 0x80 > 0x7F (unsigned comparison)
    // Result: 0x80 - 0x7F = 0x01
    assert!(!cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(!cpu.flag_n()); // Result is 0x01, bit 7 is 0
}

// ========== Y Register Preservation Tests ==========

#[test]
fn test_cpy_preserves_y_register() {
    let mut cpu = setup_cpu();

    // Test that Y register is NOT modified
    cpu.memory_mut().write(0x8000, 0xC0);
    cpu.memory_mut().write(0x8001, 0x30);

    cpu.set_y(0x42);

    cpu.step().unwrap();

    // Y register should still be 0x42
    assert_eq!(cpu.y(), 0x42);
}

#[test]
fn test_cpy_multiple_comparisons() {
    let mut cpu = setup_cpu();

    // Test multiple CPY instructions in sequence
    cpu.memory_mut().write(0x8000, 0xC0);
    cpu.memory_mut().write(0x8001, 0x30);
    cpu.memory_mut().write(0x8002, 0xC0);
    cpu.memory_mut().write(0x8003, 0x42);
    cpu.memory_mut().write(0x8004, 0xC0);
    cpu.memory_mut().write(0x8005, 0x50);

    cpu.set_y(0x42);

    // First comparison: 0x42 > 0x30
    cpu.step().unwrap();
    assert_eq!(cpu.y(), 0x42);
    assert!(cpu.flag_c());
    assert!(!cpu.flag_z());

    // Second comparison: 0x42 == 0x42
    cpu.step().unwrap();
    assert_eq!(cpu.y(), 0x42);
    assert!(cpu.flag_c());
    assert!(cpu.flag_z());

    // Third comparison: 0x42 < 0x50
    cpu.step().unwrap();
    assert_eq!(cpu.y(), 0x42);
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
}

// ========== Other Flags Preservation Tests ==========

#[test]
fn test_cpy_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xC0);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_y(0x42);
    cpu.set_flag_v(true); // Set overflow flag

    cpu.step().unwrap();

    // Overflow flag should be unchanged
    assert!(cpu.flag_v());
}

#[test]
fn test_cpy_preserves_decimal_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xC0);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_y(0x42);
    cpu.set_flag_d(true); // Set decimal flag

    cpu.step().unwrap();

    // Decimal flag should be unchanged
    assert!(cpu.flag_d());
}

// ========== Practical Use Cases ==========

#[test]
fn test_cpy_range_check_lower_bound() {
    let mut cpu = setup_cpu();

    // Check if Y >= lower bound (0x10)
    cpu.memory_mut().write(0x8000, 0xC0);
    cpu.memory_mut().write(0x8001, 0x10);

    cpu.set_y(0x20);

    cpu.step().unwrap();

    // Y >= 0x10, so carry is set
    assert!(cpu.flag_c());
}

#[test]
fn test_cpy_range_check_upper_bound() {
    let mut cpu = setup_cpu();

    // Check if Y < upper bound (0x80)
    cpu.memory_mut().write(0x8000, 0xC0);
    cpu.memory_mut().write(0x8001, 0x80);

    cpu.set_y(0x50);

    cpu.step().unwrap();

    // Y < 0x80, so carry is clear
    assert!(!cpu.flag_c());
}

#[test]
fn test_cpy_equality_check() {
    let mut cpu = setup_cpu();

    // Check if Y == expected value
    cpu.memory_mut().write(0x8000, 0xC0);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_y(0x42);

    cpu.step().unwrap();

    // Y == 0x42, so zero flag is set
    assert!(cpu.flag_z());
}

#[test]
fn test_cpy_loop_counter() {
    let mut cpu = setup_cpu();

    // Common use case: counting down with Y register
    // CPY #$00 to check if we've reached zero
    cpu.memory_mut().write(0x8000, 0xC0);
    cpu.memory_mut().write(0x8001, 0x00);

    // Test with Y = 1
    cpu.set_y(0x01);
    cpu.step().unwrap();

    // Y > 0, so not zero yet
    assert!(!cpu.flag_z());
    assert!(cpu.flag_c());
}
