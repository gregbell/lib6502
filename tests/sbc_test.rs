//! Comprehensive tests for the SBC (Subtract with Carry) instruction.
//!
//! Tests cover:
//! - All 8 addressing modes
//! - Flag updates (C, Z, V, N)
//! - Various operand values (0, positive, negative)
//! - Overflow/underflow conditions
//! - Cycle counts including page crossing penalties

use cpu6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic SBC Operation Tests ==========

#[test]
fn test_sbc_immediate_basic() {
    let mut cpu = setup_cpu();

    // SBC #$05 (0xE9 0x05)
    cpu.memory_mut().write(0x8000, 0xE9);
    cpu.memory_mut().write(0x8001, 0x05);

    cpu.set_a(0x10);
    cpu.set_flag_c(true); // Carry set = no borrow

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x0B); // 0x10 - 0x05 = 0x0B
    assert!(cpu.flag_c()); // No borrow
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_v());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 2);
}

#[test]
fn test_sbc_with_borrow() {
    let mut cpu = setup_cpu();

    // SBC #$05
    cpu.memory_mut().write(0x8000, 0xE9);
    cpu.memory_mut().write(0x8001, 0x05);

    cpu.set_a(0x10);
    cpu.set_flag_c(false); // Carry clear = borrow

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x0A); // 0x10 - 0x05 - 1 = 0x0A
}

// ========== Flag Tests ==========

#[test]
fn test_sbc_carry_flag_no_borrow() {
    let mut cpu = setup_cpu();

    // SBC #$05
    cpu.memory_mut().write(0x8000, 0xE9);
    cpu.memory_mut().write(0x8001, 0x05);

    cpu.set_a(0x10);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x0B);
    assert!(cpu.flag_c()); // Carry set (no borrow)
}

#[test]
fn test_sbc_carry_flag_with_borrow() {
    let mut cpu = setup_cpu();

    // SBC #$10
    cpu.memory_mut().write(0x8000, 0xE9);
    cpu.memory_mut().write(0x8001, 0x10);

    cpu.set_a(0x05);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xF5); // 0x05 - 0x10 = -11 = 0xF5
    assert!(!cpu.flag_c()); // Carry clear (borrow occurred)
}

#[test]
fn test_sbc_zero_flag() {
    let mut cpu = setup_cpu();

    // SBC #$05
    cpu.memory_mut().write(0x8000, 0xE9);
    cpu.memory_mut().write(0x8001, 0x05);

    cpu.set_a(0x05);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00);
    assert!(cpu.flag_z());
    assert!(cpu.flag_c()); // No borrow
}

#[test]
fn test_sbc_negative_flag() {
    let mut cpu = setup_cpu();

    // SBC #$10
    cpu.memory_mut().write(0x8000, 0xE9);
    cpu.memory_mut().write(0x8001, 0x10);

    cpu.set_a(0x00);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xF0); // 0x00 - 0x10 = -16 = 0xF0
    assert!(cpu.flag_n()); // Bit 7 is set
    assert!(!cpu.flag_c()); // Borrow occurred
}

#[test]
fn test_sbc_overflow_positive_to_negative() {
    let mut cpu = setup_cpu();

    // Subtracting a negative from a positive that overflows to negative
    // 0x50 (80) - 0xB0 (-80) = 0xA0 (-96 signed, overflow)
    cpu.memory_mut().write(0x8000, 0xE9);
    cpu.memory_mut().write(0x8001, 0xB0);

    cpu.set_a(0x50);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xA0);
    assert!(cpu.flag_v()); // Overflow occurred
    assert!(cpu.flag_n()); // Result is negative
}

#[test]
fn test_sbc_overflow_negative_to_positive() {
    let mut cpu = setup_cpu();

    // Subtracting a positive from a negative that overflows to positive
    // 0x80 (-128) - 0x01 (1) = 0x7F (127, overflow)
    cpu.memory_mut().write(0x8000, 0xE9);
    cpu.memory_mut().write(0x8001, 0x01);

    cpu.set_a(0x80);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x7F);
    assert!(cpu.flag_v()); // Overflow occurred
    assert!(!cpu.flag_n()); // Result is positive
}

#[test]
fn test_sbc_no_overflow_positive_positive() {
    let mut cpu = setup_cpu();

    // Subtracting two positive numbers without overflow
    // 0x50 (80) - 0x20 (32) = 0x30 (48)
    cpu.memory_mut().write(0x8000, 0xE9);
    cpu.memory_mut().write(0x8001, 0x20);

    cpu.set_a(0x50);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x30);
    assert!(!cpu.flag_v()); // No overflow
}

#[test]
fn test_sbc_no_overflow_negative_negative() {
    let mut cpu = setup_cpu();

    // Subtracting two negative numbers without signed overflow
    // 0xFE (-2) - 0xFF (-1) = 0xFF (-1)
    cpu.memory_mut().write(0x8000, 0xE9);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_a(0xFE);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF);
    assert!(!cpu.flag_v()); // No signed overflow
}

// ========== Addressing Mode Tests ==========

#[test]
fn test_sbc_zero_page() {
    let mut cpu = setup_cpu();

    // SBC $42 (0xE5 0x42)
    cpu.memory_mut().write(0x8000, 0xE5);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x33); // Value at zero page address

    cpu.set_a(0x50);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x1D); // 0x50 - 0x33 = 0x1D
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 3);
}

#[test]
fn test_sbc_zero_page_x() {
    let mut cpu = setup_cpu();

    // SBC $40,X (0xF5 0x40)
    cpu.memory_mut().write(0x8000, 0xF5);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0045, 0x22); // Value at 0x40 + 0x05

    cpu.set_a(0x50);
    cpu.set_x(0x05);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x2E); // 0x50 - 0x22 = 0x2E
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_sbc_zero_page_x_wrap() {
    let mut cpu = setup_cpu();

    // SBC $FF,X with X=2 should wrap to 0x01 within zero page
    cpu.memory_mut().write(0x8000, 0xF5);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x0001, 0x10); // Value at (0xFF + 0x02) % 256 = 0x01

    cpu.set_a(0x50);
    cpu.set_x(0x02);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x40); // 0x50 - 0x10 = 0x40
}

#[test]
fn test_sbc_absolute() {
    let mut cpu = setup_cpu();

    // SBC $1234 (0xED 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0xED);
    cpu.memory_mut().write(0x8001, 0x34); // Low byte
    cpu.memory_mut().write(0x8002, 0x12); // High byte
    cpu.memory_mut().write(0x1234, 0x33);

    cpu.set_a(0x50);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x1D); // 0x50 - 0x33 = 0x1D
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_sbc_absolute_x_no_page_cross() {
    let mut cpu = setup_cpu();

    // SBC $1200,X (0xFD 0x00 0x12)
    cpu.memory_mut().write(0x8000, 0xFD);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1205, 0x22); // Value at 0x1200 + 0x05

    cpu.set_a(0x50);
    cpu.set_x(0x05);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x2E);
    assert_eq!(cpu.cycles(), 4); // No page cross, base cycles only
}

#[test]
fn test_sbc_absolute_x_with_page_cross() {
    let mut cpu = setup_cpu();

    // SBC $12FF,X with X=2 crosses page boundary (0x12FF -> 0x1301)
    cpu.memory_mut().write(0x8000, 0xFD);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1301, 0x11); // Value at 0x12FF + 0x02

    cpu.set_a(0x50);
    cpu.set_x(0x02);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x3F);
    assert_eq!(cpu.cycles(), 5); // Page cross adds 1 cycle
}

#[test]
fn test_sbc_absolute_y_no_page_cross() {
    let mut cpu = setup_cpu();

    // SBC $1200,Y (0xF9 0x00 0x12)
    cpu.memory_mut().write(0x8000, 0xF9);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1203, 0x22); // Value at 0x1200 + 0x03

    cpu.set_a(0x50);
    cpu.set_y(0x03);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x2E);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_sbc_absolute_y_with_page_cross() {
    let mut cpu = setup_cpu();

    // SBC $10FE,Y with Y=3 crosses page boundary (0x10FE -> 0x1101)
    cpu.memory_mut().write(0x8000, 0xF9);
    cpu.memory_mut().write(0x8001, 0xFE);
    cpu.memory_mut().write(0x8002, 0x10);
    cpu.memory_mut().write(0x1101, 0x11); // Value at 0x10FE + 0x03

    cpu.set_a(0x50);
    cpu.set_y(0x03);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x3F);
    assert_eq!(cpu.cycles(), 5); // Page cross adds 1 cycle
}

#[test]
fn test_sbc_indirect_x() {
    let mut cpu = setup_cpu();

    // SBC ($40,X) (0xE1 0x40)
    // With X=5, reads address from zero page 0x45/0x46
    cpu.memory_mut().write(0x8000, 0xE1);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0045, 0x00); // Low byte of target address
    cpu.memory_mut().write(0x0046, 0x20); // High byte of target address (0x2000)
    cpu.memory_mut().write(0x2000, 0x11); // Value at target address

    cpu.set_a(0x50);
    cpu.set_x(0x05);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x3F); // 0x50 - 0x11 = 0x3F
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_sbc_indirect_x_zero_page_wrap() {
    let mut cpu = setup_cpu();

    // SBC ($FF,X) with X=1
    // Address pointer wraps: 0xFF + 1 = 0x00 in zero page
    cpu.memory_mut().write(0x8000, 0xE1);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x0000, 0x34); // Low byte at 0x00
    cpu.memory_mut().write(0x0001, 0x12); // High byte at 0x01 (0x1234)
    cpu.memory_mut().write(0x1234, 0x11);

    cpu.set_a(0x50);
    cpu.set_x(0x01);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x3F); // 0x50 - 0x11 = 0x3F
}

#[test]
fn test_sbc_indirect_y_no_page_cross() {
    let mut cpu = setup_cpu();

    // SBC ($40),Y (0xF1 0x40)
    // Reads base address from 0x40/0x41, then adds Y
    cpu.memory_mut().write(0x8000, 0xF1);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0040, 0x00); // Low byte of base address
    cpu.memory_mut().write(0x0041, 0x20); // High byte of base address (0x2000)
    cpu.memory_mut().write(0x2003, 0x11); // Value at 0x2000 + Y(3)

    cpu.set_a(0x50);
    cpu.set_y(0x03);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x3F); // 0x50 - 0x11 = 0x3F
    assert_eq!(cpu.cycles(), 5); // No page cross
}

#[test]
fn test_sbc_indirect_y_with_page_cross() {
    let mut cpu = setup_cpu();

    // SBC ($40),Y with page crossing
    cpu.memory_mut().write(0x8000, 0xF1);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0040, 0xFF); // Low byte of base address
    cpu.memory_mut().write(0x0041, 0x20); // High byte (0x20FF)
    cpu.memory_mut().write(0x2101, 0x11); // Value at 0x20FF + Y(2) = 0x2101

    cpu.set_a(0x50);
    cpu.set_y(0x02);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x3F); // 0x50 - 0x11 = 0x3F
    assert_eq!(cpu.cycles(), 6); // Page cross adds 1 cycle
}

// ========== Edge Case Tests ==========

#[test]
fn test_sbc_zero_from_zero_with_carry() {
    let mut cpu = setup_cpu();

    // SBC #$00 with A=$00 and C=1 (no borrow)
    cpu.memory_mut().write(0x8000, 0xE9);
    cpu.memory_mut().write(0x8001, 0x00);

    cpu.set_a(0x00);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00);
    assert!(cpu.flag_c()); // No borrow
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_sbc_zero_from_zero_without_carry() {
    let mut cpu = setup_cpu();

    // SBC #$00 with A=$00 and C=0 (borrow)
    cpu.memory_mut().write(0x8000, 0xE9);
    cpu.memory_mut().write(0x8001, 0x00);

    cpu.set_a(0x00);
    cpu.set_flag_c(false);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF); // 0 - 0 - 1 = -1 = 0xFF
    assert!(!cpu.flag_c()); // Borrow occurred
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
}

#[test]
fn test_sbc_all_ones() {
    let mut cpu = setup_cpu();

    // SBC #$FF with A=$FF and C=1
    cpu.memory_mut().write(0x8000, 0xE9);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_a(0xFF);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00); // 0xFF - 0xFF = 0x00
    assert!(cpu.flag_c());
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
}

// ========== Multiple Instructions Test ==========

#[test]
fn test_sbc_sequence() {
    let mut cpu = setup_cpu();

    // First SBC: 0x50 - 0x20 = 0x30
    cpu.memory_mut().write(0x8000, 0xE9);
    cpu.memory_mut().write(0x8001, 0x20);

    // Second SBC: 0x30 - 0x10 = 0x20
    cpu.memory_mut().write(0x8002, 0xE9);
    cpu.memory_mut().write(0x8003, 0x10);

    cpu.set_a(0x50);
    cpu.set_flag_c(true);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x30);
    assert_eq!(cpu.pc(), 0x8002);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x20);
    assert_eq!(cpu.pc(), 0x8004);
}

#[test]
fn test_sbc_borrow_chain() {
    let mut cpu = setup_cpu();

    // SBC sequence that borrows through multiple operations
    // 0x05 - 0x10 = 0xF5 (borrow, carry=0)
    cpu.memory_mut().write(0x8000, 0xE9);
    cpu.memory_mut().write(0x8001, 0x10);

    // 0xF5 - 0x00 - 1 (borrow) = 0xF4
    cpu.memory_mut().write(0x8002, 0xE9);
    cpu.memory_mut().write(0x8003, 0x00);

    cpu.set_a(0x05);
    cpu.set_flag_c(true);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0xF5);
    assert!(!cpu.flag_c()); // Borrow occurred
    assert!(cpu.flag_n());

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0xF4);
    assert!(cpu.flag_c()); // No new borrow (0xF5 - 0 - 1 still positive in unsigned)
}

// ========== Decimal Mode (BCD) Tests ==========

#[test]
fn test_sbc_decimal_mode_basic() {
    let mut cpu = setup_cpu();

    // SBC #$25 in decimal mode: 50 - 25 = 25 (BCD)
    cpu.memory_mut().write(0x8000, 0xE9);
    cpu.memory_mut().write(0x8001, 0x25);

    cpu.set_a(0x50);
    cpu.set_flag_c(true); // No borrow
    cpu.set_flag_d(true); // Enable decimal mode

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x25); // BCD result: 50 - 25 = 25
    assert!(cpu.flag_c()); // No borrow
    assert!(!cpu.flag_z());
}

#[test]
fn test_sbc_decimal_mode_with_borrow_in() {
    let mut cpu = setup_cpu();

    // SBC #$25 in decimal mode with borrow: 50 - 25 - 1 = 24 (BCD)
    cpu.memory_mut().write(0x8000, 0xE9);
    cpu.memory_mut().write(0x8001, 0x25);

    cpu.set_a(0x50);
    cpu.set_flag_c(false); // Borrow in
    cpu.set_flag_d(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x24); // BCD result: 50 - 25 - 1 = 24
    assert!(cpu.flag_c()); // No borrow out
}

#[test]
fn test_sbc_decimal_mode_with_borrow_out() {
    let mut cpu = setup_cpu();

    // SBC #$50 in decimal mode: 25 - 50 requires borrow (BCD)
    // Result should wrap in BCD
    cpu.memory_mut().write(0x8000, 0xE9);
    cpu.memory_mut().write(0x8001, 0x50);

    cpu.set_a(0x25);
    cpu.set_flag_c(true); // No borrow in
    cpu.set_flag_d(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x75); // BCD result: 25 - 50 = -25 -> 75 (with borrow)
    assert!(!cpu.flag_c()); // Borrow occurred
}

#[test]
fn test_sbc_decimal_mode_low_nibble_borrow() {
    let mut cpu = setup_cpu();

    // SBC #$08 in decimal mode: 15 - 08 = 07 (BCD)
    // Tests low nibble borrow (5 - 8 requires borrowing from tens)
    cpu.memory_mut().write(0x8000, 0xE9);
    cpu.memory_mut().write(0x8001, 0x08);

    cpu.set_a(0x15);
    cpu.set_flag_c(true);
    cpu.set_flag_d(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x07); // BCD result: 15 - 08 = 07
    assert!(cpu.flag_c()); // No borrow out
}

#[test]
fn test_sbc_decimal_mode_high_nibble_borrow() {
    let mut cpu = setup_cpu();

    // SBC #$35 in decimal mode: 42 - 35 = 07 (BCD)
    cpu.memory_mut().write(0x8000, 0xE9);
    cpu.memory_mut().write(0x8001, 0x35);

    cpu.set_a(0x42);
    cpu.set_flag_c(true);
    cpu.set_flag_d(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x07); // BCD result: 42 - 35 = 07
    assert!(cpu.flag_c()); // No borrow
}

#[test]
fn test_sbc_decimal_mode_zero_result() {
    let mut cpu = setup_cpu();

    // SBC #$50 in decimal mode: 50 - 50 = 00 (BCD)
    cpu.memory_mut().write(0x8000, 0xE9);
    cpu.memory_mut().write(0x8001, 0x50);

    cpu.set_a(0x50);
    cpu.set_flag_c(true);
    cpu.set_flag_d(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00); // BCD result: 50 - 50 = 00
    assert!(cpu.flag_c()); // No borrow
    assert!(cpu.flag_z()); // Zero flag set
}

#[test]
fn test_sbc_decimal_mode_99_minus_1() {
    let mut cpu = setup_cpu();

    // SBC #$01 in decimal mode: 99 - 01 = 98 (BCD)
    cpu.memory_mut().write(0x8000, 0xE9);
    cpu.memory_mut().write(0x8001, 0x01);

    cpu.set_a(0x99);
    cpu.set_flag_c(true);
    cpu.set_flag_d(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x98); // BCD result: 99 - 01 = 98
    assert!(cpu.flag_c()); // No borrow
    assert!(!cpu.flag_z());
}

#[test]
fn test_sbc_decimal_mode_subtract_from_zero() {
    let mut cpu = setup_cpu();

    // SBC #$01 in decimal mode: 00 - 01 requires borrow (BCD)
    cpu.memory_mut().write(0x8000, 0xE9);
    cpu.memory_mut().write(0x8001, 0x01);

    cpu.set_a(0x00);
    cpu.set_flag_c(true);
    cpu.set_flag_d(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x99); // BCD result: 00 - 01 = -01 -> 99 (with borrow)
    assert!(!cpu.flag_c()); // Borrow occurred
}

#[test]
fn test_sbc_decimal_vs_binary_mode() {
    let mut cpu = setup_cpu();

    // Compare binary vs decimal mode for same inputs
    // Binary: 0x50 - 0x25 = 0x2B
    // Decimal: 50 - 25 = 25 (BCD)

    // Test in binary mode first
    cpu.memory_mut().write(0x8000, 0xE9);
    cpu.memory_mut().write(0x8001, 0x25);
    cpu.set_a(0x50);
    cpu.set_flag_c(true);
    cpu.set_flag_d(false); // Binary mode

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x2B); // Binary result

    // Reset and test in decimal mode
    cpu.set_pc(0x8000);
    cpu.set_a(0x50);
    cpu.set_flag_c(true);
    cpu.set_flag_d(true); // Decimal mode

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x25); // BCD result: 50 - 25 = 25
}
