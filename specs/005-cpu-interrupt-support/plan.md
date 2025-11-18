# Implementation Plan: CPU Interrupt Support

**Branch**: `005-cpu-interrupt-support` | **Date**: 2025-11-18 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/005-cpu-interrupt-support/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Implement hardware-accurate interrupt support for the 6502 CPU emulator using a level-sensitive IRQ line. Devices expose memory-mapped status/control registers through the MemoryBus trait, enabling the CPU to detect and service interrupts with cycle-accurate timing (7 cycles). The ISR explicitly acknowledges interrupts by reading/writing device registers, matching real 6502 hardware behavior.

**Key Design Decisions** (from clarifications):
- Level-sensitive IRQ line (no queueing) matching real hardware
- Memory-mapped device registers via MemoryBus trait
- Devices declare address ranges at construction; system validates no overlap
- 7-cycle interrupt processing sequence per 6502 specification
- ISR polling and explicit acknowledgment (no automatic notification)

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**: None (zero external dependencies for core library - `no_std` compatible)
**Storage**: N/A (in-memory CPU and device state only)
**Testing**: cargo test (unit tests in src/, integration tests in tests/)
**Target Platform**: Native + WebAssembly (via wasm32-unknown-unknown target)
**Project Type**: Single library crate with examples
**Performance Goals**: Cycle-accurate emulation, zero overhead when no interrupts pending
**Constraints**: No OS dependencies, WASM-compatible, deterministic execution, no panics in hot paths
**Scale/Scope**: Core CPU interrupt mechanism only (no specific device implementations beyond test fixtures)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Principle I: Modularity & Separation of Concerns
✅ **PASS** - Interrupt mechanism integrates through existing CPU/MemoryBus abstraction. No hardware assumptions in CPU core. Devices implement MemoryBus trait to expose registers.

**Verification**:
- CPU struct contains only IRQ line state (boolean flag)
- All device interaction through MemoryBus reads/writes
- No direct device references or callbacks in CPU implementation

### Principle II: WebAssembly Portability
✅ **PASS** - Pure Rust, no OS dependencies, deterministic interrupt checking at instruction boundaries.

**Verification**:
- No threading, async, or syscalls
- Interrupt state is simple boolean (IRQ line active/inactive)
- All timing based on deterministic cycle counter

### Principle III: Cycle Accuracy
✅ **PASS** - 7-cycle interrupt sequence matches real 6502 hardware. IRQ checked at instruction boundaries.

**Verification**:
- FR-010 specifies exactly 7 cycles for interrupt processing
- Interrupt check occurs after each instruction completes
- No sub-instruction interrupt handling

### Principle IV: Clarity & Hackability
✅ **PASS** - Level-sensitive IRQ model is simpler than queued interrupts. Clear separation between CPU interrupt logic and device acknowledgment.

**Verification**:
- Single boolean flag for IRQ line state
- No complex state machines or event queues
- ISR acknowledgment is explicit memory read/write (visible in code)

### Principle V: Table-Driven Design
✅ **PASS** - No changes to opcode table. Interrupt checking is orthogonal to instruction decode.

**Verification**:
- Interrupt check added after instruction execution loop
- BRK instruction remains in opcode table
- No duplication of interrupt logic across opcodes

**Overall Status**: ✅ ALL GATES PASS - No violations. Design aligns with all constitutional principles.

## Project Structure

### Documentation (this feature)

```text
specs/005-cpu-interrupt-support/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   └── cpu-irq-api.md  # CPU IRQ line interface
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── cpu.rs              # CPU struct: add IRQ line state, check_interrupts() method
├── memory.rs           # MemoryBus trait (unchanged - devices already implement this)
├── lib.rs              # Public API: expose interrupt types/traits if needed
└── interrupts.rs       # NEW: Interrupt controller managing IRQ line state

tests/
├── integration/
│   └── test_interrupts.rs  # NEW: Integration tests for interrupt scenarios
└── unit/                    # Existing unit tests (may add interrupt-specific tests)

examples/
└── interrupt_device.rs  # NEW: Example device with memory-mapped registers
```

**Structure Decision**: Single project structure. The interrupt feature is a core CPU capability, not a separate subsystem. All interrupt logic lives in the main `src/` directory alongside existing CPU implementation. Memory-mapped device integration happens through the existing MemoryBus trait with no new abstractions needed.

## Complexity Tracking

No constitution violations. Complexity tracking not applicable.

## Phase 0: Research & Unknowns

**Status**: ✅ Complete (see `research.md`)

**Unknowns resolved during clarification**:
1. ✅ Interrupt mechanism architecture (level-sensitive vs queued) → Level-sensitive IRQ line
2. ✅ Device notification model (automatic vs explicit) → Explicit ISR acknowledgment
3. ✅ Device register exposure (API calls vs memory-mapped) → Memory-mapped via MemoryBus
4. ✅ Address allocation strategy (fixed vs dynamic vs device-specified) → Device specifies at construction
5. ✅ Interrupt cycle cost → 7 cycles per 6502 spec

No remaining unknowns require research agent dispatch.

## Phase 1: Design & Contracts

**Status**: ✅ Complete (see artifacts below)

**Artifacts**:
- `data-model.md` - IRQ line state, device interrupt state, interrupt sequence
- `contracts/cpu-irq-api.md` - CPU interrupt checking interface
- `quickstart.md` - How to implement an interrupt-capable device

## Phase 2: Task Generation

**Command**: Run `/speckit.tasks` to generate implementation tasks from this plan.

**Preview** (high-level breakdown):
1. Add IRQ line state to CPU struct
2. Implement interrupt checking at instruction boundaries
3. Implement 7-cycle interrupt sequence (push PC/status, read vector, jump)
4. Respect I flag (interrupt disable)
5. Add integration tests for interrupt scenarios
6. Create example device with memory-mapped registers
7. Update documentation

---

## Agent Context Update

Run after Phase 1 completion:
```bash
.specify/scripts/bash/update-agent-context.sh claude
```

This updates `.claude/context/active-technologies.md` with interrupt-specific technical context.
