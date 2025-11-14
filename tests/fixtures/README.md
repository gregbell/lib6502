# Test Fixtures

This directory contains binary test files used for validating the 6502 CPU emulator.

## Klaus Dormann's 6502 Functional Tests

The files `6502_functional_test.bin` and `6502_functional_test.lst` are from the comprehensive functional test suite by Klaus Dormann:

https://github.com/Klaus2m5/6502_65C02_functional_tests

### Files

- **6502_functional_test.bin** (65,536 bytes)
  - Complete 64KB memory image including test code and data
  - Tests all valid NMOS 6502 opcodes and addressing modes
  - Licensed under the terms provided by the original author

- **6502_functional_test.lst** (~711 KB)
  - Assembly listing with addresses and disassembly
  - Used to identify which specific test failed
  - Maps PC addresses to test names and instructions

### How the Test Works

1. Load the binary into a 64KB memory space
2. Set PC to entry point at $0400 (via reset vector)
3. Execute instructions until an infinite loop is detected
4. Success = PC stops at $3469
5. Failure = PC stops at any other address (indicates failing test)

### License

These test files are distributed as part of Klaus Dormann's test suite. Please refer to the original repository for licensing information.

### Attribution

Functional test suite created by Klaus Dormann (Klaus2m5).
