# Parser Contract: Constant Detection

**Module**: `src/assembler/parser.rs`
**Function**: `parse_line(line: &str, line_number: usize) -> Option<AssemblyLine>`

---

## Contract

The parser must recognize and parse constant assignment syntax (`NAME = VALUE`) and return it in the `AssemblyLine.constant` field.

### Input

- **line**: Source code line (may contain constant assignment, label, instruction, directive, or comment)
- **line_number**: 1-indexed line number for error reporting

### Output

- **Some(AssemblyLine)**: If line contains parseable content
- **None**: If line is empty or comment-only

### Behavior

**For constant assignment `NAME = VALUE`:**
1. Detect `=` character in non-comment portion
2. Split on first `=`
3. Extract name (left side, trimmed)
4. Extract value (right side, trimmed)
5. Validate name has no internal whitespace
6. Normalize name to UPPERCASE
7. Return `AssemblyLine` with `constant` field populated

**Detection order:**
1. Strip comments (`;` to end of line)
2. Check for `=` **before** checking for `:` (constant before label)
3. If `=` found → parse as constant
4. If `:` found → parse as label (existing behavior)
5. Otherwise → parse as instruction/directive

---

## Test Cases

### TC1: Simple constant assignment

**Input:**
```assembly
MAX = 255
```

**Expected Output:**
```rust
Some(AssemblyLine {
    line_number: 1,
    constant: Some(("MAX".into(), "255".into())),
    label: None,
    mnemonic: None,
    operand: None,
    directive: None,
    comment: None,
    span: (0, 9),
})
```

---

### TC2: Constant with hex value

**Input:**
```assembly
SCREEN_ADDR = $4000
```

**Expected Output:**
```rust
Some(AssemblyLine {
    constant: Some(("SCREEN_ADDR".into(), "$4000".into())),
    ...
})
```

---

### TC3: Constant with binary value

**Input:**
```assembly
BITS = %11110000
```

**Expected Output:**
```rust
Some(AssemblyLine {
    constant: Some(("BITS".into(), "%11110000".into())),
    ...
})
```

---

### TC4: Constant with whitespace

**Input:**
```assembly
  MAX   =   $FF
```

**Expected Output:**
```rust
Some(AssemblyLine {
    constant: Some(("MAX".into(), "$FF".into())),  // Trimmed
    ...
})
```

---

### TC5: Constant with comment

**Input:**
```assembly
PAGE_SIZE = 256  ; bytes per page
```

**Expected Output:**
```rust
Some(AssemblyLine {
    constant: Some(("PAGE_SIZE".into(), "256".into())),
    comment: Some(" bytes per page".into()),
    ...
})
```

---

### TC6: Invalid constant (space in name)

**Input:**
```assembly
MAX SIZE = 100
```

**Expected Output:**
```rust
None  // or Error - name contains whitespace
```

---

### TC7: Label (not constant)

**Input:**
```assembly
START:
```

**Expected Output:**
```rust
Some(AssemblyLine {
    constant: None,  // Not a constant
    label: Some("START".into()),
    ...
})
```

---

### TC8: Instruction with equals in comment

**Input:**
```assembly
LDA #$42  ; note=test
```

**Expected Output:**
```rust
Some(AssemblyLine {
    constant: None,  // Comment stripped before constant detection
    mnemonic: Some("LDA".into()),
    operand: Some("#$42".into()),
    comment: Some(" note=test".into()),
    ...
})
```

---

### TC9: Empty line

**Input:**
```assembly

```

**Expected Output:**
```rust
None
```

---

### TC10: Comment-only line

**Input:**
```assembly
; This is a comment
```

**Expected Output:**
```rust
Some(AssemblyLine {
    constant: None,
    label: None,
    mnemonic: None,
    comment: Some(" This is a comment".into()),
    ...
})
```

---

## Validation Requirements

### Name Validation (performed by parser)

✅ Name must not be empty
✅ Name must not contain internal whitespace
✅ Name normalized to UPPERCASE

### Value Validation (deferred to assembler main logic)

⏭️ Value parsing (hex/decimal/binary)
⏭️ Value range check (0-65535)
⏭️ Literal vs. expression detection

---

## Error Handling

**Parser does NOT throw errors** - it returns `None` or `Some(AssemblyLine)`.

Error detection happens in **assembler main logic** (Pass 1):
- Invalid name format → `InvalidLabel` error
- Invalid value format → `InvalidConstantValue` error
- Duplicate constant → `DuplicateConstant` error
- Name collision → `NameCollision` error

---

## Implementation Notes

**Insertion point**: After comment stripping, before label detection (line ~102 in parser.rs)

**Pseudocode**:
```rust
// After stripping comments:
if let Some(eq_pos) = code_part.find('=') {
    let name_part = code_part[..eq_pos].trim();
    let value_part = code_part[eq_pos + 1..].trim();

    if !name_part.is_empty() && !name_part.contains(char::is_whitespace) {
        return Some(AssemblyLine {
            constant: Some((name_part.to_uppercase(), value_part.to_string())),
            ...
        });
    }
} else if let Some(colon_pos) = code_part.find(':') {
    // Existing label detection (unchanged)
    ...
}
```

---

## Performance Requirements

- **Time complexity**: O(n) where n = line length (single pass through characters)
- **Space complexity**: O(1) additional space (reuses existing string slices)
- **No regex**: Use simple character search (`find('=')`)

---

## Dependencies

- Existing `AssemblyLine` struct (extend with `constant` field)
- Existing comment stripping logic (reuse)
- Existing label validation logic (reuse for constant names)
