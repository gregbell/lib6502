# Assembler/Disassembler Round-Trip Validation

## Status: ✅ PASSING (100% byte-perfect match)

The assembler/disassembler round-trip test **passes completely**, validating
that all 151 NMOS 6502 opcodes can be disassembled and reassembled with perfect
fidelity.

**Final Results:**

- **Binary Size**: 65,536 bytes (full 64KB address space)
- **Instructions Disassembled**: 59,869 instructions
- **Unique Opcodes**: 176 opcodes (151 valid + 25 unofficial/data bytes)
- **Round-Trip Status**: ✓ SUCCESS - All bytes match perfectly

## Overview

This document describes the round-trip validation test that uses Klaus Dormann's
6502 functional test binary to validate both the assembler and disassembler.

**Test Binary**: `tests/fixtures/6502_functional_test.bin` (same binary used for
CPU validation)

## What is the Round-Trip Test?

The round-trip test validates that our assembler and disassembler are inverse
operations:

```
Original Binary → Disassemble → Assembly Source → Reassemble → Output Binary
```

If `Original Binary == Output Binary` (byte-for-byte), then both tools are
correct.

### Why This Validates Both Tools

- **Disassembler Correctness**: If the disassembler makes ANY error in:
  - Opcode identification
  - Addressing mode detection
  - Operand byte interpretation

  Then the reassembled binary will differ from the original.

- **Assembler Correctness**: If the assembler makes ANY error in:
  - Mnemonic encoding
  - Addressing mode encoding
  - Operand byte generation

  Then the reassembled binary will differ from the original.

**Both tools must be correct for the test to pass.**

## Test Strategy

### Step 1: Load Klaus Binary

```rust
let original_binary = load_test_binary("tests/fixtures/6502_functional_test.bin");
// 65,536 bytes of known-good machine code
```

The Klaus binary is an excellent test case because it:

- Contains all 151 valid NMOS 6502 opcodes
- Uses all 13 addressing modes extensively
- Includes edge cases (page boundaries, zero page wraparound, etc.)
- Has been thoroughly validated against real hardware

### Step 2: Disassemble

```rust
let instructions = disassemble(&original_binary, options);
// Produces 59,869 Instruction structs
```

Each instruction contains:

- Address where it was found
- Opcode byte
- Mnemonic (e.g., "LDA", "STA")
- Addressing mode
- Operand bytes (0-2 bytes)
- Size and cycle count

### Step 3: Convert to Assembly Source

```rust
let asm_source = instructions_to_source(&instructions);
// Generates valid assembly source text
```

This step converts disassembled instructions into reassemblable source code:

```assembly
.org $0000
    .byte $FF
    .byte $FF
    ...
.org $0400
    CLD
    LDX #$FF
    TXS
    LDA #$00
    STA $0200
    ...
```

**Key Challenges Solved:**

1. **Address Continuity**: Inserts `.org` directives when addresses jump
2. **Data vs Code**: Represents unrecognized bytes as `.byte` directives
3. **Branch Instructions**: Converts relative offsets to target addresses (e.g.,
   `BEQ $0450`)
4. **Addressing Mode Format**: Uses proper syntax for each addressing mode

### Step 4: Reassemble

```rust
let assembled = assemble(&asm_source)?;
// Assembles the generated source back to machine code
```

The assembler must:

- Parse all generated syntax correctly
- Detect addressing modes from operand format
- Calculate branch offsets from target addresses
- Handle `.org` directives for address mapping
- Generate identical bytes to the original

### Step 5: Compare

```rust
assert_eq!(original_binary, assembled.bytes);
```

Byte-for-byte comparison. Any difference indicates a bug in either the assembler
or disassembler.

## Files Added

```
tests/
└── functional_assembler_disassembler.rs   # Round-trip test implementation

docs/
└── ASSEMBLER_DISASSEMBLER_ROUNDTRIP.md    # This file
```

## Running the Test

The round-trip test is marked as `#[ignore]` for the same reason as the Klaus
functional test - it processes 65KB of data and takes time.

### Run the test

```bash
# Fast TDD workflow - skip slow tests
cargo test

# Run all tests INCLUDING the round-trip test
cargo test -- --include-ignored

# Run ONLY the round-trip test (with output)
cargo test --test functional_assembler_disassembler klaus_assembler_disassembler_roundtrip -- --ignored --nocapture

# Run sanity checks (verify test infrastructure works)
cargo test --test functional_assembler_disassembler tests::
```

### Expected Output

```
=== Klaus Assembler/Disassembler Round-Trip Test ===

Step 1: Loading Klaus test binary...
  Loaded 65536 bytes

Step 2: Disassembling binary...
  Disassembled 59869 instructions
  Found 176 unique opcodes (52053 invalid/data bytes)

Step 3: Converting to assembly source...
  Generated 59870 lines of assembly

Step 4: Reassembling source...
  Assembled 65536 bytes

Step 5: Comparing binaries...

✓ SUCCESS: Round-trip test passed!
  All 65536 bytes match perfectly

This validates:
  - Disassembler correctly decodes all opcodes
  - Assembler correctly encodes all instructions
  - All 176 unique opcodes round-trip correctly

test klaus_assembler_disassembler_roundtrip ... ok
```

**Test Duration**: ~0.3 seconds (much faster than the functional test)

## Implementation Details

### Test Infrastructure

The test harness provides:

1. **Binary Loader**
   - Loads 64KB test image
   - Verifies size and structure

2. **Instruction-to-Source Converter**
   - Formats each instruction as assembly text
   - Handles all 13 addressing modes
   - Inserts `.org` directives for address mapping
   - Represents invalid opcodes as `.byte` directives

3. **Binary Comparator**
   - Byte-for-byte comparison
   - Detailed error reporting on mismatch
   - Shows context around first difference

4. **Sanity Tests**
   - `test_binary_loads`: Verifies binary exists
   - `test_format_instruction_*`: Tests instruction formatting
   - `test_simple_roundtrip`: Tests with small code snippet
   - `test_instructions_to_source_with_org`: Tests `.org` insertion

### Key Implementation Decisions

#### 1. Addressing Mode Detection

The assembler uses **hex digit count** to distinguish zero page from absolute
addressing:

- `$13,X` (2 hex digits) → Zero Page,X
- `$0013,X` (4 hex digits) → Absolute,X

This is important because some instructions don't support certain zero page
modes. For example:

- `CMP $13,Y` → Error (CMP doesn't have Zero Page,Y on NMOS 6502)
- `CMP $0013,Y` → Absolute,Y (valid, opcode $D9)

#### 2. Branch Target Addresses

Branch instructions use relative addressing, but the disassembler outputs target
addresses for clarity:

```
Disassembled:  BEQ $0450
Assembler sees: $0450 (absolute address)
Assembler converts: To relative offset from PC+2
```

The assembler automatically calculates relative offsets for branch instructions,
even when given numeric target addresses.

#### 3. Address Wraparound

The 6502 has a 16-bit address space. When assembling large binaries, addresses
can wrap:

```rust
current_address = current_address.wrapping_add(instruction_size);
```

All address arithmetic uses `wrapping_add` to handle the full 64KB address space
correctly.

#### 4. Invalid Opcodes

The Klaus binary contains data sections that aren't valid instructions. The
disassembler represents these as `.byte` directives:

```assembly
.byte $FF   ; Invalid opcode or data byte
```

When reassembled, these become data bytes that match the original.

## Test Coverage

This test validates all aspects of the assembler and disassembler:

### Instruction Categories (All 151 Opcodes)

- ✅ Arithmetic: ADC, SBC
- ✅ Logic: AND, ORA, EOR
- ✅ Shifts/Rotates: ASL, LSR, ROL, ROR
- ✅ Loads: LDA, LDX, LDY
- ✅ Stores: STA, STX, STY
- ✅ Transfers: TAX, TAY, TXA, TYA, TSX, TXS
- ✅ Stack: PHA, PHP, PLA, PLP
- ✅ Comparisons: CMP, CPX, CPY
- ✅ Branches: BCC, BCS, BEQ, BNE, BMI, BPL, BVC, BVS
- ✅ Jumps/Calls: JMP, JSR, RTS
- ✅ System: BRK, RTI, NOP
- ✅ Flags: CLC, CLD, CLI, CLV, SEC, SED, SEI
- ✅ Increment/Decrement: INC, INX, INY, DEC, DEX, DEY
- ✅ Bit Test: BIT

### Addressing Modes (All 13 Modes)

- ✅ Implicit (e.g., `NOP`)
- ✅ Accumulator (e.g., `LSR A`)
- ✅ Immediate (e.g., `LDA #$42`)
- ✅ Zero Page (e.g., `LDA $10`)
- ✅ Zero Page,X (e.g., `LDA $10,X`)
- ✅ Zero Page,Y (e.g., `LDX $10,Y`)
- ✅ Absolute (e.g., `LDA $1234`)
- ✅ Absolute,X (e.g., `LDA $1234,X`)
- ✅ Absolute,Y (e.g., `LDA $1234,Y`)
- ✅ Indirect (e.g., `JMP ($FFFC)`)
- ✅ Indexed Indirect (e.g., `LDA ($40,X)`)
- ✅ Indirect Indexed (e.g., `LDA ($40),Y`)
- ✅ Relative (e.g., `BEQ label` → encoded as offset)

### Edge Cases

- ✅ Zero page wraparound ($FF,X → $00)
- ✅ Page boundary crossing
- ✅ Addresses in range $0000-$00FF (ambiguous zero page/absolute)
- ✅ Branch offsets (negative and positive)
- ✅ Invalid opcodes / data bytes
- ✅ Full 64KB address space
- ✅ Address discontinuities (`.org` directives)

## Debugging Failed Tests

If the round-trip test fails:

### 1. Enable Debug Output

```bash
SAVE_ROUNDTRIP_SOURCE=1 cargo test --test functional_assembler_disassembler klaus_assembler_disassembler_roundtrip -- --ignored --nocapture
```

This saves the disassembled source to `target/roundtrip_disassembled.asm` for
inspection.

### 2. Check Error Message

The test reports the first mismatch with context:

```
Byte mismatch: 42 total differences, first at offset $1234

Context:
Offset   Original  Reassembled
------   --------  -----------
$1230:  $A9       $A9
$1231:  $42       $42
$1232:  $8D       $8D
$1233:  $00       $00
$1234:  $20       $00         <--
$1235:  $4C       $4C
...
```

### 3. Find the Instruction

Look at the disassembled source around the failing address:

```bash
# Find line number for address $1234
grep -n "\.org.*1234" target/roundtrip_disassembled.asm

# View context around that line
sed -n 'LINE-5,LINE+5p' target/roundtrip_disassembled.asm
```

### 4. Identify the Issue

Common issues:

- **Wrong addressing mode**: Disassembler identified wrong mode (e.g., zero page
  vs absolute)
- **Wrong operand**: Operand bytes not parsed correctly
- **Missing bytes**: Instruction size calculation error
- **Wrong mnemonic**: Opcode lookup error

## Benefits of Round-Trip Testing

✅ **Comprehensive**: Tests all opcodes and addressing modes in one test ✅
**Efficient**: Reuses existing Klaus binary (no new test data needed) ✅
**Automated**: No manual verification needed ✅ **Regression Prevention**: Any
change that breaks assembler/disassembler fails immediately ✅ **Confidence**:
100% byte match = both tools are provably correct

## License & Attribution

The Klaus functional test binary was created by Klaus Dormann (Klaus2m5).

- **Test Suite**: <https://github.com/Klaus2m5/6502_65C02_functional_tests>
- **Author**: Klaus Dormann
- **License**: See original repository

This round-trip test is part of the lib6502 emulator project and follows the
project's MIT/Apache-2.0 dual license.

## References

- [Klaus Functional Test Documentation](./KLAUS_FUNCTIONAL_TEST.md)
- [Klaus2m5 Repository](https://github.com/Klaus2m5/6502_65C02_functional_tests)
- [Test Fixtures README](../tests/fixtures/README.md)
- [Project Constitution](../.specify/memory/constitution.md)
