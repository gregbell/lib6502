//! Klaus Dormann's 6502 Functional Test
//!
//! This module integrates the comprehensive functional test suite from
//! https://github.com/Klaus2m5/6502_65C02_functional_tests
//!
//! The test validates all valid opcodes and addressing modes of the NMOS 6502 CPU.
//!
//! ## How the Test Works
//!
//! 1. Load the 64KB binary image into memory (includes code and data)
//! 2. Set PC to entry point ($0400)
//! 3. Execute instructions until an infinite loop is detected (PC doesn't change)
//! 4. Check if the final PC matches the success address ($3469)
//!
//! ## Success/Failure Detection
//!
//! The test uses `JMP *` (jump to current address) to create infinite loops:
//! - **Success**: PC stops at $3469 (all tests passed)
//! - **Failure**: PC stops at any other address (indicates which test failed)
//!
//! The listing file (`6502_functional_test.lst`) can be used to identify which
//! specific test failed based on the final PC value.

use lib6502::{FlatMemory, MemoryBus, CPU};
use std::fs::File;
use std::io::Read;

/// Success address - where PC ends up when all tests pass
const SUCCESS_ADDRESS: u16 = 0x3469;

/// Entry point for the functional test
const ENTRY_POINT: u16 = 0x0400;

/// Reset vector location (low byte)
const RESET_VECTOR_LOW: u16 = 0xFFFC;

/// Reset vector location (high byte)
const RESET_VECTOR_HIGH: u16 = 0xFFFD;

/// Maximum cycles to run before timing out (100 million cycles)
/// This prevents infinite loops from hanging the test suite.
/// The test should complete in far fewer cycles if working correctly.
const MAX_CYCLES: u64 = 100_000_000;

/// Number of identical PC values in a row needed to detect an infinite loop
const LOOP_DETECTION_THRESHOLD: usize = 3;

/// Load the 64KB binary test image into memory and set reset vector
fn load_test_binary(path: &str) -> FlatMemory {
    let mut file =
        File::open(path).unwrap_or_else(|e| panic!("Failed to open test binary {}: {}", path, e));

    let mut buffer = vec![0u8; 65536];
    let bytes_read = file
        .read(&mut buffer)
        .unwrap_or_else(|e| panic!("Failed to read test binary: {}", e));

    assert_eq!(bytes_read, 65536, "Test binary must be exactly 64KB");

    let mut memory = FlatMemory::new();
    for (addr, &byte) in buffer.iter().enumerate() {
        memory.write(addr as u16, byte);
    }

    // The binary already contains the full memory image including vectors,
    // but we'll verify the reset vector points to our expected entry point
    let reset_low = memory.read(RESET_VECTOR_LOW);
    let reset_high = memory.read(RESET_VECTOR_HIGH);
    let reset_vector = ((reset_high as u16) << 8) | (reset_low as u16);

    // If the binary doesn't have the reset vector set correctly, set it manually
    if reset_vector != ENTRY_POINT {
        memory.write(RESET_VECTOR_LOW, (ENTRY_POINT & 0xFF) as u8);
        memory.write(RESET_VECTOR_HIGH, ((ENTRY_POINT >> 8) & 0xFF) as u8);
    }

    memory
}

/// Run the CPU until an infinite loop is detected or max cycles is reached
///
/// Returns the PC value where execution stopped.
///
/// An infinite loop is detected when the PC doesn't change for
/// LOOP_DETECTION_THRESHOLD consecutive steps.
fn run_until_loop(
    cpu: &mut CPU<FlatMemory>,
    max_cycles: u64,
    verbose: bool,
) -> Result<u16, String> {
    let mut pc_history = [0u16; LOOP_DETECTION_THRESHOLD];
    let mut history_idx = 0;
    let start_cycles = cpu.cycles();

    loop {
        let current_pc = cpu.pc();

        // Check if we've exceeded our cycle budget
        if cpu.cycles() - start_cycles >= max_cycles {
            return Err(format!(
                "Timeout: exceeded {} cycles. PC stuck at ${:04X}. \
                 This likely means an infinite loop was encountered or an \
                 unimplemented instruction blocked progress.",
                max_cycles, current_pc
            ));
        }

        // Execute one instruction
        match cpu.step() {
            Ok(_) => {}
            Err(e) => {
                return Err(format!(
                    "Execution error at PC ${:04X}: {}. \
                     This indicates an unimplemented instruction or execution failure.",
                    current_pc, e
                ));
            }
        }

        // Update PC history
        pc_history[history_idx] = current_pc;
        history_idx = (history_idx + 1) % LOOP_DETECTION_THRESHOLD;

        // Check if PC has been the same for the last N steps (infinite loop)
        if pc_history.iter().all(|&pc| pc == current_pc) {
            if verbose {
                println!("Infinite loop detected at PC ${:04X}", current_pc);
            }
            return Ok(current_pc);
        }

        if verbose && (cpu.cycles() - start_cycles) % 100000 == 0 {
            println!(
                "Progress: {} cycles, PC: ${:04X}",
                cpu.cycles() - start_cycles,
                current_pc
            );
        }
    }
}

/// Format CPU state for diagnostic output
fn format_cpu_state(cpu: &CPU<FlatMemory>) -> String {
    format!(
        "PC:${:04X} A:${:02X} X:${:02X} Y:${:02X} SP:${:02X} P:[{}{}{}{}{}{}{}] Cycles:{}",
        cpu.pc(),
        cpu.a(),
        cpu.x(),
        cpu.y(),
        cpu.sp(),
        if cpu.flag_n() { 'N' } else { '-' },
        if cpu.flag_v() { 'V' } else { '-' },
        if cpu.flag_d() { 'D' } else { '-' },
        if cpu.flag_i() { 'I' } else { '-' },
        if cpu.flag_z() { 'Z' } else { '-' },
        if cpu.flag_c() { 'C' } else { '-' },
        if cpu.flag_b() { 'B' } else { '-' },
        cpu.cycles()
    )
}

#[test]
#[ignore = "slow functional test (~6 seconds) - run with --ignored or --include-ignored"]
fn klaus_6502_functional_test() {
    // Load the test binary (includes setting reset vector to entry point)
    let memory = load_test_binary("tests/fixtures/6502_functional_test.bin");
    let mut cpu = CPU::new(memory);

    println!("\n=== Klaus 6502 Functional Test ===");
    println!("Entry point: ${:04X}", ENTRY_POINT);
    println!("Success address: ${:04X}", SUCCESS_ADDRESS);
    println!("Initial state: {}", format_cpu_state(&cpu));
    println!("Running test...\n");

    // Verify we're starting at the right place
    assert_eq!(
        cpu.pc(),
        ENTRY_POINT,
        "CPU should start at entry point after reset"
    );

    // Run until infinite loop detected
    let verbose = false; // Set to true for progress updates
    let final_pc = match run_until_loop(&mut cpu, MAX_CYCLES, verbose) {
        Ok(pc) => pc,
        Err(e) => {
            println!("\n=== TEST FAILED ===");
            println!("{}", e);
            println!("Final state: {}", format_cpu_state(&cpu));
            panic!("{}", e);
        }
    };

    println!("Test completed!");
    println!("Final state: {}", format_cpu_state(&cpu));
    println!("Final PC: ${:04X}", final_pc);

    // Check if we reached the success address
    if final_pc == SUCCESS_ADDRESS {
        println!("\n✓ SUCCESS: All tests passed!");
    } else {
        println!("\n✗ FAILURE: Test stopped at ${:04X}", final_pc);
        println!("Expected success address: ${:04X}", SUCCESS_ADDRESS);
        println!("\nTo identify which test failed, check the listing file:");
        println!("  tests/fixtures/6502_functional_test.lst");
        println!(
            "  Search for address {:04X} to see which instruction failed.",
            final_pc
        );

        // Try to provide some context about nearby memory
        println!("\nMemory around failure point:");
        for offset in -5i16..=5 {
            let addr = (final_pc as i16 + offset) as u16;
            let byte = cpu.memory_mut().read(addr);
            let marker = if offset == 0 { " <-- PC" } else { "" };
            println!("  ${:04X}: ${:02X}{}", addr, byte, marker);
        }

        panic!(
            "Test failed at PC ${:04X} (expected ${:04X})",
            final_pc, SUCCESS_ADDRESS
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify the test binary exists and is the correct size
    #[test]
    fn test_binary_exists_and_correct_size() {
        let memory = load_test_binary("tests/fixtures/6502_functional_test.bin");
        // If we got here, the binary loaded successfully

        // Verify the entry point has a valid instruction (CLD = 0xD8)
        assert_eq!(
            memory.read(ENTRY_POINT),
            0xD8,
            "Entry point should start with CLD instruction"
        );
    }

    /// Verify success address contains the infinite loop instruction
    #[test]
    fn test_success_address_has_infinite_loop() {
        let memory = load_test_binary("tests/fixtures/6502_functional_test.bin");

        // The success address should contain JMP * (4C 69 34 = JMP $3469)
        assert_eq!(
            memory.read(SUCCESS_ADDRESS),
            0x4C,
            "Success address should have JMP opcode"
        );
        assert_eq!(
            memory.read(SUCCESS_ADDRESS + 1),
            0x69,
            "JMP target low byte"
        );
        assert_eq!(
            memory.read(SUCCESS_ADDRESS + 2),
            0x34,
            "JMP target high byte"
        );
    }
}
