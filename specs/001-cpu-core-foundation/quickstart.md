# Quickstart Guide: CPU Core Foundation

**Feature**: 001-cpu-core-foundation
**Date**: 2025-11-13
**Audience**: Developers using or contributing to the 6502 CPU core

This guide provides step-by-step instructions for setting up the development environment, understanding the core architecture, and running basic examples.

---

## Prerequisites

### Required Tools

- **Rust**: Stable toolchain (1.91.1 or later recommended)
  ```bash
  # Install via rustup if not already installed
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

  # Verify installation
  rustc --version
  cargo --version
  ```

- **WASM Target** (for WebAssembly compilation verification):
  ```bash
  rustup target add wasm32-unknown-unknown
  ```

### Optional Tools

- **IDE/Editor**: VS Code with rust-analyzer, or any Rust-compatible editor
- **Git**: For version control and pulling updates

---

## Project Structure Overview

```
6502/
├── src/
│   ├── lib.rs           # Library root, public API exports
│   ├── cpu.rs           # CPU struct and execution logic
│   ├── memory.rs        # MemoryBus trait and FlatMemory impl
│   ├── opcodes.rs       # OPCODE_TABLE (256 entries)
│   ├── addressing.rs    # AddressingMode enum
│   └── instruction.rs   # Instruction decoding (future)
│
├── tests/
│   ├── cpu_init_test.rs     # CPU initialization tests
│   ├── memory_bus_test.rs   # MemoryBus trait tests
│   └── execute_loop_test.rs # Fetch-decode-execute tests
│
├── examples/
│   └── simple_ram.rs    # Basic usage example
│
├── Cargo.toml           # Project manifest
└── specs/               # Design documentation
    └── 001-cpu-core-foundation/
        ├── spec.md
        ├── plan.md
        ├── research.md
        ├── data-model.md
        ├── quickstart.md (this file)
        └── contracts/
```

---

## Building the Project

### Standard Build

```bash
# Build the library
cargo build

# Build in release mode (optimized)
cargo build --release
```

Expected output: Successful compilation with zero warnings/errors.

### WebAssembly Build

Verify WASM portability (per constitution principle II):

```bash
cargo build --target wasm32-unknown-unknown
```

Expected output: Successful compilation. No OS-specific dependencies should cause errors.

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with verbose output
cargo test -- --nocapture

# Run specific test file
cargo test --test cpu_init_test
```

Expected output: All tests pass (100% pass rate for structural tests).

### Running Examples

```bash
# Run the simple RAM example
cargo run --example simple_ram
```

Expected output: Example demonstrates CPU initialization, memory setup, and attempted instruction execution (will report UnimplementedOpcode errors since no instructions are implemented in this feature).

---

## Quick Start: Using the CPU Core

### Step 1: Create a Memory Bus

The CPU requires a memory implementation. Use the provided `FlatMemory` for simple cases:

```rust
use cpu6502::FlatMemory;

// Create 64KB flat memory (all addresses mapped to RAM)
let mut memory = FlatMemory::new();

// Set reset vector (0xFFFC/0xFFFD) to point to program start
memory.write(0xFFFC, 0x00); // Low byte of reset address
memory.write(0xFFFD, 0x80); // High byte = 0x8000

// Load a program at 0x8000 (example: placeholder opcodes)
memory.write(0x8000, 0xEA); // NOP (opcode 0xEA - not implemented yet)
memory.write(0x8001, 0x00); // BRK (opcode 0x00 - not implemented yet)
```

### Step 2: Initialize the CPU

```rust
use cpu6502::CPU;

// Create CPU with the memory bus
let mut cpu = CPU::new(memory);

// Verify initial state (post-reset)
println!("PC: 0x{:04X}", cpu.pc());     // Should be 0x8000 (from reset vector)
println!("SP: 0x{:02X}", cpu.sp());     // Should be 0xFD
println!("I flag: {}", cpu.flag_i());   // Should be true
println!("Cycles: {}", cpu.cycles());   // Should be 0
```

### Step 3: Execute Instructions

```rust
use cpu6502::ExecutionError;

// Execute one instruction
match cpu.step() {
    Ok(()) => {
        println!("Instruction executed successfully");
        println!("PC is now: 0x{:04X}", cpu.pc());
        println!("Cycles: {}", cpu.cycles());
    }
    Err(ExecutionError::UnimplementedOpcode(opcode)) => {
        println!("Opcode 0x{:02X} not implemented", opcode);
        // This is expected in this feature - no instructions implemented yet
    }
}
```

### Step 4: Frame-Based Execution

For emulation loops (e.g., running CPU for a fixed number of cycles per frame):

```rust
// NTSC frame: ~29780 cycles (1.79 MHz / 60 Hz)
const CYCLES_PER_FRAME: u64 = 29780;

match cpu.run_for_cycles(CYCLES_PER_FRAME) {
    Ok(actual_cycles) => {
        println!("Executed {} cycles this frame", actual_cycles);
    }
    Err(e) => {
        eprintln!("Execution halted: {}", e);
    }
}
```

---

## Implementing a Custom Memory Bus

For more complex memory layouts (e.g., NES-style memory mapping, memory-mapped I/O), implement the `MemoryBus` trait:

```rust
use cpu6502::MemoryBus;

/// Custom memory with ROM at 0x8000-0xFFFF and RAM at 0x0000-0x7FFF
struct RomRamMemory {
    ram: [u8; 0x8000],  // 32KB RAM
    rom: [u8; 0x8000],  // 32KB ROM
}

impl RomRamMemory {
    pub fn new() -> Self {
        Self {
            ram: [0; 0x8000],
            rom: [0; 0x8000],
        }
    }

    /// Load ROM data at initialization
    pub fn load_rom(&mut self, data: &[u8]) {
        let copy_len = data.len().min(0x8000);
        self.rom[..copy_len].copy_from_slice(&data[..copy_len]);
    }
}

impl MemoryBus for RomRamMemory {
    fn read(&self, addr: u16) -> u8 {
        if addr < 0x8000 {
            self.ram[addr as usize]
        } else {
            self.rom[(addr - 0x8000) as usize]
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        if addr < 0x8000 {
            // Write to RAM
            self.ram[addr as usize] = value;
        }
        // Writes to ROM (0x8000+) are ignored
    }
}
```

Usage:

```rust
let mut memory = RomRamMemory::new();
memory.load_rom(&rom_data);
let mut cpu = CPU::new(memory);
```

---

## Inspecting CPU State

The CPU provides read-only getters for all internal state:

```rust
// Registers
println!("A: 0x{:02X}", cpu.a());
println!("X: 0x{:02X}", cpu.x());
println!("Y: 0x{:02X}", cpu.y());

// Program counter and stack pointer
println!("PC: 0x{:04X}", cpu.pc());
println!("SP: 0x{:02X}", cpu.sp());

// Status register (packed byte)
println!("Status: 0x{:02X} (NV-BDIZC)", cpu.status());

// Individual status flags
println!("Negative: {}", cpu.flag_n());
println!("Overflow: {}", cpu.flag_v());
println!("Break: {}", cpu.flag_b());
println!("Decimal: {}", cpu.flag_d());
println!("Interrupt Disable: {}", cpu.flag_i());
println!("Zero: {}", cpu.flag_z());
println!("Carry: {}", cpu.flag_c());

// Cycle counter
println!("Total cycles: {}", cpu.cycles());
```

---

## Exploring the Opcode Table

The opcode metadata table is publicly accessible for introspection:

```rust
use cpu6502::OPCODE_TABLE;

// Look up metadata for a specific opcode
let lda_immediate = &OPCODE_TABLE[0xA9];
println!("Opcode 0xA9:");
println!("  Mnemonic: {}", lda_immediate.mnemonic);
println!("  Mode: {:?}", lda_immediate.addressing_mode);
println!("  Cycles: {}", lda_immediate.base_cycles);
println!("  Size: {} bytes", lda_immediate.size_bytes);
println!("  Implemented: {}", lda_immediate.implemented);

// Enumerate all documented instructions
for (opcode, metadata) in OPCODE_TABLE.iter().enumerate() {
    if metadata.mnemonic != "???" {
        println!("0x{:02X}: {} ({:?}, {} cycles, {} bytes)",
                 opcode,
                 metadata.mnemonic,
                 metadata.addressing_mode,
                 metadata.base_cycles,
                 metadata.size_bytes);
    }
}
```

---

## Understanding Test Results

### Expected Test Behavior (This Feature)

Since no instructions are implemented yet, tests verify:

1. **CPU Initialization**: All registers have correct reset values
2. **MemoryBus Trait**: `FlatMemory` read/write operations work
3. **Opcode Table**: All 256 entries exist with valid metadata
4. **Execute Loop**: Fetch-decode-execute returns `UnimplementedOpcode` errors

Run tests to verify:

```bash
cargo test

# Example output:
# test cpu_init_test::test_cpu_reset_values ... ok
# test memory_bus_test::test_flat_memory_read_write ... ok
# test execute_loop_test::test_step_unimplemented ... ok
# test opcode_table_completeness ... ok
```

### Test Coverage

Per success criterion SC-010, code coverage should reach 80% of defined structures and initialization code. Run coverage tools if available:

```bash
# Using cargo-tarpaulin (optional)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

---

## Next Steps

### Implementing Your First Instruction

When implementing a new instruction in a future feature:

1. **Update Opcode Table**: Set `OPCODE_TABLE[opcode].implemented = true`
2. **Add Execution Logic**: Implement instruction behavior in `cpu.rs` or dedicated module
3. **Write Tests**: Create test cases verifying register/flag/memory changes
4. **Update Documentation**: Add rustdoc examples for the instruction

Example workflow (pseudocode for future work):

```rust
// In cpu.rs execution logic
match opcode {
    0xA9 => {
        // LDA Immediate
        self.a = self.fetch_operand_byte();
        self.update_nz_flags(self.a);
        self.pc += 1; // Advance past operand byte
    }
    _ => return Err(ExecutionError::UnimplementedOpcode(opcode)),
}
```

### Understanding Cycle Accuracy

The CPU tracks cycles for timing-accurate emulation:

- Each instruction increments the cycle counter by its `base_cycles` from the opcode table
- Future instruction implementations will add page-crossing penalties dynamically
- Use `run_for_cycles()` for frame-locked execution (e.g., 60 Hz frame timing)

### Debugging Techniques

**Enable Verbose Logging** (future enhancement):

```rust
// Example: Custom memory bus with logging
struct LoggingMemory {
    inner: FlatMemory,
}

impl MemoryBus for LoggingMemory {
    fn read(&self, addr: u16) -> u8 {
        let value = self.inner.read(addr);
        println!("READ  0x{:04X} = 0x{:02X}", addr, value);
        value
    }

    fn write(&mut self, addr: u16, value: u8) {
        println!("WRITE 0x{:04X} = 0x{:02X}", addr, value);
        self.inner.write(addr, value);
    }
}
```

---

## Common Issues & Solutions

### Issue: "Opcode 0xXX not implemented"

**Solution**: This is expected behavior for this feature. No instructions are implemented yet. The error handling mechanism is working correctly. Future features will implement specific opcodes.

### Issue: CPU PC doesn't match expected value after reset

**Solution**: Verify the reset vector is correctly set in memory:

```rust
// Reset vector at 0xFFFC/0xFFFD must point to valid program start
memory.write(0xFFFC, 0x00); // Low byte
memory.write(0xFFFD, 0x80); // High byte (PC will be 0x8000)
```

### Issue: WASM build fails with OS dependency error

**Solution**: Ensure no `std::fs`, `std::net`, or OS-specific code in the core module. The CPU core must be WASM-compatible per constitution principle II.

### Issue: Stack pointer seems incorrect

**Solution**: Stack pointer is an 8-bit value representing the low byte of the stack address. The full stack address is `0x0100 + SP`. Initial SP value is `0xFD`, meaning stack starts at `0x01FD`.

---

## Performance Considerations

### Current Performance (Foundation)

This feature prioritizes clarity and correctness over performance (per constitution principle IV). Baseline performance is adequate for:

- Educational purposes and learning the 6502
- Fantasy console development (1-2 MHz equivalent speed on modern hardware)
- Debugging and development

### Future Optimizations (Out of Scope)

Once the instruction set is complete, performance-critical sections can be profiled and optimized:

- Opcode dispatch optimization (jump tables, computed goto)
- Status flag bitfield instead of individual bools
- Instruction inlining and loop unrolling
- JIT compilation for WASM target

**Note**: Optimization must not compromise clarity without measurement-backed justification (constitution principle IV).

---

## Resources

### Reference Documentation

- **6502 Architecture**: `docs/6502-reference/Architecture.md`
- **6502 Registers**: `docs/6502-reference/Registers.md`
- **Addressing Modes**: `docs/6502-reference/Addressing-Modes.md`
- **Instruction Set**: `docs/6502-reference/Instructions.md`

### Design Documentation

- **Feature Spec**: `specs/001-cpu-core-foundation/spec.md`
- **Implementation Plan**: `specs/001-cpu-core-foundation/plan.md`
- **Research Decisions**: `specs/001-cpu-core-foundation/research.md`
- **Data Model**: `specs/001-cpu-core-foundation/data-model.md`
- **API Contracts**: `specs/001-cpu-core-foundation/contracts/`

### Project Constitution

- **Constitution**: `.specify/memory/constitution.md` (core architectural principles)

### External Resources

- [6502.org](http://www.6502.org) - Comprehensive 6502 reference
- [Visual 6502](http://www.visual6502.org) - Transistor-level 6502 simulation
- [NesDev Wiki](https://www.nesdev.org/wiki/CPU) - NES 2A03 (6502 variant) documentation

---

## Getting Help

### In-Code Documentation

All public APIs have rustdoc comments. Generate and browse local documentation:

```bash
cargo doc --open
```

### Test Examples

Tests serve as executable examples. Read test code in `tests/` to see how to use the API.

### Contributing

When adding new features or fixing bugs:

1. Follow the architecture established in this foundation
2. Maintain adherence to the project constitution (modularity, WASM portability, clarity)
3. Add tests for all new functionality
4. Update documentation as needed

---

## Summary Checklist

- [ ] Rust stable toolchain installed
- [ ] WASM target added (`rustup target add wasm32-unknown-unknown`)
- [ ] Project builds successfully (`cargo build`)
- [ ] WASM build succeeds (`cargo build --target wasm32-unknown-unknown`)
- [ ] All tests pass (`cargo test`)
- [ ] Example runs (`cargo run --example simple_ram`)
- [ ] Documentation accessible (`cargo doc --open`)

**Congratulations!** You're ready to explore the 6502 CPU core and begin implementing instructions in future features.
