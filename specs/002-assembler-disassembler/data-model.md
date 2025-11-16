# Data Model: Assembler & Disassembler

**Feature**: 002-assembler-disassembler
**Date**: 2025-11-14

## Overview

This document defines the core data structures and their relationships for the 6502 assembler and disassembler modules.

---

## Core Entities

### 1. Instruction (Disassembler Output)

Represents a single disassembled 6502 instruction with all metadata.

```rust
/// A single disassembled instruction with full metadata
pub struct Instruction {
    /// Memory address where this instruction starts
    pub address: u16,

    /// The opcode byte value (0x00-0xFF)
    pub opcode: u8,

    /// Instruction mnemonic (e.g., "LDA", "STA", "JMP")
    pub mnemonic: &'static str,

    /// Addressing mode used by this instruction
    pub addressing_mode: AddressingMode,

    /// Operand bytes (0-2 bytes depending on addressing mode)
    pub operand_bytes: Vec<u8>,

    /// Total size in bytes (1-3 bytes: opcode + operands)
    pub size_bytes: u8,

    /// Base cycle cost (excluding page-crossing penalties)
    pub base_cycles: u8,
}
```

**Validation Rules**:
- `address` can be any 16-bit value (0x0000-0xFFFF)
- `opcode` must be valid byte (0x00-0xFF)
- `mnemonic` must match OPCODE_TABLE entry for the opcode
- `operand_bytes.len()` must match addressing mode requirements (0, 1, or 2 bytes)
- `size_bytes` must equal `1 + operand_bytes.len()`

**State Transitions**: Immutable once created (no state changes)

**Relationships**:
- References `AddressingMode` enum (defined in `src/addressing.rs`)
- Mnemonic and metadata sourced from `OPCODE_TABLE` (defined in `src/opcodes.rs`)

---

### 2. AssemblyLine (Parser Internal)

Represents a single parsed line of assembly source code.

```rust
/// A parsed line of assembly source
pub struct AssemblyLine {
    /// Line number in source file (1-indexed)
    pub line_number: usize,

    /// Optional label definition (e.g., "START" from "START:")
    pub label: Option<String>,

    /// Optional mnemonic (e.g., "LDA")
    pub mnemonic: Option<String>,

    /// Optional operand text (e.g., "#$42", "$1234,X")
    pub operand: Option<String>,

    /// Optional comment text (after semicolon)
    pub comment: Option<String>,

    /// Character span in source (start, end) for error reporting
    pub span: (usize, usize),
}
```

**Validation Rules**:
- `line_number` must be > 0
- `label` if present must match pattern: `[a-zA-Z][a-zA-Z0-9_]{0,31}` (max 32 chars)
- `mnemonic` if present must be valid 6502 instruction or directive (case-insensitive)
- At least one of `label`, `mnemonic`, or `comment` must be `Some` (empty lines are skipped)

**State Transitions**:
- Created during parsing (Pass 1)
- Consumed during encoding (Pass 2)

---

### 3. Symbol (Symbol Table Entry)

Represents a label name and its resolved memory address.

```rust
/// A symbol table entry mapping a label to an address
pub struct Symbol {
    /// Label name (case-sensitive)
    pub name: String,

    /// Resolved memory address for this label
    pub address: u16,

    /// Source line where label was defined
    pub defined_at: usize,
}
```

**Validation Rules**:
- `name` must be non-empty
- `name` must match pattern: `[a-zA-Z][a-zA-Z0-9_]{0,31}`
- `address` can be any 16-bit value (0x0000-0xFFFF)
- `defined_at` must be > 0

**State Transitions**:
- Created during Pass 1 when label definition encountered
- Immutable after creation
- Queried during Pass 2 for label reference resolution

**Relationships**:
- Multiple symbols form the symbol table (collection of all labels)
- Referenced by `AssemblerOutput.symbol_table`

---

### 4. AssemblerError

Contains detailed error information for a single assembly error.

```rust
/// An error encountered during assembly
pub struct AssemblerError {
    /// Error type classification
    pub error_type: ErrorType,

    /// Line number where error occurred (1-indexed)
    pub line: usize,

    /// Column number where error starts (0-indexed)
    pub column: usize,

    /// Character span (start, end) in the source line
    pub span: (usize, usize),

    /// Human-readable error message
    pub message: String,
}

/// Classification of assembly errors
pub enum ErrorType {
    /// Syntax error (invalid format, unexpected character)
    SyntaxError,

    /// Undefined label reference
    UndefinedLabel,

    /// Duplicate label definition
    DuplicateLabel,

    /// Invalid label name (too long, starts with digit, etc.)
    InvalidLabel,

    /// Invalid mnemonic (not a recognized instruction)
    InvalidMnemonic,

    /// Invalid operand format for addressing mode
    InvalidOperand,

    /// Operand value out of range (e.g., immediate value > 255, branch too far)
    RangeError,

    /// Invalid directive usage
    InvalidDirective,
}
```

**Validation Rules**:
- `line` must be > 0
- `column` must be >= 0
- `span.0` <= `span.1`
- `message` must be non-empty

**State Transitions**:
- Created when error detected during parsing or assembly
- Collected in `Vec<AssemblerError>`
- Returned to caller if assembly fails

---

### 5. SourceMap

Maps assembled instruction addresses to source code locations for debugging.

```rust
/// Bidirectional mapping between binary and source locations
pub struct SourceMap {
    /// Forward map: instruction address → source location
    /// Sorted by address for binary search
    address_to_source: Vec<(u16, SourceLocation)>,

    /// Reverse map: source line → instruction address ranges
    /// Sorted by line number for binary search
    source_to_address: Vec<(usize, AddressRange)>,
}

/// A location in source code
pub struct SourceLocation {
    /// Line number (1-indexed)
    pub line: usize,

    /// Column where instruction starts (0-indexed)
    pub column: usize,

    /// Length of instruction in source characters
    pub length: usize,
}

/// A range of instruction addresses
pub struct AddressRange {
    /// Starting address (inclusive)
    pub start: u16,

    /// Ending address (exclusive)
    pub end: u16,
}
```

**Validation Rules**:
- `SourceLocation.line` must be > 0
- `SourceLocation.column` must be >= 0
- `SourceLocation.length` must be > 0
- `AddressRange.start` < `AddressRange.end`

**State Transitions**:
- Built incrementally during Pass 2 assembly
- Immutable once assembly complete
- Queried for debugging/IDE integration

**Operations**:
- `get_source_location(address: u16) -> Option<SourceLocation>` - Binary search in `address_to_source`
- `get_address_range(line: usize) -> Option<AddressRange>` - Binary search in `source_to_address`

---

### 6. AssemblerOutput

Contains all results from successful assembly.

```rust
/// Complete output from assembling source code
pub struct AssemblerOutput {
    /// Assembled machine code bytes
    pub bytes: Vec<u8>,

    /// Symbol table with all defined labels
    pub symbol_table: Vec<Symbol>,

    /// Source map for debugging
    pub source_map: SourceMap,

    /// Non-fatal warnings encountered during assembly
    pub warnings: Vec<AssemblerWarning>,
}

/// A non-fatal warning from the assembler
pub struct AssemblerWarning {
    /// Line number where warning occurred
    pub line: usize,

    /// Warning message
    pub message: String,
}
```

**Validation Rules**:
- `bytes` can be empty (empty source file) or up to 65536 bytes (full address space)
- `symbol_table` contains unique symbol names (no duplicates)
- `source_map` contains entries for all assembled instructions

**State Transitions**:
- Created upon successful assembly
- Immutable after creation
- Returned to caller

---

### 7. AssemblerDirective

Represents special assembler commands (`.org`, `.byte`, etc.)

```rust
/// Assembler directive types
pub enum AssemblerDirective {
    /// Set origin address (.org $XXXX)
    Origin { address: u16 },

    /// Insert literal bytes (.byte $XX, $YY, ...)
    Byte { values: Vec<u8> },

    /// Insert literal 16-bit words (.word $XXXX, $YYYY, ...)
    Word { values: Vec<u16> },
}
```

**Validation Rules**:
- `Origin.address` can be any 16-bit value
- `Byte.values` must be non-empty
- `Word.values` must be non-empty

**State Transitions**:
- Parsed from source during Pass 1
- Applied during Pass 2:
  - `Origin` sets current address counter
  - `Byte` emits bytes directly
  - `Word` emits 16-bit values in little-endian order

---

### 8. DisassemblyOptions

Configuration for disassembly behavior.

```rust
/// Options controlling disassembly output
pub struct DisassemblyOptions {
    /// Starting address for disassembly (affects address display)
    pub start_address: u16,

    /// Whether to format output as hex dump
    pub hex_dump: bool,

    /// Whether to include byte offsets in output
    pub show_offsets: bool,
}
```

**Validation Rules**:
- `start_address` can be any 16-bit value (defaults to 0x0000)

**Default Values**:
- `start_address`: 0x0000
- `hex_dump`: false
- `show_offsets`: false

---

## Relationships

```
┌─────────────────┐
│ OPCODE_TABLE    │ (existing, in src/opcodes.rs)
│ (256 entries)   │
└────────┬────────┘
         │ references
         ▼
┌─────────────────┐        ┌──────────────────┐
│ Instruction     │────────│ AddressingMode   │
│                 │  uses  │                  │
└─────────────────┘        └──────────────────┘
         ▲
         │ produces
         │
┌─────────────────┐
│ Disassembler    │
│ (function)      │
└─────────────────┘


┌─────────────────┐
│ AssemblyLine    │
│                 │
└────────┬────────┘
         │ parsed from
         │
┌─────────────────┐        ┌──────────────────┐
│ Source Text     │        │ AssemblerError   │
│                 │─error─▶│                  │
└────────┬────────┘        └──────────────────┘
         │ assembles to
         ▼
┌─────────────────┐        ┌──────────────────┐
│AssemblerOutput  │────────│ Symbol           │
│                 │contains│                  │
│                 │────────│ SourceMap        │
└─────────────────┘        └──────────────────┘
```

---

## Data Flow

### Disassembly Flow

```
Byte Slice + Options
        │
        ▼
   Disassembler
        │
        ├─ Lookup opcode in OPCODE_TABLE
        ├─ Extract operand bytes
        ├─ Format based on addressing mode
        │
        ▼
   Vec<Instruction>
```

### Assembly Flow (Two-Pass)

```
Source Text
    │
    ▼
Pass 1: Parse & Build Symbol Table
    │
    ├─ Parse each line → AssemblyLine
    ├─ Extract labels → Symbol
    ├─ Calculate instruction sizes (OPCODE_TABLE)
    │
    ▼
Symbol Table
    │
    ▼
Pass 2: Encode & Emit Bytes
    │
    ├─ Look up opcodes (OPCODE_TABLE)
    ├─ Resolve label references (Symbol Table)
    ├─ Emit bytes
    ├─ Build source map
    │
    ▼
AssemblerOutput (or Vec<AssemblerError>)
```

---

## Invariants

1. **Instruction size consistency**: `instruction.size_bytes == 1 + instruction.operand_bytes.len()`
2. **Symbol uniqueness**: Symbol table contains no duplicate names
3. **Source map coverage**: Every assembled instruction has a source map entry
4. **Address validity**: All addresses are within 0x0000-0xFFFF range
5. **Opcode table lookup**: Every opcode has a corresponding OPCODE_TABLE entry
6. **Error span validity**: `error.span.0 <= error.span.1`
7. **Symbol name validity**: All symbol names match `[a-zA-Z][a-zA-Z0-9_]{0,31}`

---

## Performance Considerations

- **Symbol lookup**: O(n) linear search acceptable for <1000 symbols
- **Source map queries**: O(log n) binary search on sorted vectors
- **Instruction decoding**: O(1) OPCODE_TABLE lookup
- **Memory usage**: Bounded by input size (64KB max program + overhead)
