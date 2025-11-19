# Data Model: Assembler Constants

**Feature**: 005-assembler-constants
**Date**: 2025-11-18
**Status**: Design Complete

This document defines the data structures and entity relationships for named constant support in the 6502 assembler.

---

## Entity Overview

The assembler constants feature extends the existing symbol system with a type distinction between labels (memory addresses) and constants (literal values).

### Entity Diagram

```
┌─────────────────────┐
│   AssemblyLine      │
├─────────────────────┤
│ line_number: usize  │
│ constant: Option<(String, String)>  ← NEW
│ label: Option<String>
│ mnemonic: Option<String>
│ operand: Option<String>
│ directive: Option<Directive>
│ comment: Option<String>
│ span: (usize, usize)
└─────────────────────┘
         │
         │ parses to
         ▼
┌─────────────────────┐
│   Symbol            │
├─────────────────────┤
│ name: String        │
│ value: u16          │ ← RENAMED from 'address'
│ kind: SymbolKind    │ ← NEW
│ defined_at: usize   │
└─────────────────────┘
         │
         │ classified by
         ▼
┌─────────────────────┐
│   SymbolKind (enum) │ ← NEW
├─────────────────────┤
│ Label               │ → Memory address
│ Constant            │ → Literal value
└─────────────────────┘
         │
         │ enforced by
         ▼
┌─────────────────────┐
│   ErrorType (enum)  │
├─────────────────────┤
│ ... (existing)      │
│ UndefinedConstant   │ ← NEW
│ DuplicateConstant   │ ← NEW
│ NameCollision       │ ← NEW
│ InvalidConstantValue│ ← NEW
└─────────────────────┘
```

---

## 1. SymbolKind (New Enum)

**Purpose**: Distinguish between labels (memory addresses) and constants (literal values) in the symbol table.

### Definition

```rust
/// Classifies symbols as labels or constants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// Memory address (defined with ':' suffix)
    Label,

    /// Literal value (defined with '=' assignment)
    Constant,
}
```

### Characteristics

- **Copyable**: Cheap to pass around (single enum discriminant)
- **Comparable**: Supports equality checks
- **Simple**: No associated data (data lives in Symbol struct)

### Usage Pattern

```rust
match symbol.kind {
    SymbolKind::Label => {
        // Use symbol.value as memory address
        let address = symbol.value;
    },
    SymbolKind::Constant => {
        // Use symbol.value as literal
        let literal = symbol.value;
    },
}
```

---

## 2. Symbol (Modified Struct)

**Purpose**: Represents a named symbol (label or constant) with its value and metadata.

### Definition

```rust
/// A symbol table entry mapping a name to a value
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    /// Symbol name (case-normalized to UPPERCASE)
    pub name: String,

    /// Value: memory address (for labels) or literal value (for constants)
    ///
    /// **BREAKING CHANGE**: Renamed from 'address' to 'value'
    /// Rationale: Field stores both label addresses and constant values,
    /// so 'value' is more semantically accurate.
    pub value: u16,

    /// Symbol classification (label or constant)
    pub kind: SymbolKind,

    /// Source line where symbol was defined (1-indexed)
    pub defined_at: usize,
}
```

### Changes from Previous Version

| Field | Previous | New | Rationale |
|-------|----------|-----|-----------|
| `address` | `u16` | **Removed** | Renamed to `value` for clarity |
| `value` | N/A | `u16` | **New name** for address/value field |
| `kind` | N/A | `SymbolKind` | **New** - distinguishes labels from constants |
| `name` | `String` | `String` | Unchanged |
| `defined_at` | `usize` | `usize` | Unchanged |

### Validation Rules

**Name validation** (enforced at parse time):
- Must start with letter `[a-zA-Z]`
- Contains only alphanumeric + underscore `[a-zA-Z0-9_]`
- Maximum 32 characters
- Case-normalized to UPPERCASE

**Value validation**:
- Range: 0-65535 (16-bit unsigned)
- For constants: Must be literal number (hex `$FF`, decimal `255`, binary `%11111111`)
- For labels: Calculated address within 64KB address space

### Uniqueness Constraint

**Namespace**: Global
- Symbol names must be unique across **both** labels and constants
- Collision error if same name used for both label and constant
- Duplicate error if same name used twice for same kind

---

## 3. AssemblyLine (Modified Struct)

**Purpose**: Represents a parsed line of assembly source code.

### Definition

```rust
/// Parsed assembly source line
pub struct AssemblyLine {
    /// Line number in source file (1-indexed)
    pub line_number: usize,

    /// Constant assignment (NAME, VALUE_STRING)
    ///
    /// **NEW FIELD**: Represents `NAME = VALUE` syntax
    /// Example: ("MAX", "$FF") for input "MAX = $FF"
    pub constant: Option<(String, String)>,

    /// Label name (without ':' suffix)
    pub label: Option<String>,

    /// Instruction mnemonic (e.g., "LDA", "STA")
    pub mnemonic: Option<String>,

    /// Operand (e.g., "#$42", "$1234,X")
    pub operand: Option<String>,

    /// Assembler directive (e.g., .org, .byte)
    pub directive: Option<AssemblerDirective>,

    /// Comment text (after ';')
    pub comment: Option<String>,

    /// Character span in source (start, end) for error reporting
    pub span: (usize, usize),
}
```

### Field Mutual Exclusivity

A single `AssemblyLine` can contain **at most one** of:
- Constant assignment (`constant`)
- Label definition (`label` + optional `mnemonic`/`operand`)
- Instruction (`mnemonic` + `operand`)
- Directive (`directive`)

**Valid combinations**:
- `constant` alone: `MAX = 255`
- `label` alone: `START:`
- `label` + `mnemonic` + `operand`: `LOOP: LDA #$10`
- `mnemonic` + `operand`: `LDA #$10`
- `directive`: `.org $8000`

**Invalid combinations** (parser prevents):
- `constant` + `label`: Cannot have both `FOO = 42` and `FOO:` on same line
- `constant` + `mnemonic`: Cannot have `FOO = 42 LDA #$10` on same line

---

## 4. ErrorType (Extended Enum)

**Purpose**: Classify assembler error types for reporting.

### New Variants

```rust
pub enum ErrorType {
    // ... existing variants ...
    SyntaxError,
    UndefinedLabel,
    DuplicateLabel,
    InvalidLabel,
    InvalidMnemonic,
    InvalidOperand,
    RangeError,
    InvalidDirective,

    // NEW: Constant-related errors
    /// Constant used but not defined
    UndefinedConstant,

    /// Constant defined multiple times
    DuplicateConstant,

    /// Same name used for both constant and label
    NameCollision,

    /// Constant value is invalid (out of range, not a literal)
    InvalidConstantValue,
}
```

### Error Descriptions

| Error Variant | Meaning | Detected When | Example |
|---------------|---------|---------------|---------|
| `UndefinedConstant` | Constant referenced but never defined | Pass 2 (encoding) | `LDA #MISSING` where MISSING not defined |
| `DuplicateConstant` | Constant defined twice | Pass 1 (definition) | `MAX = 42` then `MAX = 100` |
| `NameCollision` | Name used as both constant and label | Pass 1 (definition) | `FOO = 42` then `FOO:` |
| `InvalidConstantValue` | Value is not a literal or out of range | Pass 1 (parsing) | `FOO = $10000` (> 16 bits) or `FOO = BAR` (not literal) |

---

## 5. AssemblerError (Unchanged)

**Purpose**: Container for error information with location and message.

### Definition (for reference)

```rust
pub struct AssemblerError {
    pub error_type: ErrorType,
    pub line: usize,           // 1-indexed
    pub column: usize,         // 0-indexed
    pub span: (usize, usize),  // Character positions
    pub message: String,       // Human-readable description
}
```

### Display Format

```
Line {line}, Column {column}: {error_type} - {message}
```

**Examples**:
```
Line 8, Column 15: Undefined Constant - Undefined constant 'MAX_VALUE'
Line 12, Column 5: Duplicate Constant - Duplicate constant 'PAGE_SIZE' (previously defined at line 3)
Line 15, Column 5: Name Collision - Name collision: 'START' is already defined as a label at line 1
```

---

## 6. SymbolTable (Modified API)

**Purpose**: Storage and lookup for symbols (labels and constants).

### API Extensions

```rust
pub struct SymbolTable {
    symbols: Vec<Symbol>,  // Unified storage for both labels and constants
}

impl SymbolTable {
    /// Add a symbol to the table
    ///
    /// Returns error if:
    /// - Symbol with same name already exists (duplicate or collision)
    pub fn add_symbol(
        &mut self,
        name: String,
        value: u16,
        kind: SymbolKind,
        defined_at: usize
    ) -> Result<(), LabelError>;

    /// Look up a symbol by name (case-insensitive)
    pub fn lookup(&self, name: &str) -> Option<&Symbol>;

    /// Get all symbols
    pub fn symbols(&self) -> &[Symbol];
}
```

### Collision Detection Logic

**Implemented in `add_symbol()`**:

```rust
fn add_symbol(&mut self, name: String, value: u16, kind: SymbolKind, line: usize) -> Result<()> {
    if let Some(existing) = self.lookup(&name) {
        // Collision detected
        match (existing.kind, kind) {
            (SymbolKind::Constant, SymbolKind::Constant) => {
                Err(DuplicateConstant { name, line, original_line: existing.defined_at })
            },
            (SymbolKind::Label, SymbolKind::Label) => {
                Err(DuplicateLabel { name, line, original_line: existing.defined_at })
            },
            (_, _) => {
                Err(NameCollision {
                    name,
                    existing_kind: existing.kind,
                    existing_line: existing.defined_at,
                    new_line: line
                })
            }
        }
    } else {
        self.symbols.push(Symbol { name, value, kind, defined_at: line });
        Ok(())
    }
}
```

---

## Entity Lifecycle

### Pass 1: Symbol Table Building

```
┌─────────────────────┐
│  Parse source line  │
└──────────┬──────────┘
           │
           ▼
    Is constant (has '=')?
           │
    ┌──────┴──────┐
   YES            NO
    │              │
    ▼              ▼
Parse constant   Parse label
NAME = VALUE     NAME:
    │              │
    ▼              ▼
Extract name    Extract name
Extract value   Calculate address
    │              │
    ▼              ▼
Validate       Validate
literal number  label name
    │              │
    ▼              ▼
Add to symbol   Add to symbol
table with      table with
kind=Constant   kind=Label
    │              │
    └──────┬───────┘
           │
           ▼
    Collision check
    (unified table)
           │
    ┌──────┴──────┐
   OK           ERROR
    │              │
    ▼              ▼
Continue      Return error
next line     (DuplicateConstant,
              DuplicateLabel,
              NameCollision)
```

### Pass 2: Operand Resolution

```
┌─────────────────────┐
│  Encode instruction │
└──────────┬──────────┘
           │
           ▼
    Parse operand
    (e.g., "#MAX")
           │
           ▼
    Is identifier?
    (no prefix $, #, etc.)
           │
    ┌──────┴──────┐
   YES            NO
    │              │
    ▼              ▼
Lookup symbol    Parse as
in table         literal value
    │              │
    ▼              │
  Found?           │
    │              │
 ┌──┴──┐           │
YES    NO          │
 │      │          │
 ▼      ▼          │
Get    Error:      │
value  Undefined   │
 │     Constant    │
 │     or          │
 │     Undefined   │
 │     Label       │
 └──────┬──────────┘
        │
        ▼
Apply addressing
mode detection
        │
        ▼
Generate machine code
```

---

## Data Validation Rules

### Constant Name Validation

```rust
fn validate_constant_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Constant name cannot be empty".into());
    }

    if name.len() > 32 {
        return Err("Constant name too long (max 32 characters)".into());
    }

    let first_char = name.chars().next().unwrap();
    if !first_char.is_alphabetic() {
        return Err("Constant name must start with a letter".into());
    }

    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("Constant name contains invalid characters (only alphanumeric and underscore allowed)".into());
    }

    Ok(())
}
```

### Constant Value Validation

```rust
fn validate_constant_value(value_str: &str) -> Result<u16, String> {
    // Detect format by prefix
    let value = if value_str.starts_with('$') {
        // Hexadecimal
        u16::from_str_radix(&value_str[1..], 16)
            .map_err(|_| "Invalid hexadecimal value")?
    } else if value_str.starts_with('%') {
        // Binary
        u16::from_str_radix(&value_str[1..], 2)
            .map_err(|_| "Invalid binary value")?
    } else {
        // Decimal
        value_str.parse::<u16>()
            .map_err(|_| "Invalid decimal value")?
    };

    // Value automatically in range 0-65535 due to u16 type
    Ok(value)
}
```

---

## State Transitions

### Constant Definition States

```
┌─────────────┐
│  UNDEFINED  │ ← Initial state (no symbol exists)
└──────┬──────┘
       │
       │ Parse "NAME = VALUE"
       ▼
┌─────────────┐
│   PARSING   │
└──────┬──────┘
       │
       │ Validate name & value
       ▼
┌─────────────┐
│  VALIDATED  │
└──────┬──────┘
       │
       │ Check for collisions
       │
  ┌────┴────┐
  │         │
  ▼         ▼
COLLISION   NO COLLISION
  │         │
  │         ▼
  │    ┌─────────────┐
  │    │   DEFINED   │ ← Symbol added to table
  │    └──────┬──────┘
  │           │
  └───────────┤
              │
              ▼
         (End state)
```

### Constant Usage States

```
┌─────────────┐
│   UNKNOWN   │ ← Operand contains identifier
└──────┬──────┘
       │
       │ Lookup in symbol table
       │
  ┌────┴────┐
  │         │
  ▼         ▼
FOUND     NOT FOUND
  │         │
  │         └──→ UndefinedConstant error
  │
  ├── kind=Constant → Substitute literal value
  └── kind=Label → Use memory address
```

---

## Migration Path

### Breaking Changes

1. **Symbol struct**:
   - Field `address: u16` → `value: u16` (renamed)
   - New field `kind: SymbolKind` (added)

### Required Code Updates

| Location | Change | Count |
|----------|--------|-------|
| src/assembler.rs | Replace `symbol.address` with `symbol.value` | 5 |
| src/assembler/symbol_table.rs | Update unit tests | 3 |
| tests/assembler_tests.rs | Update test assertions | 4 |

### Backward Compatibility

- **Assembly source code**: Fully backward compatible (constants are optional)
- **Rust API**: **Breaking change** (field rename requires updates)
- **Semver impact**: Minor version bump in pre-1.0 (0.x.y → 0.(x+1).0)

---

## Summary

**New entities**: 1 (SymbolKind enum)
**Modified entities**: 3 (Symbol, AssemblyLine, ErrorType)
**Unchanged entities**: 2 (AssemblerError, SymbolTable structure)

**Data relationships**: Unified symbol table with type discrimination via enum
**Validation rules**: Strict name/value validation, collision detection
**State management**: Single-pass resolution for constants (Pass 1 definition, Pass 2 usage)

All entities align with research findings and spec requirements.
