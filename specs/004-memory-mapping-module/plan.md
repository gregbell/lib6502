# Implementation Plan: Memory Mapping Module with UART Device Support

**Branch**: `004-memory-mapping-module` | **Date**: 2025-11-17 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/004-memory-mapping-module/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Create a flexible memory mapping architecture that allows multiple hardware devices (RAM, ROM, UART, future I/O) to be attached to the 6502 memory bus. Implement a 6551 ACIA UART device as the first concrete I/O device with four memory-mapped registers (data, status, command, control). Provide browser-based terminal integration via callback interface compatible with xterm.js.

**Technical Approach**: Extend the existing `MemoryBus` trait with a device registration system that routes read/write operations based on address ranges. Implement devices as independent modules that respond to memory operations within their configured address space. UART device maintains internal state (registers, buffers) and exposes callback interface for external terminal integration.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**: None (zero external dependencies for core library - `no_std` compatible)
**Storage**: N/A (in-memory state only, no persistence)
**Testing**: `cargo test` (unit tests in source files, integration tests in `tests/` directory)
**Target Platform**: WebAssembly (WASM) + native (Linux, macOS, Windows)
**Project Type**: Single project (library)
**Performance Goals**: Handle 100+ bytes/second serial throughput, <1ms memory access latency
**Constraints**: No OS dependencies (WASM compatible), no panics in memory operations, deterministic behavior
**Scale/Scope**: 3-5 device types initially (RAM, ROM, UART, future expansion to graphics/sound)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Modularity & Separation of Concerns

- ✅ **Pass**: Memory mapping system extends existing `MemoryBus` trait without modifying CPU core
- ✅ **Pass**: Each device (RAM, ROM, UART) is implemented as independent module
- ✅ **Pass**: Device registration uses trait-based abstraction (no concrete device types in mapper)
- ✅ **Pass**: UART terminal bridge uses callback interface (no direct browser dependencies)

### II. WebAssembly Portability

- ✅ **Pass**: No external dependencies added (maintains zero-dependency requirement)
- ✅ **Pass**: UART callback interface compatible with WASM (no OS I/O, threading, or async)
- ✅ **Pass**: All device implementations use deterministic computation only
- ✅ **Pass**: Terminal integration handled via pure data exchange (no `std::net`, `std::fs`)

### III. Cycle Accuracy

- ✅ **Pass**: Memory-mapped I/O access uses same cycle model as existing memory operations
- ⚠️ **Consideration**: UART register access may need additional cycle costs (research needed)
- ✅ **Pass**: No impact on existing instruction timing

### IV. Clarity & Hackability

- ✅ **Pass**: Device trait provides clear extension point for new hardware
- ✅ **Pass**: Examples demonstrate ROM/RAM split, UART echo program, terminal integration
- ✅ **Pass**: Documentation explains memory mapping patterns and device lifecycle
- ✅ **Pass**: Code prioritizes readability over optimization

### V. Table-Driven Design

- ✅ **Pass**: Address mapping uses lookup structure (not scattered conditionals)
- ✅ **Pass**: UART register dispatch uses table/match (status/data/command/control)
- ✅ **Pass**: No duplicated routing logic

**Overall Assessment**: ✅ All gates pass. No complexity violations requiring justification.

## Project Structure

### Documentation (this feature)

```text
specs/004-memory-mapping-module/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   └── device_trait.md  # Device trait interface contract
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── memory.rs            # Existing: MemoryBus trait, FlatMemory
├── devices/             # NEW: Device implementations
│   ├── mod.rs           # Device trait, mapper implementation
│   ├── ram.rs           # Simple RAM device
│   ├── rom.rs           # Read-only memory device
│   └── uart.rs          # 6551 UART device implementation
├── cpu.rs               # Existing: CPU implementation (no changes)
├── lib.rs               # Existing: Public API (re-export new types)
└── ...                  # Other existing modules unchanged

tests/
├── memory_mapping_tests.rs  # NEW: Memory mapper integration tests
├── uart_tests.rs            # NEW: UART device tests
└── ...                      # Existing tests unchanged

examples/
├── memory_mapped_system.rs  # NEW: Example with RAM/ROM/UART
├── uart_echo.rs             # NEW: Simple echo program via UART
└── ...                      # Existing examples unchanged
```

**Structure Decision**: Extend existing single-project structure with new `src/devices/` module. All device implementations live under this module, maintaining flat hierarchy. Memory mapper implementation co-located with device trait in `devices/mod.rs`. No changes to CPU core or existing memory abstraction.

## Complexity Tracking

> **No violations detected - this section left empty per template instructions**

## Phase 0: Research & Discovery

**Status**: Complete ✓ (see [research.md](./research.md))

**Research Areas**:

1. **6551 UART Hardware Specification**
   - Register layout and behavior (data/status/command/control at base+0/1/2/3)
   - Status register bit flags (transmitter ready, receiver full, overflow, etc.)
   - Command register options (parity, echo, interrupt enables)
   - Control register settings (baud rate, word length, stop bits)

2. **Memory Mapping Patterns**
   - Address range overlap detection and priority handling
   - Unmapped address default behavior (return $FF, $00, or last value on bus)
   - Device registration order and lookup performance
   - Dynamic vs static configuration trade-offs

3. **Rust Trait Design for Devices**
   - Device trait interface (read/write with address + context)
   - Lifetime and ownership patterns (stateful devices, shared vs exclusive access)
   - Error handling patterns (no panics, silent failures vs logging)
   - Testing strategies (mock devices, state inspection)

4. **WASM Callback Interface**
   - Function pointer vs trait object for terminal callbacks
   - Data ownership across WASM boundary (byte ownership, buffer management)
   - Performance considerations (callback overhead, batching)
   - Browser integration patterns (xterm.js onData/onKey events)

**Key Decisions** (from research):
- Use `Vec<(Range<u16>, Box<dyn Device>)>` for device registration (simple, flexible)
- UART uses internal `VecDeque<u8>` for receive/transmit buffers (standard FIFO)
- Callback interface uses `Option<Box<dyn Fn(u8)>>` for terminal output (WASM-compatible)
- Unmapped reads return `0xFF` (common 6502 behavior, predictable for debugging)

## Phase 1: Architecture & Contracts

**Status**: Complete ✓ (see [data-model.md](./data-model.md), [contracts/](./contracts/))

### Core Abstractions

**Device Trait** (see [contracts/device_trait.md](./contracts/device_trait.md)):
```rust
pub trait Device {
    fn read(&self, offset: u16) -> u8;
    fn write(&mut self, offset: u16, value: u8);
    fn size(&self) -> u16;
}
```

**Memory Mapper**:
```rust
pub struct MappedMemory {
    devices: Vec<DeviceMapping>,
    unmapped_value: u8,
}

struct DeviceMapping {
    base_addr: u16,
    device: Box<dyn Device>,
}
```

**UART Device**:
```rust
pub struct Uart6551 {
    // Registers
    data_register: u8,
    status_register: u8,
    command_register: u8,
    control_register: u8,

    // Buffers
    rx_buffer: VecDeque<u8>,
    tx_buffer: VecDeque<u8>,

    // Callbacks
    on_transmit: Option<Box<dyn Fn(u8)>>,
}
```

### Integration Points

1. **CPU ↔ Memory Mapper**: CPU continues to use existing `MemoryBus` trait, mapper implements it
2. **Mapper ↔ Devices**: Mapper dispatches to devices via `Device` trait based on address
3. **UART ↔ Terminal**: UART invokes `on_transmit` callback when byte written to data register
4. **Terminal ↔ UART**: External code calls `uart.receive_byte(u8)` to inject input

### Data Model

See [data-model.md](./data-model.md) for complete entity definitions, state transitions, and validation rules.

### Quick Start

See [quickstart.md](./quickstart.md) for:
- Creating a memory-mapped system with RAM, ROM, and UART
- Running 6502 code that reads/writes UART registers
- Connecting browser terminal to UART callbacks
- Common patterns and troubleshooting

## Phase 2: Task Planning

**Status**: Not started (run `/speckit.tasks` to generate)

Task generation will produce:
- Dependency-ordered implementation tasks
- Test coverage requirements per task
- Acceptance criteria aligned with spec
- Complexity estimates and risk flags

See `tasks.md` (generated by `/speckit.tasks`) for complete task breakdown.

## Open Questions

*All research questions resolved in Phase 0. No blockers identified.*

**Deferred Decisions** (can be addressed during implementation):
- Buffer sizes for UART rx/tx (default to 256 bytes, make configurable later if needed)
- Priority handling for overlapping device mappings (disallow for now, add later if use case emerges)
- Interrupt support in UART (out of scope per spec OS-007, polling mode sufficient)

## Success Criteria Validation

Each success criterion from spec mapped to verification approach:

- **SC-001** (3+ devices): Verified by `memory_mapped_system.rs` example showing RAM/ROM/UART
- **SC-002** (100% data integrity): Verified by UART echo test (send 1000 bytes, verify all received)
- **SC-003** (<100ms transmit latency): Verified by browser integration test (measure callback to display)
- **SC-004** (<100ms receive latency): Verified by browser integration test (measure keystroke to available)
- **SC-005** (100 bytes/sec throughput): Verified by benchmark sending continuous stream
- **SC-006** (WASM compilation): Verified by `cargo build --target wasm32-unknown-unknown`
- **SC-007** (Browser compatibility): Verified by manual testing in Chrome/Firefox/Safari/Edge

## Risk Assessment

**Low Risk**:
- Device trait design (well-understood pattern, similar to existing `MemoryBus`)
- RAM/ROM device implementation (trivial wrappers over byte arrays)
- WASM compilation (existing project already WASM-compatible)

**Medium Risk**:
- UART register behavior accuracy (requires careful spec reading, validation against Ben Eater video)
- Buffer overflow handling (need clear behavior for full rx/tx buffers)
- Terminal integration testing (manual browser testing required, hard to automate)

**Mitigation**:
- Document UART behavior with references to 6551 datasheet and Ben Eater video timestamps
- Add explicit buffer size limits and overflow flag in status register
- Create comprehensive manual test plan for browser terminal integration

## Timeline Estimate

Based on task complexity and constitution alignment:

- **Phase 0 (Research)**: Complete ✓
- **Phase 1 (Design)**: Complete ✓
- **Phase 2 (Tasks)**: ~1-2 hours (run `/speckit.tasks`)
- **Implementation**: ~8-12 hours
  - Device trait + mapper: 2-3 hours
  - RAM/ROM devices: 1 hour
  - UART device: 4-5 hours
  - Examples + integration tests: 2-3 hours
- **Testing + Documentation**: ~3-4 hours
- **Browser integration**: ~2-3 hours

**Total**: ~15-20 hours for complete P1+P2+P3 implementation

## Next Steps

1. ✅ Complete Phase 0 research
2. ✅ Complete Phase 1 design and contracts
3. **→ Run `/speckit.tasks`** to generate dependency-ordered task list
4. Implement tasks in priority order (P1 → P2 → P3)
5. Verify success criteria after each priority level
6. Create PR when all tasks complete
