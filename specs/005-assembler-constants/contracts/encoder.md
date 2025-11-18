# Encoder Contract: Constant Substitution

**Module**: `src/assembler/encoder.rs`
**Function**: `resolve_operand(operand: &str, symbol_table: &SymbolTable, ...) -> Result<(AddressingMode, u16), AssemblerError>`

---

## Contract

The encoder must resolve operands by substituting constant names with their literal values and label names with their memory addresses.

### Input

- **operand**: Operand string (e.g., `"#MAX"`, `"SCREEN"`, `"$1234,X"`)
- **symbol_table**: Unified table containing both constants and labels
- **current_address**: Current instruction address (for relative addressing)
- **mnemonic**: Instruction mnemonic (for branch detection)

### Output

- **Ok((AddressingMode, u16))**: Resolved addressing mode and value
- **Err(AssemblerError)**: Resolution failed (undefined symbol, invalid operand, etc.)

### Behavior

1. Parse operand to detect addressing mode
2. If operand contains identifier (no prefix `$`, `#`, etc.):
   - Lookup in symbol table
   - If found:
     - Check `symbol.kind`
     - If `Constant`: Use `symbol.value` as literal
     - If `Label`: Use `symbol.value` as address
   - If not found: Return `UndefinedConstant` or `UndefinedLabel` error
3. Apply addressing mode detection to resolved value
4. Return (mode, value)

---

## Test Cases

### TC1: Immediate with constant

**Setup:**
```rust
symbol_table.add_symbol("MAX".into(), 255, SymbolKind::Constant, 1)?;
```

**Input:**
```rust
resolve_operand("#MAX", &symbol_table, 0x8000, "LDA")
```

**Expected Behavior:**
1. Detect `#` prefix → Immediate mode
2. Extract identifier `MAX`
3. Lookup `MAX` in symbol table → Found (Constant, value=255)
4. Substitute value → `#255` → `#$FF`
5. Return (Immediate, 0xFF)

**Expected Output:**
```rust
Ok((AddressingMode::Immediate, 0xFF))
```

---

### TC2: Zero page with constant

**Setup:**
```rust
symbol_table.add_symbol("ZP_TEMP".into(), 0x80, SymbolKind::Constant, 1)?;
```

**Input:**
```rust
resolve_operand("ZP_TEMP", &symbol_table, 0x8000, "LDA")
```

**Expected Behavior:**
1. No prefix → identifier
2. Lookup `ZP_TEMP` → Found (Constant, value=0x80)
3. Substitute value → `0x80`
4. Detect addressing mode (2-digit hex) → Zero Page
5. Return (ZeroPage, 0x80)

**Expected Output:**
```rust
Ok((AddressingMode::ZeroPage, 0x80))
```

---

### TC3: Absolute with constant

**Setup:**
```rust
symbol_table.add_symbol("SCREEN".into(), 0x4000, SymbolKind::Constant, 1)?;
```

**Input:**
```rust
resolve_operand("SCREEN", &symbol_table, 0x8000, "STA")
```

**Expected Behavior:**
1. No prefix → identifier
2. Lookup `SCREEN` → Found (Constant, value=0x4000)
3. Substitute value → `0x4000`
4. Detect addressing mode (4-digit hex) → Absolute
5. Return (Absolute, 0x4000)

**Expected Output:**
```rust
Ok((AddressingMode::Absolute, 0x4000))
```

---

### TC4: Absolute,X with constant

**Setup:**
```rust
symbol_table.add_symbol("IO_BASE".into(), 0x8000, SymbolKind::Constant, 1)?;
```

**Input:**
```rust
resolve_operand("IO_BASE,X", &symbol_table, 0x8000, "STA")
```

**Expected Behavior:**
1. Parse suffix `,X`
2. Extract identifier `IO_BASE`
3. Lookup `IO_BASE` → Found (Constant, value=0x8000)
4. Substitute value → `0x8000,X`
5. Detect addressing mode → Absolute,X
6. Return (AbsoluteX, 0x8000)

**Expected Output:**
```rust
Ok((AddressingMode::AbsoluteX, 0x8000))
```

---

### TC5: Label reference (not constant)

**Setup:**
```rust
symbol_table.add_symbol("START".into(), 0x8000, SymbolKind::Label, 1)?;
```

**Input:**
```rust
resolve_operand("START", &symbol_table, 0x8010, "JMP")
```

**Expected Behavior:**
1. No prefix → identifier
2. Lookup `START` → Found (Label, value=0x8000)
3. Use value as address → `0x8000`
4. Detect addressing mode → Absolute
5. Return (Absolute, 0x8000)

**Expected Output:**
```rust
Ok((AddressingMode::Absolute, 0x8000))
```

**Note:** Constants and labels resolved identically for most addressing modes. The `kind` field is informational.

---

### TC6: Branch with label (relative addressing)

**Setup:**
```rust
symbol_table.add_symbol("LOOP".into(), 0x8000, SymbolKind::Label, 1)?;
```

**Input:**
```rust
resolve_operand("LOOP", &symbol_table, 0x8010, "BNE")
```

**Expected Behavior:**
1. Detect branch instruction (`BNE`)
2. Lookup `LOOP` → Found (Label, value=0x8000)
3. Calculate relative offset: `0x8000 - (0x8010 + 2)` = -18
4. Validate offset in range [-128, 127]
5. Convert to two's complement
6. Return (Relative, offset)

**Expected Output:**
```rust
Ok((AddressingMode::Relative, 0xEE))  // -18 as u8 two's complement
```

---

### TC7: Undefined constant

**Setup:**
```rust
// No symbol defined
```

**Input:**
```rust
resolve_operand("#MISSING", &symbol_table, 0x8000, "LDA")
```

**Expected Behavior:**
1. Detect `#` prefix → Immediate mode
2. Extract identifier `MISSING`
3. Lookup `MISSING` → Not found
4. Return `UndefinedConstant` error

**Expected Output:**
```rust
Err(AssemblerError {
    error_type: ErrorType::UndefinedConstant,
    line: ...,
    column: ...,
    message: "Undefined constant 'MISSING'".into(),
    ...
})
```

---

### TC8: Literal value (no substitution)

**Input:**
```rust
resolve_operand("#$42", &symbol_table, 0x8000, "LDA")
```

**Expected Behavior:**
1. Detect `#$` prefix → Immediate with hex literal
2. Parse `$42` directly (no lookup)
3. Return (Immediate, 0x42)

**Expected Output:**
```rust
Ok((AddressingMode::Immediate, 0x42))
```

**Note:** No symbol lookup needed for literals.

---

### TC9: Constant vs. literal disambiguation

**Setup:**
```rust
symbol_table.add_symbol("VALUE".into(), 0xFF, SymbolKind::Constant, 1)?;
```

**Input A:**
```rust
resolve_operand("#VALUE", &symbol_table, 0x8000, "LDA")  // Constant
```

**Expected Output A:**
```rust
Ok((AddressingMode::Immediate, 0xFF))  // Substitute VALUE → 0xFF
```

**Input B:**
```rust
resolve_operand("#$42", &symbol_table, 0x8000, "LDA")  // Literal
```

**Expected Output B:**
```rust
Ok((AddressingMode::Immediate, 0x42))  // Use literal directly
```

**Disambiguation logic:**
- `#VALUE` → No `$` prefix → identifier → lookup in table
- `#$42` → Has `$` prefix → literal hex → parse directly

---

### TC10: Mixed constants and labels

**Setup:**
```rust
symbol_table.add_symbol("CONST1".into(), 42, SymbolKind::Constant, 1)?;
symbol_table.add_symbol("CONST2".into(), 100, SymbolKind::Constant, 2)?;
symbol_table.add_symbol("LABEL1".into(), 0x8000, SymbolKind::Label, 3)?;
symbol_table.add_symbol("LABEL2".into(), 0x8020, SymbolKind::Label, 4)?;
```

**Test sequence:**
```rust
resolve_operand("#CONST1", ...) → Ok((Immediate, 42))
resolve_operand("CONST2", ...) → Ok((ZeroPage, 100))
resolve_operand("LABEL1", ...) → Ok((Absolute, 0x8000))
resolve_operand("LABEL2", ...) → Ok((Absolute, 0x8020))
```

All resolve correctly based on `symbol.value`, regardless of `symbol.kind`.

---

## Resolution Algorithm

**Pseudocode:**
```rust
fn resolve_operand(operand: &str, symbol_table: &SymbolTable, ...)
    -> Result<(AddressingMode, u16), AssemblerError>
{
    // Step 1: Detect if operand is identifier or literal
    let is_identifier = !operand.starts_with('$') &&
                       !operand.starts_with('#') &&
                       !operand.starts_with('(') &&
                       !operand.starts_with('%');

    // Step 2: Resolve identifier to value
    let resolved_value = if is_identifier {
        // Extract base identifier (remove suffixes like ,X ,Y)
        let base_name = extract_identifier(operand);

        // Lookup in symbol table
        match symbol_table.lookup(base_name) {
            Some(symbol) => symbol.value,  // Use value (address or literal)
            None => return Err(UndefinedConstant or UndefinedLabel),
        }
    } else {
        // Parse literal value directly
        parse_number(operand)?
    };

    // Step 3: Detect addressing mode
    let mode = detect_addressing_mode(operand, resolved_value)?;

    // Step 4: Handle special cases (branches, etc.)
    if is_branch_instruction(mnemonic) {
        let offset = calculate_relative_offset(resolved_value, current_address)?;
        return Ok((AddressingMode::Relative, offset));
    }

    Ok((mode, resolved_value))
}
```

---

## Addressing Mode Detection

**With constant substitution:**

| Operand | Constant Value | Detected Mode | Reasoning |
|---------|----------------|---------------|-----------|
| `#MAX` | `255` | Immediate | `#` prefix |
| `ZP` | `$80` | ZeroPage | Value < 256, 2-digit hex |
| `SCREEN` | `$4000` | Absolute | Value >= 256, 4-digit hex |
| `IO_BASE,X` | `$8000` | AbsoluteX | Value >= 256 + `,X` suffix |
| `(PTR),Y` | `$40` | IndirectIndexed | `()` + `,Y` + ZP value |

**Key insight:** After substitution, addressing mode detection uses the resolved value, not the original identifier.

---

## Error Handling

**Error types:**
- `UndefinedConstant` - Constant name used but not defined
- `UndefinedLabel` - Label name used but not defined
- `InvalidOperand` - Malformed operand syntax
- `RangeError` - Value out of bounds (e.g., branch offset > ±127)

**Error context detection:**
```rust
// Determine if undefined symbol is constant or label
let error_type = if operand.starts_with('#') {
    ErrorType::UndefinedConstant  // Immediate → likely constant
} else if is_branch_instruction(mnemonic) {
    ErrorType::UndefinedLabel  // Branch target → label
} else {
    // Ambiguous - could be either (default to label for compatibility)
    ErrorType::UndefinedLabel
};
```

---

## Performance Requirements

- **Time complexity**: O(n) where n = number of symbols (symbol table lookup)
- **Space complexity**: O(1) additional space
- **No string allocations** during resolution (use slices)

---

## Dependencies

- **SymbolTable::lookup()** - Case-insensitive symbol lookup
- **SymbolKind** enum - To distinguish constants from labels (informational only for resolution)
- **AddressingMode** enum - Existing addressing mode types
- **parse_number()** - Existing literal number parser (hex/decimal/binary)
- **detect_addressing_mode()** - Existing mode detection logic (extend to handle resolved values)
