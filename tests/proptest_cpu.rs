//! Property-based tests for CPU invariants.
//!
//! These tests use proptest to verify that CPU operations maintain
//! fundamental invariants across all possible input combinations.

use lib6502::{AddressingMode, FlatMemory, MemoryBus, CPU, OPCODE_TABLE};
use proptest::prelude::*;

/// Helper function to create a CPU with reset vector at 0x8000
fn setup_cpu() -> CPU<FlatMemory> {
    let mut memory = FlatMemory::new();
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);
    CPU::new(memory)
}

/// Get all implemented opcodes from the opcode table
fn implemented_opcodes() -> Vec<u8> {
    OPCODE_TABLE
        .iter()
        .enumerate()
        .filter(|(_, m)| m.implemented)
        .map(|(i, _)| i as u8)
        .collect()
}

/// Get opcodes that don't modify PC in special ways (excludes branches, jumps, calls, returns)
fn non_branching_opcodes() -> Vec<u8> {
    OPCODE_TABLE
        .iter()
        .enumerate()
        .filter(|(_, m)| {
            m.implemented
                && !matches!(
                    m.mnemonic,
                    "BCC" | "BCS" | "BEQ" | "BMI" | "BNE" | "BPL" | "BVC" | "BVS" | "JMP" | "JSR"
                        | "RTS" | "RTI" | "BRK"
                )
        })
        .map(|(i, _)| i as u8)
        .collect()
}

/// Get opcodes that affect N and Z flags (load/transfer/ALU operations)
#[allow(dead_code)]
fn nz_affecting_opcodes() -> Vec<u8> {
    OPCODE_TABLE
        .iter()
        .enumerate()
        .filter(|(_, m)| {
            m.implemented
                && matches!(
                    m.mnemonic,
                    "LDA" | "LDX"
                        | "LDY"
                        | "TAX"
                        | "TAY"
                        | "TXA"
                        | "TYA"
                        | "TSX"
                        | "AND"
                        | "ORA"
                        | "EOR"
                        | "ADC"
                        | "SBC"
                        | "INC"
                        | "INX"
                        | "INY"
                        | "DEC"
                        | "DEX"
                        | "DEY"
                        | "ASL"
                        | "LSR"
                        | "ROL"
                        | "ROR"
                        | "CMP"
                        | "CPX"
                        | "CPY"
                        | "PLA"
                )
        })
        .map(|(i, _)| i as u8)
        .collect()
}

// ========== PC Advancement Property Tests ==========

proptest! {
    /// Property: For non-branching instructions, PC advances by exactly size_bytes
    #[test]
    fn prop_pc_advances_by_instruction_size(
        opcode in prop::sample::select(non_branching_opcodes()),
        operand1 in 0u8..=255u8,
        operand2 in 0u8..=255u8,
    ) {
        let mut cpu = setup_cpu();
        let metadata = &OPCODE_TABLE[opcode as usize];
        let expected_size = metadata.size_bytes as u16;

        // Write instruction to memory
        cpu.memory_mut().write(0x8000, opcode);
        cpu.memory_mut().write(0x8001, operand1);
        cpu.memory_mut().write(0x8002, operand2);

        // For some instructions, we need valid memory setup
        // For zero page and absolute addressing, ensure valid addresses
        setup_memory_for_instruction(&mut cpu, opcode, operand1, operand2);

        let old_pc = cpu.pc();
        let _ = cpu.step(); // Ignore errors for now

        let new_pc = cpu.pc();
        prop_assert_eq!(
            new_pc,
            old_pc.wrapping_add(expected_size),
            "PC should advance by {} bytes for opcode 0x{:02X} ({})",
            expected_size,
            opcode,
            metadata.mnemonic
        );
    }

    /// Property: Cycle counter always increases after executing an instruction
    #[test]
    fn prop_cycles_increase(
        opcode in prop::sample::select(implemented_opcodes()),
        operand1 in 0u8..=255u8,
        operand2 in 0u8..=255u8,
    ) {
        let mut cpu = setup_cpu();
        let metadata = &OPCODE_TABLE[opcode as usize];

        // Write instruction to memory
        cpu.memory_mut().write(0x8000, opcode);
        cpu.memory_mut().write(0x8001, operand1);
        cpu.memory_mut().write(0x8002, operand2);

        // Setup memory for instruction
        setup_memory_for_instruction(&mut cpu, opcode, operand1, operand2);

        let old_cycles = cpu.cycles();
        let _ = cpu.step();
        let new_cycles = cpu.cycles();

        prop_assert!(
            new_cycles >= old_cycles + metadata.base_cycles as u64,
            "Cycles should increase by at least {} for opcode 0x{:02X} ({})",
            metadata.base_cycles,
            opcode,
            metadata.mnemonic
        );
    }
}

// ========== Flag N/Z Property Tests ==========

proptest! {
    /// Property: N flag equals bit 7 of result for LDA immediate
    #[test]
    fn prop_lda_immediate_n_flag(value in 0u8..=255u8) {
        let mut cpu = setup_cpu();

        // LDA #value (0xA9)
        cpu.memory_mut().write(0x8000, 0xA9);
        cpu.memory_mut().write(0x8001, value);

        cpu.step().unwrap();

        let expected_n = (value & 0x80) != 0;
        prop_assert_eq!(
            cpu.flag_n(),
            expected_n,
            "N flag should be {} for value 0x{:02X}",
            expected_n,
            value
        );
    }

    /// Property: Z flag is set iff result is zero for LDA immediate
    #[test]
    fn prop_lda_immediate_z_flag(value in 0u8..=255u8) {
        let mut cpu = setup_cpu();

        // LDA #value (0xA9)
        cpu.memory_mut().write(0x8000, 0xA9);
        cpu.memory_mut().write(0x8001, value);

        cpu.step().unwrap();

        let expected_z = value == 0;
        prop_assert_eq!(
            cpu.flag_z(),
            expected_z,
            "Z flag should be {} for value 0x{:02X}",
            expected_z,
            value
        );
    }

    /// Property: N flag equals bit 7 of result for AND immediate
    #[test]
    fn prop_and_immediate_n_flag(a in 0u8..=255u8, operand in 0u8..=255u8) {
        let mut cpu = setup_cpu();
        cpu.set_a(a);

        // AND #operand (0x29)
        cpu.memory_mut().write(0x8000, 0x29);
        cpu.memory_mut().write(0x8001, operand);

        cpu.step().unwrap();

        let result = a & operand;
        let expected_n = (result & 0x80) != 0;
        prop_assert_eq!(
            cpu.flag_n(),
            expected_n,
            "N flag should be {} for A=0x{:02X} AND 0x{:02X} = 0x{:02X}",
            expected_n,
            a,
            operand,
            result
        );
    }

    /// Property: Z flag is set iff result is zero for AND immediate
    #[test]
    fn prop_and_immediate_z_flag(a in 0u8..=255u8, operand in 0u8..=255u8) {
        let mut cpu = setup_cpu();
        cpu.set_a(a);

        // AND #operand (0x29)
        cpu.memory_mut().write(0x8000, 0x29);
        cpu.memory_mut().write(0x8001, operand);

        cpu.step().unwrap();

        let result = a & operand;
        let expected_z = result == 0;
        prop_assert_eq!(
            cpu.flag_z(),
            expected_z,
            "Z flag should be {} for A=0x{:02X} AND 0x{:02X} = 0x{:02X}",
            expected_z,
            a,
            operand,
            result
        );
    }

    /// Property: ORA result equals A | operand and flags are correct
    #[test]
    fn prop_ora_immediate_result_and_flags(a in 0u8..=255u8, operand in 0u8..=255u8) {
        let mut cpu = setup_cpu();
        cpu.set_a(a);

        // ORA #operand (0x09)
        cpu.memory_mut().write(0x8000, 0x09);
        cpu.memory_mut().write(0x8001, operand);

        cpu.step().unwrap();

        let expected_result = a | operand;
        prop_assert_eq!(cpu.a(), expected_result);
        prop_assert_eq!(cpu.flag_n(), (expected_result & 0x80) != 0);
        prop_assert_eq!(cpu.flag_z(), expected_result == 0);
    }

    /// Property: EOR result equals A ^ operand and flags are correct
    #[test]
    fn prop_eor_immediate_result_and_flags(a in 0u8..=255u8, operand in 0u8..=255u8) {
        let mut cpu = setup_cpu();
        cpu.set_a(a);

        // EOR #operand (0x49)
        cpu.memory_mut().write(0x8000, 0x49);
        cpu.memory_mut().write(0x8001, operand);

        cpu.step().unwrap();

        let expected_result = a ^ operand;
        prop_assert_eq!(cpu.a(), expected_result);
        prop_assert_eq!(cpu.flag_n(), (expected_result & 0x80) != 0);
        prop_assert_eq!(cpu.flag_z(), expected_result == 0);
    }
}

// ========== ADC/SBC Property Tests ==========

proptest! {
    /// Property: ADC correctly computes A + M + C (unsigned)
    #[test]
    fn prop_adc_immediate_result(
        a in 0u8..=255u8,
        operand in 0u8..=255u8,
        carry_in in proptest::bool::ANY,
    ) {
        let mut cpu = setup_cpu();
        cpu.set_a(a);
        cpu.set_flag_c(carry_in);
        cpu.set_flag_d(false); // Binary mode

        // ADC #operand (0x69)
        cpu.memory_mut().write(0x8000, 0x69);
        cpu.memory_mut().write(0x8001, operand);

        cpu.step().unwrap();

        let sum = a as u16 + operand as u16 + carry_in as u16;
        let expected_result = (sum & 0xFF) as u8;
        let expected_carry = sum > 0xFF;

        prop_assert_eq!(
            cpu.a(),
            expected_result,
            "ADC result: 0x{:02X} + 0x{:02X} + {} = 0x{:02X}, got 0x{:02X}",
            a,
            operand,
            carry_in as u8,
            expected_result,
            cpu.a()
        );
        prop_assert_eq!(
            cpu.flag_c(),
            expected_carry,
            "ADC carry: sum = 0x{:03X}, expected carry = {}",
            sum,
            expected_carry
        );
    }

    /// Property: ADC overflow flag is set correctly (signed overflow)
    #[test]
    fn prop_adc_overflow_flag(
        a in 0u8..=255u8,
        operand in 0u8..=255u8,
        carry_in in proptest::bool::ANY,
    ) {
        let mut cpu = setup_cpu();
        cpu.set_a(a);
        cpu.set_flag_c(carry_in);
        cpu.set_flag_d(false); // Binary mode

        // ADC #operand (0x69)
        cpu.memory_mut().write(0x8000, 0x69);
        cpu.memory_mut().write(0x8001, operand);

        cpu.step().unwrap();

        // Overflow occurs when:
        // - Adding two positive numbers gives a negative result
        // - Adding two negative numbers gives a positive result
        let a_sign = (a & 0x80) != 0;
        let m_sign = (operand & 0x80) != 0;
        let result_sign = (cpu.a() & 0x80) != 0;

        // Overflow = same sign inputs, different sign output
        let expected_overflow = (a_sign == m_sign) && (a_sign != result_sign);

        prop_assert_eq!(
            cpu.flag_v(),
            expected_overflow,
            "ADC overflow: A=0x{:02X}, M=0x{:02X}, C={}, result=0x{:02X}, V should be {}",
            a,
            operand,
            carry_in as u8,
            cpu.a(),
            expected_overflow
        );
    }

    /// Property: SBC correctly computes A - M - !C (unsigned)
    #[test]
    fn prop_sbc_immediate_result(
        a in 0u8..=255u8,
        operand in 0u8..=255u8,
        carry_in in proptest::bool::ANY,
    ) {
        let mut cpu = setup_cpu();
        cpu.set_a(a);
        cpu.set_flag_c(carry_in);
        cpu.set_flag_d(false); // Binary mode

        // SBC #operand (0xE9)
        cpu.memory_mut().write(0x8000, 0xE9);
        cpu.memory_mut().write(0x8001, operand);

        cpu.step().unwrap();

        // SBC = A - M - !C = A + !M + C
        let borrow = !carry_in as u16;
        let diff = a as i16 - operand as i16 - borrow as i16;
        let expected_result = (diff & 0xFF) as u8;
        let expected_carry = diff >= 0; // Carry is set if no borrow occurred

        prop_assert_eq!(
            cpu.a(),
            expected_result,
            "SBC result: 0x{:02X} - 0x{:02X} - {} = 0x{:02X}, got 0x{:02X}",
            a,
            operand,
            !carry_in as u8,
            expected_result,
            cpu.a()
        );
        prop_assert_eq!(
            cpu.flag_c(),
            expected_carry,
            "SBC carry: diff = {}, expected carry = {}",
            diff,
            expected_carry
        );
    }
}

// ========== Stack Property Tests ==========

proptest! {
    /// Property: PHA followed by PLA returns the same value
    #[test]
    fn prop_pha_pla_roundtrip(value in 0u8..=255u8) {
        let mut cpu = setup_cpu();
        cpu.set_a(value);

        // PHA (0x48)
        cpu.memory_mut().write(0x8000, 0x48);
        // PLA (0x68)
        cpu.memory_mut().write(0x8001, 0x68);

        cpu.step().unwrap(); // PHA
        cpu.set_a(0x00); // Clear A to verify PLA restores it
        cpu.step().unwrap(); // PLA

        prop_assert_eq!(
            cpu.a(),
            value,
            "PHA/PLA should preserve value 0x{:02X}",
            value
        );
    }

    /// Property: Stack pointer wraps correctly (0x00 -> 0xFF on push underflow)
    #[test]
    fn prop_stack_wrap_on_push(initial_sp in 0u8..=5u8) {
        let mut cpu = setup_cpu();
        cpu.set_sp(initial_sp);
        cpu.set_a(0x42);

        // Push enough times to wrap
        for i in 0..=initial_sp {
            cpu.memory_mut().write(0x8000 + i as u16, 0x48); // PHA
        }

        for _ in 0..=initial_sp {
            cpu.step().unwrap();
        }

        // SP should have wrapped from initial_sp to 0xFF
        // Starting at initial_sp, after (initial_sp + 1) pushes:
        // SP = initial_sp - (initial_sp + 1) = -1 = 0xFF (with wrapping)
        prop_assert_eq!(
            cpu.sp(),
            0xFF,
            "SP should wrap to 0xFF after {} pushes from SP=0x{:02X}, got 0x{:02X}",
            initial_sp + 1,
            initial_sp,
            cpu.sp()
        );
    }

    /// Property: Stack pointer wraps correctly (0xFF -> 0x00 on pull underflow)
    #[test]
    fn prop_stack_wrap_on_pull(initial_sp in 250u8..=254u8) {
        let mut cpu = setup_cpu();
        cpu.set_sp(initial_sp);

        // Put some values on the stack area where we'll pull from
        for i in 0..10 {
            cpu.memory_mut().write(0x0100 + initial_sp as u16 + i + 1, 0x42);
        }

        // Pull enough times to wrap
        let pulls_to_wrap = 255 - initial_sp + 1;
        for i in 0..pulls_to_wrap {
            cpu.memory_mut().write(0x8000 + i as u16, 0x68); // PLA
        }

        for _ in 0..pulls_to_wrap {
            cpu.step().unwrap();
        }

        // SP should have wrapped to 0x00
        prop_assert_eq!(
            cpu.sp(),
            0x00,
            "SP should wrap to 0x00 after {} pulls from SP=0x{:02X}, got 0x{:02X}",
            pulls_to_wrap,
            initial_sp,
            cpu.sp()
        );
    }
}

// ========== Shift/Rotate Property Tests ==========

proptest! {
    /// Property: ASL shifts left and C gets bit 7
    #[test]
    fn prop_asl_accumulator(value in 0u8..=255u8) {
        let mut cpu = setup_cpu();
        cpu.set_a(value);

        // ASL A (0x0A)
        cpu.memory_mut().write(0x8000, 0x0A);

        cpu.step().unwrap();

        let expected_result = value << 1;
        let expected_carry = (value & 0x80) != 0;

        prop_assert_eq!(cpu.a(), expected_result);
        prop_assert_eq!(cpu.flag_c(), expected_carry);
        prop_assert_eq!(cpu.flag_n(), (expected_result & 0x80) != 0);
        prop_assert_eq!(cpu.flag_z(), expected_result == 0);
    }

    /// Property: LSR shifts right and C gets bit 0
    #[test]
    fn prop_lsr_accumulator(value in 0u8..=255u8) {
        let mut cpu = setup_cpu();
        cpu.set_a(value);

        // LSR A (0x4A)
        cpu.memory_mut().write(0x8000, 0x4A);

        cpu.step().unwrap();

        let expected_result = value >> 1;
        let expected_carry = (value & 0x01) != 0;

        prop_assert_eq!(cpu.a(), expected_result);
        prop_assert_eq!(cpu.flag_c(), expected_carry);
        prop_assert_eq!(cpu.flag_n(), false); // LSR always clears N
        prop_assert_eq!(cpu.flag_z(), expected_result == 0);
    }

    /// Property: ROL rotates left through carry
    #[test]
    fn prop_rol_accumulator(value in 0u8..=255u8, carry_in in proptest::bool::ANY) {
        let mut cpu = setup_cpu();
        cpu.set_a(value);
        cpu.set_flag_c(carry_in);

        // ROL A (0x2A)
        cpu.memory_mut().write(0x8000, 0x2A);

        cpu.step().unwrap();

        let expected_result = (value << 1) | (carry_in as u8);
        let expected_carry = (value & 0x80) != 0;

        prop_assert_eq!(cpu.a(), expected_result);
        prop_assert_eq!(cpu.flag_c(), expected_carry);
        prop_assert_eq!(cpu.flag_n(), (expected_result & 0x80) != 0);
        prop_assert_eq!(cpu.flag_z(), expected_result == 0);
    }

    /// Property: ROR rotates right through carry
    #[test]
    fn prop_ror_accumulator(value in 0u8..=255u8, carry_in in proptest::bool::ANY) {
        let mut cpu = setup_cpu();
        cpu.set_a(value);
        cpu.set_flag_c(carry_in);

        // ROR A (0x6A)
        cpu.memory_mut().write(0x8000, 0x6A);

        cpu.step().unwrap();

        let expected_result = (value >> 1) | ((carry_in as u8) << 7);
        let expected_carry = (value & 0x01) != 0;

        prop_assert_eq!(cpu.a(), expected_result);
        prop_assert_eq!(cpu.flag_c(), expected_carry);
        prop_assert_eq!(cpu.flag_n(), (expected_result & 0x80) != 0);
        prop_assert_eq!(cpu.flag_z(), expected_result == 0);
    }
}

// ========== Compare Property Tests ==========

proptest! {
    /// Property: CMP sets flags correctly
    #[test]
    fn prop_cmp_immediate_flags(a in 0u8..=255u8, operand in 0u8..=255u8) {
        let mut cpu = setup_cpu();
        cpu.set_a(a);

        // CMP #operand (0xC9)
        cpu.memory_mut().write(0x8000, 0xC9);
        cpu.memory_mut().write(0x8001, operand);

        cpu.step().unwrap();

        let result = a.wrapping_sub(operand);
        prop_assert_eq!(cpu.flag_c(), a >= operand, "C flag: A >= M");
        prop_assert_eq!(cpu.flag_z(), a == operand, "Z flag: A == M");
        prop_assert_eq!(cpu.flag_n(), (result & 0x80) != 0, "N flag: bit 7 of result");

        // A should be unchanged
        prop_assert_eq!(cpu.a(), a);
    }

    /// Property: CPX sets flags correctly
    #[test]
    fn prop_cpx_immediate_flags(x in 0u8..=255u8, operand in 0u8..=255u8) {
        let mut cpu = setup_cpu();
        cpu.set_x(x);

        // CPX #operand (0xE0)
        cpu.memory_mut().write(0x8000, 0xE0);
        cpu.memory_mut().write(0x8001, operand);

        cpu.step().unwrap();

        let result = x.wrapping_sub(operand);
        prop_assert_eq!(cpu.flag_c(), x >= operand);
        prop_assert_eq!(cpu.flag_z(), x == operand);
        prop_assert_eq!(cpu.flag_n(), (result & 0x80) != 0);

        // X should be unchanged
        prop_assert_eq!(cpu.x(), x);
    }

    /// Property: CPY sets flags correctly
    #[test]
    fn prop_cpy_immediate_flags(y in 0u8..=255u8, operand in 0u8..=255u8) {
        let mut cpu = setup_cpu();
        cpu.set_y(y);

        // CPY #operand (0xC0)
        cpu.memory_mut().write(0x8000, 0xC0);
        cpu.memory_mut().write(0x8001, operand);

        cpu.step().unwrap();

        let result = y.wrapping_sub(operand);
        prop_assert_eq!(cpu.flag_c(), y >= operand);
        prop_assert_eq!(cpu.flag_z(), y == operand);
        prop_assert_eq!(cpu.flag_n(), (result & 0x80) != 0);

        // Y should be unchanged
        prop_assert_eq!(cpu.y(), y);
    }
}

// ========== Increment/Decrement Property Tests ==========

proptest! {
    /// Property: INX wraps correctly at 0xFF
    #[test]
    fn prop_inx_wrap(x in 0u8..=255u8) {
        let mut cpu = setup_cpu();
        cpu.set_x(x);

        // INX (0xE8)
        cpu.memory_mut().write(0x8000, 0xE8);

        cpu.step().unwrap();

        let expected = x.wrapping_add(1);
        prop_assert_eq!(cpu.x(), expected);
        prop_assert_eq!(cpu.flag_n(), (expected & 0x80) != 0);
        prop_assert_eq!(cpu.flag_z(), expected == 0);
    }

    /// Property: INY wraps correctly at 0xFF
    #[test]
    fn prop_iny_wrap(y in 0u8..=255u8) {
        let mut cpu = setup_cpu();
        cpu.set_y(y);

        // INY (0xC8)
        cpu.memory_mut().write(0x8000, 0xC8);

        cpu.step().unwrap();

        let expected = y.wrapping_add(1);
        prop_assert_eq!(cpu.y(), expected);
        prop_assert_eq!(cpu.flag_n(), (expected & 0x80) != 0);
        prop_assert_eq!(cpu.flag_z(), expected == 0);
    }

    /// Property: DEX wraps correctly at 0x00
    #[test]
    fn prop_dex_wrap(x in 0u8..=255u8) {
        let mut cpu = setup_cpu();
        cpu.set_x(x);

        // DEX (0xCA)
        cpu.memory_mut().write(0x8000, 0xCA);

        cpu.step().unwrap();

        let expected = x.wrapping_sub(1);
        prop_assert_eq!(cpu.x(), expected);
        prop_assert_eq!(cpu.flag_n(), (expected & 0x80) != 0);
        prop_assert_eq!(cpu.flag_z(), expected == 0);
    }

    /// Property: DEY wraps correctly at 0x00
    #[test]
    fn prop_dey_wrap(y in 0u8..=255u8) {
        let mut cpu = setup_cpu();
        cpu.set_y(y);

        // DEY (0x88)
        cpu.memory_mut().write(0x8000, 0x88);

        cpu.step().unwrap();

        let expected = y.wrapping_sub(1);
        prop_assert_eq!(cpu.y(), expected);
        prop_assert_eq!(cpu.flag_n(), (expected & 0x80) != 0);
        prop_assert_eq!(cpu.flag_z(), expected == 0);
    }
}

// ========== Transfer Property Tests ==========

proptest! {
    /// Property: TAX correctly transfers A to X
    #[test]
    fn prop_tax_transfer(a in 0u8..=255u8) {
        let mut cpu = setup_cpu();
        cpu.set_a(a);
        cpu.set_x(0x00);

        // TAX (0xAA)
        cpu.memory_mut().write(0x8000, 0xAA);

        cpu.step().unwrap();

        prop_assert_eq!(cpu.x(), a);
        prop_assert_eq!(cpu.a(), a); // A unchanged
        prop_assert_eq!(cpu.flag_n(), (a & 0x80) != 0);
        prop_assert_eq!(cpu.flag_z(), a == 0);
    }

    /// Property: TAY correctly transfers A to Y
    #[test]
    fn prop_tay_transfer(a in 0u8..=255u8) {
        let mut cpu = setup_cpu();
        cpu.set_a(a);
        cpu.set_y(0x00);

        // TAY (0xA8)
        cpu.memory_mut().write(0x8000, 0xA8);

        cpu.step().unwrap();

        prop_assert_eq!(cpu.y(), a);
        prop_assert_eq!(cpu.a(), a); // A unchanged
        prop_assert_eq!(cpu.flag_n(), (a & 0x80) != 0);
        prop_assert_eq!(cpu.flag_z(), a == 0);
    }

    /// Property: TXA correctly transfers X to A
    #[test]
    fn prop_txa_transfer(x in 0u8..=255u8) {
        let mut cpu = setup_cpu();
        cpu.set_x(x);
        cpu.set_a(0x00);

        // TXA (0x8A)
        cpu.memory_mut().write(0x8000, 0x8A);

        cpu.step().unwrap();

        prop_assert_eq!(cpu.a(), x);
        prop_assert_eq!(cpu.x(), x); // X unchanged
        prop_assert_eq!(cpu.flag_n(), (x & 0x80) != 0);
        prop_assert_eq!(cpu.flag_z(), x == 0);
    }

    /// Property: TYA correctly transfers Y to A
    #[test]
    fn prop_tya_transfer(y in 0u8..=255u8) {
        let mut cpu = setup_cpu();
        cpu.set_y(y);
        cpu.set_a(0x00);

        // TYA (0x98)
        cpu.memory_mut().write(0x8000, 0x98);

        cpu.step().unwrap();

        prop_assert_eq!(cpu.a(), y);
        prop_assert_eq!(cpu.y(), y); // Y unchanged
        prop_assert_eq!(cpu.flag_n(), (y & 0x80) != 0);
        prop_assert_eq!(cpu.flag_z(), y == 0);
    }
}

// ========== Helper Functions ==========

/// Setup memory for an instruction to execute without crashing
fn setup_memory_for_instruction(cpu: &mut CPU<FlatMemory>, opcode: u8, operand1: u8, operand2: u8) {
    let metadata = &OPCODE_TABLE[opcode as usize];

    match metadata.addressing_mode {
        AddressingMode::ZeroPage | AddressingMode::ZeroPageX | AddressingMode::ZeroPageY => {
            // Ensure there's valid data at the zero page address
            cpu.memory_mut().write(operand1 as u16, 0x42);
        }
        AddressingMode::Absolute | AddressingMode::AbsoluteX | AddressingMode::AbsoluteY => {
            // Ensure there's valid data at the absolute address
            let addr = (operand2 as u16) << 8 | (operand1 as u16);
            if addr < 0xFF00 {
                // Avoid writing to vectors
                cpu.memory_mut().write(addr, 0x42);
            }
        }
        AddressingMode::IndirectX | AddressingMode::IndirectY => {
            // Set up indirect pointer
            let zp_addr = operand1 as u16;
            cpu.memory_mut().write(zp_addr, 0x00);
            cpu.memory_mut()
                .write(zp_addr.wrapping_add(1) & 0xFF, 0x40);
            cpu.memory_mut().write(0x4000, 0x42);
        }
        AddressingMode::Indirect => {
            // For JMP indirect
            let addr = (operand2 as u16) << 8 | (operand1 as u16);
            if addr < 0xFF00 {
                cpu.memory_mut().write(addr, 0x00);
                cpu.memory_mut().write(addr.wrapping_add(1), 0x80);
            }
        }
        _ => {}
    }
}
