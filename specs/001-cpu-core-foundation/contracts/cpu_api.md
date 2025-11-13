# API Contract: CPU Public Interface

**Feature**: 001-cpu-core-foundation
**Date**: 2025-11-13
**Contract Type**: Rust Public API

This document defines the public API contract for the CPU module. All public types, methods, and traits must maintain backward compatibility once stabilized.

## Public Types

### `struct CPU<M: MemoryBus>`

**Purpose**: 6502 CPU state and execution context.

**Generic Parameters**:
- `M: MemoryBus` - Memory bus implementation type (must implement MemoryBus trait)

**Visibility**: `pub`

**Constructor**:

```rust
/// Creates a new CPU with the given memory bus.
///
/// The CPU is initialized to the 6502 power-on reset state:
/// - Program counter (PC) is loaded from the reset vector at addresses 0xFFFC/0xFFFD
/// - Stack pointer (SP) is set to 0xFD
/// - Status register has Interrupt Disable flag set (0x24)
/// - All other registers (A, X, Y) are zeroed
/// - Cycle counter is reset to 0
///
/// # Arguments
/// * `memory` - A MemoryBus implementation that provides the reset vector
///
/// # Examples
/// ```
/// let mem = FlatMemory::new();
/// let cpu = CPU::new(mem);
/// ```
pub fn new(memory: M) -> Self
```

**Public Methods**:

```rust
/// Executes one instruction and advances the CPU state.
///
/// Performs the fetch-decode-execute cycle:
/// 1. Fetch opcode byte at current PC
/// 2. Look up instruction metadata
/// 3. Execute instruction (updates registers, flags, PC)
/// 4. Increment cycle counter
///
/// Returns an error if the instruction is not yet implemented.
///
/// # Returns
/// - `Ok(())` if instruction executed successfully
/// - `Err(ExecutionError::UnimplementedOpcode(opcode))` if instruction not implemented
///
/// # Examples
/// ```
/// match cpu.step() {
///     Ok(()) => println!("Instruction executed"),
///     Err(ExecutionError::UnimplementedOpcode(op)) => {
///         eprintln!("Opcode 0x{:02X} not implemented", op);
///     }
/// }
/// ```
pub fn step(&mut self) -> Result<(), ExecutionError>
```

```rust
/// Runs the CPU for a specified number of cycles.
///
/// Executes instructions until the cycle budget is exhausted or an error occurs.
/// Returns the actual number of cycles consumed (may be slightly more than budget
/// due to instruction granularity).
///
/// This is useful for frame-locked execution models where the CPU must run for
/// an exact number of cycles per frame (e.g., 29780 cycles for 60Hz NTSC).
///
/// # Arguments
/// * `cycle_budget` - Maximum number of cycles to execute
///
/// # Returns
/// - `Ok(cycles_consumed)` if execution completed successfully
/// - `Err(ExecutionError)` if an instruction failed
///
/// # Examples
/// ```
/// // Run CPU for one NTSC frame (60Hz, ~1.79 MHz)
/// let cycles_per_frame = 29780;
/// match cpu.run_for_cycles(cycles_per_frame) {
///     Ok(actual_cycles) => println!("Executed {} cycles", actual_cycles),
///     Err(e) => eprintln!("Execution error: {:?}", e),
/// }
/// ```
pub fn run_for_cycles(&mut self, cycle_budget: u64) -> Result<u64, ExecutionError>
```

**Public Getters** (read-only state inspection):

```rust
/// Returns the accumulator register value.
pub fn a(&self) -> u8

/// Returns the X index register value.
pub fn x(&self) -> u8

/// Returns the Y index register value.
pub fn y(&self) -> u8

/// Returns the program counter value.
pub fn pc(&self) -> u16

/// Returns the stack pointer value.
pub fn sp(&self) -> u8

/// Returns the status register as a packed byte.
///
/// Bit layout (NV-BDIZC):
/// - Bit 7: N (Negative)
/// - Bit 6: V (Overflow)
/// - Bit 5: (unused, always 1)
/// - Bit 4: B (Break)
/// - Bit 3: D (Decimal)
/// - Bit 2: I (Interrupt Disable)
/// - Bit 1: Z (Zero)
/// - Bit 0: C (Carry)
pub fn status(&self) -> u8

/// Returns the total number of CPU cycles executed since initialization.
pub fn cycles(&self) -> u64
```

**Public Flag Getters** (individual status flags):

```rust
/// Returns true if the Negative flag is set.
pub fn flag_n(&self) -> bool

/// Returns true if the Overflow flag is set.
pub fn flag_v(&self) -> bool

/// Returns true if the Break flag is set.
pub fn flag_b(&self) -> bool

/// Returns true if the Decimal mode flag is set.
pub fn flag_d(&self) -> bool

/// Returns true if the Interrupt Disable flag is set.
pub fn flag_i(&self) -> bool

/// Returns true if the Zero flag is set.
pub fn flag_z(&self) -> bool

/// Returns true if the Carry flag is set.
pub fn flag_c(&self) -> bool
```

**Invariants**:
- After `new()`, CPU state matches 6502 reset values
- `step()` always increments `cycles()` by at least the base instruction cycle cost
- `pc()` wraps at 0xFFFF to 0x0000
- `sp()` is always in range 0x00-0xFF (stack address is 0x0100 + sp)
- `status()` bit 5 is always 1 (unused bit convention)

---

## Public Traits

### `trait MemoryBus`

**Purpose**: Abstract memory interface for CPU to read/write bytes.

**Visibility**: `pub`

**Required Methods**:

```rust
/// Reads a byte from the specified 16-bit address.
///
/// This method must never panic. If the address is unmapped or invalid,
/// implementations may return garbage data (matching 6502 hardware behavior).
///
/// # Arguments
/// * `addr` - 16-bit memory address (0x0000-0xFFFF)
///
/// # Returns
/// The byte value at the specified address
///
/// # Examples
/// ```
/// let value = memory.read(0x1234);
/// ```
fn read(&self, addr: u16) -> u8;
```

```rust
/// Writes a byte to the specified 16-bit address.
///
/// This method must never panic. If the address is read-only or unmapped,
/// implementations may ignore the write (matching 6502 hardware behavior).
///
/// # Arguments
/// * `addr` - 16-bit memory address (0x0000-0xFFFF)
/// * `value` - Byte value to write
///
/// # Examples
/// ```
/// memory.write(0x1234, 0xFF);
/// ```
fn write(&mut self, addr: u16, value: u8);
```

**Design Rationale**:
- `read(&self)` uses immutable reference to allow shared reads
- `write(&mut self)` uses mutable reference to make side effects explicit
- No error types - 6502 hardware has no bus error mechanism
- Simple signatures ensure WASM compatibility

**Implementations Provided**:
- `struct FlatMemory` - Simple 64KB array (all addresses mapped to RAM)

---

## Public Enums

### `enum ExecutionError`

**Purpose**: Errors that can occur during CPU execution.

**Visibility**: `pub`

**Derives**: `Debug, Clone, PartialEq, Eq`

**Variants**:

```rust
/// Instruction opcode has not been implemented yet.
///
/// Contains the opcode byte value for debugging purposes.
UnimplementedOpcode(u8)
```

**Display Implementation**:

```rust
impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ExecutionError::UnimplementedOpcode(opcode) => {
                write!(f, "Opcode 0x{:02X} is not implemented", opcode)
            }
        }
    }
}
```

**Error Trait Implementation**:

```rust
impl std::error::Error for ExecutionError {}
```

---

### `enum AddressingMode`

**Purpose**: 6502 addressing mode enumeration.

**Visibility**: `pub`

**Derives**: `Debug, Clone, Copy, PartialEq, Eq`

**Variants**:

```rust
/// No operand, operation implied by instruction.
Implicit,

/// Operates directly on the accumulator register.
Accumulator,

/// 8-bit constant operand in instruction.
Immediate,

/// 8-bit address in zero page (0x00-0xFF).
ZeroPage,

/// Zero page address indexed by X register.
ZeroPageX,

/// Zero page address indexed by Y register.
ZeroPageY,

/// Signed 8-bit offset for branch instructions.
Relative,

/// Full 16-bit address.
Absolute,

/// 16-bit address indexed by X register.
AbsoluteX,

/// 16-bit address indexed by Y register.
AbsoluteY,

/// Indirect jump through 16-bit pointer.
Indirect,

/// Indexed indirect: (ZP + X) dereference.
IndirectX,

/// Indirect indexed: ZP dereference then + Y.
IndirectY,
```

**Usage**:
- Public primarily for opcode table introspection
- Future use in custom instruction implementations

---

## Public Constants

### `const OPCODE_TABLE: [OpcodeMetadata; 256]`

**Purpose**: Static opcode metadata table for all 256 possible opcodes.

**Visibility**: `pub`

**Element Type**: `OpcodeMetadata`

**Usage**:

```rust
// Look up metadata for LDA immediate (opcode 0xA9)
let metadata = &OPCODE_TABLE[0xA9];
assert_eq!(metadata.mnemonic, "LDA");
assert_eq!(metadata.addressing_mode, AddressingMode::Immediate);
assert_eq!(metadata.base_cycles, 2);
assert_eq!(metadata.size_bytes, 2);
```

---

### `struct OpcodeMetadata`

**Purpose**: Metadata for a single opcode.

**Visibility**: `pub`

**Derives**: `Debug, Clone, Copy, PartialEq, Eq`

**Fields**:

```rust
/// Instruction mnemonic (e.g., "LDA", "STA", "???" for illegal opcodes).
pub mnemonic: &'static str,

/// Addressing mode for this instruction.
pub addressing_mode: AddressingMode,

/// Base cycle cost (before page crossing penalties).
pub base_cycles: u8,

/// Total instruction size in bytes (opcode + operands).
pub size_bytes: u8,

/// Whether this instruction is currently implemented.
pub implemented: bool,
```

---

## Versioning & Stability

**Semantic Versioning**:
- **MAJOR**: Breaking changes to public API (e.g., changing method signatures, removing public types)
- **MINOR**: New public methods, new trait implementations, new opcodes implemented
- **PATCH**: Bug fixes, documentation updates, internal refactoring

**Current Version**: `0.1.0` (pre-1.0, API not yet stable)

**Stability Guarantees**:
- Public API may change before 1.0.0 release
- After 1.0.0, breaking changes require major version bump
- Internal implementation (private fields, helper functions) may change at any time

**Deprecation Policy**:
- Deprecated APIs marked with `#[deprecated]` attribute
- Deprecated APIs maintained for at least one minor version before removal

---

## Examples

### Basic CPU Initialization and Execution

```rust
use cpu6502::{CPU, FlatMemory, ExecutionError};

fn main() -> Result<(), ExecutionError> {
    // Create 64KB flat memory
    let mut memory = FlatMemory::new();

    // Set reset vector to 0x8000
    memory.write(0xFFFC, 0x00);
    memory.write(0xFFFD, 0x80);

    // Load a simple program at 0x8000 (placeholder - no instructions implemented yet)
    memory.write(0x8000, 0xEA); // NOP (if it were implemented)

    // Initialize CPU
    let mut cpu = CPU::new(memory);

    // Verify initial state
    assert_eq!(cpu.pc(), 0x8000);
    assert_eq!(cpu.sp(), 0xFD);
    assert_eq!(cpu.flag_i(), true);
    assert_eq!(cpu.cycles(), 0);

    // Attempt to execute one instruction
    match cpu.step() {
        Ok(()) => println!("Instruction executed successfully"),
        Err(ExecutionError::UnimplementedOpcode(opcode)) => {
            println!("Opcode 0x{:02X} not implemented yet", opcode);
        }
    }

    Ok(())
}
```

### Frame-Locked Execution

```rust
use cpu6502::{CPU, FlatMemory};

fn emulate_ntsc_frame(cpu: &mut CPU<FlatMemory>) {
    const CYCLES_PER_FRAME: u64 = 29780; // ~1.79 MHz / 60 Hz

    match cpu.run_for_cycles(CYCLES_PER_FRAME) {
        Ok(actual_cycles) => {
            println!("Frame executed {} cycles", actual_cycles);
        }
        Err(e) => {
            eprintln!("Execution halted: {}", e);
        }
    }
}
```

### Custom Memory Bus Implementation

```rust
use cpu6502::MemoryBus;

/// Memory bus with mirrored zero page (0x0000-0x00FF mirrored to 0x0100-0x01FF).
struct MirroredMemory {
    ram: [u8; 65536],
}

impl MirroredMemory {
    pub fn new() -> Self {
        Self { ram: [0; 65536] }
    }
}

impl MemoryBus for MirroredMemory {
    fn read(&self, addr: u16) -> u8 {
        // Mirror zero page to 0x0100-0x01FF
        let effective_addr = if addr >= 0x0100 && addr < 0x0200 {
            addr & 0x00FF // Mirror to zero page
        } else {
            addr
        };
        self.ram[effective_addr as usize]
    }

    fn write(&mut self, addr: u16, value: u8) {
        let effective_addr = if addr >= 0x0100 && addr < 0x0200 {
            addr & 0x00FF
        } else {
            addr
        };
        self.ram[effective_addr as usize] = value;

        // Also write to mirrored location if writing to zero page
        if effective_addr < 0x0100 {
            self.ram[(0x0100 + effective_addr) as usize] = value;
        }
    }
}
```

---

## Testing Requirements

All public API contracts must have corresponding tests:

- ✅ CPU initialization sets correct reset values
- ✅ CPU getters return correct register values
- ✅ `step()` returns UnimplementedOpcode for all opcodes (in this feature)
- ✅ `run_for_cycles()` executes approximate cycle budget
- ✅ MemoryBus trait can be implemented for custom types
- ✅ FlatMemory read/write round-trip works
- ✅ Status register packing/unpacking maintains bit layout
- ✅ OPCODE_TABLE has 256 entries with valid metadata

---

## References

- Data Model: specs/001-cpu-core-foundation/data-model.md
- Feature Spec: specs/001-cpu-core-foundation/spec.md
- Constitution: .specify/memory/constitution.md (principle IV: Clarity & Hackability)
