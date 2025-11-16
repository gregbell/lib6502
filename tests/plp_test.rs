//! Comprehensive tests for the PLP (Pull Processor Status) instruction.
//!
//! Tests cover:
//! - Basic PLP operation works correctly
//! - All addressing modes (Implicit only for PLP)
//! - Correct cycle counts
//! - Stack pointer incrementation
//! - All processor flags correctly restored from stack
//! - Other registers (A, X, Y) are not affected
//! - Edge cases (stack wraparound, various flag combinations)
//! - Integration with PHP (round-trip testing)

use cpu6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic PLP Operation Tests ==========

#[test]
fn test_plp_basic_operation() {
    let mut cpu = setup_cpu();

    // PLP (0x28)
    cpu.memory_mut().write(0x8000, 0x28);

    // Setup: Put status byte on stack
    cpu.memory_mut().write(0x01FD, 0b11000011); // N, V, C, Z set
    cpu.set_sp(0xFC); // SP points below the value

    // Clear all flags initially
    cpu.set_flag_c(false);
    cpu.set_flag_z(false);
    cpu.set_flag_n(false);
    cpu.set_flag_v(false);

    cpu.step().unwrap();

    // Flags should be restored from stack
    assert!(cpu.flag_n()); // Bit 7 set
    assert!(cpu.flag_v()); // Bit 6 set
    assert!(cpu.flag_z()); // Bit 1 set
    assert!(cpu.flag_c()); // Bit 0 set
    assert_eq!(cpu.sp(), 0xFD); // SP incremented from 0xFC to 0xFD
    assert_eq!(cpu.pc(), 0x8001); // PC advanced by 1 byte
    assert_eq!(cpu.cycles(), 4); // 4 cycles
}

#[test]
fn test_plp_pulls_status_value() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);

    // Setup: Put a specific status value on stack
    cpu.memory_mut().write(0x01FD, 0xFF); // All flags set
    cpu.set_sp(0xFC);

    cpu.step().unwrap();

    // Verify all flags are set
    assert!(cpu.flag_c());
    assert!(cpu.flag_z());
    assert!(cpu.flag_i());
    assert!(cpu.flag_d());
    assert!(cpu.flag_b());
    assert!(cpu.flag_v());
    assert!(cpu.flag_n());
}

#[test]
fn test_plp_increments_stack_pointer() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);

    // Setup stack
    cpu.memory_mut().write(0x01FD, 0x00);
    cpu.set_sp(0xFC);

    let initial_sp = cpu.sp(); // Should be 0xFC
    cpu.step().unwrap();

    // Stack pointer should be incremented by 1
    assert_eq!(cpu.sp(), initial_sp.wrapping_add(1));
}

// ========== Stack Pointer Tests ==========

#[test]
fn test_plp_stack_pointer_wraparound() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);

    // Setup: Put value at 0x0100 and set SP to 0xFF
    cpu.memory_mut().write(0x0100, 0b10101010);
    cpu.set_sp(0xFF); // SP at top

    cpu.step().unwrap();

    // Stack pointer should wrap from 0xFF to 0x00
    assert_eq!(cpu.sp(), 0x00);
    // Flags should be pulled from 0x0100
    assert!(cpu.flag_n()); // Bit 7
    assert!(!cpu.flag_v()); // Bit 6
    assert!(cpu.flag_d()); // Bit 3
    assert!(!cpu.flag_i()); // Bit 2
    assert!(cpu.flag_z()); // Bit 1
    assert!(!cpu.flag_c()); // Bit 0
}

#[test]
fn test_plp_multiple_pulls() {
    let mut cpu = setup_cpu();

    // Set up three PLP instructions
    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x8001, 0x28);
    cpu.memory_mut().write(0x8002, 0x28);

    // Setup stack with three status values
    cpu.memory_mut().write(0x01FB, 0b00000001); // Only C set
    cpu.memory_mut().write(0x01FC, 0b00000010); // Only Z set
    cpu.memory_mut().write(0x01FD, 0b10000000); // Only N set
    cpu.set_sp(0xFA); // Start below all values

    // First pull - C flag
    cpu.step().unwrap();
    assert!(cpu.flag_c());
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.sp(), 0xFB);

    // Second pull - Z flag
    cpu.step().unwrap();
    assert!(!cpu.flag_c());
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
    assert_eq!(cpu.sp(), 0xFC);

    // Third pull - N flag
    cpu.step().unwrap();
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
    assert_eq!(cpu.sp(), 0xFD);
}

// ========== Individual Flag Restoration Tests ==========

#[test]
fn test_plp_restores_carry_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0b00000001); // C set
    cpu.set_sp(0xFC);
    cpu.set_flag_c(false); // Start with C clear

    cpu.step().unwrap();

    assert!(cpu.flag_c());
}

#[test]
fn test_plp_clears_carry_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0b00000000); // C clear
    cpu.set_sp(0xFC);
    cpu.set_flag_c(true); // Start with C set

    cpu.step().unwrap();

    assert!(!cpu.flag_c());
}

#[test]
fn test_plp_restores_zero_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0b00000010); // Z set
    cpu.set_sp(0xFC);
    cpu.set_flag_z(false);

    cpu.step().unwrap();

    assert!(cpu.flag_z());
}

#[test]
fn test_plp_clears_zero_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0b00000000); // Z clear
    cpu.set_sp(0xFC);
    cpu.set_flag_z(true);

    cpu.step().unwrap();

    assert!(!cpu.flag_z());
}

#[test]
fn test_plp_restores_interrupt_disable_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0b00000100); // I set
    cpu.set_sp(0xFC);
    cpu.set_flag_i(false);

    cpu.step().unwrap();

    assert!(cpu.flag_i());
}

#[test]
fn test_plp_clears_interrupt_disable_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0b00000000); // I clear
    cpu.set_sp(0xFC);
    cpu.set_flag_i(true);

    cpu.step().unwrap();

    assert!(!cpu.flag_i());
}

#[test]
fn test_plp_restores_decimal_mode_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0b00001000); // D set
    cpu.set_sp(0xFC);
    cpu.set_flag_d(false);

    cpu.step().unwrap();

    assert!(cpu.flag_d());
}

#[test]
fn test_plp_clears_decimal_mode_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0b00000000); // D clear
    cpu.set_sp(0xFC);
    cpu.set_flag_d(true);

    cpu.step().unwrap();

    assert!(!cpu.flag_d());
}

#[test]
fn test_plp_restores_break_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0b00010000); // B set
    cpu.set_sp(0xFC);
    cpu.set_flag_b(false);

    cpu.step().unwrap();

    assert!(cpu.flag_b());
}

#[test]
fn test_plp_clears_break_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0b00000000); // B clear
    cpu.set_sp(0xFC);
    cpu.set_flag_b(true);

    cpu.step().unwrap();

    assert!(!cpu.flag_b());
}

#[test]
fn test_plp_restores_overflow_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0b01000000); // V set
    cpu.set_sp(0xFC);
    cpu.set_flag_v(false);

    cpu.step().unwrap();

    assert!(cpu.flag_v());
}

#[test]
fn test_plp_clears_overflow_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0b00000000); // V clear
    cpu.set_sp(0xFC);
    cpu.set_flag_v(true);

    cpu.step().unwrap();

    assert!(!cpu.flag_v());
}

#[test]
fn test_plp_restores_negative_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0b10000000); // N set
    cpu.set_sp(0xFC);
    cpu.set_flag_n(false);

    cpu.step().unwrap();

    assert!(cpu.flag_n());
}

#[test]
fn test_plp_clears_negative_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0b00000000); // N clear
    cpu.set_sp(0xFC);
    cpu.set_flag_n(true);

    cpu.step().unwrap();

    assert!(!cpu.flag_n());
}

#[test]
fn test_plp_ignores_bit_5() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);

    // Test with bit 5 set
    cpu.memory_mut().write(0x01FD, 0b00100000); // Only bit 5 set
    cpu.set_sp(0xFC);

    cpu.step().unwrap();

    // All flags should be clear (bit 5 is ignored)
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_i());
    assert!(!cpu.flag_d());
    assert!(!cpu.flag_b());
    assert!(!cpu.flag_v());
    assert!(!cpu.flag_n());
}

// ========== All Flags Set/Clear Tests ==========

#[test]
fn test_plp_all_flags_clear() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0b00000000); // All clear

    // Set all flags initially
    cpu.set_flag_c(true);
    cpu.set_flag_z(true);
    cpu.set_flag_i(true);
    cpu.set_flag_d(true);
    cpu.set_flag_b(true);
    cpu.set_flag_v(true);
    cpu.set_flag_n(true);

    cpu.set_sp(0xFC);
    cpu.step().unwrap();

    // All flags should be clear
    assert!(!cpu.flag_c());
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_i());
    assert!(!cpu.flag_d());
    assert!(!cpu.flag_b());
    assert!(!cpu.flag_v());
    assert!(!cpu.flag_n());
}

#[test]
fn test_plp_all_flags_set() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0xFF); // All set

    // Clear all flags initially
    cpu.set_flag_c(false);
    cpu.set_flag_z(false);
    cpu.set_flag_i(false);
    cpu.set_flag_d(false);
    cpu.set_flag_b(false);
    cpu.set_flag_v(false);
    cpu.set_flag_n(false);

    cpu.set_sp(0xFC);
    cpu.step().unwrap();

    // All flags should be set
    assert!(cpu.flag_c());
    assert!(cpu.flag_z());
    assert!(cpu.flag_i());
    assert!(cpu.flag_d());
    assert!(cpu.flag_b());
    assert!(cpu.flag_v());
    assert!(cpu.flag_n());
}

// ========== Register Preservation Tests ==========

#[test]
fn test_plp_preserves_accumulator() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0xFF);
    cpu.set_sp(0xFC);
    cpu.set_a(0x42);

    cpu.step().unwrap();

    // Accumulator should remain unchanged
    assert_eq!(cpu.a(), 0x42);
}

#[test]
fn test_plp_preserves_x_register() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0xFF);
    cpu.set_sp(0xFC);
    cpu.set_x(0x33);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x33);
}

#[test]
fn test_plp_preserves_y_register() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0xFF);
    cpu.set_sp(0xFC);
    cpu.set_y(0x44);

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x44);
}

#[test]
fn test_plp_preserves_all_registers_except_flags() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0b11111111);
    cpu.set_sp(0xFC);
    cpu.set_a(0x11);
    cpu.set_x(0x22);
    cpu.set_y(0x33);

    cpu.step().unwrap();

    // A, X, Y should be unchanged
    assert_eq!(cpu.a(), 0x11);
    assert_eq!(cpu.x(), 0x22);
    assert_eq!(cpu.y(), 0x33);
    // SP should be incremented
    assert_eq!(cpu.sp(), 0xFD);
}

// ========== Cycle Count Tests ==========

#[test]
fn test_plp_cycle_count() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0x00);
    cpu.set_sp(0xFC);

    let initial_cycles = cpu.cycles();
    cpu.step().unwrap();

    // PLP should take exactly 4 cycles
    assert_eq!(cpu.cycles() - initial_cycles, 4);
}

#[test]
fn test_plp_multiple_cycle_count() {
    let mut cpu = setup_cpu();

    // Set up 5 PLP instructions
    for i in 0..5 {
        cpu.memory_mut().write(0x8000 + i, 0x28);
    }

    // Setup stack with status values
    cpu.memory_mut().write(0x01FB, 0x00);
    cpu.memory_mut().write(0x01FC, 0x00);
    cpu.memory_mut().write(0x01FD, 0x00);
    cpu.memory_mut().write(0x01FE, 0x00);
    cpu.memory_mut().write(0x01FF, 0x00);
    cpu.set_sp(0xFA);

    for i in 1..=5 {
        cpu.step().unwrap();
        assert_eq!(cpu.cycles(), (i * 4) as u64); // Each PLP takes 4 cycles
    }
}

// ========== PC Advancement Tests ==========

#[test]
fn test_plp_advances_pc_correctly() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0x00);
    cpu.set_sp(0xFC);

    cpu.step().unwrap();

    // PC should advance by 1 byte (size of PLP instruction)
    assert_eq!(cpu.pc(), 0x8001);
}

#[test]
fn test_plp_pc_advancement_at_page_boundary() {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0xFF);
    memory.write(0xFFFD, 0x80);
    memory.write(0x80FF, 0x28); // PLP at page boundary
    memory.write(0x01FD, 0x00);

    let mut cpu = CPU::new(memory);
    cpu.set_sp(0xFC);

    cpu.step().unwrap();

    // PC should advance to next page
    assert_eq!(cpu.pc(), 0x8100);
}

// ========== Addressing Mode Tests ==========

#[test]
fn test_plp_implicit_addressing_mode() {
    let mut cpu = setup_cpu();

    // PLP uses implicit addressing mode (opcode 0x28)
    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0x00);
    cpu.set_sp(0xFC);

    cpu.step().unwrap();

    // Verify instruction executed correctly
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.cycles(), 4);
    assert_eq!(cpu.sp(), 0xFD);
}

// ========== Flag Combination Tests ==========

#[test]
fn test_plp_various_flag_combinations() {
    let mut cpu = setup_cpu();

    let test_cases = vec![
        (
            0b00000000,
            [false, false, false, false, false, false, false],
        ), // All clear
        (0b00000001, [true, false, false, false, false, false, false]), // C
        (0b00000010, [false, true, false, false, false, false, false]), // Z
        (0b00000011, [true, true, false, false, false, false, false]),  // C,Z
        (0b10000000, [false, false, false, false, false, false, true]), // N
        (0b11000000, [false, false, false, false, false, true, true]),  // N,V
        (0b11111111, [true, true, true, true, true, true, true]),       // All set
    ];

    for i in 0..test_cases.len() {
        cpu.memory_mut().write(0x8000 + i as u16, 0x28);
    }

    cpu.set_pc(0x8000);
    cpu.set_sp(0xFC);

    for &(status_byte, expected_flags) in &test_cases {
        // Put status byte on stack
        cpu.memory_mut().write(0x01FD, status_byte);
        cpu.set_sp(0xFC); // Reset SP

        cpu.step().unwrap();

        // Verify flags [C, Z, I, D, B, V, N]
        assert_eq!(
            cpu.flag_c(),
            expected_flags[0],
            "Status 0x{:02X}: C flag mismatch",
            status_byte
        );
        assert_eq!(
            cpu.flag_z(),
            expected_flags[1],
            "Status 0x{:02X}: Z flag mismatch",
            status_byte
        );
        assert_eq!(
            cpu.flag_i(),
            expected_flags[2],
            "Status 0x{:02X}: I flag mismatch",
            status_byte
        );
        assert_eq!(
            cpu.flag_d(),
            expected_flags[3],
            "Status 0x{:02X}: D flag mismatch",
            status_byte
        );
        assert_eq!(
            cpu.flag_b(),
            expected_flags[4],
            "Status 0x{:02X}: B flag mismatch",
            status_byte
        );
        assert_eq!(
            cpu.flag_v(),
            expected_flags[5],
            "Status 0x{:02X}: V flag mismatch",
            status_byte
        );
        assert_eq!(
            cpu.flag_n(),
            expected_flags[6],
            "Status 0x{:02X}: N flag mismatch",
            status_byte
        );
    }
}

// ========== Edge Case Tests ==========

#[test]
fn test_plp_does_not_modify_other_memory() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0x00);
    cpu.set_sp(0xFC);

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
fn test_plp_does_not_modify_stack_value() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x28);
    cpu.memory_mut().write(0x01FD, 0xFF);
    cpu.set_sp(0xFC);

    cpu.step().unwrap();

    // Stack value should still be present (reads are non-destructive)
    assert_eq!(cpu.memory_mut().read(0x01FD), 0xFF);
}

// ========== PHP/PLP Round-Trip Tests ==========

#[test]
fn test_php_plp_round_trip() {
    let mut cpu = setup_cpu();

    // Write PHP followed by PLP
    cpu.memory_mut().write(0x8000, 0x08); // PHP
    cpu.memory_mut().write(0x8001, 0x28); // PLP

    // Set specific flags
    cpu.set_flag_c(true);
    cpu.set_flag_z(true);
    cpu.set_flag_n(true);
    cpu.set_flag_v(false);
    cpu.set_flag_d(false);
    cpu.set_flag_i(false);
    cpu.set_flag_b(false);

    // Push
    cpu.step().unwrap();
    assert_eq!(cpu.sp(), 0xFC);

    // Modify flags
    cpu.set_flag_c(false);
    cpu.set_flag_z(false);
    cpu.set_flag_n(false);
    cpu.set_flag_v(true);

    // Pull
    cpu.step().unwrap();

    // Flags should be restored (note: B flag will be set due to PHP quirk)
    assert!(cpu.flag_c());
    assert!(cpu.flag_z());
    assert!(cpu.flag_n());
    assert!(!cpu.flag_v());
    assert!(!cpu.flag_d());
    assert!(!cpu.flag_i());
    assert!(cpu.flag_b()); // B flag forced to 1 by PHP
    assert_eq!(cpu.sp(), 0xFD); // Back to original SP
}

#[test]
fn test_multiple_php_plp_round_trips() {
    let mut cpu = setup_cpu();

    // Write sequence of PHP/PHP/PHP/PLP/PLP/PLP
    cpu.memory_mut().write(0x8000, 0x08); // PHP
    cpu.memory_mut().write(0x8001, 0x08); // PHP
    cpu.memory_mut().write(0x8002, 0x08); // PHP
    cpu.memory_mut().write(0x8003, 0x28); // PLP
    cpu.memory_mut().write(0x8004, 0x28); // PLP
    cpu.memory_mut().write(0x8005, 0x28); // PLP

    // Push three different flag states
    cpu.set_flag_c(true);
    cpu.set_flag_z(false);
    cpu.step().unwrap(); // PHP

    cpu.set_flag_c(false);
    cpu.set_flag_z(true);
    cpu.step().unwrap(); // PHP

    cpu.set_flag_c(true);
    cpu.set_flag_z(true);
    cpu.step().unwrap(); // PHP

    assert_eq!(cpu.sp(), 0xFA); // SP decremented 3 times

    // Pull in reverse order (stack is LIFO)
    cpu.step().unwrap(); // PLP - last pushed, first pulled
    assert!(cpu.flag_c());
    assert!(cpu.flag_z());

    cpu.step().unwrap(); // PLP
    assert!(!cpu.flag_c());
    assert!(cpu.flag_z());

    cpu.step().unwrap(); // PLP - first pushed, last pulled
    assert!(cpu.flag_c());
    assert!(!cpu.flag_z());

    assert_eq!(cpu.sp(), 0xFD); // Back to original SP
}

// ========== Sequential Execution Tests ==========

#[test]
fn test_plp_with_other_instructions() {
    let mut cpu = setup_cpu();

    // Write PLP followed by other instructions
    cpu.memory_mut().write(0x8000, 0x28); // PLP
    cpu.memory_mut().write(0x8001, 0xEA); // NOP
    cpu.memory_mut().write(0x8002, 0x28); // PLP

    // Setup stack
    cpu.memory_mut().write(0x01FD, 0b00000001); // C set
    cpu.memory_mut().write(0x01FE, 0b00000010); // Z set
    cpu.set_sp(0xFC);

    // First PLP
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x8001);
    assert!(cpu.flag_c());
    assert!(!cpu.flag_z());
    assert_eq!(cpu.sp(), 0xFD);

    // NOP
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x8002);

    // Second PLP
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x8003);
    assert!(!cpu.flag_c());
    assert!(cpu.flag_z());
    assert_eq!(cpu.sp(), 0xFE);
}

// ========== Integration Tests ==========

#[test]
fn test_plp_and_pla_sequence() {
    let mut cpu = setup_cpu();

    // Test PLP and PLA work correctly in sequence
    cpu.memory_mut().write(0x8000, 0x28); // PLP
    cpu.memory_mut().write(0x8001, 0x68); // PLA

    // Setup stack
    cpu.memory_mut().write(0x01FD, 0xFF); // Status byte
    cpu.memory_mut().write(0x01FE, 0x42); // Accumulator value
    cpu.set_sp(0xFC);

    // Execute PLP
    cpu.step().unwrap();
    assert!(cpu.flag_c());
    assert!(cpu.flag_z());
    assert_eq!(cpu.sp(), 0xFD);

    // Execute PLA
    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x42);
    assert_eq!(cpu.sp(), 0xFE);

    // Verify both operations completed correctly
    assert!(cpu.flag_c()); // From PLP, preserved through PLA
    assert!(!cpu.flag_z()); // Updated by PLA (0x42 is not zero)
}

#[test]
fn test_plp_restores_status_after_operations() {
    let mut cpu = setup_cpu();

    // Save status, do some operations, restore status
    cpu.memory_mut().write(0x8000, 0x08); // PHP - save status
    cpu.memory_mut().write(0x8001, 0xA9); // LDA #$FF (would set N, clear Z)
    cpu.memory_mut().write(0x8002, 0xFF);
    cpu.memory_mut().write(0x8003, 0x28); // PLP - restore status

    // Set initial flags
    cpu.set_flag_c(true);
    cpu.set_flag_z(true);
    cpu.set_flag_n(false);

    // Save status
    cpu.step().unwrap();

    // This instruction would modify flags (if LDA were implemented)
    // For now, we'll manually modify flags to simulate
    cpu.set_pc(0x8003); // Skip to PLP
    cpu.set_flag_z(false);
    cpu.set_flag_n(true);
    cpu.set_flag_c(false);

    // Restore status
    cpu.step().unwrap();

    // Flags should be restored to original values (with B forced to 1 by PHP)
    assert!(cpu.flag_c());
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
    assert!(cpu.flag_b()); // B flag forced to 1 by PHP
}
