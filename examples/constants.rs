//! Assembler constants example
//!
//! Demonstrates how to use named constants in 6502 assembly code.
//! Constants provide reusable values for I/O addresses, character codes,
//! and magic numbers.

use lib6502::assembler::assemble;

fn main() {
    println!("=== 6502 Assembler Constants Example ===\n");

    // Example program using constants
    let source = r#"
; Define I/O memory-mapped addresses
UART_DATA = $8000
UART_STATUS = $8001
SCREEN_BASE = $4000

; Define character constants
CHAR_CR = 13        ; Carriage return
CHAR_LF = 10        ; Line feed
CHAR_SPACE = 32     ; Space character

; Define zero-page variables
ZP_CHAR = $20
ZP_COUNT = $21

; Program entry point
START:
    ; Initialize counter
    LDA #0
    STA ZP_COUNT

MAIN_LOOP:
    ; Wait for UART data available
    LDA UART_STATUS
    AND #1
    BEQ MAIN_LOOP

    ; Read character from UART
    LDA UART_DATA
    STA ZP_CHAR

    ; Check for carriage return
    CMP #CHAR_CR
    BEQ HANDLE_CR

    ; Check for line feed
    CMP #CHAR_LF
    BEQ HANDLE_LF

    ; Store character to screen
    LDX ZP_COUNT
    STA SCREEN_BASE,X

    ; Increment counter
    INC ZP_COUNT

    ; Continue loop
    JMP MAIN_LOOP

HANDLE_CR:
    ; Replace CR with space
    LDA #CHAR_SPACE
    LDX ZP_COUNT
    STA SCREEN_BASE,X
    JMP MAIN_LOOP

HANDLE_LF:
    ; Move to next position
    INC ZP_COUNT
    JMP MAIN_LOOP
"#;

    // Assemble the program
    match assemble(source) {
        Ok(output) => {
            println!("✓ Assembly successful!\n");
            println!("Program size: {} bytes", output.bytes.len());
            println!("Symbol table entries: {}\n", output.symbol_table.len());

            // Show constants
            println!("=== Constants ===");
            for symbol in &output.symbol_table {
                if symbol.kind == lib6502::assembler::SymbolKind::Constant {
                    println!(
                        "  {} = ${:04X} ({})",
                        symbol.name, symbol.value, symbol.value
                    );
                }
            }

            // Show labels
            println!("\n=== Labels ===");
            for symbol in &output.symbol_table {
                if symbol.kind == lib6502::assembler::SymbolKind::Label {
                    println!("  {} @ ${:04X}", symbol.name, symbol.value);
                }
            }

            // Show first few bytes
            println!("\n=== Machine Code (first 20 bytes) ===");
            print!("  ");
            for (i, byte) in output.bytes.iter().take(20).enumerate() {
                print!("{:02X} ", byte);
                if (i + 1) % 8 == 0 {
                    print!("\n  ");
                }
            }
            println!("\n");

            println!("=== Benefits of Constants ===");
            println!("✓ Readable code: CHAR_CR instead of magic number 13");
            println!("✓ Single source of truth: Change $8000 in one place");
            println!("✓ Self-documenting: UART_DATA tells you what it is");
            println!("✓ Type safety: Constants can't be used as jump targets");
            println!("✓ Automatic addressing: Zero-page vs absolute selected automatically");
        }
        Err(errors) => {
            eprintln!("✗ Assembly failed with {} error(s):\n", errors.len());
            for error in errors {
                eprintln!("  {}", error);
            }
            std::process::exit(1);
        }
    }
}
