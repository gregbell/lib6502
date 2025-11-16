//! Comprehensive tests for the RTI (Return from Interrupt) instruction.
//!
//! Tests cover:
//! - Basic RTI operation works correctly
//! - All addressing modes (Implicit only for RTI)
//! - Correct cycle counts (6 cycles)
//! - Stack pointer incrementation (3 times: status + PC low + PC high)
//! - All processor flags correctly restored from stack
//! - Program counter correctly restored from stack
//! - Other registers (A, X, Y) are not affected
//! - Edge cases (stack wraparound, various flag combinations)
//! - Integration with BRK (round-trip testing)

use lib6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic RTI Operation Tests ==========

#[test]
fn test_rti_basic_operation() {
    let mut cpu = setup_cpu();

    // RTI (0x40)
    cpu.memory_mut().write(0x8000, 0x40);

    // Setup: Put status and PC on stack (as BRK or interrupt would)
    // BRK pushes: PC_high at SP, PC_low at SP-1, status at SP-2
    // So with final SP=0xFA: PC_high at 0x01FD, PC_low at 0x01FC, status at 0x01FB
    cpu.memory_mut().write(0x01FD, 0x12); // PC high byte
    cpu.memory_mut().write(0x01FC, 0x34); // PC low byte -> PC = 0x1234
    cpu.memory_mut().write(0x01FB, 0b00100011); // Status: C, Z set
    cpu.set_sp(0xFA); // SP points below all values

    // Clear all flags initially
    cpu.set_flag_c(false);
    cpu.set_flag_z(false);
    cpu.set_flag_n(false);
    cpu.set_flag_v(false);

    cpu.step().unwrap();

    // Flags should be restored from stack
    assert!(cpu.flag_c()); // Bit 0 set
    assert!(cpu.flag_z()); // Bit 1 set
    assert!(!cpu.flag_i()); // Bit 2 clear
    assert!(!cpu.flag_n()); // Bit 7 clear

    // PC should be restored from stack
    assert_eq!(cpu.pc(), 0x1234);

    // SP should be incremented 3 times (status + PC_low + PC_high)
    assert_eq!(cpu.sp(), 0xFD); // 0xFA + 3 = 0xFD

    // Cycles should be 6
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_rti_restores_all_flags() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);

    // Setup: All flags set in status byte
    cpu.memory_mut().write(0x01FD, 0x00); // PC high
    cpu.memory_mut().write(0x01FC, 0x00); // PC low
    cpu.memory_mut().write(0x01FB, 0xFF); // All bits set
    cpu.set_sp(0xFA);

    // Clear all flags initially
    cpu.set_flag_c(false);
    cpu.set_flag_z(false);
    cpu.set_flag_i(false);
    cpu.set_flag_d(false);
    cpu.set_flag_b(false);
    cpu.set_flag_v(false);
    cpu.set_flag_n(false);

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
fn test_rti_restores_pc_correctly() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);

    // Test various PC values
    let test_cases = vec![0x0000u16, 0x1234, 0x8000, 0xABCD, 0xFFFF];

    for &expected_pc in &test_cases {
        cpu.set_pc(0x8000);
        cpu.set_sp(0xFA);

        // Push status and PC to stack
        cpu.memory_mut().write(0x01FD, (expected_pc >> 8) as u8); // PC high
        cpu.memory_mut().write(0x01FC, (expected_pc & 0xFF) as u8); // PC low
        cpu.memory_mut().write(0x01FB, 0x00); // Status

        cpu.step().unwrap();

        assert_eq!(
            cpu.pc(),
            expected_pc,
            "Failed to restore PC 0x{:04X}",
            expected_pc
        );
    }
}

// ========== Stack Pointer Tests ==========

#[test]
fn test_rti_increments_stack_pointer_by_three() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);

    // Setup stack
    cpu.memory_mut().write(0x01FD, 0x00); // PC high
    cpu.memory_mut().write(0x01FC, 0x00); // PC low
    cpu.memory_mut().write(0x01FB, 0x00); // Status
    cpu.set_sp(0xFA);

    let initial_sp = cpu.sp(); // Should be 0xFA
    cpu.step().unwrap();

    // Stack pointer should be incremented by 3
    assert_eq!(cpu.sp(), initial_sp.wrapping_add(3));
}

#[test]
fn test_rti_stack_pointer_wraparound() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);

    // Setup: Put values near top of stack to test wraparound
    // SP=0xFE: increment to 0xFF (pull status), increment to 0x00 (pull PC low), increment to 0x01 (pull PC high)
    // So: status at 0x01FF, PC low at 0x0100, PC high at 0x0101
    cpu.memory_mut().write(0x0101, 0x12); // PC high
    cpu.memory_mut().write(0x0100, 0x34); // PC low
    cpu.memory_mut().write(0x01FF, 0b10101010); // Status
    cpu.set_sp(0xFE); // SP = 0xFE

    cpu.step().unwrap();

    // Stack pointer should wrap: 0xFE + 3 = 0x01
    assert_eq!(cpu.sp(), 0x01);
    // PC should be restored correctly
    assert_eq!(cpu.pc(), 0x1234);
    // Flags should be pulled from 0x0100
    assert!(cpu.flag_n()); // Bit 7
    assert!(!cpu.flag_v()); // Bit 6
    assert!(cpu.flag_d()); // Bit 3
}

// ========== Individual Flag Restoration Tests ==========

#[test]
fn test_rti_restores_carry_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);
    cpu.memory_mut().write(0x01FD, 0x00); // PC high
    cpu.memory_mut().write(0x01FC, 0x00); // PC low
    cpu.memory_mut().write(0x01FB, 0b00000001); // C set
    cpu.set_sp(0xFA);
    cpu.set_flag_c(false); // Start with C clear

    cpu.step().unwrap();

    assert!(cpu.flag_c());
}

#[test]
fn test_rti_clears_carry_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);
    cpu.memory_mut().write(0x01FD, 0x00); // PC high
    cpu.memory_mut().write(0x01FC, 0x00); // PC low
    cpu.memory_mut().write(0x01FB, 0b00000000); // C clear
    cpu.set_sp(0xFA);
    cpu.set_flag_c(true); // Start with C set

    cpu.step().unwrap();

    assert!(!cpu.flag_c());
}

#[test]
fn test_rti_restores_zero_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);
    cpu.memory_mut().write(0x01FD, 0x00); // PC high
    cpu.memory_mut().write(0x01FC, 0x00); // PC low
    cpu.memory_mut().write(0x01FB, 0b00000010); // Z set
    cpu.set_sp(0xFA);
    cpu.set_flag_z(false);

    cpu.step().unwrap();

    assert!(cpu.flag_z());
}

#[test]
fn test_rti_clears_zero_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);
    cpu.memory_mut().write(0x01FD, 0x00); // PC high
    cpu.memory_mut().write(0x01FC, 0x00); // PC low
    cpu.memory_mut().write(0x01FB, 0b00000000); // Z clear
    cpu.set_sp(0xFA);
    cpu.set_flag_z(true);

    cpu.step().unwrap();

    assert!(!cpu.flag_z());
}

#[test]
fn test_rti_restores_interrupt_disable_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);
    cpu.memory_mut().write(0x01FD, 0x00); // PC high
    cpu.memory_mut().write(0x01FC, 0x00); // PC low
    cpu.memory_mut().write(0x01FB, 0b00000100); // I set
    cpu.set_sp(0xFA);
    cpu.set_flag_i(false);

    cpu.step().unwrap();

    assert!(cpu.flag_i());
}

#[test]
fn test_rti_clears_interrupt_disable_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);
    cpu.memory_mut().write(0x01FD, 0x00); // PC high
    cpu.memory_mut().write(0x01FC, 0x00); // PC low
    cpu.memory_mut().write(0x01FB, 0b00000000); // I clear
    cpu.set_sp(0xFA);
    cpu.set_flag_i(true);

    cpu.step().unwrap();

    assert!(!cpu.flag_i());
}

#[test]
fn test_rti_restores_decimal_mode_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);
    cpu.memory_mut().write(0x01FD, 0x00); // PC high
    cpu.memory_mut().write(0x01FC, 0x00); // PC low
    cpu.memory_mut().write(0x01FB, 0b00001000); // D set
    cpu.set_sp(0xFA);
    cpu.set_flag_d(false);

    cpu.step().unwrap();

    assert!(cpu.flag_d());
}

#[test]
fn test_rti_clears_decimal_mode_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);
    cpu.memory_mut().write(0x01FD, 0x00); // PC high
    cpu.memory_mut().write(0x01FC, 0x00); // PC low
    cpu.memory_mut().write(0x01FB, 0b00000000); // D clear
    cpu.set_sp(0xFA);
    cpu.set_flag_d(true);

    cpu.step().unwrap();

    assert!(!cpu.flag_d());
}

#[test]
fn test_rti_restores_break_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);
    cpu.memory_mut().write(0x01FD, 0x00); // PC high
    cpu.memory_mut().write(0x01FC, 0x00); // PC low
    cpu.memory_mut().write(0x01FB, 0b00010000); // B set
    cpu.set_sp(0xFA);
    cpu.set_flag_b(false);

    cpu.step().unwrap();

    assert!(cpu.flag_b());
}

#[test]
fn test_rti_clears_break_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);
    cpu.memory_mut().write(0x01FD, 0x00); // PC high
    cpu.memory_mut().write(0x01FC, 0x00); // PC low
    cpu.memory_mut().write(0x01FB, 0b00000000); // B clear
    cpu.set_sp(0xFA);
    cpu.set_flag_b(true);

    cpu.step().unwrap();

    assert!(!cpu.flag_b());
}

#[test]
fn test_rti_restores_overflow_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);
    cpu.memory_mut().write(0x01FD, 0x00); // PC high
    cpu.memory_mut().write(0x01FC, 0x00); // PC low
    cpu.memory_mut().write(0x01FB, 0b01000000); // V set
    cpu.set_sp(0xFA);
    cpu.set_flag_v(false);

    cpu.step().unwrap();

    assert!(cpu.flag_v());
}

#[test]
fn test_rti_clears_overflow_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);
    cpu.memory_mut().write(0x01FD, 0x00); // PC high
    cpu.memory_mut().write(0x01FC, 0x00); // PC low
    cpu.memory_mut().write(0x01FB, 0b00000000); // V clear
    cpu.set_sp(0xFA);
    cpu.set_flag_v(true);

    cpu.step().unwrap();

    assert!(!cpu.flag_v());
}

#[test]
fn test_rti_restores_negative_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);
    cpu.memory_mut().write(0x01FD, 0x00); // PC high
    cpu.memory_mut().write(0x01FC, 0x00); // PC low
    cpu.memory_mut().write(0x01FB, 0b10000000); // N set
    cpu.set_sp(0xFA);
    cpu.set_flag_n(false);

    cpu.step().unwrap();

    assert!(cpu.flag_n());
}

#[test]
fn test_rti_clears_negative_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);
    cpu.memory_mut().write(0x01FD, 0x00); // PC high
    cpu.memory_mut().write(0x01FC, 0x00); // PC low
    cpu.memory_mut().write(0x01FB, 0b00000000); // N clear
    cpu.set_sp(0xFA);
    cpu.set_flag_n(true);

    cpu.step().unwrap();

    assert!(!cpu.flag_n());
}

#[test]
fn test_rti_ignores_bit_5() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);

    // Test with bit 5 set
    cpu.memory_mut().write(0x01FD, 0x00); // PC high
    cpu.memory_mut().write(0x01FC, 0x00); // PC low
    cpu.memory_mut().write(0x01FB, 0b00100000); // Only bit 5 set
    cpu.set_sp(0xFA);

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

// ========== Register Preservation Tests ==========

#[test]
fn test_rti_preserves_accumulator() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);
    cpu.memory_mut().write(0x01FD, 0xFF);
    cpu.memory_mut().write(0x01FC, 0x00);
    cpu.memory_mut().write(0x01FB, 0x00);
    cpu.set_sp(0xFA);
    cpu.set_a(0x42);

    cpu.step().unwrap();

    // Accumulator should remain unchanged
    assert_eq!(cpu.a(), 0x42);
}

#[test]
fn test_rti_preserves_x_register() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);
    cpu.memory_mut().write(0x01FD, 0xFF);
    cpu.memory_mut().write(0x01FC, 0x00);
    cpu.memory_mut().write(0x01FB, 0x00);
    cpu.set_sp(0xFA);
    cpu.set_x(0x33);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x33);
}

#[test]
fn test_rti_preserves_y_register() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);
    cpu.memory_mut().write(0x01FD, 0xFF);
    cpu.memory_mut().write(0x01FC, 0x00);
    cpu.memory_mut().write(0x01FB, 0x00);
    cpu.set_sp(0xFA);
    cpu.set_y(0x44);

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x44);
}

#[test]
fn test_rti_preserves_all_registers_except_pc_and_flags() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);
    cpu.memory_mut().write(0x01FD, 0x12); // PC high
    cpu.memory_mut().write(0x01FC, 0x34); // PC low
    cpu.memory_mut().write(0x01FB, 0b11111111); // Status
    cpu.set_sp(0xFA);
    cpu.set_a(0x11);
    cpu.set_x(0x22);
    cpu.set_y(0x33);

    cpu.step().unwrap();

    // A, X, Y should be unchanged
    assert_eq!(cpu.a(), 0x11);
    assert_eq!(cpu.x(), 0x22);
    assert_eq!(cpu.y(), 0x33);
    // PC and SP should be updated
    assert_eq!(cpu.pc(), 0x1234);
    assert_eq!(cpu.sp(), 0xFD);
}

// ========== Cycle Count Tests ==========

#[test]
fn test_rti_cycle_count() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);
    cpu.memory_mut().write(0x01FD, 0x00);
    cpu.memory_mut().write(0x01FC, 0x00);
    cpu.memory_mut().write(0x01FB, 0x00);
    cpu.set_sp(0xFA);

    let initial_cycles = cpu.cycles();
    cpu.step().unwrap();

    // RTI should take exactly 6 cycles
    assert_eq!(cpu.cycles() - initial_cycles, 6);
}

// ========== Addressing Mode Tests ==========

#[test]
fn test_rti_implicit_addressing_mode() {
    let mut cpu = setup_cpu();

    // RTI uses implicit addressing mode (opcode 0x40)
    cpu.memory_mut().write(0x8000, 0x40);
    cpu.memory_mut().write(0x01FD, 0x12); // PC high
    cpu.memory_mut().write(0x01FC, 0x34); // PC low
    cpu.memory_mut().write(0x01FB, 0x00); // Status
    cpu.set_sp(0xFA);

    cpu.step().unwrap();

    // Verify instruction executed correctly
    assert_eq!(cpu.pc(), 0x1234);
    assert_eq!(cpu.cycles(), 6);
    assert_eq!(cpu.sp(), 0xFD);
}

// ========== Flag Combination Tests ==========

#[test]
fn test_rti_various_flag_combinations() {
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

    for &(status_byte, expected_flags) in &test_cases {
        cpu.set_pc(0x8000);
        cpu.memory_mut().write(0x8000, 0x40);
        cpu.memory_mut().write(0x01FD, 0x00); // PC high
        cpu.memory_mut().write(0x01FC, 0x00); // PC low
        cpu.memory_mut().write(0x01FB, status_byte); // Status
        cpu.set_sp(0xFA);

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

// ========== BRK/RTI Round-Trip Tests ==========

#[test]
fn test_brk_rti_round_trip() {
    let mut cpu = setup_cpu();

    // Setup IRQ vector to point to interrupt handler
    cpu.memory_mut().write(0xFFFE, 0x00); // IRQ vector low
    cpu.memory_mut().write(0xFFFF, 0x90); // IRQ vector high -> 0x9000

    // Main code: BRK instruction at 0x8000
    cpu.memory_mut().write(0x8000, 0x00); // BRK

    // Interrupt handler: RTI instruction at 0x9000
    cpu.memory_mut().write(0x9000, 0x40); // RTI

    // Set specific flags before BRK
    cpu.set_flag_c(true);
    cpu.set_flag_z(true);
    cpu.set_flag_n(false);
    cpu.set_flag_v(false);
    cpu.set_flag_d(false);
    cpu.set_flag_i(false); // I will be set by BRK
    cpu.set_flag_b(false);

    let initial_sp = cpu.sp();

    // Execute BRK
    cpu.step().unwrap();

    // After BRK:
    // - PC should be at IRQ vector (0x9000)
    assert_eq!(cpu.pc(), 0x9000);
    // - I flag should be set
    assert!(cpu.flag_i());
    // - SP should be decremented by 3
    assert_eq!(cpu.sp(), initial_sp.wrapping_sub(3));

    // Execute RTI
    cpu.step().unwrap();

    // After RTI:
    // - PC should be restored to 0x8002 (BRK pushes PC+2)
    assert_eq!(cpu.pc(), 0x8002);
    // - Flags should be restored (including I flag clear from before BRK)
    assert!(cpu.flag_c());
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
    assert!(!cpu.flag_v());
    assert!(!cpu.flag_d());
    // Note: B flag will be set because BRK sets it when pushing
    assert!(cpu.flag_b());
    // SP should be restored
    assert_eq!(cpu.sp(), initial_sp);
}

// ========== Edge Case Tests ==========

#[test]
fn test_rti_does_not_modify_other_memory() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);
    cpu.memory_mut().write(0x01FD, 0x00);
    cpu.memory_mut().write(0x01FC, 0x00);
    cpu.memory_mut().write(0x01FB, 0x00);
    cpu.set_sp(0xFA);

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
fn test_rti_does_not_modify_stack_values() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);
    cpu.memory_mut().write(0x01FD, 0xFF);
    cpu.memory_mut().write(0x01FC, 0x34);
    cpu.memory_mut().write(0x01FB, 0x12);
    cpu.set_sp(0xFA);

    cpu.step().unwrap();

    // Stack values should still be present (reads are non-destructive)
    assert_eq!(cpu.memory_mut().read(0x01FD), 0xFF);
    assert_eq!(cpu.memory_mut().read(0x01FC), 0x34);
    assert_eq!(cpu.memory_mut().read(0x01FB), 0x12);
}

#[test]
fn test_rti_with_all_flags_clear() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);
    cpu.memory_mut().write(0x01FD, 0x00); // PC high
    cpu.memory_mut().write(0x01FC, 0x00); // PC low
    cpu.memory_mut().write(0x01FB, 0b00000000); // All clear

    // Set all flags initially
    cpu.set_flag_c(true);
    cpu.set_flag_z(true);
    cpu.set_flag_i(true);
    cpu.set_flag_d(true);
    cpu.set_flag_b(true);
    cpu.set_flag_v(true);
    cpu.set_flag_n(true);

    cpu.set_sp(0xFA);
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
fn test_rti_with_all_flags_set() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x40);
    cpu.memory_mut().write(0x01FD, 0x00); // PC high
    cpu.memory_mut().write(0x01FC, 0x00); // PC low
    cpu.memory_mut().write(0x01FB, 0xFF); // All set

    // Clear all flags initially
    cpu.set_flag_c(false);
    cpu.set_flag_z(false);
    cpu.set_flag_i(false);
    cpu.set_flag_d(false);
    cpu.set_flag_b(false);
    cpu.set_flag_v(false);
    cpu.set_flag_n(false);

    cpu.set_sp(0xFA);
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
