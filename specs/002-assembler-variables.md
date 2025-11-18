# Feature Specification: Assembler Variables

## Overview

Add support for named constants (variables) in the 6502 assembler, allowing users to define reusable values at the top of assembly files and reference them throughout the code.

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

### Variable Declaration

Variables are declared using the `.define` or `.equ` directive (both synonyms):

```assembly
; Define constants
.define SCREEN_ADDR $8000
.define MAX_LIVES 3
.equ CHAR_A $41
.equ SPRITE_X %00100000

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
- Directive: `.define NAME VALUE` or `.equ NAME VALUE`
- Must appear before first use
- Case-insensitive directive (`.DEFINE`, `.Define`, `.define`)
- Variable name follows same rules as labels:
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
.define ZERO_PAGE_START $00
.define STACK_START $0100
.define IO_BASE $8000

.org $8000
START:
    LDA #ZERO_PAGE_START  ; LDA #$00
    LDX #$10
    STA IO_BASE,X         ; STA $8000,X
```

### Screen/Character Constants

```assembly
.define SCREEN_BASE $4000
.define CHAR_SPACE $20
.define CHAR_STAR $2A
.define WIDTH 40
.define HEIGHT 25

CLEAR_SCREEN:
    LDX #WIDTH
    LDY #HEIGHT
    LDA #CHAR_SPACE
LOOP:
    STA SCREEN_BASE,X
    ; ... more code
```

### Zero Page Variables

```assembly
.define ZP_TEMP $80
.define ZP_COUNTER $81
.define ZP_POINTER $82

    LDA #$00
    STA ZP_TEMP          ; STA $80 (zero page)
    INC ZP_COUNTER       ; INC $81 (zero page)
    LDA (ZP_POINTER),Y   ; LDA ($82),Y (indirect indexed)
```

## Design Decisions

### 1. Variables vs. Labels

**Labels** (existing):
- Defined with `:` suffix
- Represent memory addresses
- Resolved during two-pass assembly
- Type: `SymbolKind::Label`

**Variables** (new):
- Defined with `.define` or `.equ`
- Represent literal values
- Resolved immediately (no forward references needed)
- Type: `SymbolKind::Variable`

### 2. Symbol Table Changes

Extend `Symbol` struct:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Label,      // Memory address (e.g., "START:")
    Variable,   // Literal value (e.g., ".define FOO 42")
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
1. Parse `.define`/`.equ` directives immediately
2. Add variables to symbol table as encountered
3. Variables must be defined before use (no forward references)
4. Process labels as before (addresses calculated)

**Pass 2:**
- Resolve both variables and labels when encoding operands
- Variables substitute their literal value
- Labels resolve to their memory address

### 4. Name Collision Handling

- Variables and labels share the same namespace
- Error if same name used for both variable and label
- Error if variable or label redefined

### 5. Addressing Mode Interaction

When a variable is used in an operand:

```assembly
.define VALUE $42

LDA #VALUE      ; Immediate: LDA #$42
LDA VALUE       ; Zero page or absolute, depends on value
                ; $42 = zero page (2 bytes)
                ; $0042 = force absolute (3 bytes)
```

The assembler:
1. Substitutes variable value
2. Applies normal addressing mode detection
3. Uses hex digit count heuristic (2 digits = ZP, 4 = absolute)

## Implementation Plan

### Phase 1: Core Infrastructure

1. **Extend parser** (`src/assembler/parser.rs`):
   - Add `AssemblerDirective::Define { name: String, value: u16 }`
   - Parse `.define` and `.equ` directives
   - Validate variable names (same as labels)

2. **Extend symbol system** (`src/assembler/symbol_table.rs`):
   - Add `SymbolKind` enum
   - Update `Symbol` struct with `kind` field
   - Rename `address` to `value` for clarity
   - Add methods: `add_variable()`, `lookup()`, `is_variable()`

3. **Update assembler** (`src/assembler.rs`):
   - Process `.define` directives in Pass 1
   - Add variables to symbol table immediately
   - Check for duplicate names (variable/label collision)

4. **Update operand resolution** (`src/assembler/encoder.rs`):
   - When resolving operand, check if it's a variable name
   - If variable, substitute value
   - If label, use address (existing behavior)
   - Apply addressing mode detection to resolved value

### Phase 2: Error Handling

1. **New error types**:
   - `ErrorType::UndefinedVariable` - Variable used before definition
   - `ErrorType::DuplicateVariable` - Variable defined twice
   - `ErrorType::NameCollision` - Same name used for variable and label
   - `ErrorType::InvalidVariableValue` - Value out of range

2. **Error messages**:
   - "Variable 'FOO' used but not defined"
   - "Variable 'BAR' already defined at line X"
   - "Name 'START' used as both variable and label"

### Phase 3: Testing

**Unit tests** (in `src/assembler/parser.rs`):
- Parse `.define` directive
- Parse `.equ` directive
- Parse various value formats (hex, decimal, binary)
- Validate variable names
- Reject invalid syntax

**Integration tests** (in `tests/assembler_tests.rs`):
- Basic variable definition and usage
- Multiple variables
- Variables with different number formats
- Variables in different addressing modes
- Variable name validation
- Error cases (undefined, duplicate, collision)
- Variables with labels in same program
- Complex program with many variables

**Example test case**:

```rust
#[test]
fn test_basic_variable_definition() {
    let source = r#"
        .define VALUE $42
        LDA #VALUE
    "#;

    let output = assemble(source).unwrap();
    assert_eq!(output.bytes, vec![0xA9, 0x42]);
}

#[test]
fn test_undefined_variable_error() {
    let source = "LDA #UNDEFINED";

    let result = assemble(source);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert_eq!(errors[0].error_type, ErrorType::UndefinedVariable);
}
```

## Future Enhancements

**Not in initial version:**

1. **Expression support**: `.define FOO (BAR + 2)`
2. **Arithmetic**: `.define RESULT $1000 + $0020`
3. **String constants**: `.define MSG "HELLO"`
4. **Preprocessor conditionals**: `.ifdef`, `.ifndef`
5. **Local variables**: Variables scoped to labels

## Compatibility

- Backwards compatible: existing assembly code works unchanged
- New `.define`/`.equ` directive is optional
- No changes to existing assembler behavior
- Symbol table extended but old fields preserved (renamed for clarity)

## Testing Strategy

### Test Coverage

1. **Happy path**: Variables work in all addressing modes
2. **Edge cases**: Boundary values (0, 255, 256, 65535)
3. **Error handling**: All error types covered
4. **Integration**: Variables + labels + directives together
5. **Regression**: Existing assembler tests still pass

### Example Programs

**Test program 1: Screen clear**
```assembly
.define SCREEN $4000
.define CHAR_SPACE $20
.define SCREEN_SIZE 1024

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
.define UART_DATA $8000
.define UART_STATUS $8001
.define TX_READY %00000001

SEND_CHAR:
    LDA UART_STATUS
    AND #TX_READY
    BEQ SEND_CHAR
    LDA #$41  ; 'A'
    STA UART_DATA
    RTS
```

## Success Criteria

- [ ] Variables can be defined with `.define` and `.equ`
- [ ] Variables work in all addressing modes
- [ ] Variable names follow label naming rules
- [ ] Undefined variable usage produces clear error
- [ ] Duplicate variable definition produces error
- [ ] Variable/label name collision detected
- [ ] All tests pass (unit + integration)
- [ ] Documentation updated
- [ ] Example programs provided

## Documentation Updates

1. Update `CLAUDE.md` with variable syntax
2. Add example to `examples/variables.rs`
3. Update assembler module documentation
4. Add to README if appropriate

---

## Open Questions

1. Should variables be case-sensitive or normalized to uppercase like labels?
   - **Decision**: Normalize to uppercase (consistent with labels)

2. Should `.define` and `.equ` be synonyms or have different behavior?
   - **Decision**: Synonyms (both do the same thing)

3. Should variables support forward references?
   - **Decision**: No (simpler implementation, defined-before-use is clear)

4. What's the maximum value for a variable?
   - **Decision**: 16-bit (0-65535), same as addresses
