//! Comprehensive tests for the RTS (Return from Subroutine) instruction.
//!
//! Tests cover:
//! - Basic RTS operation works correctly
//! - All addressing modes (Implicit only for RTS)
//! - Correct cycle counts (6 cycles)
//! - Stack pointer incrementation (2 times: PC low + PC high)
//! - Program counter correctly restored and incremented
//! - No flags affected
//! - Other registers (A, X, Y) are not affected
//! - Edge cases (stack wraparound, various addresses)
//! - Integration with JSR (round-trip testing)

use lib6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic RTS Operation Tests ==========

#[test]
fn test_rts_basic_operation() {
    let mut cpu = setup_cpu();

    // RTS (0x60)
    cpu.memory_mut().write(0x8000, 0x60);

    // Setup: Put return address on stack (as JSR would)
    // JSR pushes PC+2 (the address of the last byte of JSR)
    // So if JSR was at 0x1000, it pushes 0x1002
    // RTS pulls this and adds 1 to get 0x1003
    cpu.memory_mut().write(0x01FD, 0x10); // PC high byte
    cpu.memory_mut().write(0x01FC, 0x02); // PC low byte -> 0x1002
    cpu.set_sp(0xFB); // SP points below the pushed values

    cpu.step().unwrap();

    // PC should be restored and incremented: 0x1002 + 1 = 0x1003
    assert_eq!(cpu.pc(), 0x1003);

    // SP should be incremented 2 times (PC_low + PC_high)
    assert_eq!(cpu.sp(), 0xFD); // 0xFB + 2 = 0xFD

    // Cycles should be 6
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_rts_to_zero_page() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x60);

    // Return to zero page address 0x0042
    // JSR would have pushed 0x0041, RTS adds 1
    cpu.memory_mut().write(0x01FD, 0x00); // PC high byte
    cpu.memory_mut().write(0x01FC, 0x41); // PC low byte
    cpu.set_sp(0xFB);

    cpu.step().unwrap();

    assert_eq!(cpu.pc(), 0x0042); // 0x0041 + 1
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_rts_to_high_memory() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x60);

    // Return to high memory address 0xFFFC
    cpu.memory_mut().write(0x01FD, 0xFF); // PC high byte
    cpu.memory_mut().write(0x01FC, 0xFB); // PC low byte
    cpu.set_sp(0xFB);

    cpu.step().unwrap();

    assert_eq!(cpu.pc(), 0xFFFC); // 0xFFFB + 1
    assert_eq!(cpu.cycles(), 6);
}

#[test]
fn test_rts_address_wraparound() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x60);

    // Return address that wraps: 0xFFFF + 1 = 0x0000
    cpu.memory_mut().write(0x01FD, 0xFF); // PC high byte
    cpu.memory_mut().write(0x01FC, 0xFF); // PC low byte
    cpu.set_sp(0xFB);

    cpu.step().unwrap();

    assert_eq!(cpu.pc(), 0x0000); // 0xFFFF + 1 wraps to 0x0000
    assert_eq!(cpu.cycles(), 6);
}

// ========== Stack Pointer Tests ==========

#[test]
fn test_rts_increments_stack_pointer_by_two() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x60);

    cpu.memory_mut().write(0x01FD, 0x12); // PC high
    cpu.memory_mut().write(0x01FC, 0x34); // PC low
    cpu.set_sp(0xFB);

    let initial_sp = cpu.sp();
    cpu.step().unwrap();

    // Stack pointer should be incremented by 2 (PC low + PC high)
    assert_eq!(cpu.sp(), initial_sp.wrapping_add(2));
}

#[test]
fn test_rts_stack_pointer_wraparound() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x60);

    // Setup: Put values near top of stack to test wraparound
    // SP=0xFE: increment to 0xFF (pull PC low), increment to 0x00 (pull PC high)
    // So: PC low at 0x01FF, PC high at 0x0100
    cpu.memory_mut().write(0x0100, 0x12); // PC high
    cpu.memory_mut().write(0x01FF, 0x34); // PC low
    cpu.set_sp(0xFE); // SP = 0xFE

    cpu.step().unwrap();

    // Stack pointer should wrap: 0xFE + 2 = 0x00
    assert_eq!(cpu.sp(), 0x00);
    // PC should be restored correctly: 0x1234 + 1 = 0x1235
    assert_eq!(cpu.pc(), 0x1235);
}

#[test]
fn test_rts_stack_pull_order() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x60);

    // Verify RTS pulls low byte first, then high byte
    cpu.memory_mut().write(0x01FD, 0xAB); // PC high byte
    cpu.memory_mut().write(0x01FC, 0xCD); // PC low byte
    cpu.set_sp(0xFB);

    cpu.step().unwrap();

    // PC should be 0xABCD + 1 = 0xABCE
    assert_eq!(cpu.pc(), 0xABCE);
    assert_eq!(cpu.sp(), 0xFD);
}

// ========== Flag Preservation Tests ==========

#[test]
fn test_rts_preserves_all_flags() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x60);
    cpu.memory_mut().write(0x01FD, 0x12);
    cpu.memory_mut().write(0x01FC, 0x34);
    cpu.set_sp(0xFB);

    // Set all flags to known values
    cpu.set_flag_c(true);
    cpu.set_flag_z(true);
    cpu.set_flag_i(false);
    cpu.set_flag_d(true);
    cpu.set_flag_b(true);
    cpu.set_flag_v(true);
    cpu.set_flag_n(true);

    cpu.step().unwrap();

    // All flags should remain unchanged
    assert!(cpu.flag_c());
    assert!(cpu.flag_z());
    assert!(!cpu.flag_i());
    assert!(cpu.flag_d());
    assert!(cpu.flag_b());
    assert!(cpu.flag_v());
    assert!(cpu.flag_n());
}

#[test]
fn test_rts_preserves_all_flags_clear() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x60);
    cpu.memory_mut().write(0x01FD, 0x12);
    cpu.memory_mut().write(0x01FC, 0x34);
    cpu.set_sp(0xFB);

    // Clear all flags
    cpu.set_flag_c(false);
    cpu.set_flag_z(false);
    cpu.set_flag_i(false);
    cpu.set_flag_d(false);
    cpu.set_flag_b(false);
    cpu.set_flag_v(false);
    cpu.set_flag_n(false);

    cpu.step().unwrap();

    // All flags should remain unchanged
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
fn test_rts_preserves_accumulator() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x60);
    cpu.memory_mut().write(0x01FD, 0x12);
    cpu.memory_mut().write(0x01FC, 0x34);
    cpu.set_sp(0xFB);

    cpu.set_a(0x42);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x42);
}

#[test]
fn test_rts_preserves_x_register() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x60);
    cpu.memory_mut().write(0x01FD, 0x12);
    cpu.memory_mut().write(0x01FC, 0x34);
    cpu.set_sp(0xFB);

    cpu.set_x(0x55);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x55);
}

#[test]
fn test_rts_preserves_y_register() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x60);
    cpu.memory_mut().write(0x01FD, 0x12);
    cpu.memory_mut().write(0x01FC, 0x34);
    cpu.set_sp(0xFB);

    cpu.set_y(0x66);

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x66);
}

#[test]
fn test_rts_preserves_all_registers_except_pc_and_sp() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x60);
    cpu.memory_mut().write(0x01FD, 0x12); // PC high
    cpu.memory_mut().write(0x01FC, 0x34); // PC low
    cpu.set_sp(0xFB);
    cpu.set_a(0x11);
    cpu.set_x(0x22);
    cpu.set_y(0x33);

    cpu.step().unwrap();

    // A, X, Y should be unchanged
    assert_eq!(cpu.a(), 0x11);
    assert_eq!(cpu.x(), 0x22);
    assert_eq!(cpu.y(), 0x33);
    // PC and SP should be updated
    assert_eq!(cpu.pc(), 0x1235); // 0x1234 + 1
    assert_eq!(cpu.sp(), 0xFD); // 0xFB + 2
}

// ========== Cycle Count Tests ==========

#[test]
fn test_rts_cycle_count() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x60);
    cpu.memory_mut().write(0x01FD, 0x12);
    cpu.memory_mut().write(0x01FC, 0x34);
    cpu.set_sp(0xFB);

    let initial_cycles = cpu.cycles();
    cpu.step().unwrap();

    // RTS should take exactly 6 cycles
    assert_eq!(cpu.cycles() - initial_cycles, 6);
}

// ========== Addressing Mode Tests ==========

#[test]
fn test_rts_implicit_addressing_mode() {
    let mut cpu = setup_cpu();

    // RTS uses implicit addressing mode (opcode 0x60)
    cpu.memory_mut().write(0x8000, 0x60);
    cpu.memory_mut().write(0x01FD, 0x12); // PC high
    cpu.memory_mut().write(0x01FC, 0x34); // PC low
    cpu.set_sp(0xFB);

    cpu.step().unwrap();

    // Verify instruction executed correctly
    assert_eq!(cpu.pc(), 0x1235);
    assert_eq!(cpu.cycles(), 6);
    assert_eq!(cpu.sp(), 0xFD);
}

// ========== JSR/RTS Round-Trip Tests ==========

#[test]
fn test_jsr_rts_round_trip() {
    let mut cpu = setup_cpu();

    // Main code: JSR $9000 at $8000
    cpu.memory_mut().write(0x8000, 0x20); // JSR
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x90);

    // Next instruction after JSR at $8003
    cpu.memory_mut().write(0x8003, 0xEA); // NOP (for verification)

    // Subroutine: RTS at $9000
    cpu.memory_mut().write(0x9000, 0x60); // RTS

    let initial_sp = cpu.sp();

    // Execute JSR
    cpu.step().unwrap();

    // After JSR:
    // - PC should be at $9000
    assert_eq!(cpu.pc(), 0x9000);
    // - SP should be decremented by 2
    assert_eq!(cpu.sp(), initial_sp.wrapping_sub(2));
    // - Cycles should be 6
    assert_eq!(cpu.cycles(), 6);

    // Execute RTS
    cpu.step().unwrap();

    // After RTS:
    // - PC should return to $8003 (next instruction after JSR)
    assert_eq!(cpu.pc(), 0x8003);
    // - SP should be restored
    assert_eq!(cpu.sp(), initial_sp);
    // - Total cycles should be 12 (6 + 6)
    assert_eq!(cpu.cycles(), 12);
}

#[test]
fn test_nested_jsr_rts() {
    let mut cpu = setup_cpu();

    // Main: JSR $1000 at $8000
    cpu.memory_mut().write(0x8000, 0x20); // JSR
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x10);

    // Sub1: JSR $2000 at $1000
    cpu.memory_mut().write(0x1000, 0x20); // JSR
    cpu.memory_mut().write(0x1001, 0x00);
    cpu.memory_mut().write(0x1002, 0x20);

    // Sub2: RTS at $2000
    cpu.memory_mut().write(0x2000, 0x60); // RTS

    // Return point in Sub1: RTS at $1003
    cpu.memory_mut().write(0x1003, 0x60); // RTS

    let initial_sp = cpu.sp();

    // Execute first JSR (main -> sub1)
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x1000);
    assert_eq!(cpu.sp(), initial_sp.wrapping_sub(2));

    // Execute second JSR (sub1 -> sub2)
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x2000);
    assert_eq!(cpu.sp(), initial_sp.wrapping_sub(4));

    // Execute first RTS (sub2 -> sub1)
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x1003);
    assert_eq!(cpu.sp(), initial_sp.wrapping_sub(2));

    // Execute second RTS (sub1 -> main)
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.sp(), initial_sp);
}

#[test]
fn test_jsr_rts_preserves_flags() {
    let mut cpu = setup_cpu();

    // Setup JSR/RTS sequence
    cpu.memory_mut().write(0x8000, 0x20); // JSR $9000
    cpu.memory_mut().write(0x8001, 0x00);
    cpu.memory_mut().write(0x8002, 0x90);
    cpu.memory_mut().write(0x9000, 0x60); // RTS

    // Set specific flags
    cpu.set_flag_c(true);
    cpu.set_flag_z(false);
    cpu.set_flag_n(true);
    cpu.set_flag_v(false);

    // Execute JSR
    cpu.step().unwrap();

    // Flags should be preserved by JSR
    assert!(cpu.flag_c());
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
    assert!(!cpu.flag_v());

    // Execute RTS
    cpu.step().unwrap();

    // Flags should still be preserved by RTS
    assert!(cpu.flag_c());
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
    assert!(!cpu.flag_v());
}

// ========== Edge Cases ==========

#[test]
fn test_rts_with_various_addresses() {
    let mut cpu = setup_cpu();

    let test_cases = vec![
        0x0000u16, 0x0001, 0x00FF, 0x0100, 0x1234, 0x8000, 0xABCD, 0xFFFE, 0xFFFF,
    ];

    for &return_addr in &test_cases {
        cpu.set_pc(0x8000);
        cpu.memory_mut().write(0x8000, 0x60); // RTS

        // Setup stack with return_addr - 1 (as JSR would push)
        let jsr_pushed_addr = return_addr.wrapping_sub(1);
        cpu.memory_mut().write(0x01FD, (jsr_pushed_addr >> 8) as u8); // PC high
        cpu.memory_mut()
            .write(0x01FC, (jsr_pushed_addr & 0xFF) as u8); // PC low
        cpu.set_sp(0xFB);

        cpu.step().unwrap();

        assert_eq!(
            cpu.pc(),
            return_addr,
            "Failed to return to address 0x{:04X}",
            return_addr
        );
        assert_eq!(cpu.sp(), 0xFD);
    }
}

#[test]
fn test_rts_does_not_modify_stack_values() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x60);
    cpu.memory_mut().write(0x01FD, 0x12); // PC high
    cpu.memory_mut().write(0x01FC, 0x34); // PC low
    cpu.set_sp(0xFB);

    cpu.step().unwrap();

    // Stack values should still be present (reads are non-destructive)
    assert_eq!(cpu.memory_mut().read(0x01FD), 0x12);
    assert_eq!(cpu.memory_mut().read(0x01FC), 0x34);
}

#[test]
fn test_rts_does_not_modify_other_memory() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x60);
    cpu.memory_mut().write(0x01FD, 0x00);
    cpu.memory_mut().write(0x01FC, 0x00);
    cpu.set_sp(0xFB);

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
fn test_rts_at_different_stack_positions() {
    let mut cpu = setup_cpu();

    // Test RTS with various initial SP values
    let sp_values = vec![0x00, 0x01, 0x7F, 0x80, 0xFD, 0xFE, 0xFF];

    for &initial_sp in &sp_values {
        cpu.set_pc(0x8000);
        cpu.memory_mut().write(0x8000, 0x60);

        // Setup return address on stack
        let sp_after_jsr: u8 = initial_sp;
        let pc_low_addr = 0x0100 | (sp_after_jsr.wrapping_add(1) as u16);
        let pc_high_addr = 0x0100 | (sp_after_jsr.wrapping_add(2) as u16);

        cpu.memory_mut().write(pc_low_addr, 0x34);
        cpu.memory_mut().write(pc_high_addr, 0x12);
        cpu.set_sp(sp_after_jsr);

        cpu.step().unwrap();

        assert_eq!(
            cpu.pc(),
            0x1235,
            "Failed with initial SP = 0x{:02X}",
            initial_sp
        );
        assert_eq!(
            cpu.sp(),
            sp_after_jsr.wrapping_add(2),
            "SP not incremented correctly with initial SP = 0x{:02X}",
            initial_sp
        );
    }
}

#[test]
fn test_multiple_rts_in_sequence() {
    let mut cpu = setup_cpu();

    // Setup three RTS instructions in sequence
    cpu.memory_mut().write(0x8000, 0x60); // RTS -> 0x1001
    cpu.memory_mut().write(0x1001, 0x60); // RTS -> 0x2001
    cpu.memory_mut().write(0x2001, 0x60); // RTS -> 0x3001

    // Setup stack for first RTS
    cpu.memory_mut().write(0x01FD, 0x10); // Return to 0x1001
    cpu.memory_mut().write(0x01FC, 0x00);

    // Setup stack for second RTS
    cpu.memory_mut().write(0x01FF, 0x20); // Return to 0x2001
    cpu.memory_mut().write(0x01FE, 0x00);

    // Setup stack for third RTS
    cpu.memory_mut().write(0x0101, 0x30); // Return to 0x3001
    cpu.memory_mut().write(0x0100, 0x00);

    cpu.set_sp(0xFB);

    // First RTS
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x1001);
    assert_eq!(cpu.sp(), 0xFD);

    // Second RTS
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x2001);
    assert_eq!(cpu.sp(), 0xFF);

    // Third RTS
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x3001);
    assert_eq!(cpu.sp(), 0x01);
}
