//! Comprehensive tests for the PHA (Push Accumulator) instruction.
//!
//! Tests cover:
//! - Basic PHA operation works correctly
//! - All addressing modes (Implicit only for PHA)
//! - Correct cycle counts
//! - Stack pointer decrementation
//! - Accumulator value correctly pushed to stack
//! - No processor flags are affected
//! - Edge cases (stack wraparound, various accumulator values)

use lib6502::{FlatMemory, MemoryBus, CPU};

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Basic PHA Operation Tests ==========

#[test]
fn test_pha_basic_operation() {
    let mut cpu = setup_cpu();

    // PHA (0x48)
    cpu.memory_mut().write(0x8000, 0x48);
    cpu.set_a(0x42);

    cpu.step().unwrap();

    // Stack should contain the accumulator value at 0x01FD (initial SP = 0xFD)
    assert_eq!(cpu.memory_mut().read(0x01FD), 0x42);
    assert_eq!(cpu.sp(), 0xFC); // SP decremented from 0xFD to 0xFC
    assert_eq!(cpu.pc(), 0x8001); // PC advanced by 1 byte
    assert_eq!(cpu.cycles(), 3); // 3 cycles
}

#[test]
fn test_pha_pushes_accumulator_value() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x48);
    cpu.set_a(0xAB);

    cpu.step().unwrap();

    // Verify the pushed value
    assert_eq!(cpu.memory_mut().read(0x01FD), 0xAB);
}

#[test]
fn test_pha_decrements_stack_pointer() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x48);
    cpu.set_a(0x42);

    let initial_sp = cpu.sp(); // Should be 0xFD
    cpu.step().unwrap();

    // Stack pointer should be decremented by 1
    assert_eq!(cpu.sp(), initial_sp.wrapping_sub(1));
}

// ========== Stack Pointer Tests ==========

#[test]
fn test_pha_stack_pointer_wraparound() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x48);
    cpu.set_a(0x99);
    cpu.set_sp(0x00); // Set SP to 0

    cpu.step().unwrap();

    // Stack pointer should wrap from 0x00 to 0xFF
    assert_eq!(cpu.sp(), 0xFF);
    // Value should be written at 0x0100
    assert_eq!(cpu.memory_mut().read(0x0100), 0x99);
}

#[test]
fn test_pha_multiple_pushes() {
    let mut cpu = setup_cpu();

    // Set up three PHA instructions
    cpu.memory_mut().write(0x8000, 0x48);
    cpu.memory_mut().write(0x8001, 0x48);
    cpu.memory_mut().write(0x8002, 0x48);

    // First push
    cpu.set_a(0x11);
    cpu.step().unwrap();
    assert_eq!(cpu.memory_mut().read(0x01FD), 0x11);
    assert_eq!(cpu.sp(), 0xFC);

    // Second push
    cpu.set_a(0x22);
    cpu.step().unwrap();
    assert_eq!(cpu.memory_mut().read(0x01FC), 0x22);
    assert_eq!(cpu.sp(), 0xFB);

    // Third push
    cpu.set_a(0x33);
    cpu.step().unwrap();
    assert_eq!(cpu.memory_mut().read(0x01FB), 0x33);
    assert_eq!(cpu.sp(), 0xFA);
}

// ========== Accumulator Value Tests ==========

#[test]
fn test_pha_zero_value() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x48);
    cpu.set_a(0x00);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x01FD), 0x00);
}

#[test]
fn test_pha_max_value() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x48);
    cpu.set_a(0xFF);

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x01FD), 0xFF);
}

#[test]
fn test_pha_negative_value() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x48);
    cpu.set_a(0x80); // 0b10000000 (negative in signed interpretation)

    cpu.step().unwrap();

    assert_eq!(cpu.memory_mut().read(0x01FD), 0x80);
}

#[test]
fn test_pha_various_values() {
    let mut cpu = setup_cpu();

    let test_values = [0x00, 0x01, 0x7F, 0x80, 0xFE, 0xFF];

    for (i, &_value) in test_values.iter().enumerate() {
        cpu.memory_mut().write(0x8000 + i as u16, 0x48);
    }

    cpu.set_pc(0x8000);

    for &value in &test_values {
        cpu.set_a(value);
        let sp_before = cpu.sp();
        cpu.step().unwrap();

        // Verify value was pushed correctly
        let stack_addr = 0x0100 | (sp_before as u16);
        assert_eq!(cpu.memory_mut().read(stack_addr), value);
    }
}

// ========== Flag Preservation Tests ==========

#[test]
fn test_pha_preserves_all_flags_when_clear() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x48);
    cpu.set_a(0x42);

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

#[test]
fn test_pha_preserves_all_flags_when_set() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x48);
    cpu.set_a(0x42);

    // Set all flags
    cpu.set_flag_c(true);
    cpu.set_flag_z(true);
    cpu.set_flag_i(true);
    cpu.set_flag_d(true);
    cpu.set_flag_b(true);
    cpu.set_flag_v(true);
    cpu.set_flag_n(true);

    cpu.step().unwrap();

    // All flags should remain unchanged
    assert!(cpu.flag_c());
    assert!(cpu.flag_z());
    assert!(cpu.flag_i());
    assert!(cpu.flag_d());
    assert!(cpu.flag_b());
    assert!(cpu.flag_v());
    assert!(cpu.flag_n());
}

#[test]
fn test_pha_preserves_carry_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x48);
    cpu.set_a(0x42);
    cpu.set_flag_c(true);

    cpu.step().unwrap();

    assert!(cpu.flag_c());
}

#[test]
fn test_pha_preserves_zero_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x48);
    cpu.set_a(0x00);
    cpu.set_flag_z(true);

    cpu.step().unwrap();

    assert!(cpu.flag_z());
}

#[test]
fn test_pha_preserves_interrupt_disable_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x48);
    cpu.set_a(0x42);
    cpu.set_flag_i(true);

    cpu.step().unwrap();

    assert!(cpu.flag_i());
}

#[test]
fn test_pha_preserves_decimal_mode_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x48);
    cpu.set_a(0x42);
    cpu.set_flag_d(true);

    cpu.step().unwrap();

    assert!(cpu.flag_d());
}

#[test]
fn test_pha_preserves_break_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x48);
    cpu.set_a(0x42);
    cpu.set_flag_b(true);

    cpu.step().unwrap();

    assert!(cpu.flag_b());
}

#[test]
fn test_pha_preserves_overflow_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x48);
    cpu.set_a(0x42);
    cpu.set_flag_v(true);

    cpu.step().unwrap();

    assert!(cpu.flag_v());
}

#[test]
fn test_pha_preserves_negative_flag() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x48);
    cpu.set_a(0x80);
    cpu.set_flag_n(true);

    cpu.step().unwrap();

    assert!(cpu.flag_n());
}

// ========== Register Preservation Tests ==========

#[test]
fn test_pha_preserves_accumulator() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x48);
    cpu.set_a(0x42);

    cpu.step().unwrap();

    // Accumulator should remain unchanged
    assert_eq!(cpu.a(), 0x42);
}

#[test]
fn test_pha_preserves_x_register() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x48);
    cpu.set_a(0x42);
    cpu.set_x(0x33);

    cpu.step().unwrap();

    assert_eq!(cpu.x(), 0x33);
}

#[test]
fn test_pha_preserves_y_register() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x48);
    cpu.set_a(0x42);
    cpu.set_y(0x44);

    cpu.step().unwrap();

    assert_eq!(cpu.y(), 0x44);
}

#[test]
fn test_pha_preserves_all_registers_except_sp() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x48);
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
fn test_pha_cycle_count() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x48);
    cpu.set_a(0x42);

    let initial_cycles = cpu.cycles();
    cpu.step().unwrap();

    // PHA should take exactly 3 cycles
    assert_eq!(cpu.cycles() - initial_cycles, 3);
}

#[test]
fn test_pha_multiple_cycle_count() {
    let mut cpu = setup_cpu();

    // Set up 5 PHA instructions
    for i in 0..5 {
        cpu.memory_mut().write(0x8000 + i, 0x48);
    }

    cpu.set_a(0x42);

    for i in 1..=5 {
        cpu.step().unwrap();
        assert_eq!(cpu.cycles(), (i * 3) as u64); // Each PHA takes 3 cycles
    }
}

// ========== PC Advancement Tests ==========

#[test]
fn test_pha_advances_pc_correctly() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x48);
    cpu.set_a(0x42);

    cpu.step().unwrap();

    // PC should advance by 1 byte (size of PHA instruction)
    assert_eq!(cpu.pc(), 0x8001);
}

#[test]
fn test_pha_pc_advancement_at_page_boundary() {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0xFF);
    memory.write(0xFFFD, 0x80);
    memory.write(0x80FF, 0x48); // PHA at page boundary

    let mut cpu = CPU::new(memory);
    cpu.set_a(0x42);

    cpu.step().unwrap();

    // PC should advance to next page
    assert_eq!(cpu.pc(), 0x8100);
}

// ========== Addressing Mode Tests ==========

#[test]
fn test_pha_implicit_addressing_mode() {
    let mut cpu = setup_cpu();

    // PHA uses implicit addressing mode (opcode 0x48)
    cpu.memory_mut().write(0x8000, 0x48);
    cpu.set_a(0x42);

    cpu.step().unwrap();

    // Verify instruction executed correctly
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.cycles(), 3);
    assert_eq!(cpu.memory_mut().read(0x01FD), 0x42);
}

// ========== Sequential Execution Tests ==========

#[test]
fn test_pha_with_other_instructions() {
    let mut cpu = setup_cpu();

    // Write PHA followed by other instructions
    cpu.memory_mut().write(0x8000, 0x48); // PHA
    cpu.memory_mut().write(0x8001, 0xEA); // NOP
    cpu.memory_mut().write(0x8002, 0x48); // PHA

    // First PHA
    cpu.set_a(0x11);
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x8001);
    assert_eq!(cpu.memory_mut().read(0x01FD), 0x11);
    assert_eq!(cpu.sp(), 0xFC);

    // NOP
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x8002);

    // Second PHA
    cpu.set_a(0x22);
    cpu.step().unwrap();
    assert_eq!(cpu.pc(), 0x8003);
    assert_eq!(cpu.memory_mut().read(0x01FC), 0x22);
    assert_eq!(cpu.sp(), 0xFB);
}

// ========== Stack Overflow Scenario Tests ==========

#[test]
fn test_pha_stack_full_scenario() {
    let mut cpu = setup_cpu();

    // Fill the entire stack (256 values)
    for i in 0..=255 {
        cpu.memory_mut().write(0x8000 + i, 0x48);
    }

    cpu.set_a(0xAA);
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
fn test_pha_does_not_modify_other_memory() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x48);
    cpu.set_a(0x42);

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
fn test_pha_overwrites_existing_stack_data() {
    let mut cpu = setup_cpu();

    cpu.memory_mut().write(0x8000, 0x48);
    cpu.set_a(0x42);

    // Pre-populate stack with old data
    cpu.memory_mut().write(0x01FD, 0xFF);

    cpu.step().unwrap();

    // Old data should be overwritten
    assert_eq!(cpu.memory_mut().read(0x01FD), 0x42);
}
