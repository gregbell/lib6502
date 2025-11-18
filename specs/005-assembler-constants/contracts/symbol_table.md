# Symbol Table Contract: Collision Detection

**Module**: `src/assembler/symbol_table.rs`
**Function**: `add_symbol(name: String, value: u16, kind: SymbolKind, defined_at: usize) -> Result<(), LabelError>`

---

## Contract

The symbol table must maintain a unified collection of symbols (labels and constants) and detect collisions (duplicates and name conflicts).

### Input

- **name**: Symbol name (already normalized to UPPERCASE)
- **value**: 16-bit value (address for labels, literal for constants)
- **kind**: SymbolKind::Label or SymbolKind::Constant
- **defined_at**: Line number where symbol was defined (1-indexed)

### Output

- **Ok(())**: Symbol added successfully
- **Err(LabelError::Duplicate)**: Symbol already exists with same kind
- **Err(LabelError::Collision)**: Symbol exists with different kind

### Behavior

1. Lookup symbol by name (case-insensitive)
2. If symbol exists:
   - Compare `kind` with existing symbol's kind
   - If same kind → Duplicate error
   - If different kind → Collision error
3. If symbol does not exist:
   - Add symbol to internal vector
   - Return Ok(())

---

## Test Cases

### TC1: Add first constant

**Input:**
```rust
add_symbol("MAX".into(), 255, SymbolKind::Constant, 1)
```

**Expected Output:**
```rust
Ok(())
```

**State After:**
```rust
symbols = [
    Symbol { name: "MAX", value: 255, kind: Constant, defined_at: 1 }
]
```

---

### TC2: Add second constant (different name)

**Setup:**
```rust
add_symbol("MAX".into(), 255, SymbolKind::Constant, 1)?;
```

**Input:**
```rust
add_symbol("MIN".into(), 0, SymbolKind::Constant, 2)
```

**Expected Output:**
```rust
Ok(())
```

**State After:**
```rust
symbols = [
    Symbol { name: "MAX", value: 255, kind: Constant, defined_at: 1 },
    Symbol { name: "MIN", value: 0, kind: Constant, defined_at: 2 }
]
```

---

### TC3: Duplicate constant (same name, same kind)

**Setup:**
```rust
add_symbol("MAX".into(), 255, SymbolKind::Constant, 1)?;
```

**Input:**
```rust
add_symbol("MAX".into(), 100, SymbolKind::Constant, 5)
```

**Expected Output:**
```rust
Err(LabelError::Duplicate {
    name: "MAX",
    original_line: 1,
    duplicate_line: 5
})
```

**State After:**
```rust
symbols = [
    Symbol { name: "MAX", value: 255, kind: Constant, defined_at: 1 }
    // Duplicate NOT added
]
```

---

### TC4: Name collision (constant then label)

**Setup:**
```rust
add_symbol("FOO".into(), 42, SymbolKind::Constant, 3)?;
```

**Input:**
```rust
add_symbol("FOO".into(), 0x8000, SymbolKind::Label, 10)
```

**Expected Output:**
```rust
Err(LabelError::Collision {
    name: "FOO",
    existing_kind: SymbolKind::Constant,
    existing_line: 3,
    new_line: 10
})
```

**State After:**
```rust
symbols = [
    Symbol { name: "FOO", value: 42, kind: Constant, defined_at: 3 }
    // Collision NOT added
]
```

---

### TC5: Name collision (label then constant)

**Setup:**
```rust
add_symbol("START".into(), 0x8000, SymbolKind::Label, 1)?;
```

**Input:**
```rust
add_symbol("START".into(), 100, SymbolKind::Constant, 15)
```

**Expected Output:**
```rust
Err(LabelError::Collision {
    name: "START",
    existing_kind: SymbolKind::Label,
    existing_line: 1,
    new_line: 15
})
```

---

### TC6: Duplicate label (same name, same kind)

**Setup:**
```rust
add_symbol("LOOP".into(), 0x8010, SymbolKind::Label, 5)?;
```

**Input:**
```rust
add_symbol("LOOP".into(), 0x8020, SymbolKind::Label, 20)
```

**Expected Output:**
```rust
Err(LabelError::Duplicate {
    name: "LOOP",
    original_line: 5,
    duplicate_line: 20
})
```

---

### TC7: Lookup existing constant

**Setup:**
```rust
add_symbol("SCREEN".into(), 0x4000, SymbolKind::Constant, 1)?;
```

**Input:**
```rust
lookup("SCREEN")
```

**Expected Output:**
```rust
Some(&Symbol {
    name: "SCREEN",
    value: 0x4000,
    kind: SymbolKind::Constant,
    defined_at: 1
})
```

---

### TC8: Lookup non-existent symbol

**Input:**
```rust
lookup("MISSING")
```

**Expected Output:**
```rust
None
```

---

### TC9: Case-insensitive lookup

**Setup:**
```rust
add_symbol("MAX".into(), 255, SymbolKind::Constant, 1)?;
```

**Input:**
```rust
lookup("max")  // lowercase
```

**Expected Output:**
```rust
Some(&Symbol {
    name: "MAX",  // Stored as uppercase
    value: 255,
    kind: SymbolKind::Constant,
    defined_at: 1
})
```

---

### TC10: Multiple symbols of mixed kinds

**Input:**
```rust
add_symbol("FOO".into(), 42, SymbolKind::Constant, 1)?;
add_symbol("BAR".into(), 0x8000, SymbolKind::Label, 2)?;
add_symbol("BAZ".into(), 100, SymbolKind::Constant, 3)?;
add_symbol("QUX".into(), 0x8010, SymbolKind::Label, 4)?;
```

**Expected Output:**
```rust
Ok(())  // All succeed
```

**State After:**
```rust
symbols = [
    Symbol { name: "FOO", value: 42, kind: Constant, defined_at: 1 },
    Symbol { name: "BAR", value: 0x8000, kind: Label, defined_at: 2 },
    Symbol { name: "BAZ", value: 100, kind: Constant, defined_at: 3 },
    Symbol { name: "QUX", value: 0x8010, kind: Label, defined_at: 4 }
]
```

---

## Collision Detection Logic

**Algorithm:**
```rust
fn add_symbol(&mut self, name: String, value: u16, kind: SymbolKind, defined_at: usize)
    -> Result<(), LabelError>
{
    // Lookup existing symbol by name (case-insensitive)
    if let Some(existing) = self.lookup(&name) {
        // Symbol exists - check for collision
        match (existing.kind, kind) {
            (SymbolKind::Constant, SymbolKind::Constant) => {
                // Same kind → Duplicate
                Err(LabelError::Duplicate {
                    name,
                    original_line: existing.defined_at,
                    duplicate_line: defined_at,
                })
            },
            (SymbolKind::Label, SymbolKind::Label) => {
                // Same kind → Duplicate
                Err(LabelError::Duplicate {
                    name,
                    original_line: existing.defined_at,
                    duplicate_line: defined_at,
                })
            },
            (_, _) => {
                // Different kinds → Collision
                Err(LabelError::Collision {
                    name,
                    existing_kind: existing.kind,
                    existing_line: existing.defined_at,
                    new_line: defined_at,
                })
            }
        }
    } else {
        // No collision - add symbol
        self.symbols.push(Symbol {
            name,
            value,
            kind,
            defined_at,
        });
        Ok(())
    }
}
```

---

## Error Type Extensions

**Extend `LabelError` enum:**

```rust
pub enum LabelError {
    // Existing variants
    InvalidStart(String),
    InvalidCharacters(String),
    TooLong(String),

    // NEW variants
    Duplicate {
        name: String,
        original_line: usize,
        duplicate_line: usize,
    },
    Collision {
        name: String,
        existing_kind: SymbolKind,
        existing_line: usize,
        new_line: usize,
    },
}
```

---

## Performance Requirements

- **add_symbol**: O(n) where n = number of symbols (linear search)
- **lookup**: O(n) where n = number of symbols (linear search)
- **Optimization**: Acceptable for typical assembly files (< 1000 symbols)
- **Future**: Can upgrade to HashMap if performance needed

---

## Invariants

1. **Uniqueness**: No two symbols with same name (case-insensitive)
2. **Immutability**: Once added, symbols never modified (only looked up)
3. **Ordering**: Symbols stored in definition order (insertion order preserved)
4. **Case normalization**: All names stored in UPPERCASE

---

## Dependencies

- **SymbolKind** enum (new)
- **Symbol** struct (modified with `kind` field)
- **LabelError** enum (extended with Duplicate and Collision variants)
