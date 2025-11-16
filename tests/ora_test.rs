//! Comprehensive tests for the ORA (Logical Inclusive OR) instruction.
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

// ========== Basic ORA Operation Tests ==========

#[test]
fn test_ora_immediate_basic() {
    let mut cpu = setup_cpu();

    // ORA #$0F (0x09 0x0F)
    cpu.memory_mut().write(0x8000, 0x09);
    cpu.memory_mut().write(0x8001, 0x0F);

    cpu.set_a(0xF0);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF); // 0xF0 | 0x0F = 0xFF
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is set
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 2);
}

#[test]
fn test_ora_combines_bits() {
    let mut cpu = setup_cpu();

    // ORA #$55 (0b01010101)
    cpu.memory_mut().write(0x8000, 0x09);
    cpu.memory_mut().write(0x8001, 0x55);

    cpu.set_a(0xAA); // 0b10101010

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF); // 0xAA | 0x55 = 0xFF (all bits set)
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n()); // Bit 7 is set
}

#[test]
fn test_ora_same_value_preserves() {
    let mut cpu = setup_cpu();

    // ORA #$42
    cpu.memory_mut().write(0x8000, 0x09);
    cpu.memory_mut().write(0x8001, 0x42);

    cpu.set_a(0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x42); // 0x42 | 0x42 = 0x42 (same value preserved)
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

// ========== Flag Tests ==========

#[test]
fn test_ora_zero_flag() {
    let mut cpu = setup_cpu();

    // ORA #$00 with A=$00 should give zero
    cpu.memory_mut().write(0x8000, 0x09);
    cpu.memory_mut().write(0x8001, 0x00);

    cpu.set_a(0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00); // 0x00 | 0x00 = 0x00
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_ora_negative_flag() {
    let mut cpu = setup_cpu();

    // ORA #$80 sets bit 7
    cpu.memory_mut().write(0x8000, 0x09);
    cpu.memory_mut().write(0x8001, 0x80);

    cpu.set_a(0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x80); // 0x00 | 0x80 = 0x80
    assert!(cpu.flag_n()); // Bit 7 is set
    assert!(!cpu.flag_z());
}

#[test]
fn test_ora_clears_zero_flag() {
    let mut cpu = setup_cpu();

    // ORA #$01 with A=$00 clears zero flag
    cpu.memory_mut().write(0x8000, 0x09);
    cpu.memory_mut().write(0x8001, 0x01);

    cpu.set_a(0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x01); // 0x00 | 0x01 = 0x01
    assert!(!cpu.flag_n());
    assert!(!cpu.flag_z());
}

#[test]
fn test_ora_preserves_carry_flag() {
    let mut cpu = setup_cpu();

    // ORA #$FF
    cpu.memory_mut().write(0x8000, 0x09);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_a(0x42);
    cpu.set_flag_c(true); // Set carry flag

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF); // 0x42 | 0xFF = 0xFF
    assert!(cpu.flag_c()); // Carry flag should be unchanged
}

#[test]
fn test_ora_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    // ORA #$FF
    cpu.memory_mut().write(0x8000, 0x09);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_a(0x42);
    cpu.set_flag_v(true); // Set overflow flag

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF); // 0x42 | 0xFF = 0xFF
    assert!(cpu.flag_v()); // Overflow flag should be unchanged
}

// ========== Addressing Mode Tests ==========

#[test]
fn test_ora_zero_page() {
    let mut cpu = setup_cpu();

    // ORA $42 (0x05 0x42)
    cpu.memory_mut().write(0x8000, 0x05);
    cpu.memory_mut().write(0x8001, 0x42);
    cpu.memory_mut().write(0x0042, 0x0F); // Value at zero page address

    cpu.set_a(0xF0);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF); // 0xF0 | 0x0F = 0xFF
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 3);
}

#[test]
fn test_ora_zero_page_x() {
    let mut cpu = setup_cpu();

    // ORA $40,X (0x15 0x40)
    cpu.memory_mut().write(0x8000, 0x15);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0045, 0x33); // Value at 0x40 + 0x05

    cpu.set_a(0xCC);
    cpu.set_x(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF); // 0xCC | 0x33 = 0xFF
    assert_eq!(cpu.pc(), 0x8002);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_ora_zero_page_x_wrap() {
    let mut cpu = setup_cpu();

    // ORA $FF,X with X=2 should wrap to 0x01 within zero page
    cpu.memory_mut().write(0x8000, 0x15);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x0001, 0x55); // Value at (0xFF + 0x02) % 256 = 0x01

    cpu.set_a(0xAA);
    cpu.set_x(0x02);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF); // 0xAA | 0x55 = 0xFF
}

#[test]
fn test_ora_absolute() {
    let mut cpu = setup_cpu();

    // ORA $1234 (0x0D 0x34 0x12)
    cpu.memory_mut().write(0x8000, 0x0D);
    cpu.memory_mut().write(0x8001, 0x34); // Low byte
    cpu.memory_mut().write(0x8002, 0x12); // High byte
    cpu.memory_mut().write(0x1234, 0x0F);

    cpu.set_a(0xF0);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF); // 0xF0 | 0x0F = 0xFF
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_ora_absolute_x_no_page_cross() {
    let mut cpu = setup_cpu();

    // ORA $1200,X (0x1D 0x00 0x12)
    cpu.memory_mut().write(0x8000, 0x1D);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1205, 0x3C); // Value at 0x1200 + 0x05

    cpu.set_a(0xC0);
    cpu.set_x(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFC); // 0xC0 | 0x3C = 0xFC
    assert_eq!(cpu.cycles(), 4); // No page cross, base cycles only
}

#[test]
fn test_ora_absolute_x_with_page_cross() {
    let mut cpu = setup_cpu();

    // ORA $12FF,X with X=2 crosses page boundary (0x12FF -> 0x1301)
    cpu.memory_mut().write(0x8000, 0x1D);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1301, 0x0F); // Value at 0x12FF + 0x02

    cpu.set_a(0xF0);
    cpu.set_x(0x02);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF); // 0xF0 | 0x0F = 0xFF
    assert_eq!(cpu.cycles(), 5); // Page cross adds 1 cycle
}

#[test]
fn test_ora_absolute_y_no_page_cross() {
    let mut cpu = setup_cpu();

    // ORA $1200,Y (0x19 0x00 0x12)
    cpu.memory_mut().write(0x8000, 0x19);
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x12);
    cpu.memory_mut().write(0x1203, 0x66); // Value at 0x1200 + 0x03

    cpu.set_a(0x99);
    cpu.set_y(0x03);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF); // 0x99 | 0x66 = 0xFF
    assert_eq!(cpu.cycles(), 4);
}

#[test]
fn test_ora_absolute_y_with_page_cross() {
    let mut cpu = setup_cpu();

    // ORA $10FE,Y with Y=3 crosses page boundary (0x10FE -> 0x1101)
    cpu.memory_mut().write(0x8000, 0x19);
    cpu.memory_mut().write(0x8001, 0xFE);
    cpu.memory_mut().write(0x8002, 0x10);
    cpu.memory_mut().write(0x1101, 0x0F); // Value at 0x10FE + 0x03

    cpu.set_a(0xF0);
    cpu.set_y(0x03);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF); // 0xF0 | 0x0F = 0xFF
    assert_eq!(cpu.cycles(), 5); // Page cross adds 1 cycle
}

#[test]
fn test_ora_indirect_x() {
    let mut cpu = setup_cpu();

    // ORA ($40,X) (0x01 0x40)
    // With X=5, reads address from zero page 0x45/0x46
    cpu.memory_mut().write(0x8000, 0x01);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0045, 0x00); // Low byte of target address
    cpu.memory_mut().write(0x0046, 0x20); // High byte of target address (0x2000)
    cpu.memory_mut().write(0x2000, 0x33); // Value at target address

    cpu.set_a(0xCC);
    cpu.set_x(0x05);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF); // 0xCC | 0x33 = 0xFF
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_ora_indirect_x_zero_page_wrap() {
    let mut cpu = setup_cpu();

    // ORA ($FF,X) with X=1
    // Address pointer wraps: 0xFF + 1 = 0x00 in zero page
    cpu.memory_mut().write(0x8000, 0x01);
    cpu.memory_mut().write(0x8001, 0xFF);
    cpu.memory_mut().write(0x0000, 0x34); // Low byte at 0x00
    cpu.memory_mut().write(0x0001, 0x12); // High byte at 0x01 (0x1234)
    cpu.memory_mut().write(0x1234, 0x0F);

    cpu.set_a(0xF0);
    cpu.set_x(0x01);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF); // 0xF0 | 0x0F = 0xFF
}

#[test]
fn test_ora_indirect_y_no_page_cross() {
    let mut cpu = setup_cpu();

    // ORA ($40),Y (0x11 0x40)
    // Reads base address from 0x40/0x41, then adds Y
    cpu.memory_mut().write(0x8000, 0x11);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0040, 0x00); // Low byte of base address
    cpu.memory_mut().write(0x0041, 0x20); // High byte of base address (0x2000)
    cpu.memory_mut().write(0x2003, 0x0F); // Value at 0x2000 + Y(3)

    cpu.set_a(0xF0);
    cpu.set_y(0x03);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF); // 0xF0 | 0x0F = 0xFF
    assert_eq!(cpu.cycles(), 5); // No page cross
}

#[test]
fn test_ora_indirect_y_with_page_cross() {
    let mut cpu = setup_cpu();

    // ORA ($40),Y with page crossing
    cpu.memory_mut().write(0x8000, 0x11);
    cpu.memory_mut().write(0x8001, 0x40);
    cpu.memory_mut().write(0x0040, 0xFF); // Low byte of base address
    cpu.memory_mut().write(0x0041, 0x20); // High byte (0x20FF)
    cpu.memory_mut().write(0x2101, 0x0F); // Value at 0x20FF + Y(2) = 0x2101

    cpu.set_a(0xF0);
    cpu.set_y(0x02);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF); // 0xF0 | 0x0F = 0xFF
    assert_eq!(cpu.cycles(), 6); // Page cross adds 1 cycle
}

// ========== Edge Case Tests ==========

#[test]
fn test_ora_with_zero() {
    let mut cpu = setup_cpu();

    // ORA #$00 with A=$42 should leave A unchanged
    cpu.memory_mut().write(0x8000, 0x09);
    cpu.memory_mut().write(0x8001, 0x00);

    cpu.set_a(0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x42); // 0x42 | 0x00 = 0x42
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_ora_with_ff_gives_ff() {
    let mut cpu = setup_cpu();

    // ORA #$FF always gives 0xFF
    cpu.memory_mut().write(0x8000, 0x09);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_a(0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF); // 0x42 | 0xFF = 0xFF
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
}

#[test]
fn test_ora_all_ones() {
    let mut cpu = setup_cpu();

    // ORA #$FF with A=$FF
    cpu.memory_mut().write(0x8000, 0x09);
    cpu.memory_mut().write(0x8001, 0xFF);

    cpu.set_a(0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF); // 0xFF | 0xFF = 0xFF
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
}

#[test]
fn test_ora_all_zeros() {
    let mut cpu = setup_cpu();

    // ORA #$00 with A=$00
    cpu.memory_mut().write(0x8000, 0x09);
    cpu.memory_mut().write(0x8001, 0x00);

    cpu.set_a(0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00);
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_ora_set_bit() {
    let mut cpu = setup_cpu();

    // ORA #$01 - set bit 0
    cpu.memory_mut().write(0x8000, 0x09);
    cpu.memory_mut().write(0x8001, 0x01);

    cpu.set_a(0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x01); // 0x00 | 0x01 = 0x01 (bit 0 set)
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_ora_set_bit_idempotent() {
    let mut cpu = setup_cpu();

    // ORA #$01 twice should still give same result
    cpu.memory_mut().write(0x8000, 0x09);
    cpu.memory_mut().write(0x8001, 0x01);
    cpu.memory_mut().write(0x8002, 0x09);
    cpu.memory_mut().write(0x8003, 0x01);

    cpu.set_a(0x42);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x43); // 0x42 | 0x01 = 0x43

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x43); // 0x43 | 0x01 = 0x43 (idempotent)
}

#[test]
fn test_ora_alternating_bits() {
    let mut cpu = setup_cpu();

    // Test with alternating bit patterns
    // 0b10101010 | 0b01010101 = 0b11111111
    cpu.memory_mut().write(0x8000, 0x09);
    cpu.memory_mut().write(0x8001, 0x55); // 0b01010101

    cpu.set_a(0xAA); // 0b10101010

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
}

// ========== Multiple Instructions Test ==========

#[test]
fn test_ora_sequence() {
    let mut cpu = setup_cpu();

    // First ORA: 0x00 | 0xF0 = 0xF0
    cpu.memory_mut().write(0x8000, 0x09);
    cpu.memory_mut().write(0x8001, 0xF0);

    // Second ORA: 0xF0 | 0x0F = 0xFF
    cpu.memory_mut().write(0x8002, 0x09);
    cpu.memory_mut().write(0x8003, 0x0F);

    cpu.set_a(0x00);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0xF0);
    assert_eq!(cpu.pc(), 0x8002);
    assert!(cpu.flag_n());
    assert!(!cpu.flag_z());

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0xFF);
    assert_eq!(cpu.pc(), 0x8004);
    assert!(cpu.flag_n());
    assert!(!cpu.flag_z());
}

#[test]
fn test_ora_mask_building() {
    let mut cpu = setup_cpu();

    // Start with 0b00000000
    // ORA #$F0: 0b00000000 | 0b11110000 = 0b11110000
    cpu.memory_mut().write(0x8000, 0x09);
    cpu.memory_mut().write(0x8001, 0xF0);

    // ORA #$0F: 0b11110000 | 0b00001111 = 0b11111111
    cpu.memory_mut().write(0x8002, 0x09);
    cpu.memory_mut().write(0x8003, 0x0F);

    // ORA #$00: 0b11111111 | 0b00000000 = 0b11111111
    cpu.memory_mut().write(0x8004, 0x09);
    cpu.memory_mut().write(0x8005, 0x00);

    cpu.set_a(0x00);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0xF0);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0xFF);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0xFF);
}

#[test]
fn test_ora_bit_masking() {
    let mut cpu = setup_cpu();

    // Set specific bits using ORA
    // Start with 0b00000000
    // Set bit 0: 0b00000000 | 0b00000001 = 0b00000001
    cpu.memory_mut().write(0x8000, 0x09);
    cpu.memory_mut().write(0x8001, 0x01);

    // Set bit 4: 0b00000001 | 0b00010000 = 0b00010001
    cpu.memory_mut().write(0x8002, 0x09);
    cpu.memory_mut().write(0x8003, 0x10);

    // Set bit 7: 0b00010001 | 0b10000000 = 0b10010001
    cpu.memory_mut().write(0x8004, 0x09);
    cpu.memory_mut().write(0x8005, 0x80);

    cpu.set_a(0x00);

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x01);
    assert!(!cpu.flag_n());
    assert!(!cpu.flag_z());

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x11);
    assert!(!cpu.flag_n());
    assert!(!cpu.flag_z());

    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x91);
    assert!(cpu.flag_n());
    assert!(!cpu.flag_z());
}
