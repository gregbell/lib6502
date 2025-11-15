//! Comprehensive tests for the PHP (Push Processor Status) instruction.
//!
//! Tests cover:
//! - Basic PHP operation works correctly
//! - All addressing modes (Implicit only for PHP)
//! - Correct cycle counts
//! - Stack pointer decrementation
//! - Processor status correctly pushed to stack
//! - No processor flags are affected after PHP
//! - Bits 4 and 5 are always set in the pushed value (hardware quirk)
//! - Edge cases (stack wraparound, various flag combinations)

use cpu6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic PHP Operation Tests ==========

#[test]
fn test_php_basic_operation() {
    let mut cpu = setup_cpu();

    // PHP (0x08)
    cpu.memory_mut().write(0x8000, 0x08);

    // Set some flags
    cpu.set_flag_c(true);
    cpu.set_flag_z(true);

    cpu.step().unwrap();

    // Stack should contain the status byte at 0x01FD (initial SP = 0xFD)
    let status = cpu.memory_mut().read(0x01FD);
    assert_eq!(status & 0b00000001, 0b00000001); // Carry set
    assert_eq!(status & 0b00000010, 0b00000010); // Zero set
    assert_eq!(status & 0b00110000, 0b00110000); // Bits 4 and 5 always set
    assert_eq!(cpu.sp(), 0xFC); // SP decremented from 0xFD to 0xFC
    assert_eq!(cpu.pc(), 0x8001); // PC advanced by 1 byte
    assert_eq!(cpu.cycles(), 3); // 3 cycles
}

#[test]
fn test_php_pushes_status_value() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);

    // Set specific flag combination: N, V, C set; others clear
    cpu.set_flag_n(true);
    cpu.set_flag_v(true);
    cpu.set_flag_c(true);
    cpu.set_flag_z(false);
    cpu.set_flag_i(false);
    cpu.set_flag_d(false);
    cpu.set_flag_b(false);

    cpu.step().unwrap();

    // Verify the pushed value
    let status = cpu.memory_mut().read(0x01FD);
    // Expected: N=1, V=1, bit5=1, B=1 (forced), D=0, I=0, Z=0, C=1
    // Binary: 11110001 = 0xF1
    assert_eq!(status, 0xF1);
}

#[test]
fn test_php_decrements_stack_pointer() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);

    let initial_sp = cpu.sp(); // Should be 0xFD
    cpu.step().unwrap();

    // Stack pointer should be decremented by 1
    assert_eq!(cpu.sp(), initial_sp.wrapping_sub(1));
}

// ========== Stack Pointer Tests ==========

#[test]
fn test_php_stack_pointer_wraparound() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);
    cpu.set_sp(0x00); // Set SP to 0

    cpu.step().unwrap();

    // Stack pointer should wrap from 0x00 to 0xFF
    assert_eq!(cpu.sp(), 0xFF);
    // Value should be written at 0x0100
    let status = cpu.memory_mut().read(0x0100);
    // Should have bits 4 and 5 set at minimum
    assert_eq!(status & 0b00110000, 0b00110000);
}

#[test]
fn test_php_multiple_pushes() {
    let mut cpu = setup_cpu();

    // Set up three PHP instructions
    cpu.memory_mut().write(0x8000, 0x08);
    cpu.memory_mut().write(0x8001, 0x08);
    cpu.memory_mut().write(0x8002, 0x08);

    // First push - set carry
    cpu.set_flag_c(true);
    cpu.step().unwrap();
    let status1 = cpu.memory_mut().read(0x01FD);
    assert_eq!(status1 & 0b00000001, 0b00000001);
    assert_eq!(cpu.sp(), 0xFC);

    // Second push - set zero
    cpu.set_flag_c(false);
    cpu.set_flag_z(true);
    cpu.step().unwrap();
    let status2 = cpu.memory_mut().read(0x01FC);
    assert_eq!(status2 & 0b00000010, 0b00000010);
    assert_eq!(cpu.sp(), 0xFB);

    // Third push - set negative
    cpu.set_flag_z(false);
    cpu.set_flag_n(true);
    cpu.step().unwrap();
    let status3 = cpu.memory_mut().read(0x01FB);
    assert_eq!(status3 & 0b10000000, 0b10000000);
    assert_eq!(cpu.sp(), 0xFA);
}

// ========== Status Flag Value Tests ==========

#[test]
fn test_php_all_flags_clear() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);

    // Clear all flags except I (which is set on reset)
    cpu.set_flag_c(false);
    cpu.set_flag_z(false);
    cpu.set_flag_i(false);
    cpu.set_flag_d(false);
    cpu.set_flag_b(false);
    cpu.set_flag_v(false);
    cpu.set_flag_n(false);

    cpu.step().unwrap();

    let status = cpu.memory_mut().read(0x01FD);
    // Only bits 4 and 5 should be set (0b00110000 = 0x30)
    assert_eq!(status, 0x30);
}

#[test]
fn test_php_all_flags_set() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);

    // Set all flags
    cpu.set_flag_c(true);
    cpu.set_flag_z(true);
    cpu.set_flag_i(true);
    cpu.set_flag_d(true);
    cpu.set_flag_b(true);
    cpu.set_flag_v(true);
    cpu.set_flag_n(true);

    cpu.step().unwrap();

    let status = cpu.memory_mut().read(0x01FD);
    // All bits should be set (0xFF)
    assert_eq!(status, 0xFF);
}

#[test]
fn test_php_carry_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    let status = cpu.memory_mut().read(0x01FD);
    assert_eq!(status & 0b00000001, 0b00000001);
}

#[test]
fn test_php_zero_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);
    cpu.set_flag_z(true);

    cpu.step().unwrap();

    let status = cpu.memory_mut().read(0x01FD);
    assert_eq!(status & 0b00000010, 0b00000010);
}

#[test]
fn test_php_interrupt_disable_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);
    cpu.set_flag_i(true);

    cpu.step().unwrap();

    let status = cpu.memory_mut().read(0x01FD);
    assert_eq!(status & 0b00000100, 0b00000100);
}

#[test]
fn test_php_decimal_mode_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);
    cpu.set_flag_d(true);

    cpu.step().unwrap();

    let status = cpu.memory_mut().read(0x01FD);
    assert_eq!(status & 0b00001000, 0b00001000);
}

#[test]
fn test_php_break_flag_forced_to_1() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);

    // Explicitly clear the B flag in CPU
    cpu.set_flag_b(false);

    cpu.step().unwrap();

    let status = cpu.memory_mut().read(0x01FD);
    // Bit 4 (B flag) should be set in the pushed value even though CPU B flag is clear
    assert_eq!(status & 0b00010000, 0b00010000);
}

#[test]
fn test_php_overflow_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);
    cpu.set_flag_v(true);

    cpu.step().unwrap();

    let status = cpu.memory_mut().read(0x01FD);
    assert_eq!(status & 0b01000000, 0b01000000);
}

#[test]
fn test_php_negative_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);
    cpu.set_flag_n(true);

    cpu.step().unwrap();

    let status = cpu.memory_mut().read(0x01FD);
    assert_eq!(status & 0b10000000, 0b10000000);
}

#[test]
fn test_php_bit5_always_set() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);

    // Clear all flags
    cpu.set_flag_c(false);
    cpu.set_flag_z(false);
    cpu.set_flag_i(false);
    cpu.set_flag_d(false);
    cpu.set_flag_b(false);
    cpu.set_flag_v(false);
    cpu.set_flag_n(false);

    cpu.step().unwrap();

    let status = cpu.memory_mut().read(0x01FD);
    // Bit 5 should always be set
    assert_eq!(status & 0b00100000, 0b00100000);
}

// ========== Flag Preservation Tests ==========

#[test]
fn test_php_preserves_all_flags_when_clear() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);

    // Clear all flags
    cpu.set_flag_c(false);
    cpu.set_flag_z(false);
    cpu.set_flag_i(false);
    cpu.set_flag_d(false);
    cpu.set_flag_b(false);
    cpu.set_flag_v(false);
    cpu.set_flag_n(false);

    cpu.step().unwrap();

    // All flags in CPU should remain unchanged
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_i());
    assert!(!cpu.flag_d());
    assert!(!cpu.flag_b());
    assert!(!cpu.flag_v());
    assert!(!cpu.flag_n());
}

#[test]
fn test_php_preserves_all_flags_when_set() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);

    // Set all flags
    cpu.set_flag_c(true);
    cpu.set_flag_z(true);
    cpu.set_flag_i(true);
    cpu.set_flag_d(true);
    cpu.set_flag_b(true);
    cpu.set_flag_v(true);
    cpu.set_flag_n(true);

    cpu.step().unwrap();

    // All flags in CPU should remain unchanged
    assert!(cpu.flag_c());
    assert!(cpu.flag_z());
    assert!(cpu.flag_i());
    assert!(cpu.flag_d());
    assert!(cpu.flag_b());
    assert!(cpu.flag_v());
    assert!(cpu.flag_n());
}

#[test]
fn test_php_preserves_carry_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert!(cpu.flag_c());
}

#[test]
fn test_php_preserves_zero_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);
    cpu.set_flag_z(true);

    cpu.step().unwrap();

    assert!(cpu.flag_z());
}

#[test]
fn test_php_preserves_interrupt_disable_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);
    cpu.set_flag_i(true);

    cpu.step().unwrap();

    assert!(cpu.flag_i());
}

#[test]
fn test_php_preserves_decimal_mode_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);
    cpu.set_flag_d(true);

    cpu.step().unwrap();

    assert!(cpu.flag_d());
}

#[test]
fn test_php_preserves_break_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);
    cpu.set_flag_b(true);

    cpu.step().unwrap();

    // CPU's B flag should remain set
    assert!(cpu.flag_b());
}

#[test]
fn test_php_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);
    cpu.set_flag_v(true);

    cpu.step().unwrap();

    assert!(cpu.flag_v());
}

#[test]
fn test_php_preserves_negative_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);
    cpu.set_flag_n(true);

    cpu.step().unwrap();

    assert!(cpu.flag_n());
}

// ========== Register Preservation Tests ==========

#[test]
fn test_php_preserves_accumulator() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);
    cpu.set_a(0x42);

    cpu.step().unwrap();

    // Accumulator should remain unchanged
    assert_eq!(cpu.a(), 0x42);
}

#[test]
fn test_php_preserves_x_register() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);
    cpu.set_x(0x33);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x33);
}

#[test]
fn test_php_preserves_y_register() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);
    cpu.set_y(0x44);

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x44);
}

#[test]
fn test_php_preserves_all_registers_except_sp() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);
    cpu.set_a(0x11);
    cpu.set_x(0x22);
    cpu.set_y(0x33);

    cpu.step().unwrap();

    // A, X, Y should be unchanged
    assert_eq!(cpu.a(), 0x11);
    assert_eq!(cpu.x(), 0x22);
    assert_eq!(cpu.y(), 0x33);
    // SP should be decremented
    assert_eq!(cpu.sp(), 0xFC);
}

// ========== Cycle Count Tests ==========

#[test]
fn test_php_cycle_count() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);

    let initial_cycles = cpu.cycles();
    cpu.step().unwrap();

    // PHP should take exactly 3 cycles
    assert_eq!(cpu.cycles() - initial_cycles, 3);
}

#[test]
fn test_php_multiple_cycle_count() {
    let mut cpu = setup_cpu();

    // Set up 5 PHP instructions
    for i in 0..5 {
        cpu.memory_mut().write(0x8000 + i, 0x08);
    }

    for i in 1..=5 {
        cpu.step().unwrap();
        assert_eq!(cpu.cycles(), (i * 3) as u64); // Each PHP takes 3 cycles
    }
}

// ========== PC Advancement Tests ==========

#[test]
fn test_php_advances_pc_correctly() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);

    cpu.step().unwrap();

    // PC should advance by 1 byte (size of PHP instruction)
    assert_eq!(cpu.pc(), 0x8001);
}

#[test]
fn test_php_pc_advancement_at_page_boundary() {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0xFF);
    memory.write(0xFFFD, 0x80);
    memory.write(0x80FF, 0x08); // PHP at page boundary

    let mut cpu = CPU::new(memory);

    cpu.step().unwrap();

    // PC should advance to next page
    assert_eq!(cpu.pc(), 0x8100);
}

// ========== Addressing Mode Tests ==========

#[test]
fn test_php_implicit_addressing_mode() {
    let mut cpu = setup_cpu();

    // PHP uses implicit addressing mode (opcode 0x08)
    cpu.memory_mut().write(0x8000, 0x08);

    cpu.step().unwrap();

    // Verify instruction executed correctly
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.cycles(), 3);
    let status = cpu.memory_mut().read(0x01FD);
    // Should at least have bits 4 and 5 set
    assert_eq!(status & 0b00110000, 0b00110000);
}

// ========== Sequential Execution Tests ==========

#[test]
fn test_php_with_other_instructions() {
    let mut cpu = setup_cpu();

    // Write PHP followed by other instructions
    cpu.memory_mut().write(0x8000, 0x08); // PHP
    cpu.memory_mut().write(0x8001, 0xEA); // NOP
    cpu.memory_mut().write(0x8002, 0x08); // PHP

    // First PHP
    cpu.set_flag_c(true);
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x8001);
    let status1 = cpu.memory_mut().read(0x01FD);
    assert_eq!(status1 & 0b00000001, 0b00000001);
    assert_eq!(cpu.sp(), 0xFC);

    // NOP
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x8002);

    // Second PHP
    cpu.set_flag_c(false);
    cpu.set_flag_z(true);
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x8003);
    let status2 = cpu.memory_mut().read(0x01FC);
    assert_eq!(status2 & 0b00000010, 0b00000010);
    assert_eq!(cpu.sp(), 0xFB);
}

// ========== Stack Overflow Scenario Tests ==========

#[test]
fn test_php_stack_full_scenario() {
    let mut cpu = setup_cpu();

    // Fill the entire stack (256 values)
    for i in 0..=255 {
        cpu.memory_mut().write(0x8000 + i, 0x08);
    }

    cpu.set_sp(0xFF); // Start at top of stack

    // Push 256 times
    for i in (0u16..=255).rev() {
        let expected_sp = (i as u8).wrapping_sub(1);
        cpu.step().unwrap();
        assert_eq!(cpu.sp(), expected_sp);
    }

    // Stack pointer should have wrapped around back to 0xFF
    assert_eq!(cpu.sp(), 0xFF);
}

// ========== Edge Case Tests ==========

#[test]
fn test_php_does_not_modify_other_memory() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);

    // Write some values to memory outside the stack area
    cpu.memory_mut().write(0x0000, 0xAA);
    cpu.memory_mut().write(0x0200, 0xBB);
    cpu.memory_mut().write(0x1000, 0xCC);

    cpu.step().unwrap();

    // Memory values outside stack should remain unchanged
    assert_eq!(cpu.memory_mut().read(0x0000), 0xAA);
    assert_eq!(cpu.memory_mut().read(0x0200), 0xBB);
    assert_eq!(cpu.memory_mut().read(0x1000), 0xCC);
}

#[test]
fn test_php_overwrites_existing_stack_data() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x08);

    // Pre-populate stack with old data
    cpu.memory_mut().write(0x01FD, 0xFF);

    // Clear all flags
    cpu.set_flag_c(false);
    cpu.set_flag_z(false);
    cpu.set_flag_i(false);
    cpu.set_flag_d(false);
    cpu.set_flag_b(false);
    cpu.set_flag_v(false);
    cpu.set_flag_n(false);

    cpu.step().unwrap();

    // Old data should be overwritten with status (only bits 4 and 5 set)
    assert_eq!(cpu.memory_mut().read(0x01FD), 0x30);
}

// ========== Flag Combination Tests ==========

#[test]
fn test_php_various_flag_combinations() {
    let mut cpu = setup_cpu();

    let test_cases = vec![
        (0b00000000, 0x30), // All clear -> bits 4,5 set
        (0b00000001, 0x31), // C set
        (0b00000010, 0x32), // Z set
        (0b00000011, 0x33), // C,Z set
        (0b10000000, 0xB0), // N set
        (0b11000000, 0xF0), // N,V set
        (0b11111111, 0xFF), // All set
    ];

    for i in 0..test_cases.len() {
        cpu.memory_mut().write(0x8000 + i as u16, 0x08);
    }

    cpu.set_pc(0x8000);

    for &(flags_input, expected_status) in &test_cases {
        // Set flags based on input
        cpu.set_flag_c(flags_input & 0b00000001 != 0);
        cpu.set_flag_z(flags_input & 0b00000010 != 0);
        cpu.set_flag_i(flags_input & 0b00000100 != 0);
        cpu.set_flag_d(flags_input & 0b00001000 != 0);
        cpu.set_flag_b(flags_input & 0b00010000 != 0);
        cpu.set_flag_v(flags_input & 0b01000000 != 0);
        cpu.set_flag_n(flags_input & 0b10000000 != 0);

        let sp_before = cpu.sp();
        cpu.step().unwrap();

        // Verify value was pushed correctly
        let stack_addr = 0x0100 | (sp_before as u16);
        let pushed_status = cpu.memory_mut().read(stack_addr);
        assert_eq!(
            pushed_status, expected_status,
            "Flag input 0b{:08b} should produce 0x{:02X}, got 0x{:02X}",
            flags_input, expected_status, pushed_status
        );
    }
}

// ========== Integration Tests ==========

#[test]
fn test_php_and_pha_sequence() {
    let mut cpu = setup_cpu();

    // Test PHP and PHA work correctly in sequence
    cpu.memory_mut().write(0x8000, 0x08); // PHP
    cpu.memory_mut().write(0x8001, 0x48); // PHA

    cpu.set_a(0x42);
    cpu.set_flag_c(true);
    cpu.set_flag_z(true);

    // Execute PHP
    cpu.step().unwrap();
    let status = cpu.memory_mut().read(0x01FD);
    assert_eq!(status & 0b00000011, 0b00000011); // C and Z set
    assert_eq!(cpu.sp(), 0xFC);

    // Execute PHA
    cpu.step().unwrap();
    assert_eq!(cpu.memory_mut().read(0x01FC), 0x42); // Accumulator value
    assert_eq!(cpu.sp(), 0xFB);

    // Verify both values are on the stack correctly
    assert_eq!(cpu.memory_mut().read(0x01FD) & 0b00000011, 0b00000011);
    assert_eq!(cpu.memory_mut().read(0x01FC), 0x42);
}
