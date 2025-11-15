# Research: Assembler & Disassembler

**Date**: 2025-11-14
**Feature**: 002-assembler-disassembler

## Overview

This document captures research findings and design decisions for implementing a 6502 assembler and disassembler in Rust, targeting WebAssembly compatibility with zero external dependencies.

## Research Areas

### 1. Parser Architecture for Assembly Language

**Question**: What parsing approach works best for 6502 assembly in a `no_std` Rust environment without parser generator dependencies?

**Decision**: Hand-written recursive descent parser

**Rationale**:
- 6502 assembly syntax is extremely simple and regular (line-oriented, predictable structure)
- Each line follows pattern: `[label:] [mnemonic [operand]] [; comment]`
- Operand syntax is finite and well-defined (13 addressing modes with distinct prefixes)
- Hand-written parser avoids external dependencies (nom, pest, logos all add dependencies)
- Provides fine-grained control over error recovery for collecting multiple errors
- Educational value: clear, readable parsing logic that developers can understand

**Alternatives Considered**:
- **nom combinator library**: Rejected due to external dependency requirement
- **pest PEG parser**: Rejected due to external dependency and proc-macro complexity
- **Lexer + parser split**: Unnecessary complexity for line-oriented format; single-pass is sufficient

**Implementation Notes**:
- Parse line-by-line using `str::lines()`
- Split each line into tokens using custom tokenizer that handles whitespace and special characters
- Use string slicing and pattern matching for operand parsing
- Track line/column positions for error reporting

---

### 2. Number Format Parsing

**Question**: What number formats should be supported, and how should they be parsed without external dependencies?

**Decision**: Support hexadecimal (`$XX`), decimal (no prefix), and binary (`%XXXXXXXX`)

**Rationale**:
- Matches common modern 6502 assembler conventions (ca65, ACME, DASM)
- Hex with `$` prefix is standard in 6502 ecosystem
- Binary with `%` prefix is widely used for bit-oriented operations (status flags, bit masks)
- Decimal as default matches programmer expectations
- All three are trivial to parse in Rust without external libraries

**Alternatives Considered**:
- **Octal**: Rejected as rarely used in 6502 programming
- **0x prefix for hex**: Rejected in favor of `$` (6502 convention)
- **C-style 0b prefix for binary**: Rejected in favor of `%` (6502 convention)

**Implementation**:
```rust
fn parse_number(s: &str) -> Result<u16, ParseError> {
    if s.starts_with('$') {
        u16::from_str_radix(&s[1..], 16)
    } else if s.starts_with('%') {
        u16::from_str_radix(&s[1..], 2)
    } else {
        s.parse::<u16>() // decimal
    }
}
```

---

### 3. Multi-Pass Assembly for Label Resolution

**Question**: How many passes are required to resolve forward label references?

**Decision**: Two-pass assembler

**Rationale**:
- **Pass 1**: Build symbol table by parsing all labels and calculating their addresses
- **Pass 2**: Emit bytes using resolved label addresses from symbol table
- Two passes are sufficient for all valid 6502 programs (no macro expansion or complex expressions)
- Simple, deterministic algorithm well-documented in assembler literature

**Alternatives Considered**:
- **Single-pass with backpatching**: More complex, requires mutable output buffer with fixup logic
- **Three+ passes**: Unnecessary for simple 6502 assembly without macros

**Implementation Notes**:
- Pass 1 calculates instruction sizes using OPCODE_TABLE to determine addresses
- Pass 2 encodes instructions and replaces label references with resolved addresses
- Undefined labels detected in Pass 2 → error

---

### 4. Error Recovery Strategy

**Question**: How can we collect ALL errors in a single pass rather than failing on first error?

**Decision**: Continue parsing after errors with sentinel values

**Rationale**:
- Improves developer experience: see all errors at once rather than fix-compile loop
- Matches behavior of modern compilers (rustc, clang)
- Spec requirement FR-012 explicitly requires collecting all errors

**Implementation Strategy**:
- Parse errors: Insert placeholder instruction (e.g., NOP or skip line) and continue to next line
- Semantic errors (undefined labels): Record error but continue assembly with address 0x0000 as placeholder
- Collect all errors in `Vec<AssemblerError>`
- Return `Err(errors)` at end if any errors accumulated

**Alternatives Considered**:
- **Fail fast**: Rejected as poor developer experience
- **Panic recovery**: More complex, better suited to complex grammars

---

### 5. Source Mapping Granularity

**Question**: Should source maps track byte-level, instruction-level, or line-level mappings?

**Decision**: Instruction-level (each instruction maps to source line and column range)

**Rationale**:
- Byte-level is too granular (operand bytes don't have meaningful separate source locations)
- Line-level is too coarse (multiple instructions can appear on one line with macros/future features)
- Instruction-level balances precision with simplicity
- Supports debugging use case: set breakpoint on instruction → map to source line
- Supports IDE use case: hover over instruction → show source code

**Data Structure**:
```rust
pub struct SourceMap {
    // Maps instruction start address → source location
    mappings: Vec<(u16, SourceLocation)>,
}

pub struct SourceLocation {
    line: usize,
    column: usize,
    length: usize, // span length in characters
}
```

**Alternatives Considered**:
- **Byte-level**: Too fine-grained, operand bytes don't have separate source meaning
- **Line-level**: Too coarse, doesn't handle multiple instructions per line

---

### 6. Operand Format Patterns for Disassembly

**Question**: What is the standard human-readable format for each addressing mode?

**Decision**: Use canonical 6502 assembly syntax conventions

**Formats**:
- Implicit: (no operand, e.g., `NOP`)
- Accumulator: `A` (e.g., `LSR A`)
- Immediate: `#$XX` (e.g., `LDA #$42`)
- ZeroPage: `$XX` (e.g., `LDA $80`)
- ZeroPageX: `$XX,X` (e.g., `LDA $80,X`)
- ZeroPageY: `$XX,Y` (e.g., `LDX $80,Y`)
- Relative: `$XXXX` (target address, not raw offset - more readable)
- Absolute: `$XXXX` (e.g., `JMP $1234`)
- AbsoluteX: `$XXXX,X` (e.g., `LDA $1234,X`)
- AbsoluteY: `$XXXX,Y` (e.g., `LDA $1234,Y`)
- Indirect: `($XXXX)` (e.g., `JMP ($FFFC)`)
- IndirectX: `($XX,X)` (e.g., `LDA ($40,X)`)
- IndirectY: `($XX),Y` (e.g., `LDA ($40),Y`)

**Rationale**: Matches standard assembler output (DASM, ca65, commercial tools)

---

### 7. Symbol Table Design

**Question**: What data structure efficiently supports assembler symbol table operations?

**Decision**: `Vec<(String, u16)>` for simplicity

**Rationale**:
- Small symbol tables (<1000 labels typical for 64KB programs)
- Linear search is fast enough for this scale
- No external dependency (no HashMap/BTreeMap in `no_std` without `alloc`)
- Actually, we CAN use `alloc` crate for `Vec` and `String` in `no_std` - decision: use `Vec` with `alloc`

**Operations**:
- Insert: `symbols.push((name, address))`
- Lookup: `symbols.iter().find(|(n, _)| n == name).map(|(_, a)| a)`
- Duplicate detection: Check before insert

**Alternatives Considered**:
- **HashMap**: Would be faster for large symbol tables, but requires `std` or external dependency
- **BTreeMap**: Same issue as HashMap
- **Sorted Vec with binary search**: Premature optimization

---

### 8. Hex Dump Formatting

**Question**: What is the standard format for hex dump output?

**Decision**: `AAAA: BB BB BB  MNEMONIC OPERAND` format

**Example**:
```
8000: A9 42     LDA #$42
8002: 8D 00 80  STA $8000
8005: 4C 00 80  JMP $8000
```

**Rationale**:
- Widely recognized format (similar to hexdump, objdump, debuggers)
- Address allows correlation with memory layout
- Hex bytes show exact encoding
- Assembly provides human-readable interpretation
- Fixed-width columns enable alignment

**Formatting Details**:
- Address: 4 hex digits
- Bytes: Up to 3 bytes per instruction, space-separated, left-aligned
- Padding: Extra spaces to align mnemonic column
- Mnemonic: 3 characters, uppercase
- Operand: Variable length based on addressing mode

---

## Technology Best Practices

### Rust `no_std` String Processing

**Key Patterns**:
- Use `&str` slices for zero-copy parsing
- Use `core::str` methods (`lines()`, `split_whitespace()`, `trim()`, `starts_with()`)
- Use `alloc::string::String` and `alloc::vec::Vec` for owned data (requires `alloc` feature)
- Avoid regex (no standard regex in `no_std`)
- Use `char::is_alphabetic()`, `char::is_digit()` for character classification

**Error Handling**:
- Return `Result<T, E>` for all fallible operations
- Define custom error types with detailed information
- Implement `core::fmt::Display` for error types
- Avoid panics in public APIs

### WASM Compatibility Checklist

- ✅ No file I/O
- ✅ No network access
- ✅ No threading
- ✅ No system calls
- ✅ Deterministic execution
- ✅ Bounded memory usage (no unbounded recursion)
- ✅ No floating point (if targeting older WASM runtimes - not an issue here)

---

## Open Questions

None. All clarifications from spec are sufficient for implementation.

---

## References

- [6502 Instruction Reference](http://www.6502.org/tutorials/6502opcodes.html)
- [ca65 Assembler Documentation](https://cc65.github.io/doc/ca65.html) - Modern 6502 assembler for syntax conventions
- [DASM Assembler](https://dasm-assembler.github.io/) - Popular 6502 assembler
- Existing project files: `src/opcodes.rs`, `src/addressing.rs`, `CLAUDE.md`
