# Disassembler API Contract

**Module**: `lib6502::disassembler`
**Version**: 0.1.0

## Public API

### Function: `disassemble`

Disassembles a byte slice into a vector of instructions.

```rust
pub fn disassemble(
    bytes: &[u8],
    options: DisassemblyOptions,
) -> Vec<Instruction>
```

**Parameters**:
- `bytes`: Byte slice containing 6502 machine code
- `options`: Configuration for disassembly behavior

**Returns**: Vector of disassembled instructions

**Behavior**:
- Decodes bytes sequentially starting at offset 0
- Uses `OPCODE_TABLE` to determine instruction size and addressing mode
- Creates `Instruction` struct for each decoded instruction
- Advances by instruction size (1-3 bytes)
- Stops at end of byte slice
- Illegal opcodes (mnemonic "???") are decoded but marked as unimplemented

**Errors**: Never panics. Invalid opcodes are decoded as "???" with size 1.

**Example**:
```rust
use lib6502::disassembler::{disassemble, DisassemblyOptions};

let code = &[0xA9, 0x42, 0x8D, 0x00, 0x80];
let options = DisassemblyOptions {
    start_address: 0x8000,
    hex_dump: false,
    show_offsets: false,
};

let instructions = disassemble(code, options);

assert_eq!(instructions.len(), 2);
assert_eq!(instructions[0].mnemonic, "LDA");
assert_eq!(instructions[0].address, 0x8000);
assert_eq!(instructions[1].mnemonic, "STA");
assert_eq!(instructions[1].address, 0x8002);
```

---

### Function: `format_instruction`

Formats a single instruction as human-readable assembly text.

```rust
pub fn format_instruction(instruction: &Instruction) -> String
```

**Parameters**:
- `instruction`: Instruction to format

**Returns**: Formatted assembly string (e.g., "LDA #$42")

**Behavior**:
- Returns mnemonic + formatted operand based on addressing mode
- Uses standard 6502 assembly syntax conventions
- Illegal opcodes format as ".byte $XX"

**Format Examples**:
- Immediate: `"LDA #$42"`
- ZeroPage: `"LDA $80"`
- Absolute: `"JMP $1234"`
- IndirectY: `"LDA ($40),Y"`

**Errors**: Never panics

**Example**:
```rust
let instruction = Instruction {
    address: 0x8000,
    opcode: 0xA9,
    mnemonic: "LDA",
    addressing_mode: AddressingMode::Immediate,
    operand_bytes: vec![0x42],
    size_bytes: 2,
    base_cycles: 2,
};

let asm = format_instruction(&instruction);
assert_eq!(asm, "LDA #$42");
```

---

### Function: `format_hex_dump`

Formats instructions as hex dump with addresses and assembly.

```rust
pub fn format_hex_dump(instructions: &[Instruction]) -> String
```

**Parameters**:
- `instructions`: Slice of instructions to format

**Returns**: Multi-line hex dump string

**Format**:
```
AAAA: BB BB BB  MNEMONIC OPERAND
```

**Behavior**:
- One line per instruction
- Address in 4-digit hex
- Up to 3 hex bytes, space-separated, left-aligned
- Mnemonic and operand right-aligned after bytes

**Errors**: Never panics

**Example**:
```rust
let instructions = disassemble(&[0xA9, 0x42, 0x8D, 0x00, 0x80], options);
let dump = format_hex_dump(&instructions);

assert_eq!(dump, "8000: A9 42     LDA #$42\n8002: 8D 00 80  STA $8000\n");
```

---

## Data Structures

### `Instruction`

See [data-model.md](../data-model.md#1-instruction-disassembler-output)

### `DisassemblyOptions`

See [data-model.md](../data-model.md#8-disassemblyoptions)

---

## Error Handling

The disassembler API is **infallible**:
- Invalid opcodes decode as "???" with size 1 byte
- Truncated instructions at end of byte slice: partial instruction decoded with available bytes
- Empty byte slice returns empty vector
- Never panics

This design choice reflects the reality that disassembling arbitrary data may not align with instruction boundaries (e.g., disassembling from the middle of data, self-modifying code).

---

## Performance Guarantees

- **Throughput**: Minimum 10,000 bytes/millisecond on modern hardware
- **Memory**: O(n) where n is input size (approximately 80 bytes per instruction overhead)
- **Complexity**: O(n) linear scan through byte slice

---

## Thread Safety

All functions are thread-safe (no shared mutable state). Multiple threads can disassemble concurrently.

---

## WebAssembly Compatibility

- ✅ No OS dependencies
- ✅ Deterministic output
- ✅ No panics
- ✅ Bounded memory usage
- ✅ Pure computation (no side effects)
