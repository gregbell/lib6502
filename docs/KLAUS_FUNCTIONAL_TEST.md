# Klaus Dormann's 6502 Functional Test Integration

## Status: ✅ PASSING (100%)

The Klaus functional test **passes completely**, validating all 151 documented NMOS 6502 opcodes across 96+ million instruction cycles.

**Final Results:**
- **PC**: $3469 (exact success address)
- **Cycles**: 96,241,373
- **Test Status**: ✓ SUCCESS - All tests passed

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

The functional test runs automatically as part of the standard test suite.

### Run the test

```bash
# Run all tests (includes Klaus functional test)
cargo test

# Run only the Klaus functional test with output
cargo test --test functional_klaus klaus_6502_functional_test -- --nocapture

# Run sanity checks (verify binary loads correctly)
cargo test --test functional_klaus tests::
```

### Expected Output

```
=== Klaus 6502 Functional Test ===
Entry point: $0400
Success address: $3469
Initial state: PC:$0400 A:$00 X:$00 Y:$00 SP:$FD P:[---I---] Cycles:0
Running test...

Test completed!
Final state: PC:$3469 A:$F0 X:$0E Y:$FF SP:$FF P:[NV---CB] Cycles:96241373
Final PC: $3469

✓ SUCCESS: All tests passed!
test klaus_6502_functional_test ... ok
```

**Test Duration**: ~6 seconds (96+ million cycles)

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

## Test-Driven Development

The Klaus functional test served as an excellent tool for incremental development during implementation.

### Development Workflow (Historical)

1. **Run Test** → Identify next missing instruction by opcode error
2. **Look Up Opcode** → Use listing file to find instruction at failure address
3. **Implement Instruction** → Follow patterns in `src/instructions/`, update `OPCODE_TABLE`
4. **Repeat** → Test progresses further with each instruction added

This iterative approach helped implement all 151 opcodes systematically, with the test providing immediate validation of each addition.

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

## Implementation Status

### ✅ Complete Implementation (151 opcodes)

All NMOS 6502 instructions are fully implemented:

- ✅ **Arithmetic**: ADC (binary + BCD), SBC (binary + BCD)
- ✅ **Logic**: AND, ORA, EOR
- ✅ **Shifts/Rotates**: ASL, LSR, ROL, ROR
- ✅ **Loads**: LDA, LDX, LDY
- ✅ **Stores**: STA, STX, STY
- ✅ **Transfers**: TAX, TAY, TXA, TYA, TSX, TXS
- ✅ **Stack**: PHA, PHP, PLA, PLP
- ✅ **Comparisons**: CMP, CPX, CPY
- ✅ **Branches**: BCC, BCS, BEQ, BNE, BMI, BPL, BVC, BVS
- ✅ **Jumps/Calls**: JMP, JSR, RTS
- ✅ **System**: BRK, RTI, NOP
- ✅ **Flags**: CLC, CLD, CLI, CLV, SEC, SED, SEI
- ✅ **Increment/Decrement**: INC, INX, INY, DEC, DEX, DEY
- ✅ **Bit Test**: BIT

### Key Features Validated

- **Binary mode arithmetic**: All operations work correctly in standard mode
- **Decimal mode (BCD)**: ADC and SBC perform Binary Coded Decimal arithmetic when D flag is set
- **Cycle accuracy**: Correct cycle counts including page-crossing penalties
- **Flag behavior**: N, V, Z, C flags update correctly (D flag controls BCD mode)
- **Edge cases**: Page crossing, wraparound, overflow, all validated

## Milestones

### Milestone 1: Test Infrastructure ✅
- [x] Test binary loads correctly
- [x] CPU initializes at entry point
- [x] Infrastructure detects blocking instructions
- [x] Infinite loop detection works
- [x] Diagnostic output helps debugging

### Milestone 2: Core Instructions ✅
- [x] Implement core instructions (loads, stores, transfers)
- [x] Test progresses past initialization
- [x] Basic test loops execute
- [x] Binary mode arithmetic complete

### Milestone 3: Full Implementation ✅
- [x] All 151 documented 6502 opcodes implemented
- [x] Decimal mode (BCD) arithmetic in ADC/SBC
- [x] Test reaches success address ($3469)
- [x] All edge cases pass
- [x] 96+ million cycles execute flawlessly

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

## CI/CD Integration

The Klaus functional test is now part of the standard test suite and runs automatically:

- ✅ **Regression Testing**: Validates all 151 opcodes on every test run
- ✅ **Quality Gate**: Prevents breaking changes from being merged
- ✅ **Performance Baseline**: Tracks execution time (~6 seconds for 96M cycles)

### Continuous Validation

Every `cargo test` run includes:
- 1,471 total tests (including Klaus test)
- Complete instruction coverage validation
- BCD arithmetic verification
- Cycle-accurate timing checks

## Potential Enhancements

### Verbose Mode
Set `verbose = true` in the test to enable:
- Progress updates every 100k cycles
- PC tracking through execution
- Detailed timing information

### Breakpoint Support
Add ability to halt execution at specific addresses for debugging instruction implementations.

### Performance Benchmarking
Track cycle execution speed over time:
- Measure instructions per second
- Compare performance across platforms
- Identify optimization opportunities
