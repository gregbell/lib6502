# Assembler API Contract

**Module**: `lib6502::assembler`
**Version**: 0.1.0

## Public API

### Function: `assemble`

Assembles 6502 assembly source code into machine code.

```rust
pub fn assemble(source: &str) -> Result<AssemblerOutput, Vec<AssemblerError>>
```

**Parameters**:
- `source`: Assembly source code as string

**Returns**:
- `Ok(AssemblerOutput)`: Successful assembly with bytes, symbol table, and source map
- `Err(Vec<AssemblerError>)`: Collection of all errors encountered

**Behavior**:
- **Pass 1**: Parse all lines, build symbol table, calculate addresses
- **Pass 2**: Encode instructions, resolve label references, build source map
- Collects ALL errors (does not stop on first error)
- Returns errors if any were encountered
- Supports multi-line source with `\n` or `\r\n` line endings

**Error Conditions**:
- Syntax errors (invalid format)
- Undefined label references
- Duplicate label definitions
- Invalid label names
- Unknown mnemonics
- Invalid operand formats
- Operand values out of range
- Invalid directive usage

**Example**:
```rust
use lib6502::assembler::assemble;

let source = r#"
START:
    LDA #$42
    STA $8000
    JMP START
"#;

let result = assemble(source);
assert!(result.is_ok());

let output = result.unwrap();
assert_eq!(output.bytes, vec![0xA9, 0x42, 0x8D, 0x00, 0x80, 0x4C, 0x00, 0x00]);
assert_eq!(output.symbol_table.len(), 1);
assert_eq!(output.symbol_table[0].name, "START");
assert_eq!(output.symbol_table[0].address, 0x0000);
```

**Error Example**:
```rust
let source = "LDA #$42\nJMP UNDEFINED";
let result = assemble(source);

assert!(result.is_err());
let errors = result.unwrap_err();
assert_eq!(errors.len(), 1);
assert_eq!(errors[0].error_type, ErrorType::UndefinedLabel);
assert_eq!(errors[0].line, 2);
```

---

### Function: `assemble_with_origin`

Assembles source code with a specific starting address.

```rust
pub fn assemble_with_origin(
    source: &str,
    origin: u16,
) -> Result<AssemblerOutput, Vec<AssemblerError>>
```

**Parameters**:
- `source`: Assembly source code
- `origin`: Starting address for assembled code (equivalent to `.org` directive)

**Returns**: Same as `assemble`

**Behavior**:
- Sets initial address counter to `origin`
- Labels are calculated relative to this origin
- Otherwise identical to `assemble`

**Example**:
```rust
let source = "START:\n    LDA #$42";
let output = assemble_with_origin(source, 0x8000).unwrap();

assert_eq!(output.symbol_table[0].address, 0x8000);
```

---

### Function: `validate_label`

Validates a label name according to 6502 assembly rules.

```rust
pub fn validate_label(name: &str) -> Result<(), LabelError>
```

**Parameters**:
- `name`: Candidate label name

**Returns**:
- `Ok(())`: Label is valid
- `Err(LabelError)`: Label is invalid with reason

**Validation Rules**:
1. Must start with letter `[a-zA-Z]`
2. Remaining characters must be alphanumeric or underscore `[a-zA-Z0-9_]`
3. Maximum 32 characters
4. Case-sensitive

**Example**:
```rust
assert!(validate_label("START").is_ok());
assert!(validate_label("loop_1").is_ok());
assert!(validate_label("_invalid").is_err()); // starts with underscore
assert!(validate_label("1invalid").is_err()); // starts with digit
assert!(validate_label("a".repeat(33).as_str()).is_err()); // too long
```

---

## Data Structures

### `AssemblerOutput`

See [data-model.md](../data-model.md#6-assembleroutput)

**Fields**:
```rust
pub struct AssemblerOutput {
    pub bytes: Vec<u8>,
    pub symbol_table: Vec<Symbol>,
    pub source_map: SourceMap,
    pub warnings: Vec<AssemblerWarning>,
}
```

**Methods**:
```rust
impl AssemblerOutput {
    /// Get symbol address by name
    pub fn lookup_symbol(&self, name: &str) -> Option<u16>;

    /// Get source location for instruction at address
    pub fn get_source_location(&self, address: u16) -> Option<SourceLocation>;

    /// Get address range for source line
    pub fn get_address_range(&self, line: usize) -> Option<AddressRange>;
}
```

---

### `AssemblerError`

See [data-model.md](../data-model.md#4-assemblererror)

**Fields**:
```rust
pub struct AssemblerError {
    pub error_type: ErrorType,
    pub line: usize,
    pub column: usize,
    pub span: (usize, usize),
    pub message: String,
}
```

**Display Format**:
```
error[E001]: undefined label 'LOOP'
  --> line 5, column 10
  |
5 |     JMP LOOP
  |         ^^^^
```

---

### `Symbol`

See [data-model.md](../data-model.md#3-symbol-symbol-table-entry)

```rust
pub struct Symbol {
    pub name: String,
    pub address: u16,
    pub defined_at: usize,
}
```

---

### `SourceMap`

See [data-model.md](../data-model.md#5-sourcemap)

**Methods**:
```rust
impl SourceMap {
    /// Map instruction address to source location
    pub fn get_source_location(&self, address: u16) -> Option<SourceLocation>;

    /// Map source line to instruction address range
    pub fn get_address_range(&self, line: usize) -> Option<AddressRange>;
}
```

---

## Supported Directives

### `.org ADDRESS`

Sets the origin address for subsequent code.

```asm
.org $8000
LDA #$42    ; Assembles at address $8000
```

**Rules**:
- `ADDRESS` must be valid 16-bit value
- Can appear multiple times to set different code segments
- Default origin is `$0000` if not specified

---

### `.byte VALUE [, VALUE ...]`

Inserts literal byte values into output.

```asm
.byte $01, $02, $03
.byte 65, 66, 67     ; Decimal values
.byte %10101010      ; Binary value
```

**Rules**:
- Each VALUE must be 0-255
- At least one value required
- Values separated by commas

---

### `.word VALUE [, VALUE ...]`

Inserts 16-bit word values in little-endian format.

```asm
.word $1234          ; Emits $34, $12 (little-endian)
.word $ABCD, $BEEF
```

**Rules**:
- Each VALUE must be 0-65535
- Little-endian byte order (LSB first)

---

## Supported Number Formats

| Format     | Example    | Description           |
|------------|------------|-----------------------|
| Hexadecimal| `$FF`      | Prefix with `$`       |
| Decimal    | `255`      | No prefix             |
| Binary     | `%11111111`| Prefix with `%`       |

---

## Error Handling Strategy

The assembler follows an **error recovery** approach:

1. **Parse errors**: Continue parsing remaining lines to collect all syntax errors
2. **Semantic errors**: Use placeholder values (e.g., address 0x0000 for undefined labels) to continue assembly
3. **Collect all errors**: Return complete list in `Err(Vec<AssemblerError>)`
4. **No partial output**: On error, no `AssemblerOutput` is returned

This maximizes developer productivity by showing all issues in one pass.

---

## Performance Guarantees

- **Throughput**: Assemble 8KB source in <10ms on modern hardware
- **Memory**: O(n) where n is source size + symbol count
- **Complexity**: O(n) two-pass algorithm

---

## Thread Safety

All functions are thread-safe (no shared mutable state). Multiple threads can assemble concurrently.

---

## WebAssembly Compatibility

- ✅ No OS dependencies
- ✅ Deterministic output (same source → same bytes)
- ✅ Bounded memory usage
- ✅ Pure computation (no side effects)
- ✅ Error handling via Result (no panics in public API)

---

## Example: Full Assembly Flow

```rust
use lib6502::assembler::{assemble, AssemblerOutput};

let source = r#"
; Simple loop example
.org $8000

START:
    LDX #$00
LOOP:
    INX
    CPX #$10
    BNE LOOP
    BRK
"#;

match assemble(source) {
    Ok(output) => {
        println!("Assembled {} bytes", output.bytes.len());
        println!("Symbols:");
        for symbol in &output.symbol_table {
            println!("  {}: ${:04X}", symbol.name, symbol.address);
        }

        // Use source map for debugging
        if let Some(loc) = output.get_source_location(0x8000) {
            println!("Instruction at $8000 from line {}", loc.line);
        }
    }
    Err(errors) => {
        eprintln!("Assembly failed with {} errors:", errors.len());
        for error in errors {
            eprintln!("  {}", error);
        }
    }
}
```
