# Feature Specification: CPU Core Foundation

**Feature Branch**: `001-cpu-core-foundation`
**Created**: 2025-11-13
**Status**: Draft
**Input**: User description: "Create the initial project build along with the basic rust architecture, tests, etc. Don't implement any instructions, but sketch out the main loop that will be used. We will create a spec for every single instrution, so don't worry about that yet. This is about getting the main structures in place along with the main loop in place so that we can implement our first instruction."

## User Scenarios & Testing _(mandatory)_

### User Story 1 - Project Initialization (Priority: P1)

As a developer, I need a working Rust project structure with the core CPU module scaffolding so that I can begin implementing individual 6502 instructions in subsequent work.

**Why this priority**: This is the foundational infrastructure. Without it, no instruction implementation can begin. It establishes the architectural patterns that all future work will follow.

**Independent Test**: Can be fully tested by verifying the project compiles successfully, all module structures are defined, tests run (even if they only verify structure), and the CPU can be instantiated with default state.

**Acceptance Scenarios**:

1. **Given** a clean repository, **When** the developer runs the build command, **Then** the project compiles without errors
2. **Given** the project structure, **When** the developer inspects the CPU module, **Then** all core data structures (registers, flags, program counter, stack pointer, cycle counter) are defined
3. **Given** the CPU module, **When** the developer runs the test suite, **Then** all tests execute successfully (structural validation tests pass)
4. **Given** a CPU instance, **When** the developer inspects its initial state, **Then** all registers are initialized to documented reset values

---

### User Story 2 - Memory Bus Abstraction (Priority: P2)

As a developer, I need a trait-based memory bus interface so that the CPU core can read and write memory without depending on any specific memory implementation.

**Why this priority**: This is required by the constitution's Modularity principle and enables future work on different memory implementations (flat RAM, NES-style mapping, debugging wrappers, etc.).

**Independent Test**: Can be fully tested by implementing a simple test memory (flat 64KB array) that implements the bus trait, then verifying the CPU can read and write through the abstraction.

**Acceptance Scenarios**:

1. **Given** a memory bus trait definition, **When** a developer creates a simple RAM implementation, **Then** it implements the trait correctly
2. **Given** a CPU instance with a memory bus, **When** the CPU performs a read operation, **Then** the correct value is retrieved through the trait
3. **Given** a CPU instance with a memory bus, **When** the CPU performs a write operation, **Then** the value is stored correctly through the trait
4. **Given** multiple memory implementations (e.g., flat RAM and a debug wrapper), **When** the CPU is instantiated with either, **Then** it operates correctly with both

---

### User Story 3 - Fetch-Decode-Execute Loop (Priority: P3)

As a developer, I need a skeletal fetch-decode-execute loop that can be called to advance the CPU by one instruction so that I have a clear integration point for implementing individual instructions.

**Why this priority**: This establishes the execution model and provides the main entry point for running programs. While foundational, it depends on the CPU state and memory bus being in place first.

**Independent Test**: Can be fully tested by implementing a single no-op instruction and verifying the loop can fetch, decode (recognize the opcode), and execute it, advancing the program counter appropriately.

**Acceptance Scenarios**:

1. **Given** a CPU with initialized state and memory containing a simple program, **When** the execute loop is called once, **Then** the program counter advances correctly
2. **Given** a CPU with a placeholder instruction decoder, **When** the execute loop encounters an opcode, **Then** it correctly identifies the opcode (even if not implemented yet)
3. **Given** a CPU executing instructions, **When** tracking cycle count, **Then** the cycle counter increments appropriately for each instruction
4. **Given** a CPU with unimplemented opcodes, **When** the execute loop encounters one, **Then** it signals an error or returns a clear "not implemented" status

---

### User Story 4 - Opcode Metadata Table (Priority: P4)

As a developer, I need a table-driven opcode metadata structure that maps all 256 opcodes to their mnemonic, addressing mode, cycle cost, and size so that future instruction implementations can reference this single source of truth.

**Why this priority**: This implements the constitution's Table-Driven Design principle and prevents decode logic duplication. It's foundational but can be built incrementally as instructions are implemented.

**Independent Test**: Can be fully tested by verifying the table contains entries for all 256 opcodes (even if marked as "illegal" or "unimplemented"), and that the decoder can look up metadata for any given opcode.

**Acceptance Scenarios**:

1. **Given** the opcode metadata table, **When** a developer looks up any opcode (0x00-0xFF), **Then** an entry exists with mnemonic, addressing mode, base cycle cost, and instruction size
2. **Given** the opcode table, **When** multiple opcodes share an addressing mode, **Then** the addressing mode is defined once and referenced by all relevant opcodes
3. **Given** the decoder, **When** it needs to determine instruction length, **Then** it references the table rather than hard-coding sizes
4. **Given** the cycle counter, **When** an instruction executes, **Then** it increments by the base cycle cost from the table (plus any penalties)

---

### Edge Cases

- What happens when the CPU is instantiated without a memory bus? (Should this be a compile-time requirement via generics, or a runtime requirement?)
- What happens when an unimplemented instruction is encountered during execution? (Should it panic, return an error, or halt the CPU?)
- What happens if the opcode table is queried with an illegal opcode? (Should return metadata indicating "illegal" status)
- What happens when the program counter wraps around past 0xFFFF? (Should wrap to 0x0000 per 6502 behavior)
- What happens when the stack pointer underflows or overflows? (Should wrap per 6502 behavior, document this as expected)

## Requirements _(mandatory)_

### Functional Requirements

- **FR-001**: System MUST provide a Rust project structure that compiles successfully with no implementation code
- **FR-002**: System MUST define a CPU state structure containing all 6502 registers (A, X, Y), status flags (N, V, B, D, I, Z, C), program counter (PC), stack pointer (SP), and cycle counter
- **FR-003**: System MUST initialize all CPU registers to documented 6502 reset values (PC from reset vector, SP to 0xFD, status flags to 0x24)
- **FR-004**: System MUST define a memory bus trait with read and write operations that accept 16-bit addresses and handle 8-bit data
- **FR-005**: System MUST provide at least one simple memory bus implementation (flat 64KB RAM) for testing purposes
- **FR-006**: System MUST define a fetch-decode-execute method that can advance the CPU by one instruction
- **FR-007**: System MUST provide an opcode metadata table structure covering all 256 possible opcodes
- **FR-008**: System MUST mark unimplemented instructions clearly in the opcode table (e.g., with a status flag or enum variant)
- **FR-009**: System MUST define addressing mode types (Implied, Immediate, Absolute, ZeroPage, etc.) as a reusable enumeration or type
- **FR-010**: System MUST track cycle count accurately, incrementing for each instruction executed
- **FR-011**: System MUST provide a test suite structure with at least one passing test verifying CPU instantiation
- **FR-012**: System MUST support running the CPU for a specific number of cycles (for frame-based execution models)
- **FR-013**: System MUST expose CPU state (registers, flags, PC, SP) in a read-only manner for external inspection
- **FR-014**: System MUST compile to WebAssembly without errors (verify WASM compatibility)

### Key Entities

- **CPU State**: Represents the complete internal state of the 6502 processor, including accumulator (A), index registers (X, Y), program counter (PC), stack pointer (SP), status register (P with N, V, B, D, I, Z, C flags), and cycle counter
- **Memory Bus**: Abstract interface for reading and writing memory, decoupling the CPU from specific memory implementations
- **Opcode Metadata**: Static data structure mapping each of the 256 opcodes to its mnemonic name, addressing mode, base cycle cost, and instruction size
- **Addressing Mode**: Enumeration of the 6502's addressing modes (Implied, Accumulator, Immediate, Absolute, ZeroPage, ZeroPageX, ZeroPageY, AbsoluteX, AbsoluteY, Indirect, IndirectX, IndirectY, Relative)
- **Instruction**: Represents a decoded instruction ready for execution, containing opcode, addressing mode, operand bytes, and cycle cost

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: Project compiles successfully with zero errors and zero warnings on stable Rust
- **SC-002**: Project compiles successfully to WebAssembly target (wasm32-unknown-unknown) with zero errors
- **SC-003**: Test suite runs and all structural tests pass (100% pass rate for implemented tests)
- **SC-004**: CPU can be instantiated and all registers can be inspected with documented initial values
- **SC-005**: A simple program (even just a single NOP instruction at 0x0000) can be loaded into test memory and the fetch-decode-execute loop can be called without errors
- **SC-006**: Opcode metadata table is queryable for all 256 opcodes and returns consistent data structure for each
- **SC-007**: Cycle counter correctly increments from 0 when CPU begins execution
- **SC-008**: Developer can create a new memory bus implementation by implementing the trait in under 30 minutes (measured by implementation of a simple mirrored memory region)
- **SC-009**: Documentation exists covering how to instantiate the CPU, attach a memory bus, and execute instructions
- **SC-010**: Code coverage from structural tests reaches at least 80% of defined structures and initialization code

## Assumptions

- Rust stable toolchain is available (edition 2021 or later)
- WebAssembly toolchain (wasm32 target) is available for validation
- Initial implementation focuses on NMOS 6502 behavior (not 65C02 or other variants)
- Undocumented/illegal opcodes are marked as such but not implemented in this phase
- Reset vector behavior will be simulated (PC loaded from 0xFFFC/0xFFFD on initialization)
- Test framework will be standard Rust `#[test]` with cargo test
- Documentation will be standard Rust doc comments (rustdoc)
- No external dependencies required for core CPU module (memory bus implementations may use std collections)

## Out of Scope

This feature explicitly does NOT include:

- Implementation of any specific 6502 instructions (LDA, STA, JMP, etc.) - these will be separate features
- Interrupt handling logic (IRQ, NMI, BRK execution paths) - foundational structures only
- Debugging facilities (breakpoints, step execution, register inspection UI) - basic state inspection only
- Performance optimization or micro-benchmarking
- Comprehensive documentation beyond basic API docs
- Integration with any specific fantasy console or emulator project
- Audio or graphics subsystems
- Assembler or disassembler tools

## Reference

- 6502 reference documentation is stored in docs/6502-reference. In particular,
  you may want to review:
  - The full instruction set (names and categorization) in Instructions.md
  - Description of all the registers: Registers.md
  - Description of addressing modes in Addressing Modes.md
  - Others as required
