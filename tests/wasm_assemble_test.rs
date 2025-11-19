//! Test for WASM assemble() method fix
//!
//! Verifies that when code is assembled for a specific start address,
//! labels are correctly resolved to absolute addresses (not relative to $0000).

use lib6502::assemble;

#[test]
fn test_assemble_with_org_directive() {
    // Simple program with a label reference
    let code_without_org = r#"
        LDX #$00
    loop:
        LDA data,X
        BEQ done
        INX
        JMP loop
    done:
        BRK
    data:
        .byte "Hi"
        .byte $00
    "#;

    // Test 1: Without .org directive - assembles for $0000
    let result1 = assemble(code_without_org).unwrap();
    let bytes1 = &result1.bytes;

    println!("Assembled bytes: {:02X?}", bytes1);
    println!("Symbols: {:?}", result1.symbol_table);

    // Find LDA data,X instruction (opcode 0xBD for absolute,X or 0xB5 for zero-page,X)
    let lda_idx1 = bytes1
        .iter()
        .position(|&b| b == 0xBD || b == 0xB5)
        .expect("LDA data,X not found");
    let lda_opcode1 = bytes1[lda_idx1];

    let addr1 = if lda_opcode1 == 0xBD {
        // Absolute,X addressing - 3 byte instruction
        let addr_low1 = bytes1[lda_idx1 + 1];
        let addr_high1 = bytes1[lda_idx1 + 2];
        (addr_high1 as u16) << 8 | addr_low1 as u16
    } else {
        // Zero-page,X addressing - 2 byte instruction
        bytes1[lda_idx1 + 1] as u16
    };

    println!(
        "Without .org: data label at ${:04X} (opcode ${:02X})",
        addr1, lda_opcode1
    );
    assert!(addr1 < 0x0100, "Without .org, label should be < $0100");

    // Test 2: With .org $0600 directive - assembles for $0600
    let code_with_org = format!(".org $0600\n{}", code_without_org);
    let result2 = assemble(&code_with_org).unwrap();
    let bytes2 = &result2.bytes;

    println!("With .org - Assembled bytes: {:02X?}", bytes2);
    println!("With .org - Symbols: {:?}", result2.symbol_table);

    // Find LDA data,X instruction (opcode 0xBD for absolute,X or 0xB5 for zero-page,X)
    let lda_idx2 = bytes2
        .iter()
        .position(|&b| b == 0xBD || b == 0xB5)
        .expect("LDA data,X not found");
    let lda_opcode2 = bytes2[lda_idx2];

    let addr2 = if lda_opcode2 == 0xBD {
        // Absolute,X addressing - 3 byte instruction
        let addr_low2 = bytes2[lda_idx2 + 1];
        let addr_high2 = bytes2[lda_idx2 + 2];
        (addr_high2 as u16) << 8 | addr_low2 as u16
    } else {
        // Zero-page,X addressing - 2 byte instruction (shouldn't happen with .org $0600)
        bytes2[lda_idx2 + 1] as u16
    };

    println!(
        "With .org $0600: data label at ${:04X} (opcode ${:02X})",
        addr2, lda_opcode2
    );
    assert!(
        addr2 >= 0x0600,
        "With .org $0600, label should be >= $0600, got ${:04X}",
        addr2
    );
    assert!(
        addr2 < 0x0700,
        "With .org $0600, label should be < $0700, got ${:04X}",
        addr2
    );

    println!(
        "✓ Labels correctly adjusted by .org directive (from ${:04X} to ${:04X})",
        addr1, addr2
    );
}
