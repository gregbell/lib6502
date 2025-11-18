# Tasks: CPU Interrupt Support

**Input**: Design documents from `/specs/005-cpu-interrupt-support/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/cpu-irq-api.md

**Tests**: Tests are not explicitly requested in the feature specification, so test tasks are not included. Tests can be added during implementation if desired.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

## Path Conventions

Single project structure (Rust library crate):
- Source: `src/` at repository root
- Tests: `tests/` at repository root
- Examples: `examples/` at repository root

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Minimal project structure preparation for interrupt support

- [x] T001 Verify existing project structure matches plan.md (src/, tests/, examples/)
- [x] T002 Create new module file src/devices/interrupts.rs for InterruptDevice trait

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core interrupt infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T003 [P] Add InterruptDevice trait to src/devices/interrupts.rs with has_interrupt() method
- [x] T004 [P] Add irq_pending field (bool) to CPU struct in src/cpu.rs
- [x] T005 Add irq_active() method to MemoryBus trait in src/memory.rs that returns bool
- [x] T006 Implement irq_active() in MappedMemory (src/devices/mod.rs) to check all devices' has_interrupt()
- [x] T007 Export InterruptDevice trait from src/lib.rs public API
- [x] T008 Add documentation comments to InterruptDevice trait explaining hardware-accurate behavior

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - External Device Signals CPU (Priority: P1) 🎯 MVP

**Goal**: Enable a single device to signal interrupts to the CPU, with CPU executing the interrupt handler at the next instruction boundary. This delivers the fundamental interrupt capability matching real 6502 hardware behavior.

**Independent Test**: Create a simple timer device that signals an interrupt after N cycles, verify CPU executes interrupt handler, and ISR can read device status and clear interrupt.

### Implementation for User Story 1

#### Core CPU Interrupt Logic

- [x] T009 [US1] Add interrupt checking to CPU::step() in src/cpu.rs after instruction execution
- [x] T010 [US1] Implement check_irq_line() helper in src/cpu.rs that calls memory.irq_active() and updates irq_pending
- [x] T011 [US1] Implement should_service_interrupt() helper in src/cpu.rs (returns irq_pending && !flag_i)
- [x] T012 [US1] Implement service_interrupt() method in src/cpu.rs with 7-cycle IRQ sequence
- [x] T013 [US1] In service_interrupt(): Push PC high byte to stack (1 cycle) in src/cpu.rs
- [x] T014 [US1] In service_interrupt(): Push PC low byte to stack (1 cycle) in src/cpu.rs
- [x] T015 [US1] In service_interrupt(): Push status register to stack (1 cycle) in src/cpu.rs
- [x] T016 [US1] In service_interrupt(): Set I flag to prevent nested interrupts in src/cpu.rs
- [x] T017 [US1] In service_interrupt(): Read IRQ vector low byte from 0xFFFE (1 cycle) in src/cpu.rs
- [x] T018 [US1] In service_interrupt(): Read IRQ vector high byte from 0xFFFF (1 cycle) in src/cpu.rs
- [x] T019 [US1] In service_interrupt(): Set PC to vector address (2 cycles) in src/cpu.rs
- [x] T020 [US1] In service_interrupt(): Add exactly 7 to cycle counter in src/cpu.rs
- [x] T021 [US1] Add documentation comments to service_interrupt() explaining cycle breakdown

#### Example Timer Device

- [x] T022 [P] [US1] Create TimerDevice struct in examples/interrupt_device.rs with interrupt_pending field
- [x] T023 [P] [US1] Implement Device trait for TimerDevice in examples/interrupt_device.rs (size() returns 4)
- [x] T024 [P] [US1] Implement InterruptDevice trait for TimerDevice in examples/interrupt_device.rs
- [x] T025 [P] [US1] Implement MemoryBus trait for TimerDevice with 4 memory-mapped registers in examples/interrupt_device.rs
- [x] T026 [US1] Add tick() method to TimerDevice that sets interrupt_pending when counter expires in examples/interrupt_device.rs
- [x] T027 [US1] Implement STATUS register (offset 0) in TimerDevice showing interrupt_pending in bit 7 in examples/interrupt_device.rs
- [x] T028 [US1] Implement CONTROL register (offset 1) in TimerDevice that clears interrupt on write in examples/interrupt_device.rs
- [x] T029 [US1] Implement COUNTER_LO register (offset 2) in TimerDevice for read-only counter value in examples/interrupt_device.rs
- [x] T030 [US1] Implement COUNTER_HI register (offset 3) in TimerDevice for read-only counter value in examples/interrupt_device.rs

#### Integration & Validation

- [x] T031 [US1] Create integration test in tests/integration/test_interrupts.rs verifying single device interrupt
- [x] T032 [US1] Add test case: device asserts interrupt, CPU services it at next instruction boundary in tests/integration/test_interrupts.rs
- [x] T033 [US1] Add test case: I flag set, interrupt not serviced until I flag cleared in tests/integration/test_interrupts.rs
- [x] T034 [US1] Add test case: ISR reads device status register, device clears interrupt in tests/integration/test_interrupts.rs
- [x] T035 [US1] Add test case: verify exactly 7 cycles consumed by interrupt sequence in tests/integration/test_interrupts.rs
- [x] T036 [US1] Create example program in examples/interrupt_device.rs demonstrating timer interrupt workflow
- [x] T037 [US1] Add ISR example code (6502 assembly comments) showing how to poll and acknowledge device in examples/interrupt_device.rs

**Checkpoint**: At this point, User Story 1 should be fully functional - single device can signal interrupts, CPU services them with cycle-accurate timing, ISR can poll and acknowledge device.

---

## Phase 4: User Story 2 - Multiple Device Interrupt Coordination (Priority: P2)

**Goal**: Enable multiple devices to signal interrupts independently, with IRQ line staying active until all devices clear their requests. ISR can poll multiple devices to identify interrupt sources.

**Independent Test**: Connect multiple test devices, have them signal interrupts simultaneously or in sequence, verify CPU handles all interrupts and IRQ line behavior is correct.

### Implementation for User Story 2

#### Multi-Device IRQ Logic

- [ ] T038 [US2] Verify MappedMemory::irq_active() correctly ORs all device interrupt flags in src/devices/mod.rs
- [ ] T039 [US2] Add documentation to irq_active() explaining level-sensitive IRQ line semantics in src/devices/mod.rs
- [ ] T040 [US2] Ensure CPU re-checks IRQ line after RTI instruction (verify in src/cpu.rs)

#### Example Multi-Device System

- [ ] T041 [P] [US2] Create UartDevice struct in examples/interrupt_device.rs with interrupt_pending field
- [ ] T042 [P] [US2] Implement Device trait for UartDevice in examples/interrupt_device.rs (size() returns 4)
- [ ] T043 [P] [US2] Implement InterruptDevice trait for UartDevice in examples/interrupt_device.rs
- [ ] T044 [P] [US2] Implement MemoryBus trait for UartDevice with memory-mapped registers in examples/interrupt_device.rs
- [ ] T045 [US2] Add receive_byte() method to UartDevice that sets interrupt_pending in examples/interrupt_device.rs
- [ ] T046 [US2] Implement STATUS register for UartDevice showing interrupt_pending in bit 7 in examples/interrupt_device.rs
- [ ] T047 [US2] Implement DATA register for UartDevice that clears interrupt when read in examples/interrupt_device.rs

#### Integration & Validation

- [ ] T048 [US2] Add test case: two devices assert interrupts simultaneously, IRQ line active until both clear in tests/integration/test_interrupts.rs
- [ ] T049 [US2] Add test case: device asserts interrupt during ISR, CPU re-enters ISR after RTI in tests/integration/test_interrupts.rs
- [ ] T050 [US2] Add test case: ISR polls multiple devices, identifies and handles all interrupt sources in tests/integration/test_interrupts.rs
- [ ] T051 [US2] Add test case: verify IRQ line inactive only when all devices cleared in tests/integration/test_interrupts.rs
- [ ] T052 [US2] Create multi-device example in examples/interrupt_device.rs with timer + UART
- [ ] T053 [US2] Add ISR polling example showing how to check multiple devices in priority order in examples/interrupt_device.rs

**Checkpoint**: At this point, User Story 2 should be fully functional - multiple devices can signal interrupts, IRQ line correctly represents OR of all device states, ISR can poll and handle multiple sources.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, code cleanup, and final validation

- [x] T054 [P] Update CLAUDE.md with interrupt support implementation notes and example usage
- [x] T055 [P] Add module-level documentation to src/devices/interrupts.rs explaining interrupt model
- [x] T056 [P] Add README or comments to examples/interrupt_device.rs explaining example structure
- [x] T057 Run cargo clippy and fix any warnings in interrupt-related code
- [x] T058 Run cargo fmt on all modified files
- [ ] T059 Run full test suite (cargo test --include-ignored) to verify no regressions
- [ ] T060 [P] Review all error handling in interrupt code for potential panics
- [ ] T061 [P] Add inline comments explaining 7-cycle breakdown in service_interrupt()
- [ ] T062 Verify WASM compatibility (no std features used in interrupt code)

**Final Checkpoint**: Feature complete - all user stories functional, documented, tested, and ready for review.

---

## Dependencies & Execution Strategy

### User Story Dependencies

```
Phase 1: Setup (T001-T002)
    ↓
Phase 2: Foundational (T003-T008) ← BLOCKING
    ↓
Phase 3: US1 (T009-T037) ← MVP - Can implement first
    ↓
Phase 4: US2 (T038-T053) ← Builds on US1
    ↓
Phase 5: Polish (T054-T062)
```

**Independent Implementation**: User Story 1 and User Story 2 could theoretically be developed independently after Phase 2, but US2 builds naturally on US1's single-device foundation.

### Parallel Execution Opportunities

#### Within Phase 2 (Foundational)
```bash
# Can run in parallel (different files):
T003 (src/devices/interrupts.rs) || T004 (src/cpu.rs)
T007 (src/lib.rs) || T008 (documentation)
```

#### Within Phase 3 (User Story 1)
```bash
# Example device can be built in parallel with CPU interrupt logic:
T009-T021 (CPU interrupt logic in src/cpu.rs) || T022-T030 (TimerDevice in examples/)

# Documentation and examples can be parallelized:
T036 (example program) || T037 (ISR example code)
```

#### Within Phase 4 (User Story 2)
```bash
# UartDevice implementation fully parallel with UART logic:
T038-T040 (IRQ logic verification) || T041-T047 (UartDevice in examples/)

# Example creation parallel with testing:
T052 (multi-device example) || T053 (ISR polling example)
```

#### Within Phase 5 (Polish)
```bash
# Most polish tasks are independent:
T054 (CLAUDE.md) || T055 (module docs) || T056 (example README) || T060 (error handling) || T061 (comments)
```

### MVP Scope Recommendation

**Minimum Viable Product**: User Story 1 only (Tasks T001-T037)

This delivers:
- ✅ Single device can signal interrupts
- ✅ CPU services interrupts with cycle-accurate timing
- ✅ ISR can poll and acknowledge device
- ✅ Fundamental interrupt capability working
- ✅ Example timer device demonstrating pattern

**Value**: Enables basic interrupt-driven I/O, validates core architecture, provides template for additional devices.

**Next Increment**: Add User Story 2 (Tasks T038-T053) for multi-device support.

---

## Task Counts

- **Total Tasks**: 62
- **Setup Phase**: 2 tasks
- **Foundational Phase**: 6 tasks (BLOCKING)
- **User Story 1 (P1)**: 29 tasks 🎯 MVP
- **User Story 2 (P2)**: 16 tasks
- **Polish Phase**: 9 tasks

**Parallel Opportunities**: 23 tasks marked [P] can run in parallel within their phase

**Estimated Effort**:
- MVP (US1 only): ~29 tasks
- Full Feature (US1 + US2): ~53 tasks (excluding polish)
- Complete (with polish): 62 tasks

---

## Implementation Notes

### Critical Path

1. **Must Complete First**: Phase 2 (Foundational) - all user stories depend on InterruptDevice trait and CPU IRQ state
2. **MVP Path**: Phase 1 → Phase 2 → Phase 3 (US1) → Minimal Phase 5 (testing/docs)
3. **Full Feature Path**: Phase 1 → Phase 2 → Phase 3 (US1) → Phase 4 (US2) → Phase 5

### Validation Strategy

Each user story has integration tests verifying:
- **US1**: Single device interrupt workflow (T031-T035)
- **US2**: Multiple device coordination (T048-T051)

### WASM Considerations

- No OS dependencies (T062 verification)
- All state in simple structs (bool flags, u16 values)
- Deterministic cycle counting
- No callbacks or function pointers

### Cycle Accuracy

Tasks T013-T020 implement the exact 7-cycle sequence per MOS 6502 specification. Task T035 validates this with a test case.
