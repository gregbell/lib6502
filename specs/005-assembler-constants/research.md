# Research: Assembler Constants Implementation

**Feature**: 005-assembler-constants
**Date**: 2025-11-18
**Status**: Research Complete

This document consolidates research findings for implementing named constant support in the 6502 assembler.

---

## Research Summary

### Research Tasks Completed

1. ✅ Parser Implementation Pattern
2. ✅ Symbol Table Refactoring Strategy
3. ✅ Error Message Design
4. ✅ Forward Reference Handling
5. ✅ Name Collision Edge Cases

---

## 1. Parser Implementation Pattern

### Decision: Insert Constant Detection Before Label Detection

**Current parse order** (src/assembler/parser.rs lines 61-155):
1. Trim whitespace
2. Check if empty
3. Strip comments (`;` to end)
4. **INSERTION POINT**: Check for constant assignment (`=`)
5. Check for label (`:`)
6. Parse mnemonic and operand

### Implementation Strategy

**Detect `=` before `:` in parse flow:**
```rust
// After comment stripping, before label detection:
if let Some(eq_pos) = code_part.find('=') {
    // Constant assignment: NAME = VALUE
    let name_part = code_part[..eq_pos].trim();
    let value_part = code_part[eq_pos + 1..].trim();

    // Validate name (no internal spaces)
    if !name_part.is_empty() && !name_part.contains(char::is_whitespace) {
        return Some(AssemblyLine {
            constant: Some((name_part.to_uppercase(), value_part.to_string())),
            label: None,
            mnemonic: None,
            ...
        });
    }
} else if let Some(colon_pos) = code_part.find(':') {
    // Existing label detection (unchanged)
    ...
}
```

### Why This Works

- Comments stripped first, so `;note=test` won't confuse parser
- Checking `=` before `:` ensures no ambiguity
- Lines have either `=` (constant) or `:` (label), not both
- Operands come after mnemonics on different lines, so `LDA #VALUE` won't trigger

### Edge Cases Handled

| Case | Input | Handling |
|------|-------|----------|
| Label only | `START:` | No `=` found → label |
| Constant only | `MAX = $FF` | `=` found first → constant |
| Label + instruction | `LOOP: LDA #$10` | No `=`, has `:` → label + instruction |
| Comment with `=` | `FOO = $42 ; test=note` | Comment stripped first → safe |
| Multiple `=` | `X = Y = Z` | First `=` found, rest is value (validation catches error) |

### Files to Modify

- **src/assembler/parser.rs** (line ~102): Insert constant detection
- **src/assembler/parser.rs** (lines 6-28): Extend `AssemblyLine` with `constant` field

---

## 2. Symbol Table Refactoring Strategy

### Decision: Rename `address` → `value` (Breaking Change Accepted)

**Current Symbol struct** (src/assembler.rs lines 68-79):
```rust
pub struct Symbol {
    pub name: String,
    pub address: u16,
    pub defined_at: usize,
}
```

**New Symbol struct**:
```rust
pub enum SymbolKind {
    Label,      // Memory address
    Constant,   // Literal value
}

pub struct Symbol {
    pub name: String,
    pub value: u16,          // Renamed from 'address'
    pub kind: SymbolKind,
    pub defined_at: usize,
}
```

### Rationale

1. **Semantic Correctness**: Field stores both addresses (labels) and values (constants). `value` is more accurate.
2. **Low Impact**: Only 12 references in codebase (5 source, 4 tests, 3 docs)
3. **Pre-1.0 Project**: Breaking changes acceptable now, not later
4. **Spec Alignment**: Spec already decided on this approach (spec.md line 144)
5. **Future-Proof**: Adding `SymbolKind` makes `address` confusing for constants

### Impact Analysis

**References to `symbol.address`: 12 locations**

- src/assembler.rs: 5 references
- src/assembler/symbol_table.rs: 3 unit tests
- tests/assembler_tests.rs: 4 assertions

**Migration effort**: ~15 minutes (find-replace + testing)

### Alternative Considered (Rejected)

**Option: Keep `address`, add `value()` accessor**
- Pros: Backward compatible
- Cons: Field name still semantically incorrect, doesn't solve confusion

**Decision**: Rename field. Project is pre-1.0, breaking changes are acceptable for better API design.

---

## 3. Error Message Design

### Decision: Follow Existing Error Patterns

**Existing error format** (src/assembler.rs lines 165-185):
```
Line {}, Column {}: {} - {}
└─ line │ column │ ErrorType │ message
```

### New Error Types

Add 4 variants to `ErrorType` enum:

```rust
pub enum ErrorType {
    // Existing...
    SyntaxError,
    UndefinedLabel,
    DuplicateLabel,
    InvalidLabel,
    InvalidMnemonic,
    InvalidOperand,
    RangeError,
    InvalidDirective,

    // NEW for constants:
    UndefinedConstant,
    DuplicateConstant,
    NameCollision,
    InvalidConstantValue,
}
```

### Error Message Templates

| Error Type | Message Template | Example |
|-----------|------------------|---------|
| `UndefinedConstant` | `Undefined constant '{}'` | `Line 8, Column 15: Undefined Constant - Undefined constant 'MAX_VALUE'` |
| `DuplicateConstant` | `Duplicate constant '{}' (previously defined at line {})` | `Line 12, Column 5: Duplicate Constant - Duplicate constant 'PAGE_SIZE' (previously defined at line 3)` |
| `NameCollision` | `Name collision: '{}' is already defined as a {} at line {}` | `Line 15, Column 5: Name Collision - Name collision: 'START' is already defined as a label at line 1` |
| `InvalidConstantValue` | `Constant '{}' value ${:04X} is out of range (must be $0000-$FFFF)` | `Line 6, Column 20: Invalid Constant Value - Constant 'ADDR' value $10000 is out of range (must be $0000-$FFFF)` |

### Error Collection Strategy

**Pass 1**: Collect definition errors
- `DuplicateConstant`
- `NameCollision`
- `InvalidConstantValue`

**Pass 2**: Collect usage errors
- `UndefinedConstant`

**Pattern**: Match existing assembler behavior (collect all errors, return at end)

---

## 4. Forward Reference Handling

### Decision: No Forward References in Version 1

**Scope for v1**: Constants must be **literal numbers only**

```assembly
; ✅ VALID in v1:
SCREEN = $4000      ; Hex literal
MAX = 255           ; Decimal literal
CHAR_A = $41        ; Another literal

; ❌ NOT SUPPORTED in v1 (future enhancement):
OFFSET = MAX - 1    ; Expression
DERIVED = SCREEN    ; Constant reference
DOUBLE = SCREEN * 2 ; Arithmetic expression
```

### Rationale

1. **Spec Alignment**: "No expressions in initial version" (spec.md line 59)
2. **Simplicity**: Single-pass resolution during Parse Phase 1
3. **Clarity**: Self-documenting (each constant shows its value explicitly)
4. **Constitution**: Clarity & Hackability principle favors simple implementation
5. **Use Case**: Uncommon in 6502 assembly (literals are typical)

### What "No Forward References" Means

**Defined-before-use applies to usage in code**, not definitions:
- ✅ OK: Define `FOO = 42` on line 5, use `LDA #FOO` on line 10
- ❌ NOT OK: Use `LDA #BAR` on line 5, define `BAR = 42` on line 10

**Constant-to-constant references not supported**:
- ❌ NOT OK: `A = 42`, then `B = A` (even if A defined earlier)
- **Reason**: Treated as expression, not literal

### Resolution Algorithm (v1)

**Single-pass during Parse Phase 1:**
1. Encounter `NAME = VALUE`
2. Validate `VALUE` is a literal number
3. Parse number (hex/decimal/binary)
4. Add to symbol table immediately with `SymbolKind::Constant`
5. No evaluation or dependency resolution needed

### Error Handling

**For non-literal values**:
```rust
if !is_literal_number(value_part) {
    return Err(InvalidConstantValue {
        message: "Constants must be literal numbers (hex, decimal, or binary)"
    });
}
```

### Future Enhancement (v2+)

When expressions are added:
1. Collect all constant assignments
2. Build dependency graph
3. Topological sort
4. Evaluate in order
5. Detect circular references

---

## 5. Name Collision Edge Cases

### Decision: Unified Symbol Table with SymbolKind Enum

**Approach**: Single symbol table containing both labels and constants, distinguished by `SymbolKind`.

### Why Unified Table?

**Pros**:
- Single lookup for collision detection
- Consistent error logic
- No synchronization issues
- Better for future features (scoped symbols, etc.)

**Cons** (none significant):
- Symbol struct slightly larger (+1 enum field)

### Collision Detection Algorithm

**Check at symbol definition time (Pass 1)**:

```rust
fn add_symbol(name: &str, value: u16, kind: SymbolKind, line: usize) -> Result<()> {
    if let Some(existing) = symbol_table.lookup(name) {
        // Collision detected
        match (existing.kind, kind) {
            (SymbolKind::Constant, SymbolKind::Constant) => {
                Err(DuplicateConstant {
                    name,
                    original_line: existing.defined_at,
                    duplicate_line: line
                })
            },
            (SymbolKind::Label, SymbolKind::Label) => {
                Err(DuplicateLabel {
                    name,
                    original_line: existing.defined_at,
                    duplicate_line: line
                })
            },
            (_, _) => {
                // One is constant, one is label
                let existing_type = match existing.kind {
                    SymbolKind::Label => "label",
                    SymbolKind::Constant => "constant",
                };
                Err(NameCollision {
                    name,
                    existing_type,
                    existing_line: existing.defined_at,
                    new_line: line
                })
            }
        }
    } else {
        // No collision, add to table
        symbol_table.insert(Symbol { name, value, kind, defined_at: line });
        Ok(())
    }
}
```

### Detection Timing

**All collisions: Pass 1** (at definition time)
- Constants processed as encountered
- Labels processed as encountered
- Immediate lookup and collision check

**Undefined symbols: Pass 2** (at usage time)
- Operand resolution looks up symbol by name
- If not found: `UndefinedConstant` or `UndefinedLabel` (depending on context)

### Error Priority

If multiple errors on same symbol:
1. **NameCollision** (mixing types) - highest priority
2. **DuplicateConstant** / **DuplicateLabel** (same type)

**No deferral**: All detected and reported in Pass 1.

### Edge Cases

| Case | Input | Expected Behavior |
|------|-------|-------------------|
| Constant then label | `FOO = 42`<br>`FOO:` | NameCollision error at line 2 |
| Label then constant | `START:`<br>`START = 100` | NameCollision error at line 2 |
| Duplicate constant | `MAX = 42`<br>`MAX = 100` | DuplicateConstant error at line 2 |
| Duplicate label | `LOOP:`<br>`LOOP:` | DuplicateLabel error at line 2 |
| Forward reference (constant) | `LDA #BAR`<br>`BAR = 42` | UndefinedConstant error at line 1 (Pass 2) |
| Forward reference (label) | `JMP TARGET`<br>`TARGET:` | OK - labels support forward refs |
| Undefined constant | `LDA #MISSING` (never defined) | UndefinedConstant error (Pass 2) |
| Constant + label coexist | `FOO = 42` and `FOO:` anywhere | NameCollision - **not allowed** |

### Scope Handling

**Global namespace only** (v1):
- Constants and labels share the same global namespace
- No scoping (local constants, function-level, etc.)
- Simple collision detection (single lookup)

**Future enhancement** (v2+): Scoped constants (e.g., labels have local constants)

---

## Implementation Checklist

Based on all research findings:

### Phase 1: Core Infrastructure

- [ ] **Parser** (src/assembler/parser.rs)
  - [ ] Add `constant` field to `AssemblyLine` struct
  - [ ] Insert constant detection before label detection
  - [ ] Validate constant name (no spaces, valid identifier)
  - [ ] Return parsed constant assignment

- [ ] **Symbol System** (src/assembler.rs)
  - [ ] Add `SymbolKind` enum (Label, Constant)
  - [ ] Extend `Symbol` struct with `kind` field
  - [ ] Rename `address` to `value` (12 locations)
  - [ ] Update all references to `symbol.address`

- [ ] **Symbol Table** (src/assembler/symbol_table.rs)
  - [ ] Update `add_symbol()` to accept `kind` parameter
  - [ ] Add collision detection logic
  - [ ] Support lookup by name (already exists)

- [ ] **Assembler Main** (src/assembler.rs)
  - [ ] Process constant assignments in Pass 1
  - [ ] Add constants to symbol table with `SymbolKind::Constant`
  - [ ] Check for name collisions when adding

- [ ] **Encoder** (src/assembler/encoder.rs)
  - [ ] Resolve operands checking symbol kind
  - [ ] Substitute constant values (not addresses)
  - [ ] Apply addressing mode detection to resolved values

### Phase 2: Error Handling

- [ ] **Error Types** (src/assembler.rs)
  - [ ] Add `UndefinedConstant` to `ErrorType` enum
  - [ ] Add `DuplicateConstant` to `ErrorType` enum
  - [ ] Add `NameCollision` to `ErrorType` enum
  - [ ] Add `InvalidConstantValue` to `ErrorType` enum
  - [ ] Update `Display` impl with new error messages

### Phase 3: Testing

- [ ] **Unit Tests** (src/assembler/parser.rs)
  - [ ] Parse `NAME = VALUE` syntax
  - [ ] Parse hex/decimal/binary literals
  - [ ] Validate constant names
  - [ ] Reject invalid syntax

- [ ] **Integration Tests** (tests/assembler_tests.rs)
  - [ ] Basic constant definition and usage
  - [ ] Multiple constants
  - [ ] Constants in all addressing modes
  - [ ] Undefined constant error
  - [ ] Duplicate constant error
  - [ ] Name collision error
  - [ ] Constants + labels together
  - [ ] Klaus functional test still passes

---

## Technology Best Practices Applied

### Rust Enum Design (SymbolKind)

**Pattern**: Simple enum with no data, pattern matching for type discrimination

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Label,
    Constant,
}

// Usage: Pattern matching
match symbol.kind {
    SymbolKind::Label => { /* use symbol.value as address */ },
    SymbolKind::Constant => { /* use symbol.value as literal */ },
}
```

**Best practices followed**:
- Derive `Copy` for cheap passing
- Derive `PartialEq, Eq` for comparisons
- Simple variants (no data - data lives in Symbol struct)
- Explicit matching (no default cases)

### Assembler Two-Pass Architecture

**Pattern confirmed**: Standard for 6502 assemblers (ca65, DASM, ASM6, etc.)

**Pass 1**: Build symbol table
- Collect all labels (calculate addresses)
- Collect all constants (parse values)
- Detect duplicate definitions
- Detect name collisions

**Pass 2**: Encode instructions
- Resolve label references to addresses
- Resolve constant references to values
- Detect undefined symbols
- Generate machine code

**Constants fit naturally**: Processed in Pass 1, resolved in Pass 2 (same as labels).

---

## Open Questions Resolved

### Question: Parser insertion point?
**Answer**: Before label detection, after comment stripping (line ~102 in parser.rs)

### Question: Rename field or add accessor?
**Answer**: Rename `address` → `value` (breaking change accepted for better API)

### Question: Separate or unified table?
**Answer**: Unified symbol table with `SymbolKind` enum

### Question: Support forward references?
**Answer**: No in v1 (literals only). Future enhancement for expressions.

### Question: When to check collisions?
**Answer**: Pass 1 at definition time (immediate detection)

---

## Next Steps

With research complete, proceed to **Phase 1: Design Artifacts**:

1. Generate `data-model.md` - Entity designs (SymbolKind, Symbol, AssemblyLine, ErrorType)
2. Generate `contracts/` - Internal API contracts for testing
3. Generate `quickstart.md` - User-facing guide for constant syntax
4. Update agent context - Run update-agent-context.sh
5. Re-evaluate Constitution Check - Verify compliance post-design

All research findings documented here will inform those artifacts.
