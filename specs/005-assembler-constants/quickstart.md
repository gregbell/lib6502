# Quickstart: Assembler Constants

**Feature**: Named Constants for 6502 Assembler
**Syntax**: `NAME = VALUE`

---

## Overview

Constants let you define reusable values at the top of your assembly file and reference them throughout your code. This makes your programs more maintainable and self-documenting.

**Before (using literals):**
```assembly
.org $8000
START:
    LDA #$20        ; What is $20?
    STA $4000       ; What is $4000?
    LDA #40         ; What is 40?
```

**After (using constants):**
```assembly
CHAR_SPACE = $20
SCREEN_BASE = $4000
SCREEN_WIDTH = 40

.org $8000
START:
    LDA #CHAR_SPACE      ; Clear screen with spaces
    STA SCREEN_BASE      ; Write to screen
    LDA #SCREEN_WIDTH    ; Use screen width
```

---

## Syntax

### Defining Constants

```assembly
NAME = VALUE
```

**Rules:**
- `NAME` must start with a letter [a-zA-Z]
- `NAME` can contain letters, digits, and underscores [a-zA-Z0-9_]
- `NAME` is case-insensitive (normalized to UPPERCASE)
- `NAME` maximum 32 characters
- `VALUE` must be a literal number (hex, decimal, or binary)

**Valid examples:**
```assembly
MAX = 255              ; Decimal
SCREEN_ADDR = $4000    ; Hexadecimal
BITS = %11110000       ; Binary
ZP_TEMP = $80          ; Short hex (zero page)
```

**Invalid examples:**
```assembly
1ST_VALUE = 42         ; ❌ Name starts with digit
MAX SIZE = 100         ; ❌ Name contains space
FOO = BAR              ; ❌ Value is not a literal (expressions not supported in v1)
BAZ = $1000 + $20      ; ❌ Expressions not supported in v1
```

---

## Using Constants

### Immediate Addressing

```assembly
MAX_LIVES = 3

    LDA #MAX_LIVES     ; LDA #3
    STA $8000
```

### Zero Page Addressing

```assembly
ZP_COUNTER = $80

    INC ZP_COUNTER     ; INC $80
    LDA ZP_COUNTER     ; LDA $80
```

### Absolute Addressing

```assembly
SCREEN_BASE = $4000

    STA SCREEN_BASE    ; STA $4000
    LDA SCREEN_BASE    ; LDA $4000
```

### Indexed Addressing

```assembly
IO_BASE = $8000

    STA IO_BASE,X      ; STA $8000,X
    LDA IO_BASE,Y      ; LDA $8000,Y
```

### Indirect Addressing

```assembly
ZP_POINTER = $82

    LDA (ZP_POINTER),Y   ; LDA ($82),Y
```

---

## Complete Example: Screen Clear

```assembly
; Define constants
SCREEN_BASE = $4000
SCREEN_SIZE = 1000
CHAR_SPACE = $20

; Define program origin
.org $8000

; Clear screen routine
CLEAR_SCREEN:
    LDX #$00
    LDA #CHAR_SPACE
LOOP:
    STA SCREEN_BASE,X
    STA SCREEN_BASE+256,X
    STA SCREEN_BASE+512,X
    STA SCREEN_BASE+768,X
    INX
    CPX #(SCREEN_SIZE/4)  ; Note: Division not yet supported, use literal
    BNE LOOP
    RTS
```

---

## Complete Example: UART Communication

```assembly
; UART memory-mapped I/O
UART_DATA = $8000
UART_STATUS = $8001
TX_READY_BIT = %00000001

; ASCII characters
CHAR_A = $41
CHAR_NEWLINE = $0A

.org $8000

; Send a character via UART
SEND_CHAR:
    LDA UART_STATUS
    AND #TX_READY_BIT
    BEQ SEND_CHAR        ; Wait until TX ready
    LDA #CHAR_A          ; Load 'A'
    STA UART_DATA        ; Send
    RTS

; Send newline
SEND_NEWLINE:
    LDA UART_STATUS
    AND #TX_READY_BIT
    BEQ SEND_NEWLINE
    LDA #CHAR_NEWLINE
    STA UART_DATA
    RTS
```

---

## Complete Example: Zero Page Variables

```assembly
; Define zero page locations
ZP_TEMP = $80
ZP_COUNTER = $81
ZP_POINTER = $82
ZP_POINTER_HI = $83

; Screen buffer
SCREEN = $4000
CHAR_STAR = $2A

.org $8000

DRAW_PATTERN:
    ; Initialize pointer
    LDA #<SCREEN
    STA ZP_POINTER
    LDA #>SCREEN
    STA ZP_POINTER_HI

    ; Set counter
    LDA #10
    STA ZP_COUNTER

LOOP:
    ; Draw character
    LDA #CHAR_STAR
    STA (ZP_POINTER),Y

    ; Increment pointer
    INC ZP_POINTER
    BNE NO_CARRY
    INC ZP_POINTER_HI
NO_CARRY:

    ; Decrement counter
    DEC ZP_COUNTER
    BNE LOOP

    RTS
```

---

## Common Patterns

### Character Codes

```assembly
CHAR_SPACE = $20
CHAR_0 = $30
CHAR_A = $41
CHAR_a = $61
CHAR_NEWLINE = $0A
CHAR_RETURN = $0D
```

### Screen Addresses

```assembly
SCREEN_BASE = $4000
SCREEN_WIDTH = 40
SCREEN_HEIGHT = 25
SCREEN_SIZE = 1000     ; 40 * 25
```

### Memory Regions

```assembly
ZERO_PAGE_START = $00
ZERO_PAGE_END = $FF
STACK_START = $0100
STACK_END = $01FF
ROM_START = $8000
ROM_END = $FFFF
```

### I/O Ports

```assembly
IO_BASE = $8000
UART_DATA = $8000
UART_STATUS = $8001
GPIO_DATA = $8010
GPIO_DIRECTION = $8011
```

### Bit Masks

```assembly
BIT_0 = %00000001
BIT_1 = %00000010
BIT_2 = %00000100
BIT_3 = %00001000
BIT_7 = %10000000
ALL_BITS = %11111111
```

---

## Constants vs. Labels

### Labels (memory addresses)

```assembly
START:              ; Label (address of this location)
    LDA #$42
    JMP LOOP        ; Reference label address

LOOP:               ; Another label
    STA $8000
    JMP LOOP
```

Labels are defined with `:` and represent **memory addresses**.

### Constants (literal values)

```assembly
MAX = 255           ; Constant (literal value)

    LDA #MAX        ; Use constant value
    CMP #MAX
```

Constants are defined with `=` and represent **literal values**.

### Key Differences

| Feature | Labels | Constants |
|---------|--------|-----------|
| **Syntax** | `NAME:` | `NAME = VALUE` |
| **Represents** | Memory address | Literal value |
| **Usage** | Jump targets, data locations | Magic numbers, config values |
| **Forward references** | Allowed | **Not allowed** (must define before use) |
| **Example** | `LOOP:`, `START:` | `MAX = 255`, `SCREEN = $4000` |

---

## Rules and Constraints

### Definition Order

**Constants must be defined before use:**

✅ **Valid:**
```assembly
MAX = 255
    LDA #MAX        ; MAX defined above
```

❌ **Invalid:**
```assembly
    LDA #MAX        ; ❌ Error: MAX not defined yet
MAX = 255
```

### Namespace

**Constants and labels share the same namespace:**

❌ **Invalid:**
```assembly
FOO = 42
FOO:                ; ❌ Error: Name collision
    LDA #$10
```

✅ **Valid:**
```assembly
FOO = 42
BAR:                ; Different names
    LDA #FOO
```

### Uniqueness

**Constants cannot be redefined:**

❌ **Invalid:**
```assembly
MAX = 100
MAX = 200           ; ❌ Error: Duplicate constant
```

### Literal Values Only (v1)

**Constants must be literal numbers:**

✅ **Valid:**
```assembly
FOO = $4000         ; Hex literal
BAR = 255           ; Decimal literal
BAZ = %11110000     ; Binary literal
```

❌ **Not supported in v1** (future enhancement):
```assembly
DERIVED = FOO       ; ❌ Constant reference
OFFSET = FOO + 1    ; ❌ Expression
DOUBLE = FOO * 2    ; ❌ Arithmetic
```

---

## Error Messages

### Undefined Constant

**Code:**
```assembly
    LDA #MISSING
```

**Error:**
```
Line 5, Column 10: Undefined Constant - Undefined constant 'MISSING'
```

**Fix:** Define the constant before using it:
```assembly
MISSING = 42
    LDA #MISSING
```

---

### Duplicate Constant

**Code:**
```assembly
MAX = 100
MAX = 200
```

**Error:**
```
Line 2, Column 1: Duplicate Constant - Duplicate constant 'MAX' (previously defined at line 1)
```

**Fix:** Use different names or remove duplicate:
```assembly
MAX = 100
MAX2 = 200
```

---

### Name Collision

**Code:**
```assembly
START = $8000
START:
    LDA #$42
```

**Error:**
```
Line 2, Column 1: Name Collision - Name collision: 'START' is already defined as a constant at line 1
```

**Fix:** Use different names for constant and label:
```assembly
ROM_START = $8000
ENTRY_POINT:
    LDA #$42
```

---

### Invalid Constant Value

**Code:**
```assembly
TOO_BIG = $10000
```

**Error:**
```
Line 1, Column 11: Invalid Constant Value - Constant 'TOO_BIG' value $10000 is out of range (must be $0000-$FFFF)
```

**Fix:** Use a value within 16-bit range:
```assembly
MAX_ADDR = $FFFF
```

---

## Migration Guide

### From Literals to Constants

**Step 1: Identify repeated values**
```assembly
; Before
    LDA #$20
    STA $4000
    LDA #$20
    STA $4001
```

**Step 2: Define constants**
```assembly
; After
CHAR_SPACE = $20
SCREEN = $4000

    LDA #CHAR_SPACE
    STA SCREEN
    LDA #CHAR_SPACE
    STA SCREEN+1
```

### No Breaking Changes

**Existing assembly code works unchanged:**
- Constants are optional
- Literals still supported everywhere
- Labels work as before

---

## Best Practices

### 1. Group Related Constants

```assembly
; Screen configuration
SCREEN_BASE = $4000
SCREEN_WIDTH = 40
SCREEN_HEIGHT = 25

; Character codes
CHAR_SPACE = $20
CHAR_STAR = $2A

; Zero page locations
ZP_TEMP = $80
ZP_COUNTER = $81
```

### 2. Use Descriptive Names

```assembly
; ✅ Good
UART_TX_READY_BIT = %00000001

; ❌ Poor
BIT = %00000001
```

### 3. Define Constants at Top of File

```assembly
; Constants first
MAX_LIVES = 3
SCREEN = $4000

; Program code second
.org $8000
START:
    LDA #MAX_LIVES
```

### 4. Use ALL_CAPS for Constants

```assembly
; ✅ Conventional
MAX_VALUE = 255

; ✅ Also works (normalized to UPPERCASE)
max_value = 255   ; → MAX_VALUE
```

### 5. Document Magic Numbers

```assembly
; Good: Explain purpose
BAUD_DIVISOR = 104   ; 9600 baud @ 1MHz clock

; Better: Use constant name that self-documents
BAUD_9600_DIVISOR = 104
```

---

## Limitations (Version 1)

### Not Supported in v1

❌ **Expressions:**
```assembly
RESULT = FOO + BAR    ; Not supported
```

❌ **Constant references:**
```assembly
A = 42
B = A                 ; Not supported
```

❌ **Arithmetic:**
```assembly
DOUBLE = VALUE * 2    ; Not supported
```

❌ **Forward references:**
```assembly
A = B                 ; B not defined yet
B = 42
```

### Planned for v2

These features may be added in future versions:
- Expression evaluation
- Constant-to-constant references
- Arithmetic operators (+, -, *, /)
- Bitwise operators (&, |, ^, ~)

---

## Summary

**What you can do:**
- ✅ Define constants: `NAME = VALUE`
- ✅ Use hex, decimal, binary: `$FF`, `255`, `%11110000`
- ✅ Reference constants in any addressing mode
- ✅ Mix constants and labels in same program

**What you cannot do (v1):**
- ❌ Use forward references (must define before use)
- ❌ Create expressions (literals only)
- ❌ Reference other constants in definitions
- ❌ Redefine constants
- ❌ Use same name for constant and label

**Get help:**
- See error messages for specific fixes
- Check examples above for common patterns
- Refer to spec.md for detailed syntax rules

---

## Additional Examples

### Complete Program: Fibonacci Sequence

```assembly
; Constants
SCREEN = $4000
CHAR_0 = $30
MAX_ITERATIONS = 10
ZP_A = $80
ZP_B = $81
ZP_COUNT = $82

.org $8000

FIBONACCI:
    ; Initialize
    LDA #0
    STA ZP_A
    LDA #1
    STA ZP_B
    LDA #MAX_ITERATIONS
    STA ZP_COUNT

LOOP:
    ; Output ZP_A to screen (simplified)
    LDA ZP_A
    CLC
    ADC #CHAR_0
    STA SCREEN

    ; Calculate next: temp = A + B
    LDA ZP_A
    CLC
    ADC ZP_B
    PHA              ; Save result

    ; Shift: A = B
    LDA ZP_B
    STA ZP_A

    ; B = temp
    PLA
    STA ZP_B

    ; Decrement counter
    DEC ZP_COUNT
    BNE LOOP

    RTS
```

This quickstart guide provides everything users need to start using constants in their 6502 assembly programs!
