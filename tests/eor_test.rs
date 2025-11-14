//! Comprehensive tests for the EOR (Exclusive OR) instruction.
//!
//! Tests cover:
//! - All 8 addressing modes
//! - Flag updates (Z, N)
//! - Various operand values (0, positive, negative)
//! - Cycle counts including page crossing penalties

use cpu6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic EOR Operation Tests ==========

#[test]
fn test_eor_immediate_basic() {
    let mut cpu = setup_cpu();

    // EOR #$0F (0x49 0x0F)
    cpu.memory_mut().write(0x8000, 0x49);
    cpu.memory_mut().write(0x8001, 0x0F);

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xF0); // 0xFF ^ 0x0F = 0xF0
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is set
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 2);
}

#[test]
fn test_eor_flips_bits() {
    let mut cpu = setup_cpu();

    // EOR #$55 (0b01010101)
    cpu.memory_mut().write(0x8000, 0x49);
    cpu.memory_mut().write(0x8001, 0x55);

    cpu.set_a(0xAA); // 0b10101010

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF); // 0xAA ^ 0x55 = 0xFF (all bits flipped)
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is set
}

#[test]
fn test_eor_same_value_gives_zero() {
    let mut cpu = setup_cpu();

    // EOR #$42
    cpu.memory_mut().write(0x8000, 0x49);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_a(0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00); // 0x42 ^ 0x42 = 0x00 (same values XOR to zero)
    assert!(cpu.flag_z()); // Result is zero
    assert!(!cpu.flag_n());
}

// ========== Flag Tests ==========

#[test]
fn test_eor_zero_flag() {
    let mut cpu = setup_cpu();

    // EOR #$FF with A=$FF should give zero
    cpu.memory_mut().write(0x8000, 0x49);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00); // 0xFF ^ 0xFF = 0x00
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_eor_negative_flag() {
    let mut cpu = setup_cpu();

    // EOR #$7F with A=$FF gives 0x80 (bit 7 set)
    cpu.memory_mut().write(0x8000, 0x49);
    cpu.memory_mut().write(0x8001, 0x7F);

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x80); // 0xFF ^ 0x7F = 0x80
    assert!(cpu.flag_n()); // Bit 7 is set
    assert!(!cpu.flag_z());
}

#[test]
fn test_eor_clears_negative_flag() {
    let mut cpu = setup_cpu();

    // EOR #$80 with A=$80 gives 0x00 (clears negative flag)
    cpu.memory_mut().write(0x8000, 0x49);
    cpu.memory_mut().write(0x8001, 0x80);

    cpu.set_a(0x80);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00); // 0x80 ^ 0x80 = 0x00
    assert!(!cpu.flag_n()); // Bit 7 is cleared
    assert!(cpu.flag_z());
}

#[test]
fn test_eor_preserves_carry_flag() {
    let mut cpu = setup_cpu();

    // EOR #$FF
    cpu.memory_mut().write(0x8000, 0x49);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_a(0x42);
    cpu.set_flag_c(true); // Set carry flag

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xBD); // 0x42 ^ 0xFF = 0xBD
    assert!(cpu.flag_c()); // Carry flag should be unchanged
}

#[test]
fn test_eor_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    // EOR #$FF
    cpu.memory_mut().write(0x8000, 0x49);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_a(0x42);
    cpu.set_flag_v(true); // Set overflow flag

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xBD); // 0x42 ^ 0xFF = 0xBD
    assert!(cpu.flag_v()); // Overflow flag should be unchanged
}

// ========== Addressing Mode Tests ==========

#[test]
fn test_eor_zero_page() {
    let mut cpu = setup_cpu();

    // EOR $42 (0x45 0x42)
    cpu.memory_mut().write(0x8000, 0x45);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x0F); // Value at zero page address

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xF0); // 0xFF ^ 0x0F = 0xF0
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 3);
}

#[test]
fn test_eor_zero_page_x() {
    let mut cpu = setup_cpu();

    // EOR $40,X (0x55 0x40)
    cpu.memory_mut().write(0x8000, 0x55);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0045, 0x33); // Value at 0x40 + 0x05

    cpu.set_a(0xFF);
    cpu.set_x(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xCC); // 0xFF ^ 0x33 = 0xCC
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_eor_zero_page_x_wrap() {
    let mut cpu = setup_cpu();

    // EOR $FF,X with X=2 should wrap to 0x01 within zero page
    cpu.memory_mut().write(0x8000, 0x55);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x0001, 0x55); // Value at (0xFF + 0x02) % 256 = 0x01

    cpu.set_a(0xFF);
    cpu.set_x(0x02);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xAA); // 0xFF ^ 0x55 = 0xAA
}

#[test]
fn test_eor_absolute() {
    let mut cpu = setup_cpu();

    // EOR $1234 (0x4D 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0x4D);
    cpu.memory_mut().write(0x8001, 0x34); // Low byte
    cpu.memory_mut().write(0x8002, 0x12); // High byte
    cpu.memory_mut().write(0x1234, 0xF0);

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x0F); // 0xFF ^ 0xF0 = 0x0F
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_eor_absolute_x_no_page_cross() {
    let mut cpu = setup_cpu();

    // EOR $1200,X (0x5D 0x00 0x12)
    cpu.memory_mut().write(0x8000, 0x5D);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1205, 0x3C); // Value at 0x1200 + 0x05

    cpu.set_a(0xFF);
    cpu.set_x(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xC3); // 0xFF ^ 0x3C = 0xC3
    assert_eq!(cpu.cycles(), 4); // No page cross, base cycles only
}

#[test]
fn test_eor_absolute_x_with_page_cross() {
    let mut cpu = setup_cpu();

    // EOR $12FF,X with X=2 crosses page boundary (0x12FF -> 0x1301)
    cpu.memory_mut().write(0x8000, 0x5D);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1301, 0xAA); // Value at 0x12FF + 0x02

    cpu.set_a(0xFF);
    cpu.set_x(0x02);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x55); // 0xFF ^ 0xAA = 0x55
    assert_eq!(cpu.cycles(), 5); // Page cross adds 1 cycle
}

#[test]
fn test_eor_absolute_y_no_page_cross() {
    let mut cpu = setup_cpu();

    // EOR $1200,Y (0x59 0x00 0x12)
    cpu.memory_mut().write(0x8000, 0x59);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1203, 0x66); // Value at 0x1200 + 0x03

    cpu.set_a(0xFF);
    cpu.set_y(0x03);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x99); // 0xFF ^ 0x66 = 0x99
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_eor_absolute_y_with_page_cross() {
    let mut cpu = setup_cpu();

    // EOR $10FE,Y with Y=3 crosses page boundary (0x10FE -> 0x1101)
    cpu.memory_mut().write(0x8000, 0x59);
    cpu.memory_mut().write(0x8001, 0xFE);
    cpu.memory_mut().write(0x8002, 0x10);
    cpu.memory_mut().write(0x1101, 0x99); // Value at 0x10FE + 0x03

    cpu.set_a(0xFF);
    cpu.set_y(0x03);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x66); // 0xFF ^ 0x99 = 0x66
    assert_eq!(cpu.cycles(), 5); // Page cross adds 1 cycle
}

#[test]
fn test_eor_indirect_x() {
    let mut cpu = setup_cpu();

    // EOR ($40,X) (0x41 0x40)
    // With X=5, reads address from zero page 0x45/0x46
    cpu.memory_mut().write(0x8000, 0x41);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0045, 0x00); // Low byte of target address
    cpu.memory_mut().write(0x0046, 0x20); // High byte of target address (0x2000)
    cpu.memory_mut().write(0x2000, 0xCC); // Value at target address

    cpu.set_a(0xFF);
    cpu.set_x(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x33); // 0xFF ^ 0xCC = 0x33
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_eor_indirect_x_zero_page_wrap() {
    let mut cpu = setup_cpu();

    // EOR ($FF,X) with X=1
    // Address pointer wraps: 0xFF + 1 = 0x00 in zero page
    cpu.memory_mut().write(0x8000, 0x41);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x0000, 0x34); // Low byte at 0x00
    cpu.memory_mut().write(0x0001, 0x12); // High byte at 0x01 (0x1234)
    cpu.memory_mut().write(0x1234, 0x77);

    cpu.set_a(0xFF);
    cpu.set_x(0x01);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x88); // 0xFF ^ 0x77 = 0x88
}

#[test]
fn test_eor_indirect_y_no_page_cross() {
    let mut cpu = setup_cpu();

    // EOR ($40),Y (0x51 0x40)
    // Reads base address from 0x40/0x41, then adds Y
    cpu.memory_mut().write(0x8000, 0x51);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0040, 0x00); // Low byte of base address
    cpu.memory_mut().write(0x0041, 0x20); // High byte of base address (0x2000)
    cpu.memory_mut().write(0x2003, 0x88); // Value at 0x2000 + Y(3)

    cpu.set_a(0xFF);
    cpu.set_y(0x03);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x77); // 0xFF ^ 0x88 = 0x77
    assert_eq!(cpu.cycles(), 5); // No page cross
}

#[test]
fn test_eor_indirect_y_with_page_cross() {
    let mut cpu = setup_cpu();

    // EOR ($40),Y with page crossing
    cpu.memory_mut().write(0x8000, 0x51);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0040, 0xFF); // Low byte of base address
    cpu.memory_mut().write(0x0041, 0x20); // High byte (0x20FF)
    cpu.memory_mut().write(0x2101, 0x55); // Value at 0x20FF + Y(2) = 0x2101

    cpu.set_a(0xFF);
    cpu.set_y(0x02);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xAA); // 0xFF ^ 0x55 = 0xAA
    assert_eq!(cpu.cycles(), 6); // Page cross adds 1 cycle
}

// ========== Edge Case Tests ==========

#[test]
fn test_eor_with_zero() {
    let mut cpu = setup_cpu();

    // EOR #$00 with A=$42 should leave A unchanged
    cpu.memory_mut().write(0x8000, 0x49);
    cpu.memory_mut().write(0x8001, 0x00);

    cpu.set_a(0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x42); // 0x42 ^ 0x00 = 0x42
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_eor_with_ff_inverts() {
    let mut cpu = setup_cpu();

    // EOR #$FF inverts all bits
    cpu.memory_mut().write(0x8000, 0x49);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_a(0xAA); // 0b10101010

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x55); // 0xAA ^ 0xFF = 0x55 (all bits inverted)
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_eor_all_ones() {
    let mut cpu = setup_cpu();

    // EOR #$FF with A=$FF
    cpu.memory_mut().write(0x8000, 0x49);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00); // 0xFF ^ 0xFF = 0x00
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_eor_all_zeros() {
    let mut cpu = setup_cpu();

    // EOR #$00 with A=$00
    cpu.memory_mut().write(0x8000, 0x49);
    cpu.memory_mut().write(0x8001, 0x00);

    cpu.set_a(0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00);
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_eor_toggle_bit() {
    let mut cpu = setup_cpu();

    // EOR #$01 - toggle bit 0
    cpu.memory_mut().write(0x8000, 0x49);
    cpu.memory_mut().write(0x8001, 0x01);

    cpu.set_a(0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x01); // 0x00 ^ 0x01 = 0x01 (bit 0 toggled on)
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_eor_toggle_bit_twice() {
    let mut cpu = setup_cpu();

    // EOR #$01 twice should restore original value
    cpu.memory_mut().write(0x8000, 0x49);
    cpu.memory_mut().write(0x8001, 0x01);
    cpu.memory_mut().write(0x8002, 0x49);
    cpu.memory_mut().write(0x8003, 0x01);

    cpu.set_a(0x42);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x43); // 0x42 ^ 0x01 = 0x43

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x42); // 0x43 ^ 0x01 = 0x42 (back to original)
}

#[test]
fn test_eor_alternating_bits() {
    let mut cpu = setup_cpu();

    // Test with alternating bit patterns
    // 0b10101010 ^ 0b01010101 = 0b11111111
    cpu.memory_mut().write(0x8000, 0x49);
    cpu.memory_mut().write(0x8001, 0x55); // 0b01010101

    cpu.set_a(0xAA); // 0b10101010

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
}

// ========== Multiple Instructions Test ==========

#[test]
fn test_eor_sequence() {
    let mut cpu = setup_cpu();

    // First EOR: 0xFF ^ 0xF0 = 0x0F
    cpu.memory_mut().write(0x8000, 0x49);
    cpu.memory_mut().write(0x8001, 0xF0);

    // Second EOR: 0x0F ^ 0x0F = 0x00
    cpu.memory_mut().write(0x8002, 0x49);
    cpu.memory_mut().write(0x8003, 0x0F);

    cpu.set_a(0xFF);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x0F);
    assert_eq!(cpu.pc(), 0x8002);
    assert!(!cpu.flag_n());
    assert!(!cpu.flag_z());

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x00);
    assert_eq!(cpu.pc(), 0x8004);
    assert!(!cpu.flag_n());
    assert!(cpu.flag_z());
}

#[test]
fn test_eor_mask_manipulation() {
    let mut cpu = setup_cpu();

    // Start with 0b00000000
    // EOR #$F0: 0b00000000 ^ 0b11110000 = 0b11110000
    cpu.memory_mut().write(0x8000, 0x49);
    cpu.memory_mut().write(0x8001, 0xF0);

    // EOR #$0F: 0b11110000 ^ 0b00001111 = 0b11111111
    cpu.memory_mut().write(0x8002, 0x49);
    cpu.memory_mut().write(0x8003, 0x0F);

    // EOR #$FF: 0b11111111 ^ 0b11111111 = 0b00000000
    cpu.memory_mut().write(0x8004, 0x49);
    cpu.memory_mut().write(0x8005, 0xFF);

    cpu.set_a(0x00);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0xF0);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0xFF);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x00);
}
