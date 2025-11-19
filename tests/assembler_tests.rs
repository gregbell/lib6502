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
    assert_eq!(symbol.value, 0, "START should be at address 0");

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
        symbol.value, 5,
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
    assert_eq!(symbol.unwrap().value, 0);

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
        symbol.unwrap().value,
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

// Tests for branch instructions with numeric addresses
// Discovered via Klaus round-trip test - branches should accept numeric
// target addresses and automatically calculate relative offsets

#[test]
fn test_branch_with_numeric_address_forward() {
    // Branch forward to absolute address
    let source = r#"
        .org $1000
        BEQ $1010
    "#;

    let result = assemble(source).unwrap();

    // BEQ opcode is $F0
    // Instruction at $1000, next instruction at $1002
    // Target is $1010, offset = $1010 - $1002 = $000E = 14
    assert_eq!(result.bytes, vec![0xF0, 0x0E]);
}

#[test]
fn test_branch_with_numeric_address_backward() {
    // Branch backward to absolute address
    let source = r#"
        .org $1010
        BNE $1000
    "#;

    let result = assemble(source).unwrap();

    // BNE opcode is $D0
    // Instruction at $1010, next instruction at $1012
    // Target is $1000, offset = $1000 - $1012 = -18 = $EE (two's complement)
    assert_eq!(result.bytes, vec![0xD0, 0xEE]);
}

#[test]
fn test_branch_with_numeric_address_zero_offset() {
    // Branch to the next instruction (offset = 0)
    let source = r#"
        .org $1000
        BCC $1002
    "#;

    let result = assemble(source).unwrap();

    // BCC opcode is $90
    // Instruction at $1000, next instruction at $1002
    // Target is $1002, offset = $1002 - $1002 = 0
    assert_eq!(result.bytes, vec![0x90, 0x00]);
}

#[test]
fn test_branch_with_numeric_address_max_forward() {
    // Branch with maximum forward offset (+127)
    let source = r#"
        .org $1000
        BPL $1081
    "#;

    let result = assemble(source).unwrap();

    // BPL opcode is $10
    // Instruction at $1000, next instruction at $1002
    // Target is $1081, offset = $1081 - $1002 = $007F = 127
    assert_eq!(result.bytes, vec![0x10, 0x7F]);
}

#[test]
fn test_branch_with_numeric_address_max_backward() {
    // Branch with maximum backward offset (-128)
    let source = r#"
        .org $1000
        BMI $0F82
    "#;

    let result = assemble(source).unwrap();

    // BMI opcode is $30
    // Instruction at $1000, next instruction at $1002
    // Target is $0F82, offset = $0F82 - $1002 = -128 = $80 (two's complement)
    assert_eq!(result.bytes, vec![0x30, 0x80]);
}

#[test]
fn test_branch_with_numeric_address_out_of_range_forward() {
    // Branch target too far forward (> +127)
    let source = r#"
        .org $1000
        BEQ $1082
    "#;

    let result = assemble(source);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].error_type, ErrorType::RangeError);
    assert!(errors[0].message.contains("out of range"));
}

#[test]
fn test_branch_with_numeric_address_out_of_range_backward() {
    // Branch target too far backward (< -128)
    let source = r#"
        .org $1000
        BNE $0F81
    "#;

    let result = assemble(source);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].error_type, ErrorType::RangeError);
    assert!(errors[0].message.contains("out of range"));
}

#[test]
fn test_all_branch_instructions_with_numeric_addresses() {
    // Test all 8 branch instructions to ensure they all support numeric addresses
    let source = r#"
        .org $2000
        BCC $2010
        BCS $2010
        BEQ $2010
        BMI $2010
        BNE $2010
        BPL $2010
        BVC $2010
        BVS $2010
    "#;

    let result = assemble(source).unwrap();

    // Each branch is 2 bytes (opcode + offset)
    assert_eq!(result.bytes.len(), 16);

    // All should have offset $0E (target $2010 - next instruction address)
    // Opcodes: BCC=$90, BCS=$B0, BEQ=$F0, BMI=$30, BNE=$D0, BPL=$10, BVC=$50, BVS=$70
    assert_eq!(
        result.bytes,
        vec![
            0x90, 0x0E, // BCC
            0xB0, 0x0C, // BCS (from $2002)
            0xF0, 0x0A, // BEQ (from $2004)
            0x30, 0x08, // BMI (from $2006)
            0xD0, 0x06, // BNE (from $2008)
            0x10, 0x04, // BPL (from $200A)
            0x50, 0x02, // BVC (from $200C)
            0x70, 0x00, // BVS (from $200E, target is $2010)
        ]
    );
}

#[test]
fn test_branch_numeric_address_with_hex_prefix() {
    // Ensure hex prefix works ($XXXX format)
    let source = r#"
        .org $1000
        BEQ $1010
    "#;

    let result = assemble(source).unwrap();
    assert_eq!(result.bytes, vec![0xF0, 0x0E]);
}

#[test]
fn test_branch_numeric_address_decimal_format() {
    // Test decimal format (without $ prefix)
    let source = r#"
        .org $1000
        BEQ 4112
    "#; // 4112 decimal = $1010 hex

    let result = assemble(source).unwrap();
    assert_eq!(result.bytes, vec![0xF0, 0x0E]);
}

#[test]
fn test_branch_still_works_with_labels() {
    // Ensure label-based branches still work (existing functionality)
    let source = r#"
.org $1000
START:
    NOP
LOOP:
    NOP
    BNE START
    RTS
"#;

    let result = assemble(source).unwrap();
    // START: NOP ($1000-$1001)
    // LOOP: NOP ($1001-$1002)
    // BNE START ($1002-$1004, offset back to $1000)
    // Offset = $1000 - $1004 = -4 = $FC (two's complement)
    // RTS ($1004-$1005)
    assert_eq!(result.bytes, vec![0xEA, 0xEA, 0xD0, 0xFC, 0x60]);
}

// T016: Integration test for basic constant definition
#[test]
fn test_constant_definition_basic() {
    let source = r#"
MAX = 255
SCREEN = $4000
BITS = %11110000

START:
    NOP
"#;

    let result = assemble(source);
    assert!(result.is_ok(), "Assembly with constants should succeed");

    let output = result.unwrap();

    // Verify constants are in symbol table
    let max_symbol = output.lookup_symbol("MAX");
    assert!(
        max_symbol.is_some(),
        "MAX constant should be in symbol table"
    );
    let max_symbol = max_symbol.unwrap();
    assert_eq!(max_symbol.value, 255);
    assert_eq!(max_symbol.kind, lib6502::assembler::SymbolKind::Constant);

    let screen_symbol = output.lookup_symbol("SCREEN");
    assert!(
        screen_symbol.is_some(),
        "SCREEN constant should be in symbol table"
    );
    let screen_symbol = screen_symbol.unwrap();
    assert_eq!(screen_symbol.value, 0x4000);
    assert_eq!(screen_symbol.kind, lib6502::assembler::SymbolKind::Constant);

    let bits_symbol = output.lookup_symbol("BITS");
    assert!(
        bits_symbol.is_some(),
        "BITS constant should be in symbol table"
    );
    let bits_symbol = bits_symbol.unwrap();
    assert_eq!(bits_symbol.value, 0b11110000);
    assert_eq!(bits_symbol.kind, lib6502::assembler::SymbolKind::Constant);

    // Verify label is also in symbol table with correct kind
    let start_symbol = output.lookup_symbol("START");
    assert!(
        start_symbol.is_some(),
        "START label should be in symbol table"
    );
    let start_symbol = start_symbol.unwrap();
    assert_eq!(start_symbol.value, 0); // Address 0
    assert_eq!(start_symbol.kind, lib6502::assembler::SymbolKind::Label);

    // Verify the NOP instruction assembled correctly
    assert_eq!(output.bytes, vec![0xEA]); // NOP opcode
}

// T020: Integration test for immediate addressing with constant (LDA #MAX)
#[test]
fn test_constant_usage_immediate() {
    let source = r#"
MAX = 255
SMALL = 42

    LDA #MAX
    LDX #SMALL
    LDY #10
"#;

    let result = assemble(source);
    assert!(
        result.is_ok(),
        "Assembly with constants in immediate mode should succeed"
    );

    let output = result.unwrap();

    // Verify assembled bytes
    // LDA #255 = A9 FF
    // LDX #42 = A2 2A
    // LDY #10 = A0 0A
    assert_eq!(output.bytes, vec![0xA9, 0xFF, 0xA2, 0x2A, 0xA0, 0x0A]);
}

// T021: Integration test for zero page with constant (LDA ZP_TEMP)
#[test]
fn test_constant_usage_zero_page() {
    let source = r#"
ZP_TEMP = $20
ZP_PTR = $30

    LDA ZP_TEMP
    STA ZP_PTR
"#;

    let result = assemble(source);
    assert!(
        result.is_ok(),
        "Assembly with constants in zero page mode should succeed"
    );

    let output = result.unwrap();

    // Verify assembled bytes
    // LDA $20 = A5 20 (zero page)
    // STA $30 = 85 30 (zero page)
    assert_eq!(output.bytes, vec![0xA5, 0x20, 0x85, 0x30]);
}

// T022: Integration test for absolute with constant (STA SCREEN)
#[test]
fn test_constant_usage_absolute() {
    let source = r#"
SCREEN = $4000
IO_PORT = $8000

    LDA #$42
    STA SCREEN
    STX IO_PORT
"#;

    let result = assemble(source);
    assert!(
        result.is_ok(),
        "Assembly with constants in absolute mode should succeed"
    );

    let output = result.unwrap();

    // Verify assembled bytes
    // LDA #$42 = A9 42
    // STA $4000 = 8D 00 40 (absolute, little-endian)
    // STX $8000 = 8E 00 80 (absolute, little-endian)
    assert_eq!(
        output.bytes,
        vec![0xA9, 0x42, 0x8D, 0x00, 0x40, 0x8E, 0x00, 0x80]
    );
}

// T023: Integration test for indexed with constant (LDA IO_BASE,X)
#[test]
fn test_constant_usage_indexed() {
    let source = r#"
IO_BASE = $200
BUFFER = $50

    LDA IO_BASE,X
    STA IO_BASE,Y
    LDA BUFFER,X
"#;

    let result = assemble(source);
    assert!(
        result.is_ok(),
        "Assembly with constants in indexed mode should succeed"
    );

    let output = result.unwrap();

    // Verify assembled bytes
    // LDA $200,X = BD 00 02 (absolute,X - little-endian)
    // STA $200,Y = 99 00 02 (absolute,Y - little-endian)
    // LDA $50,X = B5 50 (zero page,X)
    assert_eq!(
        output.bytes,
        vec![0xBD, 0x00, 0x02, 0x99, 0x00, 0x02, 0xB5, 0x50]
    );
}

// T031: Integration test for undefined constant error
#[test]
fn test_error_undefined_constant() {
    let source = r#"
    LDA #MISSING
"#;

    let result = assemble(source);
    assert!(result.is_err(), "Should fail with undefined constant");

    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].error_type, ErrorType::UndefinedLabel); // Undefined symbol
    assert!(errors[0].message.contains("MISSING"));
}

// T032: Integration test for duplicate constant error
#[test]
fn test_error_duplicate_constant() {
    let source = r#"
MAX = 100
MAX = 200
"#;

    let result = assemble(source);
    assert!(result.is_err(), "Should fail with duplicate constant");

    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].error_type, ErrorType::DuplicateConstant);
    assert!(errors[0].message.contains("MAX"));
    assert!(errors[0].message.contains("previously defined"));
}

// T033: Integration test for name collision error (constant then label)
#[test]
fn test_error_name_collision_constant_then_label() {
    let source = r#"
FOO = 42
FOO:
    NOP
"#;

    let result = assemble(source);
    assert!(result.is_err(), "Should fail with name collision");

    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].error_type, ErrorType::NameCollision);
    assert!(errors[0].message.contains("FOO"));
    assert!(errors[0].message.contains("already defined as a constant"));
}

// T034: Integration test for name collision error (label then constant)
#[test]
fn test_error_name_collision_label_then_constant() {
    let source = r#"
BAR:
    NOP
BAR = 100
"#;

    let result = assemble(source);
    assert!(result.is_err(), "Should fail with name collision");

    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].error_type, ErrorType::NameCollision);
    assert!(errors[0].message.contains("BAR"));
    assert!(errors[0].message.contains("already defined as a label"));
}

// T038: Integration test for mixed constants and labels in same program
#[test]
fn test_mixed_constants_and_labels() {
    let source = r#"
; Define constants
MAX_LIVES = 3
SCREEN_ADDR = $4000
ZP_TEMP = $20

; Define labels and code
START:
    LDA #MAX_LIVES
    STA ZP_TEMP

LOOP:
    LDA ZP_TEMP
    STA SCREEN_ADDR
    JMP LOOP
"#;

    let result = assemble(source);
    assert!(
        result.is_ok(),
        "Should successfully assemble mixed constants and labels"
    );

    let output = result.unwrap();

    // Verify constants are in symbol table
    let max_lives = output.lookup_symbol("MAX_LIVES").unwrap();
    assert_eq!(max_lives.kind, lib6502::assembler::SymbolKind::Constant);
    assert_eq!(max_lives.value, 3);

    let screen = output.lookup_symbol("SCREEN_ADDR").unwrap();
    assert_eq!(screen.kind, lib6502::assembler::SymbolKind::Constant);
    assert_eq!(screen.value, 0x4000);

    let zp = output.lookup_symbol("ZP_TEMP").unwrap();
    assert_eq!(zp.kind, lib6502::assembler::SymbolKind::Constant);
    assert_eq!(zp.value, 0x20);

    // Verify labels are in symbol table
    let start = output.lookup_symbol("START").unwrap();
    assert_eq!(start.kind, lib6502::assembler::SymbolKind::Label);
    assert_eq!(start.value, 0); // Address 0

    let loop_label = output.lookup_symbol("LOOP").unwrap();
    assert_eq!(loop_label.kind, lib6502::assembler::SymbolKind::Label);
    // LOOP address depends on instruction sizes - just verify it exists

    // Verify program assembled successfully
    assert!(!output.bytes.is_empty());
    assert_eq!(output.bytes[0], 0xA9); // LDA immediate
    assert_eq!(output.bytes[1], 0x03); // value 3
}

// T039: Integration test for constants in all addressing modes
#[test]
fn test_constants_all_addressing_modes() {
    let source = r#"
; Define test constants
IMM_VAL = 42
ZP_ADDR = $50
ABS_ADDR = $2000
ZP_IDX = $60
ABS_IDX = $3000

    ; Immediate mode
    LDA #IMM_VAL

    ; Zero page
    STA ZP_ADDR

    ; Absolute
    STA ABS_ADDR

    ; Zero page indexed
    LDA ZP_IDX,X
    LDX ZP_IDX,Y  ; LDX supports ZP,Y

    ; Absolute indexed
    STA ABS_IDX,X
    STA ABS_IDX,Y
"#;

    let result = assemble(source);
    if result.is_err() {
        eprintln!("Assembly errors: {:#?}", result.as_ref().unwrap_err());
    }
    assert!(
        result.is_ok(),
        "Should successfully use constants in all addressing modes"
    );

    let output = result.unwrap();

    // Verify all constants are in symbol table
    assert_eq!(output.lookup_symbol("IMM_VAL").unwrap().value, 42);
    assert_eq!(output.lookup_symbol("ZP_ADDR").unwrap().value, 0x50);
    assert_eq!(output.lookup_symbol("ABS_ADDR").unwrap().value, 0x2000);
    assert_eq!(output.lookup_symbol("ZP_IDX").unwrap().value, 0x60);
    assert_eq!(output.lookup_symbol("ABS_IDX").unwrap().value, 0x3000);

    // Verify assembled bytes
    let expected = vec![
        0xA9, 0x2A, // LDA #42
        0x85, 0x50, // STA $50 (zero page)
        0x8D, 0x00, 0x20, // STA $2000 (absolute)
        0xB5, 0x60, // LDA $60,X (zero page,X)
        0xB6, 0x60, // LDX $60,Y (zero page,Y)
        0x9D, 0x00, 0x30, // STA $3000,X (absolute,X)
        0x99, 0x00, 0x30, // STA $3000,Y (absolute,Y)
    ];
    assert_eq!(output.bytes, expected);
}

// T043: Integration test for complex program with many constants
#[test]
fn test_complex_program_with_many_constants() {
    let source = r#"
; I/O addresses
UART_DATA = $8000
UART_STATUS = $8001
SCREEN_BASE = $4000
KEYBOARD_PORT = $8002

; Character constants
CHAR_CR = 13
CHAR_LF = 10
CHAR_SPACE = 32

; System constants
MAX_BUFFER = 128
ZP_PTR = $40
ZP_COUNTER = $42

; Program
START:
    LDX #0
    STX ZP_COUNTER

MAIN_LOOP:
    LDA UART_STATUS
    AND #1
    BEQ MAIN_LOOP

    LDA UART_DATA
    CMP #CHAR_CR
    BEQ HANDLE_CR
    CMP #CHAR_LF
    BEQ HANDLE_LF

    STA SCREEN_BASE,X
    INX
    CPX #MAX_BUFFER
    BNE MAIN_LOOP

HANDLE_CR:
    LDA #CHAR_SPACE
    STA SCREEN_BASE,X
    JMP MAIN_LOOP

HANDLE_LF:
    INX
    JMP MAIN_LOOP
"#;

    let result = assemble(source);
    if result.is_err() {
        eprintln!("Assembly errors: {:#?}", result.as_ref().unwrap_err());
    }
    assert!(
        result.is_ok(),
        "Complex program with many constants should assemble successfully"
    );

    let output = result.unwrap();

    // Verify all constants exist
    assert_eq!(output.lookup_symbol("UART_DATA").unwrap().value, 0x8000);
    assert_eq!(output.lookup_symbol("CHAR_CR").unwrap().value, 13);
    assert_eq!(output.lookup_symbol("MAX_BUFFER").unwrap().value, 128);
    assert_eq!(output.lookup_symbol("ZP_PTR").unwrap().value, 0x40);

    // Verify all constants have correct kind
    assert_eq!(
        output.lookup_symbol("UART_DATA").unwrap().kind,
        lib6502::assembler::SymbolKind::Constant
    );

    // Verify all labels exist
    assert!(output.lookup_symbol("START").is_some());
    assert!(output.lookup_symbol("MAIN_LOOP").is_some());
    assert!(output.lookup_symbol("HANDLE_CR").is_some());

    // Verify labels have correct kind
    assert_eq!(
        output.lookup_symbol("START").unwrap().kind,
        lib6502::assembler::SymbolKind::Label
    );

    // Verify program assembled successfully
    assert!(
        !output.bytes.is_empty(),
        "Program should have assembled code"
    );
}

// T044: Integration test for backward compatibility (existing code without constants)
#[test]
fn test_backward_compatibility_no_constants() {
    // This is an existing test pattern that should continue to work unchanged
    let source = r#"
.org $8000

START:
    LDA #$42
    STA $2000
    LDX #$00

LOOP:
    LDA $2000,X
    STA $4000,X
    INX
    CPX #$10
    BNE LOOP

    JMP START
"#;

    let result = assemble(source);
    assert!(
        result.is_ok(),
        "Existing assembly code without constants should work unchanged"
    );

    let output = result.unwrap();

    // Verify labels still work as before
    let start = output.lookup_symbol("START").unwrap();
    assert_eq!(start.kind, lib6502::assembler::SymbolKind::Label);
    assert_eq!(start.value, 0x8000);

    let loop_label = output.lookup_symbol("LOOP").unwrap();
    assert_eq!(loop_label.kind, lib6502::assembler::SymbolKind::Label);

    // Verify code still assembles correctly
    assert!(!output.bytes.is_empty());
    assert_eq!(output.bytes[0], 0xA9); // LDA immediate
    assert_eq!(output.bytes[1], 0x42); // value 0x42
}

// =============================================================================
// Low/High Byte Operators (<symbol and >symbol)
// =============================================================================

#[test]
fn test_low_byte_operator() {
    let source = r#"
        .org $8000
isr:
        NOP
start:
        LDA #<isr
        STA $FFFE
    "#;

    let result = assemble(source).unwrap();

    // LDA #<isr should load the low byte of the isr address ($8000)
    // Low byte of $8000 is $00
    assert_eq!(result.bytes[1], 0xA9); // LDA immediate opcode
    assert_eq!(result.bytes[2], 0x00); // Low byte of $8000
}

#[test]
fn test_high_byte_operator() {
    let source = r#"
        .org $8000
isr:
        NOP
start:
        LDA #>isr
        STA $FFFF
    "#;

    let result = assemble(source).unwrap();

    // LDA #>isr should load the high byte of the isr address ($8000)
    // High byte of $8000 is $80
    assert_eq!(result.bytes[1], 0xA9); // LDA immediate opcode
    assert_eq!(result.bytes[2], 0x80); // High byte of $8000
}

#[test]
fn test_low_high_byte_operators_together() {
    let source = r#"
        .org $8000
vector:
        NOP
        NOP
main:
        LDA #<vector
        STA $FFFE
        LDA #>vector
        STA $FFFF
    "#;

    let result = assemble(source).unwrap();

    // vector is at $8000
    // First LDA #<vector loads $00 (low byte of $8000)
    assert_eq!(result.bytes[2], 0xA9); // First LDA immediate opcode
    assert_eq!(result.bytes[3], 0x00); // Low byte

    // Second LDA #>vector loads $80 (high byte of $8000)
    assert_eq!(result.bytes[7], 0xA9); // Second LDA immediate opcode
    assert_eq!(result.bytes[8], 0x80); // High byte
}

#[test]
fn test_low_high_byte_operators_with_different_address() {
    let source = r#"
        .org $1234
handler:
        RTI
main:
        LDA #<handler
        LDX #>handler
    "#;

    let result = assemble(source).unwrap();

    // handler is at $1234
    // LDA #<handler loads $34 (low byte of $1234)
    // LDX #>handler loads $12 (high byte of $1234)
    assert_eq!(result.bytes[1], 0xA9); // LDA immediate opcode
    assert_eq!(result.bytes[2], 0x34); // Low byte of $1234

    assert_eq!(result.bytes[3], 0xA2); // LDX immediate opcode
    assert_eq!(result.bytes[4], 0x12); // High byte of $1234
}

// =============================================================================
// Integration Tests - UART Patterns
// =============================================================================

#[test]
fn test_uart_hello_pattern() {
    // This tests the pattern used in uart-hello.asm
    let source = r#"
        .org $8000
UART_DATA = $A000

        LDX #$00
print_loop:
        LDA message,X
        BEQ done
        STA UART_DATA
        INX
        JMP print_loop
done:
        BRK

message:
        .byte "Hello, 6502!"
        .byte $0D, $0A
        .byte $00
    "#;

    let result = assemble(source).unwrap();

    // Verify the message bytes are correct
    // The message starts after the code (which we need to calculate)
    // We can search for "Hello" in the bytes
    let hello = b"Hello, 6502!";
    let mut found = false;
    for i in 0..result.bytes.len() - hello.len() {
        if &result.bytes[i..i + hello.len()] == hello {
            found = true;
            // Verify the CR, LF, and null terminator follow
            assert_eq!(result.bytes[i + hello.len()], 0x0D);
            assert_eq!(result.bytes[i + hello.len() + 1], 0x0A);
            assert_eq!(result.bytes[i + hello.len() + 2], 0x00);
            break;
        }
    }
    assert!(found, "Message string not found in assembled bytes");
}

#[test]
fn test_uart_echo_pattern() {
    // This tests the pattern used in uart-echo.asm
    let source = r#"
        .org $8000
UART_DATA    = $A000
UART_COMMAND = $A002
IRQ_EN       = $02

        LDA #<isr
        STA $FFFE
        LDA #>isr
        STA $FFFF

        LDA #IRQ_EN
        STA UART_COMMAND

        CLI

idle_loop:
        NOP
        JMP idle_loop

isr:
        LDA UART_DATA
        STA UART_DATA
        RTI
    "#;

    let result = assemble(source).unwrap();

    // Just verify it assembles successfully
    // The first instruction should be LDA #<isr
    assert_eq!(result.bytes[0], 0xA9); // LDA immediate
                                       // The low byte of isr depends on where it's assembled, but we know it assembles
    assert!(result.bytes.len() > 20); // Should have reasonable size
}
