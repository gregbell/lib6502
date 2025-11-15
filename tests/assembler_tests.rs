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
