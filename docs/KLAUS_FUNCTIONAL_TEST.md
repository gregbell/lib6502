# Klaus Dormann's 6502 Functional Test Integration

## Overview

This document describes the integration of Klaus Dormann's comprehensive 6502 functional test suite into our emulator. This test validates all valid opcodes and addressing modes of the NMOS 6502 processor.

**Test Source**: https://github.com/Klaus2m5/6502_65C02_functional_tests

## What is the Functional Test?

The Klaus functional test is a self-contained assembly program that:

1. Tests every valid 6502 opcode with multiple addressing modes
2. Validates processor flags (N, V, Z, C) for each operation
3. Checks edge cases (overflow, underflow, page crossing, etc.)
4. Uses an elegant pass/fail mechanism based on infinite loops

### Test Mechanics

- **Binary**: 64KB memory image (`6502_functional_test.bin`)
- **Entry Point**: $0400
- **Success Address**: $3469 - PC stops here when all tests pass
- **Failure Detection**: PC stops at any other address when test fails
- **Loop Detection**: Test uses `JMP *` (infinite loop) to signal completion

## Files Added

```
tests/
├── fixtures/
│   ├── README.md                       # Documentation about test files
│   ├── 6502_functional_test.bin        # 64KB test binary (from Klaus2m5)
│   └── 6502_functional_test.lst        # Assembly listing (711KB, for debugging)
└── functional_klaus.rs                 # Test harness implementation

.gitattributes                          # Binary file handling
docs/
└── KLAUS_FUNCTIONAL_TEST.md            # This file
```

## Running the Test

The functional test is marked as `#[ignore]` because it requires more instructions to be implemented.

### Run the test

```bash
# Run only the Klaus functional test
cargo test --test functional_klaus klaus_6502_functional_test -- --ignored --nocapture

# Run all non-ignored tests (Klaus test will be skipped)
cargo test

# Run sanity checks (verify binary loads correctly)
cargo test --test functional_klaus tests::
```

### Expected Output

**Current State** (with only 8 instructions implemented):
```
=== Klaus 6502 Functional Test ===
Entry point: $0400
Success address: $3469
Initial state: PC:$0400 A:$00 X:$00 Y:$00 SP:$FD P:[---I---] Cycles:0
Running test...

=== TEST FAILED ===
Execution error at PC $0400: Opcode 0xD8 is not implemented.
Final state: PC:$0401 A:$00 X:$00 Y:$00 SP:$FD P:[---I---] Cycles:2
```

The first instruction is **CLD (0xD8)** - Clear Decimal Mode.

**When Test Passes** (after all instructions implemented):
```
=== Klaus 6502 Functional Test ===
Entry point: $0400
Success address: $3469
...
Test completed!
Final state: PC:$3469 ...
Final PC: $3469

✓ SUCCESS: All tests passed!
```

## Implementation Details

### Test Infrastructure

The test harness (`tests/functional_klaus.rs`) provides:

1. **Binary Loader**
   - Loads 64KB test image into `FlatMemory`
   - Verifies and sets reset vector to entry point ($0400)
   - Validates binary size and structure

2. **Infinite Loop Detection**
   - Tracks PC history (last 3 values)
   - Detects when PC stops changing (indicates `JMP *` loop)
   - Prevents infinite hangs with cycle budget (100M cycles)

3. **Diagnostic Output**
   - Shows CPU state (registers, flags, cycles)
   - Displays memory around failure point
   - Provides listing file reference for debugging

4. **Sanity Tests**
   - `test_binary_exists_and_correct_size`: Verifies binary loads
   - `test_success_address_has_infinite_loop`: Confirms success marker exists

### How It Works

```rust
// 1. Load test binary into memory
let memory = load_test_binary("tests/fixtures/6502_functional_test.bin");

// 2. Create CPU (reads PC from reset vector)
let mut cpu = CPU::new(memory);

// 3. Execute until infinite loop detected
let final_pc = run_until_loop(&mut cpu, MAX_CYCLES, verbose)?;

// 4. Check if we reached success address
assert_eq!(final_pc, SUCCESS_ADDRESS);
```

### Key Constants

```rust
const SUCCESS_ADDRESS: u16 = 0x3469;  // Where PC ends on success
const ENTRY_POINT: u16 = 0x0400;      // Test start address
const MAX_CYCLES: u64 = 100_000_000;  // Timeout protection
const LOOP_DETECTION_THRESHOLD: usize = 3; // PC unchanged count
```

## Using the Test to Guide Implementation

The functional test is an excellent tool for incremental development:

### 1. Run Test to Find Next Missing Instruction

```bash
cargo test --test functional_klaus klaus_6502_functional_test -- --ignored --nocapture 2>&1 | grep "Opcode"
```

Output shows which opcode blocked progress:
```
Execution error at PC $0400: Opcode 0xD8 is not implemented.
```

### 2. Look Up Opcode

```bash
# Find opcode in listing file
grep "^0400" tests/fixtures/6502_functional_test.lst
```

Output:
```
0400 : d8          start   cld
```

Opcode `0xD8` = **CLD** (Clear Decimal Mode)

### 3. Implement the Instruction

Follow the existing pattern in `src/instructions/` and update `OPCODE_TABLE`.

### 4. Repeat

Run test again to find the next missing instruction.

## Debugging Failed Tests

When a test fails at a specific address (not the success address):

### 1. Note the Failure Address

```
Test failed at PC $1234
```

### 2. Check the Listing File

```bash
grep "^1234" tests/fixtures/6502_functional_test.lst
```

This shows which instruction and which test was running at that address.

### 3. Examine Context

The test output shows memory around the failure point and CPU state, helping identify what went wrong.

## Test Coverage

Once fully passing, this test validates:

### Instruction Categories
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

### Addressing Modes (13 total)
- Implied
- Accumulator
- Immediate
- Zero Page
- Zero Page,X
- Zero Page,Y
- Absolute
- Absolute,X
- Absolute,Y
- Indirect (JMP only)
- Indexed Indirect (Indirect,X)
- Indirect Indexed (Indirect),Y
- Relative (branches)

### Edge Cases
- Page crossing penalties
- Zero page wraparound
- Stack operations
- Flag updates (N, V, Z, C)
- Signed vs unsigned behavior
- Overflow detection

## Implementation Progress

### Currently Implemented (8 instructions, 27 opcodes)
- ADC (8 addressing modes)
- AND (8 addressing modes)
- ASL (5 addressing modes)
- BCC, BCS, BEQ, BMI (1 each)
- BIT (2 addressing modes)

### Status: First Instruction Blocking
- **CLD (0xD8)** - Clear Decimal Mode

This is the very first instruction in the test, so we haven't gotten far yet!

### Remaining Work
~229 opcodes across:
- Remaining branches (BNE, BPL, BVC, BVS)
- Loads/Stores (LDA, LDX, LDY, STA, STX, STY)
- Transfers (TAX, TAY, TXA, TYA, TSX, TXS)
- Stack operations (PHA, PHP, PLA, PLP)
- Comparisons (CMP, CPX, CPY)
- Remaining logic (EOR, ORA)
- Remaining shifts (LSR, ROL, ROR)
- Inc/Dec (INC, INX, INY, DEC, DEX, DEY)
- Jumps (JMP, JSR, RTS, RTI)
- System (BRK, NOP)
- Flag manipulation (CLC, CLD, CLI, CLV, SEC, SED, SEI)
- SBC (Subtract with Carry)

## Milestones

### Milestone 1: Test Starts ✅
- [x] Test binary loads correctly
- [x] CPU initializes at entry point
- [x] Infrastructure detects first blocking instruction

### Milestone 2: Basic Instructions
- [ ] Implement core instructions (loads, stores, transfers)
- [ ] Test progresses past initialization
- [ ] Basic test loops execute

### Milestone 3: Full Implementation
- [ ] All 151 documented 6502 opcodes implemented
- [ ] Test reaches success address ($3469)
- [ ] All edge cases pass

## License & Attribution

The functional test suite was created by Klaus Dormann (Klaus2m5).

- **Test Suite**: https://github.com/Klaus2m5/6502_65C02_functional_tests
- **Author**: Klaus Dormann
- **License**: See original repository

This integration is part of the cpu6502 emulator project and follows the project's MIT/Apache-2.0 dual license.

## References

- [Klaus2m5 Functional Tests Repository](https://github.com/Klaus2m5/6502_65C02_functional_tests)
- [6502.org Discussion Thread](http://forum.6502.org/viewtopic.php?f=8&t=5298)
- [Test Fixtures README](../tests/fixtures/README.md)
- [Project Constitution](../.specify/memory/constitution.md)

## Future Enhancements

### Verbose Mode
Currently disabled by default. Set `verbose = true` in the test to see:
- Progress updates every 100k cycles
- PC tracking through execution
- Detailed timing information

### Breakpoint Support
Add ability to halt execution at specific addresses for debugging.

### Test Reports
Generate detailed reports showing:
- Which instruction categories pass/fail
- Cycle count statistics
- Coverage metrics

### CI Integration
Once sufficient instructions are implemented, add to CI pipeline as:
- Regression test for all PRs
- Performance benchmark
- Coverage validator
