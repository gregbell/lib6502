//! Property-based tests for addressing mode calculations.
//!
//! These tests verify that all 13 addressing modes correctly calculate
//! effective addresses and handle edge cases like zero-page wraparound
//! and page boundary crossing.

use lib6502::{FlatMemory, MemoryBus, CPU};
use proptest::prelude::*;

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

// ========== Zero Page Addressing Tests ==========

proptest! {
    /// Property: Zero page addressing reads from address 0x00XX
    #[test]
    fn prop_zero_page_address_calculation(zp_addr in 0u8..=255u8, value in 0u8..=255u8) {
        let mut cpu = setup_cpu();

        // Store value at zero page address
        cpu.memory_mut().write(zp_addr as u16, value);

        // LDA $zp_addr (0xA5)
        cpu.memory_mut().write(0x8000, 0xA5);
        cpu.memory_mut().write(0x8001, zp_addr);

        cpu.step().unwrap();

        prop_assert_eq!(
            cpu.a(),
            value,
            "LDA ${:02X} should load value 0x{:02X}",
            zp_addr,
            value
        );
    }

    /// Property: Zero page,X addressing wraps within zero page (stays in 0x00-0xFF)
    #[test]
    fn prop_zero_page_x_wraps_in_zero_page(
        base in 0u8..=255u8,
        x in 0u8..=255u8,
        value in 0u8..=255u8,
    ) {
        let mut cpu = setup_cpu();
        cpu.set_x(x);

        // Calculate effective address with wraparound
        let effective_addr = base.wrapping_add(x);

        // Store value at effective zero page address
        cpu.memory_mut().write(effective_addr as u16, value);

        // LDA $base,X (0xB5)
        cpu.memory_mut().write(0x8000, 0xB5);
        cpu.memory_mut().write(0x8001, base);

        cpu.step().unwrap();

        prop_assert_eq!(
            cpu.a(),
            value,
            "LDA ${:02X},X with X={:02X} should read from ${:04X} and get 0x{:02X}",
            base,
            x,
            effective_addr as u16,
            value
        );
    }

    /// Property: Zero page,Y addressing wraps within zero page (for LDX)
    #[test]
    fn prop_zero_page_y_wraps_in_zero_page(
        base in 0u8..=255u8,
        y in 0u8..=255u8,
        value in 0u8..=255u8,
    ) {
        let mut cpu = setup_cpu();
        cpu.set_y(y);

        // Calculate effective address with wraparound
        let effective_addr = base.wrapping_add(y);

        // Store value at effective zero page address
        cpu.memory_mut().write(effective_addr as u16, value);

        // LDX $base,Y (0xB6)
        cpu.memory_mut().write(0x8000, 0xB6);
        cpu.memory_mut().write(0x8001, base);

        cpu.step().unwrap();

        prop_assert_eq!(
            cpu.x(),
            value,
            "LDX ${:02X},Y with Y={:02X} should read from ${:04X} and get 0x{:02X}",
            base,
            y,
            effective_addr as u16,
            value
        );
    }
}

// ========== Absolute Addressing Tests ==========

proptest! {
    /// Property: Absolute addressing reads from correct 16-bit address
    #[test]
    fn prop_absolute_address_calculation(
        addr_lo in 0u8..=255u8,
        addr_hi in 0u8..=0x7Fu8, // Avoid vectors at 0xFFxx
        value in 0u8..=255u8,
    ) {
        let mut cpu = setup_cpu();
        let addr = (addr_hi as u16) << 8 | (addr_lo as u16);

        // Store value at address
        cpu.memory_mut().write(addr, value);

        // LDA $addr (0xAD)
        cpu.memory_mut().write(0x8000, 0xAD);
        cpu.memory_mut().write(0x8001, addr_lo);
        cpu.memory_mut().write(0x8002, addr_hi);

        cpu.step().unwrap();

        prop_assert_eq!(
            cpu.a(),
            value,
            "LDA ${:04X} should load value 0x{:02X}",
            addr,
            value
        );
    }
}

// ========== Page Crossing Detection Tests ==========

proptest! {
    /// Property: Absolute,X page crossing adds 1 cycle
    #[test]
    fn prop_absolute_x_page_crossing_detection(
        addr_lo in 0u8..=255u8,
        addr_hi in 0u8..=0x7Eu8, // Avoid vectors
        x in 0u8..=255u8,
        value in 0u8..=255u8,
    ) {
        let mut cpu = setup_cpu();
        cpu.set_x(x);

        let base_addr = (addr_hi as u16) << 8 | (addr_lo as u16);
        let effective_addr = base_addr.wrapping_add(x as u16);
        let page_crossed = (base_addr & 0xFF00) != (effective_addr & 0xFF00);

        // Store value at effective address
        if effective_addr < 0xFF00 {
            cpu.memory_mut().write(effective_addr, value);
        }

        // LDA $addr,X (0xBD) - 4 cycles base, +1 if page crossed
        cpu.memory_mut().write(0x8000, 0xBD);
        cpu.memory_mut().write(0x8001, addr_lo);
        cpu.memory_mut().write(0x8002, addr_hi);

        cpu.step().unwrap();

        let expected_cycles = if page_crossed { 5 } else { 4 };
        prop_assert_eq!(
            cpu.cycles(),
            expected_cycles,
            "LDA ${:04X},X with X={:02X} -> ${:04X}: page_crossed={}, cycles should be {}",
            base_addr,
            x,
            effective_addr,
            page_crossed,
            expected_cycles
        );
    }

    /// Property: Absolute,Y page crossing adds 1 cycle
    #[test]
    fn prop_absolute_y_page_crossing_detection(
        addr_lo in 0u8..=255u8,
        addr_hi in 0u8..=0x7Eu8,
        y in 0u8..=255u8,
        value in 0u8..=255u8,
    ) {
        let mut cpu = setup_cpu();
        cpu.set_y(y);

        let base_addr = (addr_hi as u16) << 8 | (addr_lo as u16);
        let effective_addr = base_addr.wrapping_add(y as u16);
        let page_crossed = (base_addr & 0xFF00) != (effective_addr & 0xFF00);

        // Store value at effective address
        if effective_addr < 0xFF00 {
            cpu.memory_mut().write(effective_addr, value);
        }

        // LDA $addr,Y (0xB9) - 4 cycles base, +1 if page crossed
        cpu.memory_mut().write(0x8000, 0xB9);
        cpu.memory_mut().write(0x8001, addr_lo);
        cpu.memory_mut().write(0x8002, addr_hi);

        cpu.step().unwrap();

        let expected_cycles = if page_crossed { 5 } else { 4 };
        prop_assert_eq!(
            cpu.cycles(),
            expected_cycles,
            "LDA ${:04X},Y with Y={:02X} -> ${:04X}: page_crossed={}, cycles should be {}",
            base_addr,
            y,
            effective_addr,
            page_crossed,
            expected_cycles
        );
    }
}

// ========== Indirect Addressing Tests ==========

proptest! {
    /// Property: Indexed indirect (zp,X) correctly dereferences pointer
    #[test]
    fn prop_indexed_indirect_dereference(
        base in 0u8..=255u8,
        x in 0u8..=255u8,
        target_lo in 0u8..=255u8,
        target_hi in 1u8..=0x7Fu8,  // High byte >= 1 to avoid zero page overlap
        value in 0u8..=255u8,
    ) {
        let mut cpu = setup_cpu();
        cpu.set_x(x);

        // Calculate zero page address (wraps within zero page)
        let zp_addr = base.wrapping_add(x);
        let target_addr = (target_hi as u16) << 8 | (target_lo as u16);

        // Skip if target address would overlap with zero page pointer
        // (target_hi >= 1 ensures this, but double-check)
        if target_addr <= 0xFF {
            return Ok(());
        }

        // Store pointer at zero page
        cpu.memory_mut().write(zp_addr as u16, target_lo);
        cpu.memory_mut().write(zp_addr.wrapping_add(1) as u16, target_hi);

        // Store value at target address (safe now - no overlap with ZP)
        cpu.memory_mut().write(target_addr, value);

        // LDA ($base,X) (0xA1)
        cpu.memory_mut().write(0x8000, 0xA1);
        cpu.memory_mut().write(0x8001, base);

        cpu.step().unwrap();

        prop_assert_eq!(
            cpu.a(),
            value,
            "LDA (${:02X},X) with X={:02X} should read pointer at ${:02X}, target ${:04X}, value 0x{:02X}",
            base,
            x,
            zp_addr,
            target_addr,
            value
        );
    }

    /// Property: Indirect indexed (zp),Y correctly adds Y after dereference
    #[test]
    fn prop_indirect_indexed_dereference(
        zp_addr in 0u8..=254u8, // Avoid wrap in pointer read
        base_lo in 0u8..=255u8,
        base_hi in 1u8..=0x7Eu8,  // High byte >= 1 to avoid zero page overlap
        y in 0u8..=255u8,
        value in 0u8..=255u8,
    ) {
        let mut cpu = setup_cpu();
        cpu.set_y(y);

        let base_addr = (base_hi as u16) << 8 | (base_lo as u16);
        let effective_addr = base_addr.wrapping_add(y as u16);

        // Skip if effective address would overlap with zero page (where pointer is stored)
        if effective_addr <= 0xFF {
            return Ok(());
        }

        // Store pointer at zero page
        cpu.memory_mut().write(zp_addr as u16, base_lo);
        cpu.memory_mut().write((zp_addr + 1) as u16, base_hi);

        // Store value at effective address (safe now - no overlap with ZP)
        if effective_addr < 0xFF00 {
            cpu.memory_mut().write(effective_addr, value);
        }

        // LDA ($zp),Y (0xB1)
        cpu.memory_mut().write(0x8000, 0xB1);
        cpu.memory_mut().write(0x8001, zp_addr);

        cpu.step().unwrap();

        if effective_addr < 0xFF00 {
            prop_assert_eq!(
                cpu.a(),
                value,
                "LDA (${:02X}),Y with Y={:02X} should read base ${:04X}, effective ${:04X}, value 0x{:02X}",
                zp_addr,
                y,
                base_addr,
                effective_addr,
                value
            );
        }
    }

    /// Property: Indirect indexed (zp),Y page crossing adds 1 cycle
    #[test]
    fn prop_indirect_indexed_page_crossing(
        zp_addr in 0u8..=254u8,
        base_lo in 0u8..=255u8,
        base_hi in 0u8..=0x7Eu8,
        y in 0u8..=255u8,
    ) {
        let mut cpu = setup_cpu();
        cpu.set_y(y);

        let base_addr = (base_hi as u16) << 8 | (base_lo as u16);
        let effective_addr = base_addr.wrapping_add(y as u16);
        let page_crossed = (base_addr & 0xFF00) != (effective_addr & 0xFF00);

        // Store pointer at zero page
        cpu.memory_mut().write(zp_addr as u16, base_lo);
        cpu.memory_mut().write((zp_addr + 1) as u16, base_hi);

        // Store some value at effective address
        if effective_addr < 0xFF00 {
            cpu.memory_mut().write(effective_addr, 0x42);
        }

        // LDA ($zp),Y (0xB1) - 5 cycles base, +1 if page crossed
        cpu.memory_mut().write(0x8000, 0xB1);
        cpu.memory_mut().write(0x8001, zp_addr);

        cpu.step().unwrap();

        let expected_cycles = if page_crossed { 6 } else { 5 };
        prop_assert_eq!(
            cpu.cycles(),
            expected_cycles,
            "LDA (${:02X}),Y with Y={:02X}: base=${:04X}, eff=${:04X}, page_crossed={}, cycles={}",
            zp_addr,
            y,
            base_addr,
            effective_addr,
            page_crossed,
            expected_cycles
        );
    }
}

// ========== Indirect JMP Bug Tests ==========

proptest! {
    /// Property: JMP ($xxFF) exhibits the page boundary bug (wraps within page)
    #[test]
    fn prop_jmp_indirect_page_boundary_bug(
        page in 0x10u8..=0x7Fu8, // Use mid-range pages to avoid code area
        target_lo in 0u8..=255u8,
        target_hi in 0u8..=255u8,
    ) {
        let mut cpu = setup_cpu();

        // Setup indirect pointer at page boundary ($xxFF)
        let pointer_addr = (page as u16) << 8 | 0xFF;

        // On real 6502, JMP ($xxFF) reads low byte from $xxFF and high byte from $xx00
        // (it wraps within the page, not to the next page)
        cpu.memory_mut().write(pointer_addr, target_lo);
        cpu.memory_mut().write((page as u16) << 8, target_hi); // Wraps to $xx00

        // Also set up what would be at $xx00+1 (which is NOT used due to bug)
        cpu.memory_mut().write(((page as u16) << 8) + 0x100, 0xFF);

        // JMP ($xxFF) (0x6C)
        cpu.memory_mut().write(0x8000, 0x6C);
        cpu.memory_mut().write(0x8001, 0xFF);
        cpu.memory_mut().write(0x8002, page);

        cpu.step().unwrap();

        let expected_target = (target_hi as u16) << 8 | (target_lo as u16);
        prop_assert_eq!(
            cpu.pc(),
            expected_target,
            "JMP (${:02X}FF) should jump to ${:04X} (bug: reads hi byte from ${:02X}00)",
            page,
            expected_target,
            page
        );
    }

    /// Property: JMP ($xxYY) where YY != 0xFF reads consecutive bytes correctly
    #[test]
    fn prop_jmp_indirect_normal(
        addr_lo in 0u8..=254u8, // Not 0xFF
        addr_hi in 0x10u8..=0x7Fu8,
        target_lo in 0u8..=255u8,
        target_hi in 0u8..=255u8,
    ) {
        let mut cpu = setup_cpu();

        let pointer_addr = (addr_hi as u16) << 8 | (addr_lo as u16);

        // Normal case: reads two consecutive bytes
        cpu.memory_mut().write(pointer_addr, target_lo);
        cpu.memory_mut().write(pointer_addr + 1, target_hi);

        // JMP ($addr) (0x6C)
        cpu.memory_mut().write(0x8000, 0x6C);
        cpu.memory_mut().write(0x8001, addr_lo);
        cpu.memory_mut().write(0x8002, addr_hi);

        cpu.step().unwrap();

        let expected_target = (target_hi as u16) << 8 | (target_lo as u16);
        prop_assert_eq!(
            cpu.pc(),
            expected_target,
            "JMP (${:04X}) should jump to ${:04X}",
            pointer_addr,
            expected_target
        );
    }
}

// ========== Branch Addressing Tests ==========

proptest! {
    /// Property: Forward branch calculates correct target
    #[test]
    fn prop_branch_forward(offset in 1i8..=127i8) {
        let mut cpu = setup_cpu();
        cpu.set_flag_z(true); // Set Z so BEQ will branch

        // BEQ offset (0xF0)
        cpu.memory_mut().write(0x8000, 0xF0);
        cpu.memory_mut().write(0x8001, offset as u8);

        // Put NOP at target to prevent issues
        let target = 0x8002u16.wrapping_add(offset as u16);
        cpu.memory_mut().write(target, 0xEA);

        cpu.step().unwrap();

        prop_assert_eq!(
            cpu.pc(),
            target,
            "BEQ with offset {} should branch to ${:04X}",
            offset,
            target
        );
    }

    /// Property: Backward branch calculates correct target
    #[test]
    fn prop_branch_backward(offset in -128i8..=-3i8) {
        // Exclude -1 and -2 because target would overlap with branch instruction
        let mut cpu = setup_cpu();
        cpu.set_flag_z(true); // Set Z so BEQ will branch

        // BEQ offset (0xF0)
        cpu.memory_mut().write(0x8000, 0xF0);
        cpu.memory_mut().write(0x8001, offset as u8);

        let target = 0x8002u16.wrapping_add(offset as i16 as u16);

        // Put NOP at target (safe since target won't overlap with instruction)
        cpu.memory_mut().write(target, 0xEA);

        cpu.step().unwrap();

        prop_assert_eq!(
            cpu.pc(),
            target,
            "BEQ with offset {} should branch to ${:04X}",
            offset,
            target
        );
    }

    /// Property: Branch page crossing adds 2 cycles (1 for taken + 1 for page)
    #[test]
    fn prop_branch_page_crossing_cycles(offset in 1u8..=127u8) {
        // Start at address that will cause page crossing with forward branch
        let mut cpu = setup_cpu();

        // Position PC near page boundary
        let start_pc = 0x80F0u16;
        cpu.set_pc(start_pc);
        cpu.set_flag_z(true); // BEQ will branch

        // BEQ with large forward offset
        cpu.memory_mut().write(start_pc, 0xF0);
        cpu.memory_mut().write(start_pc + 1, offset);

        let target = (start_pc + 2).wrapping_add(offset as u16);
        let page_crossed = (start_pc & 0xFF00) != (target & 0xFF00);

        // Put NOP at target
        cpu.memory_mut().write(target, 0xEA);

        cpu.step().unwrap();

        // BEQ: 2 base + 1 taken + 1 if page crossed
        let expected_cycles = if page_crossed { 4 } else { 3 };
        prop_assert_eq!(
            cpu.cycles(),
            expected_cycles,
            "BEQ from ${:04X} to ${:04X}: page_crossed={}, expected {} cycles",
            start_pc,
            target,
            page_crossed,
            expected_cycles
        );
    }

    /// Property: Branch not taken is always 2 cycles
    #[test]
    fn prop_branch_not_taken_cycles(offset in 0u8..=255u8) {
        let mut cpu = setup_cpu();
        cpu.set_flag_z(false); // BEQ will NOT branch

        // BEQ offset (0xF0)
        cpu.memory_mut().write(0x8000, 0xF0);
        cpu.memory_mut().write(0x8001, offset);

        cpu.step().unwrap();

        prop_assert_eq!(
            cpu.cycles(),
            2,
            "BEQ not taken should always be 2 cycles"
        );
        prop_assert_eq!(
            cpu.pc(),
            0x8002,
            "BEQ not taken should advance PC by 2"
        );
    }
}

// ========== Store Addressing Tests ==========

proptest! {
    /// Property: STA zero page writes to correct address
    #[test]
    fn prop_sta_zero_page_writes_correctly(zp_addr in 0u8..=255u8, value in 0u8..=255u8) {
        let mut cpu = setup_cpu();
        cpu.set_a(value);

        // STA $zp (0x85)
        cpu.memory_mut().write(0x8000, 0x85);
        cpu.memory_mut().write(0x8001, zp_addr);

        cpu.step().unwrap();

        let stored = cpu.memory_mut().read(zp_addr as u16);
        prop_assert_eq!(
            stored,
            value,
            "STA ${:02X} should store 0x{:02X}",
            zp_addr,
            value
        );
    }

    /// Property: STA absolute writes to correct address
    #[test]
    fn prop_sta_absolute_writes_correctly(
        addr_lo in 0u8..=255u8,
        addr_hi in 0u8..=0x7Fu8,
        value in 0u8..=255u8,
    ) {
        let mut cpu = setup_cpu();
        cpu.set_a(value);
        let addr = (addr_hi as u16) << 8 | (addr_lo as u16);

        // STA $addr (0x8D)
        cpu.memory_mut().write(0x8000, 0x8D);
        cpu.memory_mut().write(0x8001, addr_lo);
        cpu.memory_mut().write(0x8002, addr_hi);

        cpu.step().unwrap();

        let stored = cpu.memory_mut().read(addr);
        prop_assert_eq!(
            stored,
            value,
            "STA ${:04X} should store 0x{:02X}",
            addr,
            value
        );
    }

    /// Property: STA indexed (absolute,X) writes to correct address
    #[test]
    fn prop_sta_absolute_x_writes_correctly(
        addr_lo in 0u8..=255u8,
        addr_hi in 0u8..=0x7Eu8,
        x in 0u8..=255u8,
        value in 0u8..=255u8,
    ) {
        let mut cpu = setup_cpu();
        cpu.set_a(value);
        cpu.set_x(x);

        let base_addr = (addr_hi as u16) << 8 | (addr_lo as u16);
        let effective_addr = base_addr.wrapping_add(x as u16);

        // STA $addr,X (0x9D)
        cpu.memory_mut().write(0x8000, 0x9D);
        cpu.memory_mut().write(0x8001, addr_lo);
        cpu.memory_mut().write(0x8002, addr_hi);

        cpu.step().unwrap();

        if effective_addr < 0xFF00 {
            let stored = cpu.memory_mut().read(effective_addr);
            prop_assert_eq!(
                stored,
                value,
                "STA ${:04X},X with X={:02X} should store 0x{:02X} at ${:04X}",
                base_addr,
                x,
                value,
                effective_addr
            );
        }
    }
}

// ========== Read-Modify-Write Addressing Tests ==========

proptest! {
    /// Property: INC zero page modifies correct address
    #[test]
    fn prop_inc_zero_page_modifies_correctly(zp_addr in 1u8..=254u8, initial in 0u8..=255u8) {
        let mut cpu = setup_cpu();
        cpu.memory_mut().write(zp_addr as u16, initial);

        // INC $zp (0xE6)
        cpu.memory_mut().write(0x8000, 0xE6);
        cpu.memory_mut().write(0x8001, zp_addr);

        cpu.step().unwrap();

        let result = cpu.memory_mut().read(zp_addr as u16);
        let expected = initial.wrapping_add(1);
        prop_assert_eq!(
            result,
            expected,
            "INC ${:02X} with initial 0x{:02X} should give 0x{:02X}",
            zp_addr,
            initial,
            expected
        );
    }

    /// Property: DEC zero page modifies correct address
    #[test]
    fn prop_dec_zero_page_modifies_correctly(zp_addr in 1u8..=254u8, initial in 0u8..=255u8) {
        let mut cpu = setup_cpu();
        cpu.memory_mut().write(zp_addr as u16, initial);

        // DEC $zp (0xC6)
        cpu.memory_mut().write(0x8000, 0xC6);
        cpu.memory_mut().write(0x8001, zp_addr);

        cpu.step().unwrap();

        let result = cpu.memory_mut().read(zp_addr as u16);
        let expected = initial.wrapping_sub(1);
        prop_assert_eq!(
            result,
            expected,
            "DEC ${:02X} with initial 0x{:02X} should give 0x{:02X}",
            zp_addr,
            initial,
            expected
        );
    }

    /// Property: ASL zero page shifts and updates memory correctly
    #[test]
    fn prop_asl_zero_page_modifies_correctly(zp_addr in 1u8..=254u8, initial in 0u8..=255u8) {
        let mut cpu = setup_cpu();
        cpu.memory_mut().write(zp_addr as u16, initial);

        // ASL $zp (0x06)
        cpu.memory_mut().write(0x8000, 0x06);
        cpu.memory_mut().write(0x8001, zp_addr);

        cpu.step().unwrap();

        let result = cpu.memory_mut().read(zp_addr as u16);
        let expected = initial << 1;
        let expected_carry = (initial & 0x80) != 0;

        prop_assert_eq!(result, expected);
        prop_assert_eq!(cpu.flag_c(), expected_carry);
    }
}
