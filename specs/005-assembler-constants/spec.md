# Feature Specification: Assembler Constants

## Overview

Add support for named constants in the 6502 assembler, allowing users to define reusable values at the top of assembly files and reference them throughout the code.

## Background

Currently, the assembler supports:
- Labels (memory addresses defined with `:`)
- Literal values (`#$42`, `$1234`, `255`, `%11111111`)
- Directives (`.org`, `.byte`, `.word`)

Missing: Named constants for frequently used values (e.g., screen addresses, character codes, magic numbers).

## User Story

As an assembly programmer, I want to:
1. Define named constants at the top of my assembly file
2. Use those constants anywhere in my code
3. Get clear errors if I reference undefined variables
4. Distinguish between address labels and value constants

## Syntax

### Constant Declaration

Constants are declared using simple assignment syntax:

```assembly
; Define constants
SCREEN_ADDR = $8000
MAX_LIVES = 3
CHAR_A = $41
SPRITE_X = %00100000

; Use in code
START:
    LDA #MAX_LIVES
    STA SCREEN_ADDR
    LDA #CHAR_A
    ORA #SPRITE_X
    RTS
```

### Syntax Rules

**Declaration:**
- Syntax: `NAME = VALUE`
- Must appear before first use
- Constant name follows same rules as labels:
  - Start with letter [a-zA-Z]
  - Contain only alphanumeric + underscore [a-zA-Z0-9_]
  - Maximum 32 characters
  - Case-normalized to UPPERCASE

**Value:**
- Must be a literal number (hex `$FF`, decimal `255`, binary `%11111111`)
- No expressions in initial version (future enhancement)
- Range: 0-65535 (16-bit value)

**Usage:**
- Reference by name in any operand position
- Assembler substitutes the literal value
- Works with all addressing modes

## Examples

### Basic Usage

```assembly
ZERO_PAGE_START = $00
STACK_START = $0100
IO_BASE = $8000

.org $8000
START:
    LDA #ZERO_PAGE_START  ; LDA #$00
    LDX #$10
    STA IO_BASE,X         ; STA $8000,X
```

### Screen/Character Constants

```assembly
SCREEN_BASE = $4000
CHAR_SPACE = $20
CHAR_STAR = $2A
WIDTH = 40
HEIGHT = 25

CLEAR_SCREEN:
    LDX #WIDTH
    LDY #HEIGHT
    LDA #CHAR_SPACE
LOOP:
    STA SCREEN_BASE,X
    ; ... more code
```

### Zero Page Constants

```assembly
ZP_TEMP = $80
ZP_COUNTER = $81
ZP_POINTER = $82

    LDA #$00
    STA ZP_TEMP          ; STA $80 (zero page)
    INC ZP_COUNTER       ; INC $81 (zero page)
    LDA (ZP_POINTER),Y   ; LDA ($82),Y (indirect indexed)
```

## Design Decisions

### 1. Constants vs. Labels

**Labels** (existing):
- Defined with `:` suffix
- Represent memory addresses
- Resolved during two-pass assembly
- Type: `SymbolKind::Label`

**Constants** (new):
- Defined with `=` assignment syntax
- Represent literal values
- Resolved immediately (no forward references needed)
- Type: `SymbolKind::Constant`

### 2. Symbol Table Changes

Extend `Symbol` struct:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Label,      // Memory address (e.g., "START:")
    Constant,   // Literal value (e.g., "FOO = 42")
}

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub value: u16,          // Renamed from 'address'
    pub kind: SymbolKind,
    pub defined_at: usize,
}
```

### 3. Resolution Order

**Pass 1:**
1. Parse constant assignments (`NAME = VALUE`) immediately
2. Add constants to symbol table as encountered
3. Constants must be defined before use (no forward references)
4. Process labels as before (addresses calculated)

**Pass 2:**
- Resolve both constants and labels when encoding operands
- Constants substitute their literal value
- Labels resolve to their memory address

### 4. Name Collision Handling

- Constants and labels share the same namespace
- Error if same name used for both constant and label
- Error if constant or label redefined

### 5. Addressing Mode Interaction

When a constant is used in an operand:

```assembly
VALUE = $42

LDA #VALUE      ; Immediate: LDA #$42
LDA VALUE       ; Zero page or absolute, depends on value
                ; $42 = zero page (2 bytes)
                ; $0042 = force absolute (3 bytes)
```

The assembler:
1. Substitutes constant value
2. Applies normal addressing mode detection
3. Uses hex digit count heuristic (2 digits = ZP, 4 = absolute)

## Implementation Plan

### Phase 1: Core Infrastructure

1. **Extend parser** (`src/assembler/parser.rs`):
   - Detect constant assignment syntax (`NAME = VALUE`)
   - Parse constant name and value
   - Validate constant names (same as labels)
   - Distinguish from label definitions (which have `:`)

2. **Extend symbol system** (`src/assembler/symbol_table.rs`):
   - Add `SymbolKind` enum
   - Update `Symbol` struct with `kind` field
   - Rename `address` to `value` for clarity
   - Add methods: `add_constant()`, `lookup()`, `is_constant()`

3. **Update assembler** (`src/assembler.rs`):
   - Process constant assignments in Pass 1
   - Add constants to symbol table immediately
   - Check for duplicate names (constant/label collision)

4. **Update operand resolution** (`src/assembler/encoder.rs`):
   - When resolving operand, check if it's a constant name
   - If constant, substitute value
   - If label, use address (existing behavior)
   - Apply addressing mode detection to resolved value

### Phase 2: Error Handling

1. **New error types**:
   - `ErrorType::UndefinedConstant` - Constant used before definition
   - `ErrorType::DuplicateConstant` - Constant defined twice
   - `ErrorType::NameCollision` - Same name used for constant and label
   - `ErrorType::InvalidConstantValue` - Value out of range

2. **Error messages**:
   - "Constant 'FOO' used but not defined"
   - "Constant 'BAR' already defined at line X"
   - "Name 'START' used as both constant and label"

### Phase 3: Testing

**Unit tests** (in `src/assembler/parser.rs`):
- Parse constant assignment (`NAME = VALUE`)
- Parse various value formats (hex, decimal, binary)
- Validate constant names
- Reject invalid syntax
- Distinguish constant assignment from label definition

**Integration tests** (in `tests/assembler_tests.rs`):
- Basic constant definition and usage
- Multiple constants
- Constants with different number formats
- Constants in different addressing modes
- Constant name validation
- Error cases (undefined, duplicate, collision)
- Constants with labels in same program
- Complex program with many constants

**Example test case**:

```rust
#[test]
fn test_basic_constant_definition() {
    let source = r#"
        VALUE = $42
        LDA #VALUE
    "#;

    let output = assemble(source).unwrap();
    assert_eq!(output.bytes, vec![0xA9, 0x42]);
}

#[test]
fn test_undefined_constant_error() {
    let source = "LDA #UNDEFINED";

    let result = assemble(source);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert_eq!(errors[0].error_type, ErrorType::UndefinedConstant);
}
```

## Future Enhancements

**Not in initial version:**

1. **Expression support**: `FOO = (BAR + 2)`
2. **Arithmetic**: `RESULT = $1000 + $0020`
3. **String constants**: `MSG = "HELLO"`
4. **Preprocessor conditionals**: `.ifdef`, `.ifndef`
5. **Local constants**: Constants scoped to labels

## Compatibility

- Backwards compatible: existing assembly code works unchanged
- New constant assignment syntax is optional
- No changes to existing assembler behavior
- Symbol table extended but old fields preserved (renamed for clarity)

## Testing Strategy

### Test Coverage

1. **Happy path**: Constants work in all addressing modes
2. **Edge cases**: Boundary values (0, 255, 256, 65535)
3. **Error handling**: All error types covered
4. **Integration**: Constants + labels + directives together
5. **Regression**: Existing assembler tests still pass

### Example Programs

**Test program 1: Screen clear**
```assembly
SCREEN = $4000
CHAR_SPACE = $20
SCREEN_SIZE = 1024

.org $8000
CLEAR:
    LDX #$00
    LDA #CHAR_SPACE
LOOP:
    STA SCREEN,X
    STA SCREEN+256,X
    STA SCREEN+512,X
    STA SCREEN+768,X
    INX
    BNE LOOP
    RTS
```

**Test program 2: I/O ports**
```assembly
UART_DATA = $8000
UART_STATUS = $8001
TX_READY = %00000001

SEND_CHAR:
    LDA UART_STATUS
    AND #TX_READY
    BEQ SEND_CHAR
    LDA #$41  ; 'A'
    STA UART_DATA
    RTS
```

## Success Criteria

- [ ] Constants can be defined with `=` assignment syntax
- [ ] Constants work in all addressing modes
- [ ] Constant names follow label naming rules
- [ ] Undefined constant usage produces clear error
- [ ] Duplicate constant definition produces error
- [ ] Constant/label name collision detected
- [ ] All tests pass (unit + integration)
- [ ] Documentation updated
- [ ] Example programs provided

## Documentation Updates

1. Update `CLAUDE.md` with constant syntax
2. Add example to `examples/constants.rs`
3. Update assembler module documentation
4. Add to README if appropriate

---

## Open Questions

1. Should constants be case-sensitive or normalized to uppercase like labels?
   - **Decision**: Normalize to uppercase (consistent with labels)

2. Should constants support forward references?
   - **Decision**: No (simpler implementation, defined-before-use is clear)

3. What's the maximum value for a constant?
   - **Decision**: 16-bit (0-65535), same as addresses
