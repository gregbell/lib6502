# Opcode Table Schema Contract

**Feature**: 001-cpu-core-foundation
**Date**: 2025-11-13
**Contract Type**: Static Data Structure

This document defines the schema and validation rules for the 256-entry opcode metadata table that serves as the single source of truth for all 6502 instruction metadata.

## Schema Definition

### Table Structure

```rust
pub const OPCODE_TABLE: [OpcodeMetadata; 256] = [
    // 256 entries indexed by opcode byte (0x00-0xFF)
    OpcodeMetadata { /* ... */ }, // 0x00
    OpcodeMetadata { /* ... */ }, // 0x01
    // ... 254 more entries
    OpcodeMetadata { /* ... */ }, // 0xFF
];
```

### Entry Schema: `OpcodeMetadata`

| Field | Type | Constraints | Validation |
|-------|------|-------------|------------|
| `mnemonic` | `&'static str` | 3 chars for documented, "???" for illegal | Must be uppercase ASCII, non-empty |
| `addressing_mode` | `AddressingMode` | Must be valid enum variant | Compile-time check via enum |
| `base_cycles` | `u8` | 0-7 for documented, 0 for illegal | 0 = illegal, 1-7 = valid instruction |
| `size_bytes` | `u8` | 1-3 bytes | 1 = no operand, 2 = 1-byte operand, 3 = 2-byte operand |
| `implemented` | `bool` | `false` for all in this feature | Set to `true` when instruction added in future features |

## Addressing Mode Size Mapping

The `size_bytes` field must match the addressing mode's operand size:

| Addressing Mode | Operand Size | Total Size | Examples |
|-----------------|--------------|------------|----------|
| `Implicit` | 0 bytes | 1 byte | BRK, CLC, NOP |
| `Accumulator` | 0 bytes | 1 byte | LSR A, ROL A |
| `Immediate` | 1 byte | 2 bytes | LDA #$10 |
| `ZeroPage` | 1 byte | 2 bytes | LDA $80 |
| `ZeroPageX` | 1 byte | 2 bytes | LDA $80,X |
| `ZeroPageY` | 1 byte | 2 bytes | LDX $80,Y |
| `Relative` | 1 byte | 2 bytes | BEQ label |
| `Absolute` | 2 bytes | 3 bytes | JMP $1234 |
| `AbsoluteX` | 2 bytes | 3 bytes | LDA $1234,X |
| `AbsoluteY` | 2 bytes | 3 bytes | LDA $1234,Y |
| `Indirect` | 2 bytes | 3 bytes | JMP ($FFFC) |
| `IndirectX` | 1 byte | 2 bytes | LDA ($40,X) |
| `IndirectY` | 1 byte | 2 bytes | LDA ($40),Y |

## Documented Instruction Metadata

The table includes 151 documented NMOS 6502 instructions. Below are representative examples (full table data sourced from 6502 reference docs):

### Example Entries (Documented Instructions)

```rust
// 0x00: BRK - Force interrupt
OpcodeMetadata {
    mnemonic: "BRK",
    addressing_mode: AddressingMode::Implicit,
    base_cycles: 7,
    size_bytes: 1,
    implemented: false,
},

// 0xA9: LDA - Load Accumulator (Immediate)
OpcodeMetadata {
    mnemonic: "LDA",
    addressing_mode: AddressingMode::Immediate,
    base_cycles: 2,
    size_bytes: 2,
    implemented: false,
},

// 0x4C: JMP - Jump Absolute
OpcodeMetadata {
    mnemonic: "JMP",
    addressing_mode: AddressingMode::Absolute,
    base_cycles: 3,
    size_bytes: 3,
    implemented: false,
},

// 0xEA: NOP - No Operation
OpcodeMetadata {
    mnemonic: "NOP",
    addressing_mode: AddressingMode::Implicit,
    base_cycles: 2,
    size_bytes: 1,
    implemented: false,
},
```

## Illegal/Undocumented Opcode Metadata

The table includes 105 illegal/undocumented opcodes. These are marked with:
- `mnemonic: "???"`
- `base_cycles: 0` (illegal opcodes have no defined cycle cost)
- `size_bytes: 1` (minimum size, effectively a no-op placeholder)
- `implemented: false`

### Example Entries (Illegal Opcodes)

```rust
// 0x02: Illegal opcode
OpcodeMetadata {
    mnemonic: "???",
    addressing_mode: AddressingMode::Implicit,
    base_cycles: 0,
    size_bytes: 1,
    implemented: false,
},

// 0x12: Illegal opcode
OpcodeMetadata {
    mnemonic: "???",
    addressing_mode: AddressingMode::Implicit,
    base_cycles: 0,
    size_bytes: 1,
    implemented: false,
},
```

**Rationale**: Illegal opcodes are included for table completeness (all 256 entries must exist). Future work can replace specific illegal opcodes with undocumented instruction behavior if needed (e.g., for Commodore 64 emulation).

## Cycle Cost Accuracy

**Base Cycles**: The `base_cycles` field represents the minimum cycle cost for the instruction without any penalties.

**Page Crossing Penalties** (NOT included in base_cycles):
- Indexed addressing modes (AbsoluteX, AbsoluteY, IndirectY) add +1 cycle if page boundary crossed
- Branch instructions add +1 cycle if branch taken, +2 if page boundary crossed

**Future Implementation**: Instruction execution logic will add page crossing penalties dynamically based on the effective address calculation. The opcode table only stores base costs.

### Cycle Cost Reference (Documented Instructions)

| Instruction Type | Base Cycles | Examples |
|------------------|-------------|----------|
| Register transfer | 2 | TAX, TXA, TAY, TYA |
| Load (immediate) | 2 | LDA #, LDX #, LDY # |
| Load (zero page) | 3 | LDA $00, LDX $00 |
| Load (absolute) | 4 | LDA $1234, STA $1234 |
| Arithmetic (immediate) | 2 | ADC #, SBC # |
| Branch (not taken) | 2 | BEQ, BNE, BCS, BCC |
| Branch (taken, same page) | 3 | BEQ +10 |
| Jump (absolute) | 3 | JMP $1234 |
| Jump (indirect) | 5 | JMP ($1234) |
| JSR (subroutine call) | 6 | JSR $1234 |
| RTS (return) | 6 | RTS |
| BRK (interrupt) | 7 | BRK |

## Validation Rules

### Compile-Time Validations

1. **Table Completeness**: Array length must be exactly 256 entries (enforced by type system)
2. **Addressing Mode**: Must be a valid `AddressingMode` enum variant (enforced by type system)
3. **String Literals**: All mnemonics must be static string slices (enforced by `&'static str` type)

### Runtime Validations (Test Suite)

```rust
#[test]
fn opcode_table_completeness() {
    // Verify all 256 entries exist
    assert_eq!(OPCODE_TABLE.len(), 256);

    for (opcode, metadata) in OPCODE_TABLE.iter().enumerate() {
        // All mnemonics must be non-empty
        assert!(!metadata.mnemonic.is_empty(),
                "Opcode 0x{:02X} has empty mnemonic", opcode);

        // Size must be 1-3 bytes
        assert!(metadata.size_bytes >= 1 && metadata.size_bytes <= 3,
                "Opcode 0x{:02X} has invalid size: {}", opcode, metadata.size_bytes);

        // Documented instructions must have non-zero cycles
        if metadata.mnemonic != "???" {
            assert!(metadata.base_cycles > 0,
                    "Opcode 0x{:02X} ({}) has zero cycles", opcode, metadata.mnemonic);
        }

        // Implemented flag must be false for this feature
        assert_eq!(metadata.implemented, false,
                   "Opcode 0x{:02X} marked as implemented in foundation", opcode);
    }
}

#[test]
fn opcode_table_size_mode_consistency() {
    for (opcode, metadata) in OPCODE_TABLE.iter().enumerate() {
        let expected_size = match metadata.addressing_mode {
            AddressingMode::Implicit => 1,
            AddressingMode::Accumulator => 1,
            AddressingMode::Immediate => 2,
            AddressingMode::ZeroPage => 2,
            AddressingMode::ZeroPageX => 2,
            AddressingMode::ZeroPageY => 2,
            AddressingMode::Relative => 2,
            AddressingMode::Absolute => 3,
            AddressingMode::AbsoluteX => 3,
            AddressingMode::AbsoluteY => 3,
            AddressingMode::Indirect => 3,
            AddressingMode::IndirectX => 2,
            AddressingMode::IndirectY => 2,
        };

        assert_eq!(metadata.size_bytes, expected_size,
                   "Opcode 0x{:02X} size mismatch: mode {:?} expects {} bytes, got {}",
                   opcode, metadata.addressing_mode, expected_size, metadata.size_bytes);
    }
}
```

## Data Source References

### Official 6502 Documentation
- **Instruction Set**: docs/6502-reference/Instructions.md (56 documented instructions)
- **Addressing Modes**: docs/6502-reference/Addressing-Modes.md (13 modes)

### Opcode Matrix (for illegal opcodes)
External reference: NMOS 6502 opcode matrix (cross-reference for undocumented opcodes)
- 151 documented opcodes (56 instructions × multiple addressing modes)
- 105 illegal/undocumented opcodes

### Known Documented Opcode Examples

**Load/Store**: LDA, LDX, LDY, STA, STX, STY
**Arithmetic**: ADC, SBC, INC, DEC, INX, DEX, INY, DEY
**Logical**: AND, ORA, EOR, BIT
**Shifts**: ASL, LSR, ROL, ROR
**Branches**: BEQ, BNE, BCS, BCC, BMI, BPL, BVS, BVC
**Jumps**: JMP, JSR, RTS, RTI
**Stack**: PHA, PLA, PHP, PLP
**Transfers**: TAX, TXA, TAY, TYA, TSX, TXS
**Flags**: CLC, SEC, CLI, SEI, CLD, SED, CLV
**Comparisons**: CMP, CPX, CPY
**System**: BRK, NOP

## Update Protocol

When implementing a new instruction in a future feature:

1. Update `OPCODE_TABLE[opcode].implemented = true`
2. Add test case verifying instruction execution behavior
3. Update feature spec referencing this opcode table schema
4. Ensure cycle accuracy matches base_cycles (plus any penalties)

**Versioning**: The opcode table schema is versioned with the library. Breaking changes to `OpcodeMetadata` struct require a major version bump.

---

## References

- Data Model: specs/001-cpu-core-foundation/data-model.md
- API Contract: specs/001-cpu-core-foundation/contracts/cpu_api.md
- 6502 Instructions: docs/6502-reference/Instructions.md
- Constitution: .specify/memory/constitution.md (principle V: Table-Driven Design)
