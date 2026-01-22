//! Fuzz target for the assembler.
//!
//! This target feeds arbitrary strings to the assembler to find
//! edge cases, panics, and crashes in parsing and encoding.

#![no_main]

use lib6502::assemble;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Try to interpret the data as UTF-8
    if let Ok(source) = std::str::from_utf8(data) {
        // Attempt to assemble the source
        // We don't care about errors - just ensure no panics
        let _ = assemble(source);
    }

    // Also try assembling with lossy UTF-8 conversion
    // This exercises more edge cases in string handling
    let lossy_source = String::from_utf8_lossy(data);
    let _ = assemble(&lossy_source);
});
