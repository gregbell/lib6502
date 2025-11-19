//! Lexer unit tests
//!
//! Tests for the assembler lexer (tokenization phase)

use lib6502::assembler::{tokenize, TokenType};

#[test]
fn test_identifier_tokenization() {
    // Test basic identifier (mnemonic)
    let tokens = tokenize("LDA").unwrap();
    assert_eq!(tokens.len(), 2); // LDA + EOF
    assert_eq!(tokens[0].token_type, TokenType::Identifier("LDA".to_string()));
    assert_eq!(tokens[0].line, 1);
    assert_eq!(tokens[0].column, 0);
    assert_eq!(tokens[0].length, 3);

    // Test mixed case (should normalize to uppercase)
    let tokens = tokenize("lda").unwrap();
    assert_eq!(tokens[0].token_type, TokenType::Identifier("LDA".to_string()));

    // Test identifier with underscores and numbers
    let tokens = tokenize("LABEL_123").unwrap();
    assert_eq!(tokens[0].token_type, TokenType::Identifier("LABEL_123".to_string()));

    // Test multiple identifiers
    let tokens = tokenize("LDA STA").unwrap();
    assert_eq!(tokens.len(), 4); // LDA + whitespace + STA + EOF
    assert_eq!(tokens[0].token_type, TokenType::Identifier("LDA".to_string()));
    assert_eq!(tokens[1].token_type, TokenType::Whitespace);
    assert_eq!(tokens[2].token_type, TokenType::Identifier("STA".to_string()));
}

#[test]
fn test_hex_number_tokenization() {
    // Test hex number
    let tokens = tokenize("$42").unwrap();
    assert_eq!(tokens.len(), 2); // $42 + EOF
    assert_eq!(tokens[0].token_type, TokenType::HexNumber(0x42));
    assert_eq!(tokens[0].line, 1);
    assert_eq!(tokens[0].column, 0);
    assert_eq!(tokens[0].length, 3); // $ + 42

    // Test hex with uppercase
    let tokens = tokenize("$ABCD").unwrap();
    assert_eq!(tokens[0].token_type, TokenType::HexNumber(0xABCD));

    // Test hex with lowercase
    let tokens = tokenize("$abcd").unwrap();
    assert_eq!(tokens[0].token_type, TokenType::HexNumber(0xABCD));

    // Test max value
    let tokens = tokenize("$FFFF").unwrap();
    assert_eq!(tokens[0].token_type, TokenType::HexNumber(0xFFFF));
}

#[test]
fn test_binary_number_tokenization() {
    // Test binary number
    let tokens = tokenize("%01000010").unwrap();
    assert_eq!(tokens.len(), 2); // %01000010 + EOF
    assert_eq!(tokens[0].token_type, TokenType::BinaryNumber(66));
    assert_eq!(tokens[0].line, 1);
    assert_eq!(tokens[0].column, 0);
    assert_eq!(tokens[0].length, 9); // % + 01000010

    // Test all zeros
    let tokens = tokenize("%00000000").unwrap();
    assert_eq!(tokens[0].token_type, TokenType::BinaryNumber(0));

    // Test all ones (8-bit)
    let tokens = tokenize("%11111111").unwrap();
    assert_eq!(tokens[0].token_type, TokenType::BinaryNumber(255));

    // Test 16-bit binary
    let tokens = tokenize("%1111111111111111").unwrap();
    assert_eq!(tokens[0].token_type, TokenType::BinaryNumber(65535));
}

#[test]
fn test_decimal_number_tokenization() {
    // Test decimal number
    let tokens = tokenize("42").unwrap();
    assert_eq!(tokens.len(), 2); // 42 + EOF
    assert_eq!(tokens[0].token_type, TokenType::DecimalNumber(42));
    assert_eq!(tokens[0].line, 1);
    assert_eq!(tokens[0].column, 0);
    assert_eq!(tokens[0].length, 2);

    // Test zero
    let tokens = tokenize("0").unwrap();
    assert_eq!(tokens[0].token_type, TokenType::DecimalNumber(0));

    // Test max value
    let tokens = tokenize("65535").unwrap();
    assert_eq!(tokens[0].token_type, TokenType::DecimalNumber(65535));

    // Test leading zeros
    let tokens = tokenize("007").unwrap();
    assert_eq!(tokens[0].token_type, TokenType::DecimalNumber(7));
}

#[test]
fn test_operator_tokenization() {
    // Test all single-character operators
    let tokens = tokenize(":,#$%=().").unwrap();
    assert_eq!(tokens.len(), 10); // 9 operators + EOF

    assert_eq!(tokens[0].token_type, TokenType::Colon);
    assert_eq!(tokens[1].token_type, TokenType::Comma);
    assert_eq!(tokens[2].token_type, TokenType::Hash);
    assert_eq!(tokens[3].token_type, TokenType::Dollar);
    assert_eq!(tokens[4].token_type, TokenType::Percent);
    assert_eq!(tokens[5].token_type, TokenType::Equal);
    assert_eq!(tokens[6].token_type, TokenType::LParen);
    assert_eq!(tokens[7].token_type, TokenType::RParen);
    assert_eq!(tokens[8].token_type, TokenType::Dot);

    // All should have length 1
    for token in tokens.iter().take(9) {
        assert_eq!(token.length, 1);
    }
}

#[test]
fn test_comment_preservation() {
    // Test simple comment
    let tokens = tokenize("; This is a comment").unwrap();
    assert_eq!(tokens.len(), 2); // comment + EOF
    assert_eq!(tokens[0].token_type, TokenType::Comment(" This is a comment".to_string()));
    assert_eq!(tokens[0].line, 1);
    assert_eq!(tokens[0].column, 0);

    // Test comment after code
    let tokens = tokenize("LDA #$42 ; Load accumulator").unwrap();
    assert_eq!(tokens.len(), 7); // LDA + ws + # + $42 + ws + comment + EOF
    if let TokenType::Comment(text) = &tokens[5].token_type {
        assert_eq!(text, " Load accumulator");
    } else {
        panic!("Expected comment token");
    }

    // Test empty comment
    let tokens = tokenize(";").unwrap();
    assert_eq!(tokens[0].token_type, TokenType::Comment("".to_string()));
}

#[test]
fn test_invalid_hex_digit_error() {
    // Test invalid hex digit
    let result = tokenize("$ZZ");
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);

    // Check error type
    if let lib6502::assembler::LexerError::InvalidHexDigit { ch, line, column } = errors[0] {
        assert_eq!(ch, 'Z');
        assert_eq!(line, 1);
        assert_eq!(column, 1);
    } else {
        panic!("Expected InvalidHexDigit error");
    }
}

#[test]
fn test_number_overflow_error() {
    // Test decimal overflow
    let result = tokenize("99999");
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert!(!errors.is_empty());

    // Check error type
    if let lib6502::assembler::LexerError::NumberTooLarge { value, max, .. } = &errors[0] {
        assert_eq!(value, "99999");
        assert_eq!(*max, 65535);
    } else {
        panic!("Expected NumberTooLarge error");
    }

    // Test hex overflow
    let result = tokenize("$FFFFF");
    assert!(result.is_err());
}

#[test]
fn test_line_column_tracking() {
    // Test multiline source
    let source = "LDA #$42\nSTA $1000\n";
    let tokens = tokenize(source).unwrap();

    // Find LDA token (should be on line 1, column 0)
    assert_eq!(tokens[0].token_type, TokenType::Identifier("LDA".to_string()));
    assert_eq!(tokens[0].line, 1);
    assert_eq!(tokens[0].column, 0);

    // Find STA token (should be on line 2, column 0)
    // tokens: LDA + ws + # + $42 + newline + STA + ws + $1000 + newline + EOF
    // Index:  0    1    2   3     4         5     6    7       8         9
    assert_eq!(tokens[5].token_type, TokenType::Identifier("STA".to_string()));
    assert_eq!(tokens[5].line, 2);
    assert_eq!(tokens[5].column, 0);

    // Check newline tokens
    assert_eq!(tokens[4].token_type, TokenType::Newline);
    assert_eq!(tokens[4].line, 1);

    assert_eq!(tokens[8].token_type, TokenType::Newline);
    assert_eq!(tokens[8].line, 2);
}

#[test]
fn test_whitespace_handling() {
    // Test spaces
    let tokens = tokenize("LDA  STA").unwrap();
    assert_eq!(tokens[1].token_type, TokenType::Whitespace);
    assert_eq!(tokens[1].length, 2);

    // Test tabs
    let tokens = tokenize("LDA\t\tSTA").unwrap();
    assert_eq!(tokens[1].token_type, TokenType::Whitespace);
    assert_eq!(tokens[1].length, 2);

    // Test mixed
    let tokens = tokenize("LDA \t STA").unwrap();
    assert_eq!(tokens[1].token_type, TokenType::Whitespace);
    assert_eq!(tokens[1].length, 3);
}

#[test]
fn test_newline_normalization() {
    // Test LF
    let tokens = tokenize("LDA\nSTA").unwrap();
    assert_eq!(tokens[1].token_type, TokenType::Newline);
    assert_eq!(tokens[1].length, 1);

    // Test CRLF
    let tokens = tokenize("LDA\r\nSTA").unwrap();
    assert_eq!(tokens[1].token_type, TokenType::Newline);
    assert_eq!(tokens[1].length, 2);

    // Test CR (old Mac)
    let tokens = tokenize("LDA\rSTA").unwrap();
    assert_eq!(tokens[1].token_type, TokenType::Newline);
    assert_eq!(tokens[1].length, 1);
}

#[test]
fn test_complete_instruction() {
    // Test a complete instruction with all token types
    let tokens = tokenize("START: LDA #$42 ; Load value").unwrap();

    // Expected tokens: START + : + ws + LDA + ws + # + $42 + ws + comment + EOF
    assert_eq!(tokens[0].token_type, TokenType::Identifier("START".to_string()));
    assert_eq!(tokens[1].token_type, TokenType::Colon);
    assert_eq!(tokens[2].token_type, TokenType::Whitespace);
    assert_eq!(tokens[3].token_type, TokenType::Identifier("LDA".to_string()));
    assert_eq!(tokens[4].token_type, TokenType::Whitespace);
    assert_eq!(tokens[5].token_type, TokenType::Hash);
    assert_eq!(tokens[6].token_type, TokenType::HexNumber(0x42));
    assert_eq!(tokens[7].token_type, TokenType::Whitespace);
    if let TokenType::Comment(text) = &tokens[8].token_type {
        assert_eq!(text, " Load value");
    } else {
        panic!("Expected comment token");
    }
    assert_eq!(tokens[9].token_type, TokenType::Eof);
}

#[test]
fn test_eof_token() {
    // Every tokenize() call should end with EOF
    let tokens = tokenize("").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token_type, TokenType::Eof);

    let tokens = tokenize("LDA").unwrap();
    assert_eq!(tokens[tokens.len() - 1].token_type, TokenType::Eof);
}
