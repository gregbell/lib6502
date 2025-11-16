//! Basic assembler usage example

use lib6502::assembler::assemble;

fn main() {
    let source = r#"
        LDA #$42
        STA $8000
        JMP $8000
    "#;

    match assemble(source) {
        Ok(output) => {
            println!("Assembled {} bytes:", output.bytes.len());
            for (i, byte) in output.bytes.iter().enumerate() {
                print!("{:02X} ", byte);
                if (i + 1) % 8 == 0 {
                    println!();
                }
            }
            println!();
        }
        Err(errors) => {
            eprintln!("Assembly failed:");
            for error in errors {
                eprintln!("  Line {}: {}", error.line, error.message);
            }
        }
    }
}
