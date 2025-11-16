//! Integration tests for the 6502 assembler

use lib6502::assembler::{assemble, ErrorType};

// T028: Integration test for single instruction assembly (LDA #$42)
#[test]
fn test_single_instruction_assembly() {
    let source = "LDA #$42";

    let result = assemble(source);
    assert!(result.is_ok(), "Assembly should succeed");

    let output = result.unwrap();
    assert_eq!(
        output.bytes,
        vec![0xA9, 0x42],
        "Should assemble to LDA immediate"
    );
}

// T029: Integration test for multi-line assembly
#[test]
fn test_multi_line_assembly() {
    let source = r#"
        LDA #$42
        STA $8000
        JMP $8000
    "#;

    let result = assemble(source);
    assert!(result.is_ok(), "Assembly should succeed");

    let output = result.unwrap();
    assert_eq!(
        output.bytes,
        vec![0xA9, 0x42, 0x8D, 0x00, 0x80, 0x4C, 0x00, 0x80],
        "Should assemble all three instructions"
    );
}

// T030: Integration test for number format parsing (hex $42, decimal 66, binary %01000010)
#[test]
fn test_number_format_parsing() {
    let source_hex = "LDA #$42";
    let source_dec = "LDA #66";
    let source_bin = "LDA #%01000010";

    let result_hex = assemble(source_hex).unwrap();
    let result_dec = assemble(source_dec).unwrap();
    let result_bin = assemble(source_bin).unwrap();

    // All three should produce the same output
    assert_eq!(result_hex.bytes, vec![0xA9, 0x42]);
    assert_eq!(result_dec.bytes, vec![0xA9, 0x42]);
    assert_eq!(result_bin.bytes, vec![0xA9, 0x42]);
}

// T031: Integration test for case-insensitive and whitespace-tolerant parsing
#[test]
fn test_case_insensitive_and_whitespace() {
    let variations = vec![
        "LDA #$42",
        "lda #$42",
        "LdA #$42",
        "  LDA   #$42  ",
        "\tLDA\t#$42\t",
    ];

    for source in variations {
        let result = assemble(source);
        assert!(
            result.is_ok(),
            "Should handle case and whitespace: '{}'",
            source
        );
        assert_eq!(result.unwrap().bytes, vec![0xA9, 0x42]);
    }
}

// T032: Integration test for syntax error reporting with line/column info
#[test]
fn test_syntax_error_reporting() {
    let source = r#"
        LDA #$42
        INVALID_MNEMONIC #$10
        STA $8000
    "#;

    let result = assemble(source);
    assert!(result.is_err(), "Should fail on invalid mnemonic");

    let errors = result.unwrap_err();
    assert!(!errors.is_empty(), "Should have at least one error");

    let error = &errors[0];
    assert_eq!(error.error_type, ErrorType::InvalidMnemonic);
    assert!(error.line > 0, "Should have line number");
    // Column is 0-indexed; just ensure span is consistent and message exists
    assert!(!error.message.is_empty(), "Should have error message");
}

// T033: Integration test for multiple error collection
#[test]
fn test_multiple_error_collection() {
    let source = r#"
        INVALID1 #$42
        LDA #$42
        INVALID2 $8000
        STA #$1234
    "#;

    let result = assemble(source);
    assert!(result.is_err(), "Should fail on invalid mnemonics");

    let errors = result.unwrap_err();
    assert!(
        errors.len() >= 2,
        "Should collect multiple errors, got {}",
        errors.len()
    );

    // Should have at least the two invalid mnemonics
    let invalid_mnemonics = errors
        .iter()
        .filter(|e| e.error_type == ErrorType::InvalidMnemonic)
        .count();
    assert!(
        invalid_mnemonics >= 2,
        "Should detect at least 2 invalid mnemonics"
    );
}

// T046: Integration test for error reporting with line/column/span
#[test]
fn test_error_span_information() {
    let source = r#"
LDA #$42
BADMNEM #$10
STA $8000
"#;

    let result = assemble(source);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    let error = &errors[0];

    // Error should have detailed location info
    assert_eq!(error.line, 3, "Error on line 3");
    assert!(error.span.0 < error.span.1, "Span should have start < end");
    assert_eq!(error.error_type, ErrorType::InvalidMnemonic);
}

// T047: Integration test for source map query by instruction address
#[test]
fn test_source_map_by_address() {
    let source = r#"
LDA #$42
STA $8000
NOP
"#;

    let result = assemble(source);
    assert!(result.is_ok());

    let output = result.unwrap();

    // Query source location for first instruction (LDA at address 0)
    let loc = output.get_source_location(0);
    assert!(loc.is_some(), "Should find source location for address 0");
    let loc = loc.unwrap();
    assert_eq!(loc.line, 2, "LDA is on line 2");

    // Query source location for second instruction (STA at address 2)
    let loc = output.get_source_location(2);
    assert!(loc.is_some(), "Should find source location for address 2");
    let loc = loc.unwrap();
    assert_eq!(loc.line, 3, "STA is on line 3");
}

// T048: Integration test for source map query by source line
#[test]
fn test_source_map_by_line() {
    let source = r#"
LDA #$42
STA $8000
"#;

    let result = assemble(source);
    assert!(result.is_ok());

    let output = result.unwrap();

    // Query address range for line 2 (LDA #$42)
    let range = output.get_address_range(2);
    assert!(range.is_some(), "Should find address range for line 2");
    let range = range.unwrap();
    assert_eq!(range.start, 0);
    assert_eq!(range.end, 2); // LDA #$42 is 2 bytes

    // Query address range for line 3 (STA $8000)
    let range = output.get_address_range(3);
    assert!(range.is_some(), "Should find address range for line 3");
    let range = range.unwrap();
    assert_eq!(range.start, 2);
    assert_eq!(range.end, 5); // STA $8000 is 3 bytes
}

// T049: Integration test for symbol table access
#[test]
fn test_symbol_table_access() {
    let source = "LDA #$42\nSTA $8000";

    let result = assemble(source);
    assert!(result.is_ok());

    let output = result.unwrap();

    // No labels in this simple code, but symbol table should be accessible
    assert_eq!(output.symbol_table.len(), 0, "No labels defined");

    // Lookup should return None for non-existent symbols
    assert!(output.lookup_symbol("NONEXISTENT").is_none());
}

// T050: Integration test for structured Instruction data (already tested in disassembler_tests.rs)
// The Instruction struct is already being validated in disassembler tests

// ========== Phase 6: User Story 3 - Labels ==========

// T062: Integration test for simple label definition and reference (JMP START)
#[test]
fn test_simple_label_definition_and_reference() {
    let source = r#"
START:
    LDA #$42
    JMP START
"#;

    let result = assemble(source);
    assert!(result.is_ok(), "Should successfully assemble with label");

    let output = result.unwrap();

    // Verify symbol table contains START label
    assert_eq!(output.symbol_table.len(), 1, "Should have 1 label");
    let symbol = output.lookup_symbol("START");
    assert!(symbol.is_some(), "Should find START label");
    let symbol = symbol.unwrap();
    assert_eq!(symbol.name, "START");
    assert_eq!(symbol.address, 0, "START should be at address 0");

    // Verify bytes: LDA #$42 (A9 42) + JMP $0000 (4C 00 00)
    assert_eq!(output.bytes, vec![0xA9, 0x42, 0x4C, 0x00, 0x00]);
}

// T063: Integration test for forward label reference
#[test]
fn test_forward_label_reference() {
    let source = r#"
    JMP END
    LDA #$42
END:
    NOP
"#;

    let result = assemble(source);
    assert!(
        result.is_ok(),
        "Should successfully assemble with forward reference"
    );

    let output = result.unwrap();

    // Verify symbol table contains END label
    let symbol = output.lookup_symbol("END");
    assert!(symbol.is_some(), "Should find END label");
    let symbol = symbol.unwrap();
    assert_eq!(
        symbol.address, 5,
        "END should be at address 5 (after JMP + LDA)"
    );

    // Verify JMP instruction targets correct address (0x0005)
    // JMP $0005 = 4C 05 00 (little endian)
    assert_eq!(output.bytes[0], 0x4C); // JMP opcode
    assert_eq!(output.bytes[1], 0x05); // Low byte of address
    assert_eq!(output.bytes[2], 0x00); // High byte of address
}

// T064: Integration test for relative branch to label (BEQ LOOP)
#[test]
fn test_relative_branch_to_label() {
    let source = r#"
LOOP:
    LDA #$42
    BEQ LOOP
"#;

    let result = assemble(source);
    assert!(
        result.is_ok(),
        "Should successfully assemble with branch to label"
    );

    let output = result.unwrap();

    // Verify symbol table
    let symbol = output.lookup_symbol("LOOP");
    assert!(symbol.is_some());
    assert_eq!(symbol.unwrap().address, 0);

    // BEQ LOOP should branch back
    // From address 2 (after LDA #$42), branch to address 0
    // Offset = 0 - (2 + 2) = -4 = 0xFC in two's complement
    assert_eq!(output.bytes[0], 0xA9); // LDA #$42
    assert_eq!(output.bytes[1], 0x42);
    assert_eq!(output.bytes[2], 0xF0); // BEQ opcode
    assert_eq!(output.bytes[3], 0xFC); // Offset -4
}

// Test for forward branch to confirm Pass-1 sizing bug is fixed
#[test]
fn test_forward_branch_label_sizing() {
    let source = r#"
    BEQ FORWARD   ; Should be 2 bytes at address 0-1
    NOP           ; Should be at address 2
FORWARD:
    LDA #$42      ; Should be at address 3
"#;

    let result = assemble(source);
    assert!(
        result.is_ok(),
        "Should successfully assemble with forward branch to label"
    );

    let output = result.unwrap();

    // Verify symbol table - FORWARD should be at address 3
    let symbol = output.lookup_symbol("FORWARD");
    assert!(
        symbol.is_some(),
        "FORWARD label should exist in symbol table"
    );
    assert_eq!(
        symbol.unwrap().address,
        3,
        "FORWARD should be at address 3 (BEQ=2 bytes, NOP=1 byte)"
    );

    // Verify assembled bytes
    assert_eq!(output.bytes[0], 0xF0); // BEQ opcode
                                       // From address 0, branch to address 3
                                       // Offset = 3 - (0 + 2) = 1
    assert_eq!(output.bytes[1], 0x01); // Offset +1
    assert_eq!(output.bytes[2], 0xEA); // NOP opcode
    assert_eq!(output.bytes[3], 0xA9); // LDA #$42
    assert_eq!(output.bytes[4], 0x42);
}

// T065: Integration test for undefined label error
#[test]
fn test_undefined_label_error() {
    let source = r#"
    JMP UNDEFINED
    LDA #$42
"#;

    let result = assemble(source);
    assert!(result.is_err(), "Should fail on undefined label");

    let errors = result.unwrap_err();
    assert!(!errors.is_empty());

    // Should have an undefined label error
    let undefined_errors = errors
        .iter()
        .filter(|e| e.error_type == ErrorType::UndefinedLabel)
        .count();
    assert!(undefined_errors >= 1, "Should have undefined label error");
}

// T066: Integration test for duplicate label error
#[test]
fn test_duplicate_label_error() {
    let source = r#"
START:
    LDA #$42
START:
    NOP
"#;

    let result = assemble(source);
    assert!(result.is_err(), "Should fail on duplicate label");

    let errors = result.unwrap_err();
    assert!(!errors.is_empty());

    // Should have a duplicate label error
    let duplicate_errors = errors
        .iter()
        .filter(|e| e.error_type == ErrorType::DuplicateLabel)
        .count();
    assert!(duplicate_errors >= 1, "Should have duplicate label error");
}

// T067: Integration test for invalid label validation
#[test]
fn test_invalid_label_validation() {
    // Test label starting with digit
    let source1 = r#"
1START:
    LDA #$42
"#;
    let result = assemble(source1);
    assert!(result.is_err(), "Should fail on label starting with digit");

    // Test label that's too long (>32 chars)
    let source2 = format!("{}:\n    LDA #$42", "A".repeat(33));
    let result = assemble(&source2);
    assert!(result.is_err(), "Should fail on label that's too long");

    // Test label with invalid characters
    let source3 = r#"
MY-LABEL:
    LDA #$42
"#;
    let result = assemble(source3);
    assert!(
        result.is_err(),
        "Should fail on label with invalid characters"
    );
}

// ========== Phase 8: User Story 5 - Comments and Directives ==========

// T089: Integration test for comment parsing and ignoring
#[test]
fn test_comment_parsing() {
    let source = r#"
; This is a full-line comment
LDA #$42  ; Load the value 42
STA $8000 ; Store it
; Another comment
NOP       ; No operation
"#;

    let result = assemble(source);
    assert!(
        result.is_ok(),
        "Should successfully assemble code with comments"
    );

    let output = result.unwrap();

    // Verify the instructions are assembled correctly (comments ignored)
    assert_eq!(output.bytes.len(), 6); // LDA (2) + STA (3) + NOP (1)
    assert_eq!(output.bytes[0], 0xA9); // LDA immediate
    assert_eq!(output.bytes[1], 0x42);
    assert_eq!(output.bytes[2], 0x8D); // STA absolute
    assert_eq!(output.bytes[5], 0xEA); // NOP
}

// T090: Integration test for .org directive setting origin address
#[test]
fn test_org_directive() {
    let source = r#"
.org $8000
LDA #$42
STA $8005
"#;

    let result = assemble(source);
    assert!(
        result.is_ok(),
        "Should successfully assemble with .org directive"
    );

    let output = result.unwrap();

    // Origin should be set, but bytes are just the instructions
    assert_eq!(output.bytes.len(), 5); // LDA (2) + STA (3)

    // Check source map reflects the org address
    let loc = output.get_source_location(0x8000);
    assert!(
        loc.is_some(),
        "First instruction should be at $8000 due to .org"
    );
}

// T091: Integration test for .byte directive inserting literal bytes
#[test]
fn test_byte_directive() {
    let source = r#"
.byte $42, $43, $44
LDA #$FF
"#;

    let result = assemble(source);
    assert!(
        result.is_ok(),
        "Should successfully assemble with .byte directive"
    );

    let output = result.unwrap();

    // .byte inserts 3 bytes, then LDA adds 2 more
    assert_eq!(output.bytes.len(), 5);
    assert_eq!(output.bytes[0], 0x42);
    assert_eq!(output.bytes[1], 0x43);
    assert_eq!(output.bytes[2], 0x44);
    assert_eq!(output.bytes[3], 0xA9); // LDA
    assert_eq!(output.bytes[4], 0xFF);
}

// T092: Integration test for .word directive with little-endian encoding
#[test]
fn test_word_directive() {
    let source = r#"
.word $1234, $5678
LDA #$FF
"#;

    let result = assemble(source);
    assert!(
        result.is_ok(),
        "Should successfully assemble with .word directive"
    );

    let output = result.unwrap();

    // .word inserts 4 bytes (2 words in little-endian), then LDA adds 2 more
    assert_eq!(output.bytes.len(), 6);

    // $1234 in little-endian: $34 $12
    assert_eq!(output.bytes[0], 0x34);
    assert_eq!(output.bytes[1], 0x12);

    // $5678 in little-endian: $78 $56
    assert_eq!(output.bytes[2], 0x78);
    assert_eq!(output.bytes[3], 0x56);

    // LDA #$FF
    assert_eq!(output.bytes[4], 0xA9);
    assert_eq!(output.bytes[5], 0xFF);
}

// T093: Integration test for invalid directive error
#[test]
fn test_invalid_directive_error() {
    let source = r#"
.invalid $1234
LDA #$42
"#;

    let result = assemble(source);
    assert!(result.is_err(), "Should fail on invalid directive");

    let errors = result.unwrap_err();
    assert!(!errors.is_empty());

    // Should have an invalid directive error
    let invalid_directive_errors = errors
        .iter()
        .filter(|e| e.error_type == ErrorType::InvalidDirective)
        .count();
    assert!(
        invalid_directive_errors >= 1,
        "Should have invalid directive error"
    );
}

// ========== Tests for Case-Insensitive Operands and Whitespace Tolerance ==========

// Test lowercase register names in indexed modes
#[test]
fn test_lowercase_register_indexed_modes() {
    // ZeroPageX with lowercase
    let source1 = "lda $10,x";
    let result1 = assemble(source1);
    assert!(result1.is_ok(), "Should handle lowercase ,x: {:?}", result1);
    let output1 = result1.unwrap();
    assert_eq!(output1.bytes[0], 0xB5); // LDA ZeroPageX opcode
    assert_eq!(output1.bytes[1], 0x10);

    // AbsoluteX with lowercase
    let source2 = "lda $1234,x";
    let result2 = assemble(source2);
    assert!(
        result2.is_ok(),
        "Should handle lowercase ,x in absolute: {:?}",
        result2
    );
    let output2 = result2.unwrap();
    assert_eq!(output2.bytes[0], 0xBD); // LDA AbsoluteX opcode

    // ZeroPageY with lowercase
    let source3 = "ldx $10,y";
    let result3 = assemble(source3);
    assert!(result3.is_ok(), "Should handle lowercase ,y: {:?}", result3);
    let output3 = result3.unwrap();
    assert_eq!(output3.bytes[0], 0xB6); // LDX ZeroPageY opcode
}

#[test]
fn test_lowercase_register_indirect_modes() {
    // IndirectX with lowercase
    let source1 = "lda ($20,x)";
    let result1 = assemble(source1);
    assert!(
        result1.is_ok(),
        "Should handle lowercase ($20,x): {:?}",
        result1
    );
    let output1 = result1.unwrap();
    assert_eq!(output1.bytes[0], 0xA1); // LDA IndirectX opcode
    assert_eq!(output1.bytes[1], 0x20);

    // IndirectY with lowercase
    let source2 = "lda ($20),y";
    let result2 = assemble(source2);
    assert!(
        result2.is_ok(),
        "Should handle lowercase ($20),y: {:?}",
        result2
    );
    let output2 = result2.unwrap();
    assert_eq!(output2.bytes[0], 0xB1); // LDA IndirectY opcode
    assert_eq!(output2.bytes[1], 0x20);
}

#[test]
fn test_lowercase_accumulator_mode() {
    // Accumulator mode with lowercase 'a'
    let source = "asl a";
    let result = assemble(source);
    assert!(
        result.is_ok(),
        "Should handle lowercase 'a' for accumulator: {:?}",
        result
    );
    let output = result.unwrap();
    assert_eq!(output.bytes[0], 0x0A); // ASL Accumulator opcode
}

// Test whitespace tolerance around commas and parentheses
#[test]
fn test_whitespace_around_comma_indexed() {
    // Space before comma
    let source1 = "lda $10 ,x";
    let result1 = assemble(source1);
    assert!(
        result1.is_ok(),
        "Should handle space before comma: {:?}",
        result1
    );

    // Space after comma
    let source2 = "lda $10, x";
    let result2 = assemble(source2);
    assert!(
        result2.is_ok(),
        "Should handle space after comma: {:?}",
        result2
    );

    // Spaces both sides
    let source3 = "lda $10 , x";
    let result3 = assemble(source3);
    assert!(
        result3.is_ok(),
        "Should handle spaces around comma: {:?}",
        result3
    );
}

#[test]
fn test_whitespace_in_indirect_modes() {
    // IndirectX with spaces
    let source1 = "lda ( $20 , x )";
    let result1 = assemble(source1);
    assert!(
        result1.is_ok(),
        "Should handle spaces in ($20,x): {:?}",
        result1
    );
    let output1 = result1.unwrap();
    assert_eq!(output1.bytes[0], 0xA1); // LDA IndirectX opcode

    // IndirectY with spaces
    let source2 = "lda ( $20 ) , y";
    let result2 = assemble(source2);
    assert!(
        result2.is_ok(),
        "Should handle spaces in ($20),y: {:?}",
        result2
    );
    let output2 = result2.unwrap();
    assert_eq!(output2.bytes[0], 0xB1); // LDA IndirectY opcode
}

// Test mixed case scenarios
#[test]
fn test_mixed_case_mnemonic_and_register() {
    // Uppercase mnemonic, lowercase register
    let source1 = "LDA $10,x";
    let result1 = assemble(source1);
    assert!(
        result1.is_ok(),
        "Should handle mixed case LDA $10,x: {:?}",
        result1
    );

    // Lowercase mnemonic, uppercase register
    let source2 = "lda $10,X";
    let result2 = assemble(source2);
    assert!(
        result2.is_ok(),
        "Should handle mixed case lda $10,X: {:?}",
        result2
    );

    // Mixed everything
    let source3 = "LdA $10,x";
    let result3 = assemble(source3);
    assert!(
        result3.is_ok(),
        "Should handle mixed case LdA $10,x: {:?}",
        result3
    );
}
