//! Tests for new assembler features: string literals, low/high byte operators

use lib6502::assembler::assemble;

#[test]
fn test_string_literal_in_byte_directive() {
    let source = r#"
        .org $8000
        .byte "Hello"
    "#;

    let result = assemble(source).unwrap();

    // "Hello" = 5 bytes (H, e, l, l, o)
    assert_eq!(result.bytes.len(), 5);
    assert_eq!(result.bytes[0], b'H');
    assert_eq!(result.bytes[1], b'e');
    assert_eq!(result.bytes[2], b'l');
    assert_eq!(result.bytes[3], b'l');
    assert_eq!(result.bytes[4], b'o');
}

#[test]
fn test_string_literal_with_escape_sequences() {
    let source = r#"
        .org $8000
        .byte "Line1\nLine2\tTab"
    "#;

    let result = assemble(source).unwrap();

    // "Line1\nLine2\tTab" = L, i, n, e, 1, \n, L, i, n, e, 2, \t, T, a, b
    assert_eq!(result.bytes.len(), 15);
    assert_eq!(result.bytes[0], b'L');
    assert_eq!(result.bytes[5], b'\n');
    assert_eq!(result.bytes[11], b'\t');
}

#[test]
fn test_string_literal_with_numbers() {
    let source = r#"
        .org $8000
        .byte "Hello", $0D, $0A, "World"
    "#;

    let result = assemble(source).unwrap();

    // "Hello" + $0D + $0A + "World" = 5 + 1 + 1 + 5 = 12 bytes
    assert_eq!(result.bytes.len(), 12);
    assert_eq!(result.bytes[0], b'H');
    assert_eq!(result.bytes[4], b'o');
    assert_eq!(result.bytes[5], 0x0D);
    assert_eq!(result.bytes[6], 0x0A);
    assert_eq!(result.bytes[7], b'W');
    assert_eq!(result.bytes[11], b'd');
}

#[test]
fn test_empty_string_literal() {
    let source = r#"
        .org $8000
        .byte ""
    "#;

    let result = assemble(source).unwrap();

    // Empty string contributes 0 bytes
    assert_eq!(result.bytes.len(), 0);
}

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

#[test]
fn test_string_in_word_directive_should_error() {
    let source = r#"
        .org $8000
        .word "test"
    "#;

    let result = assemble(source);

    // String literals should not be allowed in .word directive
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors[0]
        .message
        .contains("String literals are not supported in .word directive"));
}

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
