# Quickstart: Assembler & Disassembler

**Feature**: 002-assembler-disassembler
**Date**: 2025-11-14

This guide provides quick examples for using the 6502 assembler and disassembler modules.

---

## Disassembler

### Basic Disassembly

```rust
use lib6502::disassembler::{disassemble, DisassemblyOptions};

fn main() {
    // Machine code bytes
    let code = &[
        0xA9, 0x42,       // LDA #$42
        0x8D, 0x00, 0x80, // STA $8000
        0x4C, 0x00, 0x80, // JMP $8000
    ];

    // Disassemble with default options
    let options = DisassemblyOptions {
        start_address: 0x8000,
        hex_dump: false,
        show_offsets: false,
    };

    let instructions = disassemble(code, options);

    // Print instructions
    for instr in instructions {
        println!("{:04X}: {}", instr.address, format_instruction(&instr));
    }
}
```

**Output**:
```
8000: LDA #$42
8002: STA $8000
8005: JMP $8000
```

---

### Hex Dump Format

```rust
use lib6502::disassembler::{disassemble, format_hex_dump, DisassemblyOptions};

fn main() {
    let code = &[0xA9, 0x42, 0x8D, 0x00, 0x80, 0x4C, 0x00, 0x80];

    let options = DisassemblyOptions {
        start_address: 0x8000,
        hex_dump: true,
        show_offsets: false,
    };

    let instructions = disassemble(code, options);
    let dump = format_hex_dump(&instructions);

    print!("{}", dump);
}
```

**Output**:
```
8000: A9 42     LDA #$42
8002: 8D 00 80  STA $8000
8005: 4C 00 80  JMP $8000
```

---

### Inspecting Instruction Metadata

```rust
use lib6502::disassembler::disassemble;

fn main() {
    let code = &[0xA9, 0x42]; // LDA #$42

    let instructions = disassemble(code, Default::default());
    let instr = &instructions[0];

    println!("Mnemonic: {}", instr.mnemonic);
    println!("Opcode: ${:02X}", instr.opcode);
    println!("Size: {} bytes", instr.size_bytes);
    println!("Cycles: {}", instr.base_cycles);
    println!("Addressing mode: {:?}", instr.addressing_mode);
}
```

**Output**:
```
Mnemonic: LDA
Opcode: $A9
Size: 2 bytes
Cycles: 2
Addressing mode: Immediate
```

---

## Assembler

### Basic Assembly

```rust
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
        }
        Err(errors) => {
            eprintln!("Assembly failed:");
            for error in errors {
                eprintln!("  Line {}: {}", error.line, error.message);
            }
        }
    }
}
```

**Output**:
```
Assembled 8 bytes:
A9 42 8D 00 80 4C 00 80
```

---

### Assembly with Labels

```rust
use lib6502::assembler::assemble;

fn main() {
    let source = r#"
        .org $8000

    START:
        LDX #$00
    LOOP:
        INX
        CPX #$10
        BNE LOOP
        JMP START
    "#;

    match assemble(source) {
        Ok(output) => {
            println!("Assembled {} bytes", output.bytes.len());

            // Print symbol table
            println!("\nSymbol Table:");
            for symbol in &output.symbol_table {
                println!("  {}: ${:04X}", symbol.name, symbol.address);
            }
        }
        Err(errors) => {
            for error in errors {
                eprintln!("Error on line {}: {}", error.line, error.message);
            }
        }
    }
}
```

**Output**:
```
Assembled 9 bytes

Symbol Table:
  START: $8000
  LOOP: $8002
```

---

### Using Source Maps for Debugging

```rust
use lib6502::assembler::assemble;

fn main() {
    let source = r#"
START:
    LDA #$42
    STA $8000
"#;

    let output = assemble(source).unwrap();

    // Map instruction address to source line
    if let Some(loc) = output.get_source_location(0x0000) {
        println!("Instruction at $0000 is from line {}, column {}",
                 loc.line, loc.column);
    }

    // Map source line to instruction address
    if let Some(range) = output.get_address_range(2) {
        println!("Line 2 contains instructions from ${:04X} to ${:04X}",
                 range.start, range.end);
    }
}
```

**Output**:
```
Instruction at $0000 is from line 2, column 4
Line 2 contains instructions from $0000 to $0002
```

---

### Handling Assembly Errors

```rust
use lib6502::assembler::{assemble, ErrorType};

fn main() {
    let source = r#"
        LDA #$42
        JMP UNDEFINED    ; Error: undefined label
        STA #$1234       ; Error: invalid addressing mode
    "#;

    match assemble(source) {
        Ok(_) => println!("Assembly succeeded"),
        Err(errors) => {
            println!("Assembly failed with {} errors:\n", errors.len());

            for error in errors {
                println!("Error at line {}, column {}:", error.line, error.column);
                println!("  Type: {:?}", error.error_type);
                println!("  Message: {}", error.message);
                println!();
            }
        }
    }
}
```

**Output**:
```
Assembly failed with 2 errors:

Error at line 3, column 12:
  Type: UndefinedLabel
  Message: undefined label 'UNDEFINED'

Error at line 4, column 12:
  Type: InvalidOperand
  Message: invalid operand for STA: immediate mode not supported
```

---

### Using Directives

```rust
use lib6502::assembler::assemble;

fn main() {
    let source = r#"
        .org $8000

        .byte $48, $65, $6C, $6C, $6F  ; "Hello"

        .word $1234, $5678              ; Two 16-bit values

    START:
        LDA #$00
    "#;

    let output = assemble(source).unwrap();

    println!("Bytes:");
    for byte in &output.bytes {
        print!("{:02X} ", byte);
    }
    println!();
}
```

**Output**:
```
Bytes:
48 65 6C 6C 6F 34 12 78 56 A9 00
```

---

## Round-Trip: Assemble → Disassemble

```rust
use lib6502::assembler::assemble;
use lib6502::disassembler::{disassemble, format_instruction, DisassemblyOptions};

fn main() {
    let source = r#"
        LDA #$42
        STA $8000
        JMP $8000
    "#;

    // Assemble
    let output = assemble(source).unwrap();
    println!("Assembled bytes: {:?}", output.bytes);

    // Disassemble
    let options = DisassemblyOptions::default();
    let instructions = disassemble(&output.bytes, options);

    println!("\nDisassembled:");
    for instr in instructions {
        println!("  {}", format_instruction(&instr));
    }
}
```

**Output**:
```
Assembled bytes: [169, 66, 141, 0, 128, 76, 0, 128]

Disassembled:
  LDA #$42
  STA $8000
  JMP $8000
```

---

## Integration with CPU Emulator

```rust
use lib6502::{CPU, FlatMemory};
use lib6502::assembler::assemble;

fn main() {
    // Assemble program
    let source = r#"
        .org $8000

        LDA #$42
        STA $00
        BRK
    "#;

    let output = assemble(source).unwrap();

    // Load into memory
    let mut memory = FlatMemory::new();
    for (i, &byte) in output.bytes.iter().enumerate() {
        memory.write(0x8000 + i as u16, byte);
    }

    // Create CPU and run
    let mut cpu = CPU::new(memory);
    cpu.set_pc(0x8000);

    // Run until BRK
    loop {
        match cpu.step() {
            Ok(()) => continue,
            Err(e) => {
                // BRK triggers an error in this simple example
                println!("CPU stopped");
                break;
            }
        }
    }

    println!("Value at $00: ${:02X}", cpu.memory().read(0x00));
}
```

**Output**:
```
CPU stopped
Value at $00: $42
```

---

## Number Format Examples

```rust
use lib6502::assembler::assemble;

fn main() {
    let source = r#"
        LDA #$FF      ; Hexadecimal
        LDX #255      ; Decimal
        LDY #%11111111 ; Binary

        ; All three are equivalent!
    "#;

    let output = assemble(source).unwrap();

    // All three LDA/LDX/LDY instructions load the same value (255)
    println!("Bytes: {:02X?}", output.bytes);
}
```

**Output**:
```
Bytes: [A9, FF, A2, FF, A0, FF]
```

---

## WebAssembly Usage

```rust
use wasm_bindgen::prelude::*;
use lib6502::assembler::assemble;
use lib6502::disassembler::{disassemble, format_hex_dump};

#[wasm_bindgen]
pub fn assemble_6502(source: &str) -> Result<Vec<u8>, JsValue> {
    match assemble(source) {
        Ok(output) => Ok(output.bytes),
        Err(errors) => {
            let msg = errors.iter()
                .map(|e| format!("Line {}: {}", e.line, e.message))
                .collect::<Vec<_>>()
                .join("\n");
            Err(JsValue::from_str(&msg))
        }
    }
}

#[wasm_bindgen]
pub fn disassemble_6502(bytes: &[u8]) -> String {
    let instructions = disassemble(bytes, Default::default());
    format_hex_dump(&instructions)
}
```

---

## Next Steps

- Read [data-model.md](./data-model.md) for detailed data structure documentation
- Read [assembler-api.md](./contracts/assembler-api.md) for complete API reference
- Read [disassembler-api.md](./contracts/disassembler-api.md) for disassembler details
- See [tasks.md](./tasks.md) for implementation plan (generated by `/speckit.tasks`)
