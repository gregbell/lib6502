# Data Model: CPU Core Foundation

**Feature**: 001-cpu-core-foundation
**Date**: 2025-11-13
**Phase**: 1 - Design & Contracts

This document defines the core data structures and entities that comprise the 6502 CPU emulator foundation. All entities are extracted from the feature specification requirements and research decisions.

## Entity Overview

The CPU core foundation consists of five primary entities:

1. **CPU** - The processor state and execution context
2. **MemoryBus** - Abstract memory interface trait
3. **OpcodeMetadata** - Static instruction metadata table
4. **AddressingMode** - Enumeration of addressing modes
5. **ExecutionError** - Error types for execution failures

---

## Entity: CPU

**Purpose**: Represents the complete internal state of the 6502 processor.

**Rust Type**: `struct CPU<M: MemoryBus>`

**Fields**:

| Field | Type | Description | Initial Value | Validation |
|-------|------|-------------|---------------|------------|
| `a` | `u8` | Accumulator register | `0x00` | N/A (all u8 values valid) |
| `x` | `u8` | X index register | `0x00` | N/A (all u8 values valid) |
| `y` | `u8` | Y index register | `0x00` | N/A (all u8 values valid) |
| `pc` | `u16` | Program counter | From reset vector (0xFFFC/0xFFFD) | N/A (wraps at 0xFFFF) |
| `sp` | `u8` | Stack pointer (0x0100-0x01FF) | `0xFD` | N/A (wraps at 0xFF) |
| `flag_n` | `bool` | Negative flag (bit 7 of result) | `false` | N/A |
| `flag_v` | `bool` | Overflow flag (signed overflow) | `false` | N/A |
| `flag_b` | `bool` | Break flag (BRK instruction executed) | `false` | N/A |
| `flag_d` | `bool` | Decimal mode flag (BCD arithmetic) | `false` | N/A |
| `flag_i` | `bool` | Interrupt disable flag | `true` | N/A (per reset value 0x24) |
| `flag_z` | `bool` | Zero flag (result was zero) | `false` | N/A |
| `flag_c` | `bool` | Carry flag (overflow/underflow) | `false` | N/A |
| `cycles` | `u64` | Total CPU cycles executed | `0` | N/A (monotonically increasing) |
| `memory` | `M` | Memory bus implementation (generic) | Provided at instantiation | Must implement MemoryBus trait |

**Invariants**:
- Stack pointer always wraps within 0x00-0xFF range (full address is 0x0100 + SP)
- Program counter wraps at 0xFFFF to 0x0000
- Cycle counter never decreases (monotonic)
- Status flag bit 5 (unused in 6502) is implicitly always 1 when status register is pushed to stack (not stored in CPU struct)

**State Transitions**:
- **Power-On Reset**: PC loaded from reset vector, SP set to 0xFD, I flag set, all other registers zeroed
- **Instruction Execution**: PC advances by instruction size, flags updated per instruction semantics, cycles incremented
- **Wrap Conditions**: PC and SP wrap naturally via u16/u8 arithmetic overflow

**Relationships**:
- CPU owns a MemoryBus implementation (generic type parameter M)
- CPU references OpcodeMetadata table (global const, no ownership)
- CPU execution may produce ExecutionError results

---

## Entity: MemoryBus

**Purpose**: Abstract trait defining memory read/write interface, enabling CPU decoupling from specific memory implementations.

**Rust Type**: `trait MemoryBus`

**Methods**:

| Method | Signature | Description | Constraints |
|--------|-----------|-------------|-------------|
| `read` | `fn read(&self, addr: u16) -> u8` | Read byte from memory address | Must never panic, may return garbage for unmapped addresses |
| `write` | `fn write(&mut self, addr: u16, value: u8)` | Write byte to memory address | Must never panic, may ignore writes to unmapped/ROM addresses |

**Implementations** (this feature):
- **FlatMemory**: Simple 64KB array implementation for testing (all addresses mapped to RAM)

**Rationale**:
- Immutable `&self` for read enables shared references
- Mutable `&mut self` for write makes memory modification explicit
- No error return types - 6502 hardware has no bus error mechanism (reads/writes always succeed, hardware determines behavior)

**Future Implementations** (out of scope):
- NES-style memory mapping (mirrored regions, memory-mapped I/O)
- Debugging wrappers (logging reads/writes, access breakpoints)
- ROM/RAM separation (writes to ROM ignored)

---

## Entity: OpcodeMetadata

**Purpose**: Static metadata table providing mnemonic, addressing mode, cycle cost, and size for all 256 possible 6502 opcodes.

**Rust Type**: `struct OpcodeMetadata`

**Fields**:

| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| `mnemonic` | `&'static str` | Instruction mnemonic (e.g., "LDA", "STA", "???") | 3 chars for documented, "???" for illegal |
| `addressing_mode` | `AddressingMode` | Addressing mode for this opcode | Must be valid enum variant |
| `base_cycles` | `u8` | Base cycle cost (before page crossing penalties) | 1-7 cycles for documented instructions |
| `size_bytes` | `u8` | Total instruction size including opcode and operands | 1-3 bytes |
| `implemented` | `bool` | Whether instruction execution is implemented | `false` for all opcodes in this feature |

**Table Structure**:
```rust
const OPCODE_TABLE: [OpcodeMetadata; 256] = [ /* 256 entries indexed by opcode */ ];
```

**Example Entries**:
- `0xA9`: `{ mnemonic: "LDA", addressing_mode: Immediate, base_cycles: 2, size_bytes: 2, implemented: false }`
- `0x00`: `{ mnemonic: "BRK", addressing_mode: Implicit, base_cycles: 7, size_bytes: 1, implemented: false }`
- `0x02`: `{ mnemonic: "???", addressing_mode: Implicit, base_cycles: 0, size_bytes: 1, implemented: false }` (illegal opcode)

**Data Source**: Opcode metadata derived from:
- docs/6502-reference/Instructions.md (documented instructions)
- External opcode tables for illegal/undocumented opcodes (e.g., NMOS 6502 opcode matrix)

**Validation Rules**:
- All 256 opcodes (0x00-0xFF) must have entries
- Documented opcodes (151 total) must have accurate cycle/size data
- Illegal opcodes (105 total) marked with "???" mnemonic and `implemented: false`

---

## Entity: AddressingMode

**Purpose**: Enumeration of all 6502 addressing modes, used by opcode table and future addressing resolution logic.

**Rust Type**: `enum AddressingMode`

**Variants**:

| Variant | Operand Size | Description | Example Opcodes |
|---------|--------------|-------------|-----------------|
| `Implicit` | 0 bytes | No operand, operation implied | CLC, RTS, NOP |
| `Accumulator` | 0 bytes | Operates on accumulator register | LSR A, ROL A |
| `Immediate` | 1 byte | 8-bit constant operand | LDA #$10 |
| `ZeroPage` | 1 byte | 8-bit zero page address (0x00-0xFF) | LDA $80 |
| `ZeroPageX` | 1 byte | Zero page address + X register | LDA $80,X |
| `ZeroPageY` | 1 byte | Zero page address + Y register | LDX $80,Y |
| `Relative` | 1 byte | Signed 8-bit branch offset | BEQ label |
| `Absolute` | 2 bytes | Full 16-bit address | JMP $1234 |
| `AbsoluteX` | 2 bytes | 16-bit address + X register | LDA $1234,X |
| `AbsoluteY` | 2 bytes | 16-bit address + Y register | LDA $1234,Y |
| `Indirect` | 2 bytes | Indirect jump (JMP only) | JMP ($FFFC) |
| `IndirectX` | 1 byte | Indexed indirect (ZP + X) then deref | LDA ($40,X) |
| `IndirectY` | 1 byte | Indirect indexed (ZP deref then + Y) | LDA ($40),Y |

**Relationship to Instruction Size**:
- `size_bytes = 1 + operand_size` (opcode byte + addressing mode operand bytes)
- This relationship is encoded in the OpcodeMetadata table

**Future Use**:
- Instruction decoding: Determine how many operand bytes to fetch
- Address calculation: Compute effective address from mode + operands + registers
- Cycle penalty calculation: Page crossings on indexed modes add +1 cycle

---

## Entity: ExecutionError

**Purpose**: Error type representing failures during CPU execution.

**Rust Type**: `enum ExecutionError`

**Variants**:

| Variant | Data | Description | When Produced |
|---------|------|-------------|---------------|
| `UnimplementedOpcode` | `u8` (opcode byte) | Instruction not yet implemented | Opcode table entry has `implemented: false` |

**Future Variants** (out of scope for this feature):
- `InvalidOpcode(u8)`: Illegal opcode with no defined behavior
- `MemoryAccessViolation(u16)`: Read/write to protected region (if memory protection added)

**Error Handling Strategy**:
- `step()` method returns `Result<(), ExecutionError>`
- Caller decides how to handle errors (halt, log, skip, etc.)
- No panicking on unimplemented instructions (violates Rust best practices)

---

## Data Flow

```
┌─────────────────────────────────────────────────────────┐
│                     CPU Execution Loop                   │
│                                                           │
│  ┌─────────┐      ┌──────────────┐      ┌─────────────┐│
│  │ Fetch   │─────▶│ Decode       │─────▶│ Execute     ││
│  │ (PC)    │      │ (Opcode Tbl) │      │ (Instr Fn)  ││
│  └────┬────┘      └──────┬───────┘      └──────┬──────┘│
│       │                  │                     │        │
│       │                  │                     │        │
│       ▼                  ▼                     ▼        │
│  ┌─────────────────────────────────────────────────────┐│
│  │           MemoryBus Trait (read/write)              ││
│  └─────────────────────────────────────────────────────┘│
│                          │                               │
│                          ▼                               │
│                  ┌───────────────┐                       │
│                  │ FlatMemory    │                       │
│                  │ (64KB array)  │                       │
│                  └───────────────┘                       │
└─────────────────────────────────────────────────────────┘

1. Fetch: Read opcode byte from memory[PC] via MemoryBus
2. Decode: Look up OpcodeMetadata in OPCODE_TABLE[opcode]
3. Execute: Check implemented flag, return error if false
4. Update: Increment cycles, advance PC (future: execute instruction logic)
```

---

## Validation & Constraints

**Type Safety**:
- Rust's type system enforces:
  - Register values are u8/u16 (no out-of-range values)
  - MemoryBus trait must be implemented for generic type M
  - AddressingMode is exhaustive enum (match statements checked at compile time)

**Runtime Constraints**:
- CPU state is self-contained (no external dependencies beyond MemoryBus)
- No heap allocations in core execution loop (all data on stack or in memory bus)
- Deterministic execution (no randomness, no system time dependencies)

**WASM Compatibility**:
- All types are `no_std` compatible (or can be made so by conditional compilation)
- No OS dependencies (no file I/O, networking, threading)
- MemoryBus implementations may use `std::vec::Vec` for allocation but trait remains WASM-safe

---

## Testing Requirements

**Data Model Tests** (from spec):
- CPU initialization: Verify all register fields match documented reset values
- MemoryBus trait: Implement FlatMemory, verify read returns written values
- OpcodeMetadata table: Verify all 256 entries exist and have non-empty mnemonics
- AddressingMode enum: Verify all 13 variants are defined (compile-time check via exhaustive match)
- ExecutionError: Verify UnimplementedOpcode error contains correct opcode byte

**Validation Tests**:
- PC wrap: Set PC to 0xFFFF, fetch opcode, verify PC wraps to 0x0000
- SP wrap: Set SP to 0x00, decrement (simulate push), verify wraps to 0xFF
- Cycle monotonicity: Execute multiple instructions, verify cycles only increase

---

## References

- Feature Spec: specs/001-cpu-core-foundation/spec.md
- Research Decisions: specs/001-cpu-core-foundation/research.md
- Constitution: .specify/memory/constitution.md (principles I, II, III)
- 6502 Architecture: docs/6502-reference/Architecture.md
- 6502 Registers: docs/6502-reference/Registers.md
