//! Comprehensive tests for assembler directive enhancements
//!
//! Tests cover:
//! - Label references in .byte and .word directives
//! - ROM image generation with to_rom_image()
//! - Multiple segments and address layout
//! - Error handling for undefined symbols in directives

use lib6502::assembler::{assemble, ErrorType};

// =============================================================================
// Label References in .word Directive
// =============================================================================

#[test]
fn test_word_directive_with_label_reference() {
    let source = r#"
START:
    LDA #$42
    RTS

.org $FFFC
.word START
"#;

    let result = assemble(source);
    assert!(result.is_ok(), "Should assemble successfully");

    let output = result.unwrap();

    // Check bytes
    assert_eq!(output.bytes.len(), 5); // LDA + RTS + .word (2 bytes)
    assert_eq!(output.bytes[0], 0xA9); // LDA #$42
    assert_eq!(output.bytes[1], 0x42);
    assert_eq!(output.bytes[2], 0x60); // RTS
    assert_eq!(output.bytes[3], 0x00); // Low byte of START ($0000)
    assert_eq!(output.bytes[4], 0x00); // High byte of START

    // Check symbol table
    let start_addr = output.lookup_symbol_addr("START");
    assert_eq!(start_addr, Some(0x0000));
}

#[test]
fn test_word_directive_with_label_at_origin() {
    let source = r#"
.org $8000
START:
    NOP
    NOP

.org $FFFC
.word START
"#;

    let output = assemble(source).unwrap();

    // Check that START is at $8000
    assert_eq!(output.lookup_symbol_addr("START"), Some(0x8000));

    // Check reset vector points to START
    // Last two bytes should be $00 $80 (little-endian)
    assert_eq!(output.bytes[2], 0x00); // Low byte
    assert_eq!(output.bytes[3], 0x80); // High byte
}

#[test]
fn test_word_directive_with_multiple_labels() {
    let source = r#"
RESET:
    JMP MAIN

NMI:
    RTI

IRQ:
    RTI

MAIN:
    LDA #$00
    BRK

.org $FFFA
.word NMI
.word RESET
.word IRQ
"#;

    let output = assemble(source).unwrap();

    // Check vectors
    let nmi_addr = output.lookup_symbol_addr("NMI").unwrap();
    let reset_addr = output.lookup_symbol_addr("RESET").unwrap();
    let irq_addr = output.lookup_symbol_addr("IRQ").unwrap();

    // Find where .word directives start in bytes
    let bytes_len = output.bytes.len();

    // Last 6 bytes should be the three vectors
    assert_eq!(output.bytes[bytes_len - 6], (nmi_addr & 0xFF) as u8);
    assert_eq!(output.bytes[bytes_len - 5], ((nmi_addr >> 8) & 0xFF) as u8);
    assert_eq!(output.bytes[bytes_len - 4], (reset_addr & 0xFF) as u8);
    assert_eq!(
        output.bytes[bytes_len - 3],
        ((reset_addr >> 8) & 0xFF) as u8
    );
    assert_eq!(output.bytes[bytes_len - 2], (irq_addr & 0xFF) as u8);
    assert_eq!(output.bytes[bytes_len - 1], ((irq_addr >> 8) & 0xFF) as u8);
}

#[test]
fn test_word_directive_with_undefined_label() {
    let source = r#"
.org $FFFC
.word UNDEFINED_LABEL
"#;

    let result = assemble(source);
    assert!(result.is_err(), "Should fail with undefined label");

    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].error_type, ErrorType::UndefinedLabel);
    assert!(errors[0].message.contains("UNDEFINED_LABEL"));
}

#[test]
fn test_word_directive_mixed_literals_and_labels() {
    let source = r#"
START:
    NOP

.org $1000
.word START, $ABCD, START, $1234
"#;

    let output = assemble(source).unwrap();

    // Find .word bytes
    let word_bytes = &output.bytes[1..]; // Skip the NOP

    assert_eq!(word_bytes[0], 0x00); // START low
    assert_eq!(word_bytes[1], 0x00); // START high
    assert_eq!(word_bytes[2], 0xCD); // $ABCD low
    assert_eq!(word_bytes[3], 0xAB); // $ABCD high
    assert_eq!(word_bytes[4], 0x00); // START low
    assert_eq!(word_bytes[5], 0x00); // START high
    assert_eq!(word_bytes[6], 0x34); // $1234 low
    assert_eq!(word_bytes[7], 0x12); // $1234 high
}

// =============================================================================
// Label References in .byte Directive
// =============================================================================

#[test]
fn test_byte_directive_with_constant() {
    let source = r#"
CHAR_A = 65
CHAR_B = 66

.byte CHAR_A, CHAR_B, $43
"#;

    let output = assemble(source).unwrap();
    assert_eq!(output.bytes, vec![65, 66, 0x43]);
}

#[test]
fn test_byte_directive_with_label_low_byte() {
    let source = r#"
.org $00FF
START:
    NOP

.org $0000
.byte START
"#;

    let output = assemble(source).unwrap();

    // START is at $00FF, so low byte should be $FF
    assert_eq!(output.bytes[1], 0xFF);
}

#[test]
fn test_byte_directive_with_label_exceeding_byte_range() {
    let source = r#"
.org $8000
START:
    NOP

.org $0000
.byte START
"#;

    let result = assemble(source);
    assert!(result.is_err(), "Should fail when label value > $FF");

    let errors = result.unwrap_err();
    assert_eq!(errors[0].error_type, ErrorType::RangeError);
    assert!(errors[0].message.contains("exceeds 8-bit range"));
}

#[test]
fn test_byte_directive_with_undefined_symbol() {
    let source = ".byte UNDEFINED";

    let result = assemble(source);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert_eq!(errors[0].error_type, ErrorType::UndefinedLabel);
}

// =============================================================================
// ROM Image Generation
// =============================================================================

#[test]
fn test_to_rom_image_simple() {
    let source = r#"
.org $8000
    LDA #$42
    STA $00
"#;

    let output = assemble(source).unwrap();
    let rom = output.to_rom_image(0xFF);

    // ROM should start at $8000 and contain 4 bytes
    assert_eq!(rom.len(), 4);
    assert_eq!(rom[0], 0xA9); // LDA #$42
    assert_eq!(rom[1], 0x42);
    assert_eq!(rom[2], 0x85); // STA $00
    assert_eq!(rom[3], 0x00);
}

#[test]
fn test_to_rom_image_with_gap() {
    let source = r#"
.org $8000
START:
    LDA #$42

.org $FFFC
.word START
"#;

    let output = assemble(source).unwrap();
    let rom = output.to_rom_image(0xFF);

    // ROM should span from $8000 to $FFFD (inclusive)
    // Size = $FFFD - $8000 + 1 = $7FFE = 32766 bytes
    assert_eq!(rom.len(), 0x7FFE);

    // First bytes are the program
    assert_eq!(rom[0], 0xA9); // LDA #$42 at $8000
    assert_eq!(rom[1], 0x42);

    // Gap should be filled with 0xFF
    assert_eq!(rom[2], 0xFF);
    assert_eq!(rom[100], 0xFF);
    assert_eq!(rom[1000], 0xFF);

    // Reset vector at end ($FFFC - $8000 = $7FFC offset)
    assert_eq!(rom[0x7FFC], 0x00); // Low byte of START ($8000)
    assert_eq!(rom[0x7FFD], 0x80); // High byte of START
}

#[test]
fn test_to_rom_image_multiple_segments() {
    let source = r#"
.org $0200
SEGMENT1:
    NOP
    NOP

.org $0300
SEGMENT2:
    LDA #$11

.org $0400
SEGMENT3:
    STA $00
"#;

    let output = assemble(source).unwrap();
    let rom = output.to_rom_image(0x00);

    // ROM from $0200 to $0401 (inclusive) = 514 bytes
    assert_eq!(rom.len(), 514);

    // SEGMENT1 at offset 0
    assert_eq!(rom[0], 0xEA); // NOP
    assert_eq!(rom[1], 0xEA); // NOP

    // Gap filled with 0x00
    assert!(
        rom[2..=255].iter().all(|&b| b == 0x00),
        "Gap should be filled with 0x00"
    );

    // SEGMENT2 at offset $100 (256)
    assert_eq!(rom[256], 0xA9); // LDA #$11
    assert_eq!(rom[257], 0x11);

    // Another gap
    assert!(
        rom[258..=511].iter().all(|&b| b == 0x00),
        "Gap should be filled with 0x00"
    );

    // SEGMENT3 at offset $200 (512)
    assert_eq!(rom[512], 0x85); // STA $00
    assert_eq!(rom[513], 0x00);
}

#[test]
fn test_to_rom_image_empty_output() {
    let source = "; Just a comment";

    let output = assemble(source).unwrap();
    let rom = output.to_rom_image(0xFF);

    // No code, so ROM should be empty
    assert_eq!(rom.len(), 0);
}

#[test]
fn test_to_rom_image_fill_byte_zero() {
    let source = r#"
.org $1000
    NOP

.org $1010
    NOP
"#;

    let output = assemble(source).unwrap();
    let rom = output.to_rom_image(0x00);

    assert_eq!(rom[0], 0xEA); // First NOP

    // Gap filled with 0x00
    assert!(
        rom[1..16].iter().all(|&b| b == 0x00),
        "Gap should be filled with 0x00"
    );

    assert_eq!(rom[16], 0xEA); // Second NOP
}

#[test]
fn test_to_rom_image_fill_byte_custom() {
    let source = r#"
.org $2000
    NOP

.org $2004
    NOP
"#;

    let output = assemble(source).unwrap();
    let rom = output.to_rom_image(0xAA);

    assert_eq!(rom[0], 0xEA); // First NOP

    // Gap filled with 0xAA
    assert!(
        rom[1..4].iter().all(|&b| b == 0xAA),
        "Gap should be filled with 0xAA"
    );

    assert_eq!(rom[4], 0xEA); // Second NOP
}

// =============================================================================
// Segment Tracking
// =============================================================================

#[test]
fn test_segments_single_segment() {
    let source = "LDA #$42\nSTA $00";

    let output = assemble(source).unwrap();

    assert_eq!(output.segments.len(), 1);
    assert_eq!(output.segments[0].address, 0x0000);
    assert_eq!(output.segments[0].length, 4); // 2 + 2 bytes
}

#[test]
fn test_segments_multiple_org_directives() {
    let source = r#"
.org $8000
    NOP
    NOP

.org $9000
    LDA #$42

.org $FFFC
    .word $8000
"#;

    let output = assemble(source).unwrap();

    assert_eq!(output.segments.len(), 3);

    // First segment
    assert_eq!(output.segments[0].address, 0x8000);
    assert_eq!(output.segments[0].length, 2);

    // Second segment
    assert_eq!(output.segments[1].address, 0x9000);
    assert_eq!(output.segments[1].length, 2);

    // Third segment
    assert_eq!(output.segments[2].address, 0xFFFC);
    assert_eq!(output.segments[2].length, 2);
}

#[test]
fn test_segments_with_labels_only() {
    let source = r#"
START:
LOOP:
END:
"#;

    let output = assemble(source).unwrap();

    // No code, no segments
    assert_eq!(output.segments.len(), 0);
    assert_eq!(output.bytes.len(), 0);
}

// =============================================================================
// Integration Tests
// =============================================================================

#[test]
fn test_realistic_rom_with_vectors() {
    let source = r#"
; 6502 ROM with interrupt vectors

.org $8000

RESET:
    LDX #$FF        ; Initialize stack pointer
    TXS
    LDA #$00        ; Clear accumulator
    JMP MAIN

NMI:
    RTI             ; Non-maskable interrupt handler

IRQ:
    RTI             ; Interrupt request handler

MAIN:
    LDA #$42
    STA $0200
LOOP:
    JMP LOOP

; Interrupt vectors
.org $FFFA
.word NMI
.word RESET
.word IRQ
"#;

    let output = assemble(source).unwrap();

    // Check symbols
    assert_eq!(output.lookup_symbol_addr("RESET"), Some(0x8000));
    // RESET: LDX #$FF (2) + TXS (1) + LDA #$00 (2) + JMP MAIN (3) = 8 bytes
    assert_eq!(output.lookup_symbol_addr("NMI"), Some(0x8008));
    // NMI: RTI (1) = 1 byte
    assert_eq!(output.lookup_symbol_addr("IRQ"), Some(0x8009));
    // IRQ: RTI (1) = 1 byte
    assert_eq!(output.lookup_symbol_addr("MAIN"), Some(0x800A));

    // Generate ROM image
    let rom = output.to_rom_image(0xFF);

    // Check ROM size (from $8000 to $FFFF = 32768 bytes)
    assert_eq!(rom.len(), 0x7FFF + 1);

    // Check first instruction (LDX #$FF at $8000)
    assert_eq!(rom[0], 0xA2); // LDX immediate
    assert_eq!(rom[1], 0xFF);

    // Check vectors at end
    let nmi_addr = 0x8008u16;
    let reset_addr = 0x8000u16;
    let irq_addr = 0x8009u16;

    let vector_offset = 0xFFFA - 0x8000;

    assert_eq!(rom[vector_offset], (nmi_addr & 0xFF) as u8);
    assert_eq!(rom[vector_offset + 1], ((nmi_addr >> 8) & 0xFF) as u8);
    assert_eq!(rom[vector_offset + 2], (reset_addr & 0xFF) as u8);
    assert_eq!(rom[vector_offset + 3], ((reset_addr >> 8) & 0xFF) as u8);
    assert_eq!(rom[vector_offset + 4], (irq_addr & 0xFF) as u8);
    assert_eq!(rom[vector_offset + 5], ((irq_addr >> 8) & 0xFF) as u8);
}

#[test]
fn test_word_directive_with_constant() {
    // Note: Constants are resolved to their literal values
    let source = r#"
IO_BASE = $8000

.word IO_BASE
"#;

    let output = assemble(source).unwrap();

    assert_eq!(output.bytes[0], 0x00); // Low byte of $8000
    assert_eq!(output.bytes[1], 0x80); // High byte of $8000
}

// =============================================================================
// String Literals in .byte Directive
// =============================================================================

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
