//! Integration tests for the 6502 assembler

use cpu6502::assembler::{assemble, ErrorType};

// T028: Integration test for single instruction assembly (LDA #$42)
#[test]
fn test_single_instruction_assembly() {
    let source = "LDA #$42";

    let result = assemble(source);
    assert!(result.is_ok(), "Assembly should succeed");

    let output = result.unwrap();
    assert_eq!(output.bytes, vec![0xA9, 0x42], "Should assemble to LDA immediate");
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
        assert!(result.is_ok(), "Should handle case and whitespace: '{}'", source);
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
    assert!(error.column >= 0, "Should have column number");
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
    assert!(errors.len() >= 2, "Should collect multiple errors, got {}", errors.len());

    // Should have at least the two invalid mnemonics
    let invalid_mnemonics = errors.iter()
        .filter(|e| e.error_type == ErrorType::InvalidMnemonic)
        .count();
    assert!(invalid_mnemonics >= 2, "Should detect at least 2 invalid mnemonics");
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
