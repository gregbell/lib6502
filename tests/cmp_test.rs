//! Comprehensive tests for the CMP (Compare Accumulator) instruction.
//!
//! Tests cover:
//! - All 8 addressing modes
//! - Flag updates (C, Z, N)
//! - Various comparison scenarios (A > M, A == M, A < M)
//! - Edge cases and boundary conditions
//! - Correct cycle counts (including page crossing)
//! - Accumulator is NOT modified

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
fn test_cmp_immediate_equal() {
    let mut cpu = setup_cpu();

    // CMP #$42 (0xC9 0x42) - A = M = 0x42
    cpu.memory_mut().write(0x8000, 0xC9);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_a(0x42);

    cpu.step().unwrap();

    // A == M, so Z should be set, C should be set (A >= M)
    assert!(cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(!cpu.flag_n()); // Result is 0, so N is clear

    // Accumulator should NOT be modified
    assert_eq!(cpu.a(), 0x42);

    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 2);
}

#[test]
fn test_cmp_immediate_greater() {
    let mut cpu = setup_cpu();

    // CMP #$30 (0xC9 0x30) - A = 0x42, M = 0x30
    cpu.memory_mut().write(0x8000, 0xC9);
    cpu.memory_mut().write(0x8001, 0x30);

    cpu.set_a(0x42);

    cpu.step().unwrap();

    // A > M, so C should be set, Z should be clear
    assert!(!cpu.flag_z());
    assert!(cpu.flag_c());
    // Result is 0x42 - 0x30 = 0x12, bit 7 is 0, so N is clear
    assert!(!cpu.flag_n());

    // Accumulator should NOT be modified
    assert_eq!(cpu.a(), 0x42);

    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 2);
}

#[test]
fn test_cmp_immediate_less() {
    let mut cpu = setup_cpu();

    // CMP #$50 (0xC9 0x50) - A = 0x42, M = 0x50
    cpu.memory_mut().write(0x8000, 0xC9);
    cpu.memory_mut().write(0x8001, 0x50);

    cpu.set_a(0x42);

    cpu.step().unwrap();

    // A < M, so C should be clear, Z should be clear
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_c());
    // Result is 0x42 - 0x50 = 0xF2 (wrapping), bit 7 is 1, so N is set
    assert!(cpu.flag_n());

    // Accumulator should NOT be modified
    assert_eq!(cpu.a(), 0x42);

    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 2);
}

// ========== Zero Page Mode Tests ==========

#[test]
fn test_cmp_zero_page() {
    let mut cpu = setup_cpu();

    // CMP $42 (0xC5 0x42)
    cpu.memory_mut().write(0x8000, 0xC5);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x30); // Value at zero page

    cpu.set_a(0x42);

    cpu.step().unwrap();

    // A > M (0x42 > 0x30)
    assert!(!cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(!cpu.flag_n());

    assert_eq!(cpu.a(), 0x42);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 3);
}

// ========== Zero Page,X Mode Tests ==========

#[test]
fn test_cmp_zero_page_x() {
    let mut cpu = setup_cpu();

    // CMP $40,X (0xD5 0x40) with X = 0x05
    cpu.memory_mut().write(0x8000, 0xD5);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0045, 0x42); // Value at 0x40 + 0x05

    cpu.set_a(0x42);
    cpu.set_x(0x05);

    cpu.step().unwrap();

    // A == M (0x42 == 0x42)
    assert!(cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(!cpu.flag_n());

    assert_eq!(cpu.a(), 0x42);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_cmp_zero_page_x_wraparound() {
    let mut cpu = setup_cpu();

    // CMP $FF,X (0xD5 0xFF) with X = 0x05
    // Should wrap to 0x04 in zero page
    cpu.memory_mut().write(0x8000, 0xD5);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x0004, 0x20); // Value at (0xFF + 0x05) & 0xFF = 0x04

    cpu.set_a(0x30);
    cpu.set_x(0x05);

    cpu.step().unwrap();

    // A > M (0x30 > 0x20)
    assert!(!cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(!cpu.flag_n());

    assert_eq!(cpu.a(), 0x30);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 4);
}

// ========== Absolute Mode Tests ==========

#[test]
fn test_cmp_absolute() {
    let mut cpu = setup_cpu();

    // CMP $1234 (0xCD 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0xCD);
    cpu.memory_mut().write(0x8001, 0x34); // Low byte
    cpu.memory_mut().write(0x8002, 0x12); // High byte
    cpu.memory_mut().write(0x1234, 0x42);

    cpu.set_a(0x42);

    cpu.step().unwrap();

    // A == M
    assert!(cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(!cpu.flag_n());

    assert_eq!(cpu.a(), 0x42);
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 4);
}

// ========== Absolute,X Mode Tests ==========

#[test]
fn test_cmp_absolute_x_no_page_cross() {
    let mut cpu = setup_cpu();

    // CMP $1234,X (0xDD 0x34 0x12) with X = 0x05
    cpu.memory_mut().write(0x8000, 0xDD);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1239, 0x30); // Value at 0x1234 + 0x05

    cpu.set_a(0x42);
    cpu.set_x(0x05);

    cpu.step().unwrap();

    // A > M
    assert!(!cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(!cpu.flag_n());

    assert_eq!(cpu.a(), 0x42);
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 4); // No page crossing
}

#[test]
fn test_cmp_absolute_x_page_cross() {
    let mut cpu = setup_cpu();

    // CMP $12FF,X (0xDD 0xFF 0x12) with X = 0x05
    // Crosses page boundary: 0x12FF + 0x05 = 0x1304
    cpu.memory_mut().write(0x8000, 0xDD);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1304, 0x42);

    cpu.set_a(0x30);
    cpu.set_x(0x05);

    cpu.step().unwrap();

    // A < M
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_c());
    assert!(cpu.flag_n()); // 0x30 - 0x42 = 0xEE (negative)

    assert_eq!(cpu.a(), 0x30);
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 5); // +1 for page crossing
}

// ========== Absolute,Y Mode Tests ==========

#[test]
fn test_cmp_absolute_y_no_page_cross() {
    let mut cpu = setup_cpu();

    // CMP $1234,Y (0xD9 0x34 0x12) with Y = 0x05
    cpu.memory_mut().write(0x8000, 0xD9);
    cpu.memory_mut().write(0x8001, 0x34);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1239, 0x42);

    cpu.set_a(0x42);
    cpu.set_y(0x05);

    cpu.step().unwrap();

    // A == M
    assert!(cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(!cpu.flag_n());

    assert_eq!(cpu.a(), 0x42);
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 4); // No page crossing
}

#[test]
fn test_cmp_absolute_y_page_cross() {
    let mut cpu = setup_cpu();

    // CMP $12FF,Y (0xD9 0xFF 0x12) with Y = 0x10
    // Crosses page boundary: 0x12FF + 0x10 = 0x130F
    cpu.memory_mut().write(0x8000, 0xD9);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x130F, 0x20);

    cpu.set_a(0x50);
    cpu.set_y(0x10);

    cpu.step().unwrap();

    // A > M
    assert!(!cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(!cpu.flag_n());

    assert_eq!(cpu.a(), 0x50);
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 5); // +1 for page crossing
}

// ========== (Indirect,X) Mode Tests ==========

#[test]
fn test_cmp_indirect_x() {
    let mut cpu = setup_cpu();

    // CMP ($40,X) (0xC1 0x40) with X = 0x05
    // Zero page address: 0x40 + 0x05 = 0x45
    // Pointer at 0x45/0x46 points to 0x1234
    cpu.memory_mut().write(0x8000, 0xC1);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0045, 0x34); // Low byte of pointer
    cpu.memory_mut().write(0x0046, 0x12); // High byte of pointer
    cpu.memory_mut().write(0x1234, 0x42); // Value at target address

    cpu.set_a(0x42);
    cpu.set_x(0x05);

    cpu.step().unwrap();

    // A == M
    assert!(cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(!cpu.flag_n());

    assert_eq!(cpu.a(), 0x42);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_cmp_indirect_x_wraparound() {
    let mut cpu = setup_cpu();

    // CMP ($FF,X) (0xC1 0xFF) with X = 0x02
    // Zero page address: (0xFF + 0x02) & 0xFF = 0x01
    cpu.memory_mut().write(0x8000, 0xC1);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x0001, 0x00); // Low byte of pointer
    cpu.memory_mut().write(0x0002, 0x20); // High byte of pointer
    cpu.memory_mut().write(0x2000, 0x30);

    cpu.set_a(0x50);
    cpu.set_x(0x02);

    cpu.step().unwrap();

    // A > M
    assert!(!cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(!cpu.flag_n());

    assert_eq!(cpu.a(), 0x50);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 6);
}

// ========== (Indirect),Y Mode Tests ==========

#[test]
fn test_cmp_indirect_y_no_page_cross() {
    let mut cpu = setup_cpu();

    // CMP ($40),Y (0xD1 0x40) with Y = 0x05
    // Pointer at 0x40/0x41 points to 0x1234
    // Final address: 0x1234 + 0x05 = 0x1239
    cpu.memory_mut().write(0x8000, 0xD1);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0040, 0x34); // Low byte of pointer
    cpu.memory_mut().write(0x0041, 0x12); // High byte of pointer
    cpu.memory_mut().write(0x1239, 0x42);

    cpu.set_a(0x42);
    cpu.set_y(0x05);

    cpu.step().unwrap();

    // A == M
    assert!(cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(!cpu.flag_n());

    assert_eq!(cpu.a(), 0x42);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 5); // No page crossing
}

#[test]
fn test_cmp_indirect_y_page_cross() {
    let mut cpu = setup_cpu();

    // CMP ($40),Y (0xD1 0x40) with Y = 0x10
    // Pointer at 0x40/0x41 points to 0x12FF
    // Final address: 0x12FF + 0x10 = 0x130F (page cross)
    cpu.memory_mut().write(0x8000, 0xD1);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0040, 0xFF); // Low byte of pointer
    cpu.memory_mut().write(0x0041, 0x12); // High byte of pointer
    cpu.memory_mut().write(0x130F, 0x30);

    cpu.set_a(0x20);
    cpu.set_y(0x10);

    cpu.step().unwrap();

    // A < M
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_c());
    assert!(cpu.flag_n()); // 0x20 - 0x30 = 0xF0 (negative)

    assert_eq!(cpu.a(), 0x20);
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 6); // +1 for page crossing
}

// ========== Flag Behavior Tests ==========

#[test]
fn test_cmp_zero_flag() {
    let mut cpu = setup_cpu();

    // Test Z flag when A == M
    cpu.memory_mut().write(0x8000, 0xC9);
    cpu.memory_mut().write(0x8001, 0x7F);

    cpu.set_a(0x7F);

    cpu.step().unwrap();

    assert!(cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(!cpu.flag_n());
}

#[test]
fn test_cmp_carry_flag_set() {
    let mut cpu = setup_cpu();

    // Test C flag when A >= M
    cpu.memory_mut().write(0x8000, 0xC9);
    cpu.memory_mut().write(0x8001, 0x50);

    cpu.set_a(0x50);

    cpu.step().unwrap();

    assert!(cpu.flag_c()); // A == M, so C is set
}

#[test]
fn test_cmp_carry_flag_clear() {
    let mut cpu = setup_cpu();

    // Test C flag when A < M
    cpu.memory_mut().write(0x8000, 0xC9);
    cpu.memory_mut().write(0x8001, 0x80);

    cpu.set_a(0x50);

    cpu.step().unwrap();

    assert!(!cpu.flag_c()); // A < M, so C is clear
}

#[test]
fn test_cmp_negative_flag_set() {
    let mut cpu = setup_cpu();

    // Test N flag when result has bit 7 set
    // A = 0x10, M = 0x20, result = 0x10 - 0x20 = 0xF0 (bit 7 set)
    cpu.memory_mut().write(0x8000, 0xC9);
    cpu.memory_mut().write(0x8001, 0x20);

    cpu.set_a(0x10);

    cpu.step().unwrap();

    assert!(cpu.flag_n()); // Result has bit 7 set
    assert!(!cpu.flag_c()); // A < M
    assert!(!cpu.flag_z()); // A != M
}

#[test]
fn test_cmp_negative_flag_clear() {
    let mut cpu = setup_cpu();

    // Test N flag when result has bit 7 clear
    // A = 0x50, M = 0x30, result = 0x50 - 0x30 = 0x20 (bit 7 clear)
    cpu.memory_mut().write(0x8000, 0xC9);
    cpu.memory_mut().write(0x8001, 0x30);

    cpu.set_a(0x50);

    cpu.step().unwrap();

    assert!(!cpu.flag_n()); // Result has bit 7 clear
    assert!(cpu.flag_c()); // A > M
    assert!(!cpu.flag_z()); // A != M
}

// ========== Edge Cases ==========

#[test]
fn test_cmp_zero_vs_zero() {
    let mut cpu = setup_cpu();

    // CMP #$00 with A = 0x00
    cpu.memory_mut().write(0x8000, 0xC9);
    cpu.memory_mut().write(0x8001, 0x00);

    cpu.set_a(0x00);

    cpu.step().unwrap();

    // 0x00 == 0x00
    assert!(cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(!cpu.flag_n()); // Result is 0x00, bit 7 is 0
}

#[test]
fn test_cmp_max_values() {
    let mut cpu = setup_cpu();

    // CMP #$FF with A = 0xFF
    cpu.memory_mut().write(0x8000, 0xC9);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    // 0xFF == 0xFF
    assert!(cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(!cpu.flag_n()); // Result is 0x00, bit 7 is 0
}

#[test]
fn test_cmp_zero_vs_max() {
    let mut cpu = setup_cpu();

    // CMP #$FF with A = 0x00
    cpu.memory_mut().write(0x8000, 0xC9);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_a(0x00);

    cpu.step().unwrap();

    // 0x00 < 0xFF
    // Result: 0x00 - 0xFF = 0x01 (wrapping)
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_n()); // Result is 0x01, bit 7 is 0
}

#[test]
fn test_cmp_max_vs_zero() {
    let mut cpu = setup_cpu();

    // CMP #$00 with A = 0xFF
    cpu.memory_mut().write(0x8000, 0xC9);
    cpu.memory_mut().write(0x8001, 0x00);

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    // 0xFF > 0x00
    // Result: 0xFF - 0x00 = 0xFF
    assert!(!cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(cpu.flag_n()); // Result is 0xFF, bit 7 is 1
}

#[test]
fn test_cmp_signed_boundary() {
    let mut cpu = setup_cpu();

    // Test comparison across signed boundary
    // CMP #$7F with A = 0x80
    cpu.memory_mut().write(0x8000, 0xC9);
    cpu.memory_mut().write(0x8001, 0x7F);

    cpu.set_a(0x80);

    cpu.step().unwrap();

    // 0x80 > 0x7F (unsigned comparison)
    // Result: 0x80 - 0x7F = 0x01
    assert!(!cpu.flag_z());
    assert!(cpu.flag_c());
    assert!(!cpu.flag_n()); // Result is 0x01, bit 7 is 0
}

// ========== Accumulator Preservation Tests ==========

#[test]
fn test_cmp_preserves_accumulator() {
    let mut cpu = setup_cpu();

    // Test that accumulator is NOT modified
    cpu.memory_mut().write(0x8000, 0xC9);
    cpu.memory_mut().write(0x8001, 0x30);

    cpu.set_a(0x42);

    cpu.step().unwrap();

    // Accumulator should still be 0x42
    assert_eq!(cpu.a(), 0x42);
}

#[test]
fn test_cmp_multiple_comparisons() {
    let mut cpu = setup_cpu();

    // Test multiple CMP instructions in sequence
    cpu.memory_mut().write(0x8000, 0xC9);
    cpu.memory_mut().write(0x8001, 0x30);
    cpu.memory_mut().write(0x8002, 0xC9);
    cpu.memory_mut().write(0x8003, 0x42);
    cpu.memory_mut().write(0x8004, 0xC9);
    cpu.memory_mut().write(0x8005, 0x50);

    cpu.set_a(0x42);

    // First comparison: 0x42 > 0x30
    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x42);
    assert!(cpu.flag_c());
    assert!(!cpu.flag_z());

    // Second comparison: 0x42 == 0x42
    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x42);
    assert!(cpu.flag_c());
    assert!(cpu.flag_z());

    // Third comparison: 0x42 < 0x50
    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x42);
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
}

// ========== Other Flags Preservation Tests ==========

#[test]
fn test_cmp_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xC9);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_a(0x42);
    cpu.set_flag_v(true); // Set overflow flag

    cpu.step().unwrap();

    // Overflow flag should be unchanged
    assert!(cpu.flag_v());
}

#[test]
fn test_cmp_preserves_decimal_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0xC9);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_a(0x42);
    cpu.set_flag_d(true); // Set decimal flag

    cpu.step().unwrap();

    // Decimal flag should be unchanged
    assert!(cpu.flag_d());
}

// ========== Practical Use Cases ==========

#[test]
fn test_cmp_range_check_lower_bound() {
    let mut cpu = setup_cpu();

    // Check if A >= lower bound (0x10)
    cpu.memory_mut().write(0x8000, 0xC9);
    cpu.memory_mut().write(0x8001, 0x10);

    cpu.set_a(0x20);

    cpu.step().unwrap();

    // A >= 0x10, so carry is set
    assert!(cpu.flag_c());
}

#[test]
fn test_cmp_range_check_upper_bound() {
    let mut cpu = setup_cpu();

    // Check if A < upper bound (0x80)
    cpu.memory_mut().write(0x8000, 0xC9);
    cpu.memory_mut().write(0x8001, 0x80);

    cpu.set_a(0x50);

    cpu.step().unwrap();

    // A < 0x80, so carry is clear
    assert!(!cpu.flag_c());
}

#[test]
fn test_cmp_equality_check() {
    let mut cpu = setup_cpu();

    // Check if A == expected value
    cpu.memory_mut().write(0x8000, 0xC9);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_a(0x42);

    cpu.step().unwrap();

    // A == 0x42, so zero flag is set
    assert!(cpu.flag_z());
}
