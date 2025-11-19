# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A cycle-accurate NMOS 6502 CPU emulator built in Rust. The project prioritizes modularity, clarity, WebAssembly portability, and hackability. No external dependencies are used in the core library.

## Constitution

The project uses a CONSTITUTION.md file (.specify/memory/constitution.md) to manage a set of core principles to follow. Review them to ensure that new work matches hte project style, architecture, and vision.

## Architecture

### Core Abstractions

The emulator uses a **trait-based architecture** to decouple CPU logic from memory implementation:

- **CPU<M: MemoryBus>**: Generic over memory implementation, contains all processor state (registers, flags, PC, SP, cycle counter)
- **MemoryBus trait**: Provides `read(&self, addr: u16) -> u8` and `write(&mut self, addr: u16, value: u8)`
- **FlatMemory**: Simple 64KB RAM implementation of MemoryBus
- **OPCODE_TABLE**: Static 256-entry metadata table mapping opcodes to mnemonic, addressing mode, cycle cost, and size

### Execution Model

The fetch-decode-execute loop is driven by:

- `CPU::step()` - Execute one instruction and return Result
- `CPU::run_for_cycles(budget)` - Execute until cycle budget exhausted
- Unimplemented opcodes return `Err(ExecutionError::UnimplementedOpcode(u8))`

### Module Structure

```
src/
  lib.rs              - Public API and error types
  cpu.rs              - CPU state and execution logic
  memory.rs           - MemoryBus trait and FlatMemory impl
  opcodes.rs          - OPCODE_TABLE metadata
  addressing.rs       - AddressingMode enum
  assembler/
    mod.rs            - Assembler public API
    lexer.rs          - Tokenization (characters → tokens)
    parser.rs         - Syntactic analysis (tokens → AssemblyLine)
    encoder.rs        - Code generation (AssemblyLine → bytes)
    symbol_table.rs   - Label/constant resolution
    source_map.rs     - Debug information
tests/                - Integration tests (separate from unit tests in src/)
examples/             - Usage examples
specs/                - Feature specifications and planning docs
```

### Assembler Architecture

The assembler follows a **three-phase pipeline** that separates concerns:

```
Phase 1: Lexical Analysis (src/assembler/lexer.rs)
  Input:  "LDA #$42 ; comment"
  Output: [Identifier("LDA"), Whitespace, Hash, HexNumber(0x42), Comment("comment"), EOF]

  Responsibilities:
  - Tokenize characters into typed tokens
  - Parse number literals ($42 → HexNumber(0x42))
  - Track source locations (line, column)
  - Detect lexical errors (invalid hex, overflow)

Phase 2: Syntactic Analysis (src/assembler/parser.rs)
  Input:  Token stream from lexer
  Output: AssemblyLine { label, mnemonic, operand, directive, ... }

  Responsibilities:
  - Pattern match on token types (Identifier + Colon → label)
  - Build structured representation
  - Validate syntax (not semantics)
  - Preserve comments and locations

Phase 3: Code Generation (src/assembler/encoder.rs + mod.rs)
  Input:  Vector of AssemblyLine
  Output: Binary machine code + source maps

  Responsibilities:
  - Resolve labels and constants (two-pass)
  - Validate addressing modes and ranges
  - Generate machine code bytes
  - Build debug information
```

**Key Benefits of Lexer/Parser Separation:**

- **Simpler Code**: Token pattern matching instead of string manipulation
- **Better Errors**: Lexical errors (bad hex) vs syntactic errors (bad mnemonic)
- **Extensibility**: Add directives without modifying lexer
- **Reusability**: External tools can use `tokenize()` for syntax highlighting
- **Type Safety**: Compiler catches token type mismatches

**Adding New Features:**

- New directive → Modify parser only (lexer already handles `.` + identifier)
- New number format → Modify lexer only (parser uses TokenType::*Number)
- New addressing mode → Modify encoder only (parser preserves operand tokens)

### Table-Driven Design

All opcode information lives in `OPCODE_TABLE`. When implementing instructions:

1. Look up metadata via `OPCODE_TABLE[opcode as usize]`
2. Use `metadata.addressing_mode` to determine how to fetch operands
3. Add `metadata.base_cycles` to cycle counter (plus page-crossing penalties)
4. Advance PC by `metadata.size_bytes`

The `get_operand_value()` helper handles all addressing modes and returns `(value, page_crossed)`.

## Common Commands

```bash
# Build the library
cargo build

# Run fast tests (TDD workflow - skips slow functional test)
cargo test

# Run all tests including slow functional test
cargo test -- --include-ignored

# Run only the Klaus functional test
cargo test --test functional_klaus klaus_6502_functional_test -- --ignored --nocapture

# Run a specific test
cargo test test_name

# Run tests with output visible
cargo test -- --nocapture

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Format
cargo fmt

# Run examples
cargo run --example simple_ram
```

## Test Suites

The project has two test categories:

**Fast Tests** (default: `cargo test`)
- 1,470+ unit and integration tests
- Complete in ~2 seconds
- Perfect for TDD workflow
- Run automatically on every `cargo test`

**Functional Tests** (run with `--ignored`)
- Klaus Dormann's 6502 functional test (~6 seconds)
- Validates all 151 opcodes with 96M+ instruction cycles
- Marked as `#[ignore]` to skip during TDD
- Run explicitly with: `cargo test -- --ignored`
- CI runs both test suites separately

See [docs/KLAUS_FUNCTIONAL_TEST.md](docs/KLAUS_FUNCTIONAL_TEST.md) for details.

## Testing Patterns

- Unit tests live in `mod tests` at bottom of source files
- Integration tests live in `tests/` directory
- CPU state is inspectable via public getter methods: `cpu.a()`, `cpu.pc()`, `cpu.flag_c()`, etc.
- CPU state can be set via public setters for testing: `cpu.set_a(value)`, `cpu.set_flag_c(true)`
- Use `cpu.memory_mut()` to access memory for test setup

## Implementation Workflow

When adding new instructions:

1. Mark `implemented: true` in OPCODE_TABLE for the relevant opcodes
2. Add match arm in `CPU::step()` to dispatch based on `metadata.mnemonic`
3. Implement instruction logic in a private `execute_xxx()` method
4. Use `get_operand_value()` to handle addressing modes
5. Update flags (N, Z, C, V as appropriate)
6. Update cycle counter (base + page crossing)
7. Advance PC by instruction size
8. Add comprehensive tests in `tests/` directory
9. Run `cargo test`, `cargo clippy`, and `cargo fmt`

## Key Design Constraints

- **No external dependencies** in core library (only dev-dependencies for testing)
- **No OS dependencies** - must work in WebAssembly
- **No panics in MemoryBus** - reads/writes always succeed (matching real 6502 hardware)
- **Individual bool fields for flags** - not packed into status register byte (stored as `flag_n`, `flag_z`, etc.)
- **Cycle accuracy** - track exact cycle counts including page-crossing penalties

<!-- MANUAL ADDITIONS START -->

## Interrupt Support

The emulator implements hardware-accurate IRQ (Interrupt Request) support matching real 6502 behavior.

### Interrupt Model

- **Level-sensitive IRQ line**: Shared among all interrupt-capable devices via logical OR
- **No queuing**: Interrupt state reflects current device status (not edge-triggered)
- **I flag respect**: Interrupts only serviced when I flag is clear
- **7-cycle sequence**: Exact timing matching MOS 6502 specification
- **Explicit acknowledgment**: ISR must read/write device registers to clear interrupts

### Creating Interrupt-Capable Devices

Devices implement both `Device` and `InterruptDevice` traits:

```rust
use lib6502::{Device, InterruptDevice};

struct TimerDevice {
    interrupt_pending: bool,
    // ... device fields
}

impl InterruptDevice for TimerDevice {
    fn has_interrupt(&self) -> bool {
        self.interrupt_pending
    }
}

impl Device for TimerDevice {
    fn size(&self) -> u16 { 4 }  // Number of memory-mapped registers

    fn read(&self, offset: u16) -> u8 {
        match offset {
            0 => if self.interrupt_pending { 0x80 } else { 0x00 },  // STATUS
            // ... other registers
            _ => 0x00
        }
    }

    fn write(&mut self, offset: u16, value: u8) {
        match offset {
            1 if value & 0x80 != 0 => self.interrupt_pending = false,  // CONTROL
            // ... other registers
            _ => {}
        }
    }

    // ... implement as_any(), as_any_mut()

    fn has_interrupt(&self) -> bool {
        <Self as InterruptDevice>::has_interrupt(self)  // Delegate to InterruptDevice
    }
}
```

### Interrupt Service Routine (ISR) Pattern

```asm
irq_handler:
    pha                ; Save registers

    lda $D000          ; Read device STATUS register
    and #$80           ; Check interrupt pending (bit 7)
    beq not_our_irq

    lda #$80
    sta $D001          ; Acknowledge interrupt (write CONTROL)

not_our_irq:
    pla
    rti                ; Return from interrupt
```

### CPU Interrupt Sequence

When `irq_active() && !flag_i`:

1. Push PC high byte to stack (1 cycle)
2. Push PC low byte to stack (1 cycle)
3. Push status register to stack (1 cycle)
4. Set I flag to prevent nested interrupts
5. Read IRQ vector from 0xFFFE-0xFFFF (2 cycles)
6. Jump to handler address (2 cycles)

**Total: 7 cycles** (matches hardware)

### Examples

See `examples/interrupt_device.rs` for complete working timer device with:
- Memory-mapped STATUS/CONTROL/COUNTER registers
- Interrupt generation and acknowledgment
- Sample ISR code
- Full system integration example

See `tests/interrupt_test.rs` for comprehensive test coverage.
## Assembler Constants

The assembler supports named constants for defining reusable values like screen addresses, character codes, and magic numbers.

### Syntax

Define constants using `NAME = VALUE` syntax:

```assembly
; I/O addresses
UART_DATA = $8000
SCREEN_ADDR = $4000

; Character constants
CHAR_CR = 13
CHAR_LF = 10

; Zero-page addresses
ZP_TEMP = $20
```

### Usage

Constants can be used anywhere in code:

```assembly
; Immediate mode
    LDA #CHAR_CR        ; Loads 13

; Zero-page addressing
    STA ZP_TEMP         ; Stores to $20

; Absolute addressing
    STA SCREEN_ADDR     ; Stores to $4000

; Indexed addressing
    LDA UART_DATA,X     ; Indexed absolute
    STA ZP_TEMP,Y       ; Indexed zero-page
```

### Rules

- Constants must be defined before use (no forward references)
- Names follow the same rules as labels (alphanumeric + underscore, start with letter)
- Names are case-insensitive and normalized to UPPERCASE
- Values can be decimal, hex ($XXXX), or binary (%XXXXXXXX)
- Constants hold literal values, labels hold memory addresses
- Name collisions between constants and labels are detected

### Automatic Addressing Mode Selection

The assembler automatically chooses the most efficient addressing mode:

- Values 0-255: Zero-page addressing (2 bytes)
- Values >255: Absolute addressing (3 bytes)

### Examples

See `examples/constants.rs` for a complete example program.

<!-- MANUAL ADDITIONS END -->

## Active Technologies
- Rust 1.75+ (edition 2021) + None (zero external dependencies for core library - `no_std` compatible) (002-assembler-disassembler)
- N/A (operates on in-memory byte slices and strings) (002-assembler-disassembler)
- Rust 1.75+ (for WASM compilation), HTML5/CSS3/JavaScript ES6+ (for frontend) (003-wasm-web-demo)
- N/A (fully client-side, no persistence) (003-wasm-web-demo)
- N/A (in-memory state only, no persistence) (004-memory-mapping-module)
- Rust 1.75+ (edition 2021) + None (zero external dependencies for core library - `no_std` compatible per Constitution) (006-assembler-lexer)
- N/A (operates on in-memory strings and produces byte vectors) (006-assembler-lexer)
- N/A (in-memory CPU and device state only) (005-cpu-interrupt-support)

## Recent Changes
- 002-assembler-disassembler: Added Rust 1.75+ (edition 2021) + None (zero external dependencies for core library - `no_std` compatible)
