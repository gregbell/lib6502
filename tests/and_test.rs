//! Comprehensive tests for the AND (Logical AND) instruction.
//!
//! Tests cover:
//! - All 8 addressing modes
//! - Flag updates (Z, N)
//! - Various operand values (0, positive, negative)
//! - Cycle counts including page crossing penalties

use lib6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic AND Operation Tests ==========

#[test]
fn test_and_immediate_basic() {
    let mut cpu = setup_cpu();

    // AND #$0F (0x29 0x0F)
    cpu.memory_mut().write(0x8000, 0x29);
    cpu.memory_mut().write(0x8001, 0x0F);

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x0F); // 0xFF & 0x0F = 0x0F
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 2);
}

#[test]
fn test_and_clears_bits() {
    let mut cpu = setup_cpu();

    // AND #$55 (0b01010101)
    cpu.memory_mut().write(0x8000, 0x29);
    cpu.memory_mut().write(0x8001, 0x55);

    cpu.set_a(0xAA); // 0b10101010

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00); // 0xAA & 0x55 = 0x00 (complementary bits)
    assert!(cpu.flag_z()); // Result is zero
    assert!(!cpu.flag_n());
}

// ========== Flag Tests ==========

#[test]
fn test_and_zero_flag() {
    let mut cpu = setup_cpu();

    // AND #$00
    cpu.memory_mut().write(0x8000, 0x29);
    cpu.memory_mut().write(0x8001, 0x00);

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00); // 0xFF & 0x00 = 0x00
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_and_negative_flag() {
    let mut cpu = setup_cpu();

    // AND #$80 (0b10000000)
    cpu.memory_mut().write(0x8000, 0x29);
    cpu.memory_mut().write(0x8001, 0x80);

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x80); // 0xFF & 0x80 = 0x80
    assert!(cpu.flag_n()); // Bit 7 is set
    assert!(!cpu.flag_z());
}

#[test]
fn test_and_clears_negative_flag() {
    let mut cpu = setup_cpu();

    // AND #$7F (0b01111111) - clears bit 7
    cpu.memory_mut().write(0x8000, 0x29);
    cpu.memory_mut().write(0x8001, 0x7F);

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x7F); // 0xFF & 0x7F = 0x7F
    assert!(!cpu.flag_n()); // Bit 7 is cleared
    assert!(!cpu.flag_z());
}

#[test]
fn test_and_preserves_carry_flag() {
    let mut cpu = setup_cpu();

    // AND #$FF
    cpu.memory_mut().write(0x8000, 0x29);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_a(0x42);
    cpu.set_flag_c(true); // Set carry flag

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x42); // 0x42 & 0xFF = 0x42
    assert!(cpu.flag_c()); // Carry flag should be unchanged
}

#[test]
fn test_and_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    // AND #$FF
    cpu.memory_mut().write(0x8000, 0x29);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_a(0x42);
    cpu.set_flag_v(true); // Set overflow flag

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x42); // 0x42 & 0xFF = 0x42
    assert!(cpu.flag_v()); // Overflow flag should be unchanged
}

// ========== Addressing Mode Tests ==========

#[test]
fn test_and_zero_page() {
    let mut cpu = setup_cpu();

    // AND $42 (0x25 0x42)
    cpu.memory_mut().write(0x8000, 0x25);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x0F); // Value at zero page address

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x0F); // 0xFF & 0x0F = 0x0F
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 3);
}

#[test]
fn test_and_zero_page_x() {
    let mut cpu = setup_cpu();

    // AND $40,X (0x35 0x40)
    cpu.memory_mut().write(0x8000, 0x35);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0045, 0x33); // Value at 0x40 + 0x05

    cpu.set_a(0xFF);
    cpu.set_x(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x33); // 0xFF & 0x33 = 0x33
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_and_zero_page_x_wrap() {
    let mut cpu = setup_cpu();

    // AND $FF,X with X=2 should wrap to 0x01 within zero page
    cpu.memory_mut().write(0x8000, 0x35);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x0001, 0x55); // Value at (0xFF + 0x02) % 256 = 0x01

    cpu.set_a(0xFF);
    cpu.set_x(0x02);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x55); // 0xFF & 0x55 = 0x55
}

#[test]
fn test_and_absolute() {
    let mut cpu = setup_cpu();

    // AND $1234 (0x2D 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0x2D);
    cpu.memory_mut().write(0x8001, 0x34); // Low byte
    cpu.memory_mut().write(0x8002, 0x12); // High byte
    cpu.memory_mut().write(0x1234, 0xF0);

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xF0); // 0xFF & 0xF0 = 0xF0
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_and_absolute_x_no_page_cross() {
    let mut cpu = setup_cpu();

    // AND $1200,X (0x3D 0x00 0x12)
    cpu.memory_mut().write(0x8000, 0x3D);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1205, 0x3C); // Value at 0x1200 + 0x05

    cpu.set_a(0xFF);
    cpu.set_x(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x3C); // 0xFF & 0x3C = 0x3C
    assert_eq!(cpu.cycles(), 4); // No page cross, base cycles only
}

#[test]
fn test_and_absolute_x_with_page_cross() {
    let mut cpu = setup_cpu();

    // AND $12FF,X with X=2 crosses page boundary (0x12FF -> 0x1301)
    cpu.memory_mut().write(0x8000, 0x3D);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1301, 0xAA); // Value at 0x12FF + 0x02

    cpu.set_a(0xFF);
    cpu.set_x(0x02);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xAA); // 0xFF & 0xAA = 0xAA
    assert_eq!(cpu.cycles(), 5); // Page cross adds 1 cycle
}

#[test]
fn test_and_absolute_y_no_page_cross() {
    let mut cpu = setup_cpu();

    // AND $1200,Y (0x39 0x00 0x12)
    cpu.memory_mut().write(0x8000, 0x39);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1203, 0x66); // Value at 0x1200 + 0x03

    cpu.set_a(0xFF);
    cpu.set_y(0x03);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x66); // 0xFF & 0x66 = 0x66
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_and_absolute_y_with_page_cross() {
    let mut cpu = setup_cpu();

    // AND $10FE,Y with Y=3 crosses page boundary (0x10FE -> 0x1101)
    cpu.memory_mut().write(0x8000, 0x39);
    cpu.memory_mut().write(0x8001, 0xFE);
    cpu.memory_mut().write(0x8002, 0x10);
    cpu.memory_mut().write(0x1101, 0x99); // Value at 0x10FE + 0x03

    cpu.set_a(0xFF);
    cpu.set_y(0x03);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x99); // 0xFF & 0x99 = 0x99
    assert_eq!(cpu.cycles(), 5); // Page cross adds 1 cycle
}

#[test]
fn test_and_indirect_x() {
    let mut cpu = setup_cpu();

    // AND ($40,X) (0x21 0x40)
    // With X=5, reads address from zero page 0x45/0x46
    cpu.memory_mut().write(0x8000, 0x21);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0045, 0x00); // Low byte of target address
    cpu.memory_mut().write(0x0046, 0x20); // High byte of target address (0x2000)
    cpu.memory_mut().write(0x2000, 0xCC); // Value at target address

    cpu.set_a(0xFF);
    cpu.set_x(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xCC); // 0xFF & 0xCC = 0xCC
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_and_indirect_x_zero_page_wrap() {
    let mut cpu = setup_cpu();

    // AND ($FF,X) with X=1
    // Address pointer wraps: 0xFF + 1 = 0x00 in zero page
    cpu.memory_mut().write(0x8000, 0x21);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x0000, 0x34); // Low byte at 0x00
    cpu.memory_mut().write(0x0001, 0x12); // High byte at 0x01 (0x1234)
    cpu.memory_mut().write(0x1234, 0x77);

    cpu.set_a(0xFF);
    cpu.set_x(0x01);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x77); // 0xFF & 0x77 = 0x77
}

#[test]
fn test_and_indirect_y_no_page_cross() {
    let mut cpu = setup_cpu();

    // AND ($40),Y (0x31 0x40)
    // Reads base address from 0x40/0x41, then adds Y
    cpu.memory_mut().write(0x8000, 0x31);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0040, 0x00); // Low byte of base address
    cpu.memory_mut().write(0x0041, 0x20); // High byte of base address (0x2000)
    cpu.memory_mut().write(0x2003, 0x88); // Value at 0x2000 + Y(3)

    cpu.set_a(0xFF);
    cpu.set_y(0x03);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x88); // 0xFF & 0x88 = 0x88
    assert_eq!(cpu.cycles(), 5); // No page cross
}

#[test]
fn test_and_indirect_y_with_page_cross() {
    let mut cpu = setup_cpu();

    // AND ($40),Y with page crossing
    cpu.memory_mut().write(0x8000, 0x31);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0040, 0xFF); // Low byte of base address
    cpu.memory_mut().write(0x0041, 0x20); // High byte (0x20FF)
    cpu.memory_mut().write(0x2101, 0x55); // Value at 0x20FF + Y(2) = 0x2101

    cpu.set_a(0xFF);
    cpu.set_y(0x02);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x55); // 0xFF & 0x55 = 0x55
    assert_eq!(cpu.cycles(), 6); // Page cross adds 1 cycle
}

// ========== Edge Case Tests ==========

#[test]
fn test_and_all_ones() {
    let mut cpu = setup_cpu();

    // AND #$FF with A=$FF
    cpu.memory_mut().write(0x8000, 0x29);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF); // 0xFF & 0xFF = 0xFF
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is set
}

#[test]
fn test_and_all_zeros() {
    let mut cpu = setup_cpu();

    // AND #$00 with A=$00
    cpu.memory_mut().write(0x8000, 0x29);
    cpu.memory_mut().write(0x8001, 0x00);

    cpu.set_a(0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00);
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_and_mask_low_nibble() {
    let mut cpu = setup_cpu();

    // AND #$0F - mask to keep only low 4 bits
    cpu.memory_mut().write(0x8000, 0x29);
    cpu.memory_mut().write(0x8001, 0x0F);

    cpu.set_a(0xAB); // 0b10101011

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x0B); // 0xAB & 0x0F = 0x0B (keeps low nibble)
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_and_mask_high_nibble() {
    let mut cpu = setup_cpu();

    // AND #$F0 - mask to keep only high 4 bits
    cpu.memory_mut().write(0x8000, 0x29);
    cpu.memory_mut().write(0x8001, 0xF0);

    cpu.set_a(0xAB); // 0b10101011

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xA0); // 0xAB & 0xF0 = 0xA0 (keeps high nibble)
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is set
}

#[test]
fn test_and_alternating_bits() {
    let mut cpu = setup_cpu();

    // Test with alternating bit patterns
    // 0b10101010 & 0b01010101 = 0b00000000
    cpu.memory_mut().write(0x8000, 0x29);
    cpu.memory_mut().write(0x8001, 0x55); // 0b01010101

    cpu.set_a(0xAA); // 0b10101010

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00);
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
}

// ========== Multiple Instructions Test ==========

#[test]
fn test_and_sequence() {
    let mut cpu = setup_cpu();

    // First AND: 0xFF & 0xF0 = 0xF0
    cpu.memory_mut().write(0x8000, 0x29);
    cpu.memory_mut().write(0x8001, 0xF0);

    // Second AND: 0xF0 & 0x0F = 0x00
    cpu.memory_mut().write(0x8002, 0x29);
    cpu.memory_mut().write(0x8003, 0x0F);

    cpu.set_a(0xFF);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0xF0);
    assert_eq!(cpu.pc(), 0x8002);
    assert!(cpu.flag_n());
    assert!(!cpu.flag_z());

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x00);
    assert_eq!(cpu.pc(), 0x8004);
    assert!(!cpu.flag_n());
    assert!(cpu.flag_z());
}

#[test]
fn test_and_progressive_masking() {
    let mut cpu = setup_cpu();

    // Start with 0b11111111, progressively mask bits
    // AND #$FE: 0b11111111 & 0b11111110 = 0b11111110
    cpu.memory_mut().write(0x8000, 0x29);
    cpu.memory_mut().write(0x8001, 0xFE);

    // AND #$FC: 0b11111110 & 0b11111100 = 0b11111100
    cpu.memory_mut().write(0x8002, 0x29);
    cpu.memory_mut().write(0x8003, 0xFC);

    // AND #$F0: 0b11111100 & 0b11110000 = 0b11110000
    cpu.memory_mut().write(0x8004, 0x29);
    cpu.memory_mut().write(0x8005, 0xF0);

    cpu.set_a(0xFF);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0xFE);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0xFC);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0xF0);
}
