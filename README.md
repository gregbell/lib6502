# lib6502

[![CI](https://github.com/your-org/6502/workflows/CI/badge.svg)](https://github.com/your-org/6502/actions)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

> A cycle-accurate NMOS 6502 CPU emulator built for modularity, clarity, and WebAssembly portability.

**lib6502** is a faithful emulation of the iconic MOS Technology 6502 processor, written in Rust with zero external dependencies. Designed for both historical accuracy and modern usability, it powers everything from retro computing projects to educational tools and browser-based emulators.

## Features

- **🎯 Cycle-Accurate Emulation** - Precisely tracks CPU cycles including page-crossing penalties
- **🔧 Zero Dependencies** - Core library has no external dependencies, fully `no_std` compatible
- **🌐 WebAssembly Ready** - Runs in browsers with optional WASM bindings ([try the demo](https://your-org.github.io/6502/))
- **🏗️ Modular Architecture** - Trait-based design lets you plug in custom memory implementations
- **📝 Built-in Assembler/Disassembler** - Write 6502 assembly directly in your programs
- **✅ Extensively Tested** - 1,470+ unit tests plus Klaus Dormann's comprehensive functional test (96M+ cycles, all 151 opcodes validated)
- **📚 Well Documented** - Comprehensive documentation and examples

## Quick Start

```rust
use lib6502::{CPU, FlatMemory, MemoryBus};

// Create 64KB flat memory
let mut memory = FlatMemory::new();

// Set reset vector to point to program start at 0x8000
memory.write(0xFFFC, 0x00); // Low byte
memory.write(0xFFFD, 0x80); // High byte

// Load a simple program
memory.write(0x8000, 0xA9); // LDA #$42
memory.write(0x8001, 0x42);

// Initialize CPU - it will load PC from the reset vector
let mut cpu = CPU::new(memory);

// Execute one instruction
cpu.step().unwrap();

// Check the accumulator
assert_eq!(cpu.a(), 0x42);
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
lib6502 = "0.1"
```

For WebAssembly support:

```toml
[dependencies]
lib6502 = { version = "0.1", features = ["wasm"] }
```

## Usage

### Basic Emulation

```rust
use lib6502::{CPU, FlatMemory};

let mut memory = FlatMemory::new();
// Set up reset vector and load program...

let mut cpu = CPU::new(memory);

// Execute instructions one at a time
loop {
    match cpu.step() {
        Ok(()) => {
            println!("PC: {:04X}, A: {:02X}", cpu.pc(), cpu.a());
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            break;
        }
    }
}
```

### Using the Assembler

```rust
use lib6502::assemble;

let source = r#"
    LDA #$42
    STA $8000
    JMP $8000
"#;

match assemble(source) {
    Ok(output) => {
        println!("Assembled {} bytes", output.bytes.len());
        // Load output.bytes into memory...
    }
    Err(errors) => {
        for error in errors {
            eprintln!("Line {}: {}", error.line, error.message);
        }
    }
}
```

### Using the Disassembler

```rust
use lib6502::{disassemble, DisassemblyOptions};

let bytes = vec![0xA9, 0x42, 0x8D, 0x00, 0x80];
let options = DisassemblyOptions {
    start_address: 0x8000,
    show_bytes: true,
};

for instruction in disassemble(&bytes, options) {
    println!("{:04X}: {}", instruction.address, instruction.text);
}
// Output:
// 8000: A9 42     LDA #$42
// 8002: 8D 00 80  STA $8000
```

### Custom Memory Implementation

Implement the `MemoryBus` trait to create custom memory maps:

```rust
use lib6502::{MemoryBus, CPU};

struct MyMemory {
    ram: Vec<u8>,
    rom: Vec<u8>,
}

impl MemoryBus for MyMemory {
    fn read(&self, addr: u16) -> u8 {
        if addr < 0x8000 {
            self.ram[addr as usize]
        } else {
            self.rom[(addr - 0x8000) as usize]
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        if addr < 0x8000 {
            self.ram[addr as usize] = value;
        }
        // ROM writes are ignored
    }
}

let memory = MyMemory { /* ... */ };
let cpu = CPU::new(memory);
```

## Architecture

lib6502 follows a **trait-based architecture** that separates concerns:

```
┌─────────────────┐
│   CPU<M>        │  Generic over memory implementation
│  (State + Logic)│  Registers, flags, PC, SP, cycles
└────────┬────────┘
         │ uses
         ▼
┌─────────────────┐
│  MemoryBus      │  Trait: read(addr) -> u8
│  (Trait)        │         write(addr, value)
└────────┬────────┘
         │ implements
    ┌────┴──────┬──────────┐
    ▼           ▼          ▼
┌──────────┐ ┌────────┐  ┌─────────┐
│FlatMemory│ │ Custom │  │ Custom  │
│ (64KB)   │ │ Memory │  │ Memory  │
└──────────┘ └────────┘  └─────────┘
```

**Key Components:**

- **`CPU<M: MemoryBus>`** - Generic CPU implementation with all processor state
- **`MemoryBus` trait** - Abstraction for memory access (read/write)
- **`FlatMemory`** - Simple 64KB RAM implementation
- **`OPCODE_TABLE`** - Static table of all 256 opcode metadata entries
- **Assembler/Disassembler** - Convert between assembly text and bytecode

See [`CLAUDE.md`](CLAUDE.md) for detailed architecture documentation.

## Examples

The [`examples/`](examples/) directory contains:

- **`simple_ram.rs`** - Basic CPU setup and execution
- **`simple_asm.rs`** - Assembler usage
- **`simple_disasm.rs`** - Disassembler usage

Run examples with:

```bash
cargo run --example simple_ram
```

## Web Demo

An interactive 6502 assembly playground is available at the [GitHub Pages demo](https://your-org.github.io/6502/) (or run locally from the `demo/` directory).

The demo features:
- Live assembly editor
- Real-time CPU state visualization
- Memory viewer
- Cycle-accurate execution

## Development

### Prerequisites

- Rust 1.75 or later
- Standard Rust toolchain (`cargo`, `rustc`, `rustfmt`, `clippy`)

### Building

```bash
# Build the library
cargo build

# Build with WASM support
cargo build --features wasm
```

### Testing

The project has two test suites:

**Fast Tests** (~2 seconds, runs by default):
```bash
cargo test
```

**Functional Tests** (includes Klaus test, ~6 seconds):
```bash
cargo test -- --ignored
```

**All Tests** (comprehensive, ~8 seconds):
```bash
cargo test && cargo test -- --ignored
```

See [`docs/KLAUS_FUNCTIONAL_TEST.md`](docs/KLAUS_FUNCTIONAL_TEST.md) for details about the comprehensive functional test suite.

### Code Quality

```bash
# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt

# Check formatting
cargo fmt --all -- --check
```

### Project Structure

```
lib6502/
├── src/
│   ├── lib.rs           # Public API and error types
│   ├── cpu.rs           # CPU state and execution logic
│   ├── memory.rs        # MemoryBus trait and FlatMemory
│   ├── opcodes.rs       # OPCODE_TABLE metadata
│   ├── addressing.rs    # AddressingMode enum
│   ├── assembler.rs     # 6502 assembler
│   ├── disassembler.rs  # 6502 disassembler
│   ├── instructions/    # Instruction implementations
│   └── wasm.rs          # WebAssembly bindings (optional)
├── tests/               # Integration tests
│   ├── fixtures/        # Test data (Klaus binary, etc.)
│   └── *.rs             # Test implementations
├── examples/            # Usage examples
├── demo/                # WebAssembly demo webpage
├── docs/                # Documentation
└── specs/               # Feature specifications
```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and linters (`cargo test && cargo clippy && cargo fmt`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

See [`CLAUDE.md`](CLAUDE.md) for development guidelines and project constitution.

## Design Principles

- **Modularity** - Separation of concerns via traits
- **Clarity** - Readable, well-documented code
- **WebAssembly Portability** - No OS dependencies
- **Cycle Accuracy** - Faithful timing emulation
- **Hackability** - Easy to extend and modify
- **Zero Dependencies** - Core library has no external dependencies

## Testing Philosophy

The project maintains two test categories to support different workflows:

- **Fast Tests** (1,470+ tests, ~2s) - For rapid TDD iteration
- **Functional Tests** (Klaus suite, ~6s) - For comprehensive validation

CI runs both suites to ensure correctness while keeping local development fast.

## Documentation

- [Architecture Guide](CLAUDE.md) - Development guide and project overview
- [Klaus Functional Test](docs/KLAUS_FUNCTIONAL_TEST.md) - Comprehensive test suite documentation
- [Assembler/Disassembler](docs/ASSEMBLER_DISASSEMBLER_ROUNDTRIP.md) - Assembly tooling details

## Roadmap

- [x] Core CPU foundation
- [x] All 151 NMOS 6502 opcodes
- [x] Cycle-accurate timing
- [x] Assembler and disassembler
- [x] WebAssembly support
- [x] Interactive web demo
- [ ] Debugger interface
- [ ] Additional memory implementations (ROM, banked memory)
- [ ] Performance optimizations
- [ ] CMOS 65C02 support

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- **Klaus Dormann** - For the comprehensive [6502 functional test suite](https://github.com/Klaus2m5/6502_65C02_functional_tests)
- **MOS Technology** - For creating the legendary 6502 processor
- The Rust community for excellent tooling and documentation

## See Also

- [6502.org](http://www.6502.org/) - The 6502 microprocessor resource
- [Visual 6502](http://visual6502.org/) - Visual transistor-level simulation
- [Easy 6502](https://skilldrick.github.io/easy6502/) - Learn 6502 assembly

---

Built with ❤️ in Rust
