//! Comprehensive tests for the PLA (Pull Accumulator) instruction.
//!
//! Tests cover:
//! - Basic PLA operation works correctly
//! - All addressing modes (Implicit only for PLA)
//! - Correct cycle counts
//! - Stack pointer incrementation
//! - Accumulator value correctly pulled from stack
//! - Z flag set correctly (set if A = 0)
//! - N flag set correctly (set if bit 7 of A is set)
//! - Other processor flags are not affected
//! - Edge cases (stack wraparound, various pulled values)

use lib6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic PLA Operation Tests ==========

#[test]
fn test_pla_basic_operation() {
    let mut cpu = setup_cpu();

    // PLA (0x68)
    cpu.memory_mut().write(0x8000, 0x68);

    // Setup: Put value on stack and set SP to point below it
    cpu.memory_mut().write(0x01FD, 0x42);
    cpu.set_sp(0xFC); // SP points below the value

    cpu.step().unwrap();

    // Accumulator should contain the pulled value
    assert_eq!(cpu.a(), 0x42);
    assert_eq!(cpu.sp(), 0xFD); // SP incremented from 0xFC to 0xFD
    assert_eq!(cpu.pc(), 0x8001); // PC advanced by 1 byte
    assert_eq!(cpu.cycles(), 4); // 4 cycles
    assert!(!cpu.flag_z()); // 0x42 is not zero
    assert!(!cpu.flag_n()); // Bit 7 is not set
}

#[test]
fn test_pla_pulls_stack_value() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x68);

    // Setup: Put a specific value on stack
    cpu.memory_mut().write(0x01FD, 0xAB);
    cpu.set_sp(0xFC);

    cpu.step().unwrap();

    // Verify the pulled value
    assert_eq!(cpu.a(), 0xAB);
}

#[test]
fn test_pla_increments_stack_pointer() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x68);

    // Setup stack
    cpu.memory_mut().write(0x01FD, 0x42);
    cpu.set_sp(0xFC);

    let initial_sp = cpu.sp(); // Should be 0xFC
    cpu.step().unwrap();

    // Stack pointer should be incremented by 1
    assert_eq!(cpu.sp(), initial_sp.wrapping_add(1));
}

// ========== Stack Pointer Tests ==========

#[test]
fn test_pla_stack_pointer_wraparound() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x68);

    // Setup: Put value at 0x0100 and set SP to 0xFF
    cpu.memory_mut().write(0x0100, 0x99);
    cpu.set_sp(0xFF); // SP at top

    cpu.step().unwrap();

    // Stack pointer should wrap from 0xFF to 0x00
    assert_eq!(cpu.sp(), 0x00);
    // Value should be pulled from 0x0100
    assert_eq!(cpu.a(), 0x99);
}

#[test]
fn test_pla_multiple_pulls() {
    let mut cpu = setup_cpu();

    // Set up three PLA instructions
    cpu.memory_mut().write(0x8000, 0x68);
    cpu.memory_mut().write(0x8001, 0x68);
    cpu.memory_mut().write(0x8002, 0x68);

    // Setup stack with three values
    cpu.memory_mut().write(0x01FB, 0x33);
    cpu.memory_mut().write(0x01FC, 0x22);
    cpu.memory_mut().write(0x01FD, 0x11);
    cpu.set_sp(0xFA); // Start below all values

    // First pull
    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x33);
    assert_eq!(cpu.sp(), 0xFB);

    // Second pull
    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x22);
    assert_eq!(cpu.sp(), 0xFC);

    // Third pull
    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x11);
    assert_eq!(cpu.sp(), 0xFD);
}

// ========== Zero Flag Tests ==========

#[test]
fn test_pla_sets_zero_flag_when_value_is_zero() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x68);

    // Setup: Put zero on stack
    cpu.memory_mut().write(0x01FD, 0x00);
    cpu.set_sp(0xFC);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00);
    assert!(cpu.flag_z()); // Z flag should be set
}

#[test]
fn test_pla_clears_zero_flag_when_value_is_nonzero() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x68);

    // Setup: Put non-zero on stack and set Z flag
    cpu.memory_mut().write(0x01FD, 0x42);
    cpu.set_sp(0xFC);
    cpu.set_flag_z(true); // Z flag initially set

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x42);
    assert!(!cpu.flag_z()); // Z flag should be cleared
}

// ========== Negative Flag Tests ==========

#[test]
fn test_pla_sets_negative_flag_when_bit_7_set() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x68);

    // Setup: Put value with bit 7 set on stack
    cpu.memory_mut().write(0x01FD, 0x80);
    cpu.set_sp(0xFC);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x80);
    assert!(cpu.flag_n()); // N flag should be set
}

#[test]
fn test_pla_clears_negative_flag_when_bit_7_clear() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x68);

    // Setup: Put value with bit 7 clear on stack
    cpu.memory_mut().write(0x01FD, 0x7F);
    cpu.set_sp(0xFC);
    cpu.set_flag_n(true); // N flag initially set

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x7F);
    assert!(!cpu.flag_n()); // N flag should be cleared
}

#[test]
fn test_pla_negative_flag_with_0xff() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x68);

    // Setup: Put 0xFF on stack (bit 7 set)
    cpu.memory_mut().write(0x01FD, 0xFF);
    cpu.set_sp(0xFC);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF);
    assert!(cpu.flag_n()); // N flag should be set
    assert!(!cpu.flag_z()); // Z flag should not be set
}

// ========== Flag Combinations Tests ==========

#[test]
fn test_pla_both_zero_and_negative_flags_clear() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x68);

    // Setup: Put positive non-zero value on stack
    cpu.memory_mut().write(0x01FD, 0x42);
    cpu.set_sp(0xFC);
    cpu.set_flag_z(true);
    cpu.set_flag_n(true);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x42);
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_pla_zero_flag_set_negative_flag_clear() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x68);

    // Setup: Put zero on stack
    cpu.memory_mut().write(0x01FD, 0x00);
    cpu.set_sp(0xFC);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00);
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
}

// ========== Other Flags Preservation Tests ==========

#[test]
fn test_pla_preserves_carry_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x68);
    cpu.memory_mut().write(0x01FD, 0x42);
    cpu.set_sp(0xFC);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert!(cpu.flag_c()); // Carry flag should be preserved
}

#[test]
fn test_pla_preserves_interrupt_disable_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x68);
    cpu.memory_mut().write(0x01FD, 0x42);
    cpu.set_sp(0xFC);
    cpu.set_flag_i(false);

    cpu.step().unwrap();

    assert!(!cpu.flag_i()); // I flag should be preserved
}

#[test]
fn test_pla_preserves_decimal_mode_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x68);
    cpu.memory_mut().write(0x01FD, 0x42);
    cpu.set_sp(0xFC);
    cpu.set_flag_d(true);

    cpu.step().unwrap();

    assert!(cpu.flag_d()); // D flag should be preserved
}

#[test]
fn test_pla_preserves_break_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x68);
    cpu.memory_mut().write(0x01FD, 0x42);
    cpu.set_sp(0xFC);
    cpu.set_flag_b(true);

    cpu.step().unwrap();

    assert!(cpu.flag_b()); // B flag should be preserved
}

#[test]
fn test_pla_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x68);
    cpu.memory_mut().write(0x01FD, 0x42);
    cpu.set_sp(0xFC);
    cpu.set_flag_v(true);

    cpu.step().unwrap();

    assert!(cpu.flag_v()); // V flag should be preserved
}

#[test]
fn test_pla_only_affects_z_and_n_flags() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x68);
    cpu.memory_mut().write(0x01FD, 0x42);
    cpu.set_sp(0xFC);

    // Set all non-Z/N flags
    cpu.set_flag_c(true);
    cpu.set_flag_i(false);
    cpu.set_flag_d(true);
    cpu.set_flag_b(true);
    cpu.set_flag_v(true);

    cpu.step().unwrap();

    // Only Z and N should be updated (both cleared for 0x42)
    assert!(!cpu.flag_z());
    assert!(!cpu.flag_n());
    // Other flags preserved
    assert!(cpu.flag_c());
    assert!(!cpu.flag_i());
    assert!(cpu.flag_d());
    assert!(cpu.flag_b());
    assert!(cpu.flag_v());
}

// ========== Register Preservation Tests ==========

#[test]
fn test_pla_preserves_x_register() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x68);
    cpu.memory_mut().write(0x01FD, 0x42);
    cpu.set_sp(0xFC);
    cpu.set_x(0x33);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x33);
}

#[test]
fn test_pla_preserves_y_register() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x68);
    cpu.memory_mut().write(0x01FD, 0x42);
    cpu.set_sp(0xFC);
    cpu.set_y(0x44);

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x44);
}

// ========== Cycle Count Tests ==========

#[test]
fn test_pla_cycle_count() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x68);
    cpu.memory_mut().write(0x01FD, 0x42);
    cpu.set_sp(0xFC);

    let initial_cycles = cpu.cycles();
    cpu.step().unwrap();

    // PLA should take exactly 4 cycles
    assert_eq!(cpu.cycles() - initial_cycles, 4);
}

#[test]
fn test_pla_multiple_cycle_count() {
    let mut cpu = setup_cpu();

    // Set up 5 PLA instructions
    for i in 0..5 {
        cpu.memory_mut().write(0x8000 + i, 0x68);
    }

    // Setup stack with values
    cpu.memory_mut().write(0x01FB, 0x11);
    cpu.memory_mut().write(0x01FC, 0x22);
    cpu.memory_mut().write(0x01FD, 0x33);
    cpu.memory_mut().write(0x01FE, 0x44);
    cpu.memory_mut().write(0x01FF, 0x55);
    cpu.set_sp(0xFA);

    for i in 1..=5 {
        cpu.step().unwrap();
        assert_eq!(cpu.cycles(), (i * 4) as u64); // Each PLA takes 4 cycles
    }
}

// ========== PC Advancement Tests ==========

#[test]
fn test_pla_advances_pc_correctly() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x68);
    cpu.memory_mut().write(0x01FD, 0x42);
    cpu.set_sp(0xFC);

    cpu.step().unwrap();

    // PC should advance by 1 byte (size of PLA instruction)
    assert_eq!(cpu.pc(), 0x8001);
}

#[test]
fn test_pla_pc_advancement_at_page_boundary() {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0xFF);
    memory.write(0xFFFD, 0x80);
    memory.write(0x80FF, 0x68); // PLA at page boundary
    memory.write(0x01FD, 0x42);

    let mut cpu = CPU::new(memory);
    cpu.set_sp(0xFC);

    cpu.step().unwrap();

    // PC should advance to next page
    assert_eq!(cpu.pc(), 0x8100);
}

// ========== Addressing Mode Tests ==========

#[test]
fn test_pla_implicit_addressing_mode() {
    let mut cpu = setup_cpu();

    // PLA uses implicit addressing mode (opcode 0x68)
    cpu.memory_mut().write(0x8000, 0x68);
    cpu.memory_mut().write(0x01FD, 0x42);
    cpu.set_sp(0xFC);

    cpu.step().unwrap();

    // Verify instruction executed correctly
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.cycles(), 4);
    assert_eq!(cpu.a(), 0x42);
}

// ========== Value Range Tests ==========

#[test]
fn test_pla_various_values() {
    let mut cpu = setup_cpu();

    let test_values = [0x00, 0x01, 0x7F, 0x80, 0xFE, 0xFF];

    for (i, &_value) in test_values.iter().enumerate() {
        cpu.memory_mut().write(0x8000 + i as u16, 0x68);
    }

    cpu.set_pc(0x8000);
    cpu.set_sp(0xF9); // Start below all test values

    for &value in &test_values {
        // Put value on stack at current SP + 1
        let sp_before = cpu.sp();
        cpu.memory_mut()
            .write(0x0100 | ((sp_before.wrapping_add(1)) as u16), value);

        cpu.step().unwrap();

        // Verify value was pulled correctly
        assert_eq!(cpu.a(), value);

        // Verify flags
        assert_eq!(cpu.flag_z(), value == 0);
        assert_eq!(cpu.flag_n(), (value & 0x80) != 0);
    }
}

#[test]
fn test_pla_zero_value() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x68);
    cpu.memory_mut().write(0x01FD, 0x00);
    cpu.set_sp(0xFC);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0x00);
    assert!(cpu.flag_z());
    assert!(!cpu.flag_n());
}

#[test]
fn test_pla_max_value() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x68);
    cpu.memory_mut().write(0x01FD, 0xFF);
    cpu.set_sp(0xFC);

    cpu.step().unwrap();

    assert_eq!(cpu.a(), 0xFF);
    assert!(!cpu.flag_z());
    assert!(cpu.flag_n());
}

// ========== Push/Pull Round-Trip Tests ==========

#[test]
fn test_pha_pla_round_trip() {
    let mut cpu = setup_cpu();

    // Write PHA followed by PLA
    cpu.memory_mut().write(0x8000, 0x48); // PHA
    cpu.memory_mut().write(0x8001, 0x68); // PLA

    cpu.set_a(0x42);

    // Push
    cpu.step().unwrap();
    assert_eq!(cpu.sp(), 0xFC);
    assert_eq!(cpu.memory_mut().read(0x01FD), 0x42);

    // Clear accumulator to verify pull
    cpu.set_a(0x00);

    // Pull
    cpu.step().unwrap();
    assert_eq!(cpu.a(), 0x42);
    assert_eq!(cpu.sp(), 0xFD); // Back to original SP
}

#[test]
fn test_multiple_pha_pla_round_trips() {
    let mut cpu = setup_cpu();

    // Write sequence of PHA/PHA/PHA/PLA/PLA/PLA
    cpu.memory_mut().write(0x8000, 0x48); // PHA
    cpu.memory_mut().write(0x8001, 0x48); // PHA
    cpu.memory_mut().write(0x8002, 0x48); // PHA
    cpu.memory_mut().write(0x8003, 0x68); // PLA
    cpu.memory_mut().write(0x8004, 0x68); // PLA
    cpu.memory_mut().write(0x8005, 0x68); // PLA

    // Push three different values
    cpu.set_a(0x11);
    cpu.step().unwrap(); // PHA

    cpu.set_a(0x22);
    cpu.step().unwrap(); // PHA

    cpu.set_a(0x33);
    cpu.step().unwrap(); // PHA

    assert_eq!(cpu.sp(), 0xFA); // SP decremented 3 times

    // Pull in reverse order (stack is LIFO)
    cpu.step().unwrap(); // PLA
    assert_eq!(cpu.a(), 0x33); // Last pushed, first pulled

    cpu.step().unwrap(); // PLA
    assert_eq!(cpu.a(), 0x22);

    cpu.step().unwrap(); // PLA
    assert_eq!(cpu.a(), 0x11); // First pushed, last pulled

    assert_eq!(cpu.sp(), 0xFD); // Back to original SP
}

// ========== Edge Case Tests ==========

#[test]
fn test_pla_does_not_modify_other_memory() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x68);
    cpu.memory_mut().write(0x01FD, 0x42);
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
fn test_pla_does_not_modify_stack_value() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x68);
    cpu.memory_mut().write(0x01FD, 0x42);
    cpu.set_sp(0xFC);

    cpu.step().unwrap();

    // Stack value should still be present (reads are non-destructive)
    assert_eq!(cpu.memory_mut().read(0x01FD), 0x42);
}

// ========== Sequential Execution Tests ==========

#[test]
fn test_pla_with_other_instructions() {
    let mut cpu = setup_cpu();

    // Write PLA followed by other instructions
    cpu.memory_mut().write(0x8000, 0x68); // PLA
    cpu.memory_mut().write(0x8001, 0xEA); // NOP
    cpu.memory_mut().write(0x8002, 0x68); // PLA

    // Setup stack
    cpu.memory_mut().write(0x01FD, 0x11);
    cpu.memory_mut().write(0x01FE, 0x22);
    cpu.set_sp(0xFC);

    // First PLA
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.a(), 0x11);
    assert_eq!(cpu.sp(), 0xFD);

    // NOP
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x8002);

    // Second PLA
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.a(), 0x22);
    assert_eq!(cpu.sp(), 0xFE);
}
