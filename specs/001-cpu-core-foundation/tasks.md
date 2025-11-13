# Tasks: CPU Core Foundation

**Input**: Design documents from `/specs/001-cpu-core-foundation/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Not explicitly requested in the feature specification. However, SC-003, SC-010, and FR-011 require test infrastructure and basic structural tests.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Single project structure at repository root:

- `src/` - Library source code
- `tests/` - Integration tests
- `examples/` - Usage examples
- `Cargo.toml` - Project manifest

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Initialize Rust project structure and build configuration

- [ ] T001 Create Cargo.toml with library configuration (edition 2021, no dependencies)
- [ ] T002 Create src/lib.rs as library root with module declarations
- [ ] T003 Create tests/ directory for integration tests
- [ ] T004 Create examples/ directory for usage examples
- [ ] T005 Verify project builds successfully with `cargo build`
- [ ] T006 Add wasm32-unknown-unknown target and verify WASM compilation succeeds

**Checkpoint**: Project structure is ready, builds successfully for both native and WASM targets

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core type definitions and data structures that ALL user stories depend on

**CRITICAL**: No user story work can begin until this phase is complete

- [ ] T007 Define AddressingMode enum in src/addressing.rs with all 13 variants (Implicit, Accumulator, Immediate, ZeroPage, ZeroPageX, ZeroPageY, Relative, Absolute, AbsoluteX, AbsoluteY, Indirect, IndirectX, IndirectY)
- [ ] T008 Define OpcodeMetadata struct in src/opcodes.rs with fields (mnemonic, addressing_mode, base_cycles, size_bytes, implemented)
- [ ] T009 Define ExecutionError enum in src/lib.rs with UnimplementedOpcode(u8) variant
- [ ] T010 Implement Display and Error traits for ExecutionError in src/lib.rs

**Checkpoint**: Foundation types ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Project Initialization (Priority: P1) - MVP

**Goal**: Create a working Rust project with core CPU module scaffolding that compiles successfully and can instantiate a CPU with default state

**Independent Test**: Project compiles, CPU can be instantiated, all registers can be inspected with documented initial values, test suite runs successfully

### Implementation for User Story 1

- [ ] T011 [US1] Define CPU struct in src/cpu.rs with generic MemoryBus parameter and all register fields (a, x, y, pc, sp, flag_n, flag_v, flag_b, flag_d, flag_i, flag_z, flag_c, cycles, memory)
- [ ] T012 [US1] Implement CPU::new() constructor in src/cpu.rs that initializes CPU to reset state (reads reset vector from 0xFFFC/0xFFFD, sets SP to 0xFD, sets I flag to true, zeros other registers)
- [ ] T013 [P] [US1] Implement public getter methods for registers in src/cpu.rs (a, x, y, pc, sp, cycles)
- [ ] T014 [P] [US1] Implement public getter methods for status flags in src/cpu.rs (flag_n, flag_v, flag_b, flag_d, flag_i, flag_z, flag_c)
- [ ] T015 [US1] Implement status() method in src/cpu.rs that packs flags into u8 byte (NV-BDIZC format, bit 5 always 1)
- [ ] T016 [US1] Create tests/cpu_init_test.rs with test verifying CPU initialization values
- [ ] T017 [US1] Add doc comments to CPU struct and methods in src/cpu.rs per API contract

**Checkpoint**: CPU struct is complete, can be instantiated, state is inspectable, initialization test passes

---

## Phase 4: User Story 2 - Memory Bus Abstraction (Priority: P2)

**Goal**: Provide trait-based memory bus interface that decouples CPU from specific memory implementations

**Independent Test**: Simple FlatMemory implementation can be created, CPU can read and write through the abstraction, test memory round-trips correctly

### Implementation for User Story 2

- [ ] T018 [US2] Define MemoryBus trait in src/memory.rs with read(&self, addr: u16) -> u8 and write(&mut self, addr: u16, value: u8) methods
- [ ] T019 [US2] Add doc comments to MemoryBus trait per API contract in src/memory.rs
- [ ] T020 [US2] Define FlatMemory struct in src/memory.rs with 64KB array field
- [ ] T021 [US2] Implement FlatMemory::new() constructor in src/memory.rs that initializes memory to zeros
- [ ] T022 [US2] Implement MemoryBus trait for FlatMemory in src/memory.rs
- [ ] T023 [US2] Create tests/memory_bus_test.rs with tests verifying FlatMemory read/write round-trip
- [ ] T024 [US2] Update src/lib.rs to re-export MemoryBus trait and FlatMemory struct

**Checkpoint**: MemoryBus trait is defined, FlatMemory implementation works, memory abstraction tests pass

---

## Phase 5: User Story 3 - Fetch-Decode-Execute Loop (Priority: P3)

**Goal**: Provide skeletal fetch-decode-execute loop that can advance the CPU by one instruction with clear error handling for unimplemented instructions

**Independent Test**: Execute loop can fetch opcode, identify it (even if not implemented), return UnimplementedOpcode error, and increment program counter appropriately

### Implementation for User Story 3

- [ ] T025 [US3] Implement CPU::step() method in src/cpu.rs that fetches opcode at PC via MemoryBus
- [ ] T026 [US3] Add decode logic to step() in src/cpu.rs that looks up opcode in OPCODE_TABLE
- [ ] T027 [US3] Add execution check in step() that returns ExecutionError::UnimplementedOpcode if implemented flag is false
- [ ] T028 [US3] Add cycle counter increment in step() that adds base_cycles from opcode metadata
- [ ] T029 [US3] Implement CPU::run_for_cycles() method in src/cpu.rs that executes instructions until cycle budget exhausted
- [ ] T030 [US3] Create tests/execute_loop_test.rs with test that loads simple program and verifies step() returns UnimplementedOpcode
- [ ] T031 [US3] Add test in tests/execute_loop_test.rs verifying cycle counter increments correctly
- [ ] T032 [US3] Add test in tests/execute_loop_test.rs verifying run_for_cycles() respects cycle budget

**Checkpoint**: Execute loop is functional, error handling works, cycle counting is accurate, execution tests pass

---

## Phase 6: User Story 4 - Opcode Metadata Table (Priority: P4)

**Goal**: Provide complete table-driven opcode metadata structure covering all 256 opcodes as single source of truth for decode logic

**Independent Test**: Table contains entries for all 256 opcodes, decoder can look up metadata for any opcode, all documented instructions have valid data, illegal opcodes are marked appropriately

### Implementation for User Story 4

- [ ] T033 [US4] Create OPCODE_TABLE constant array in src/opcodes.rs with 256 OpcodeMetadata entries (initialize with placeholder data)
- [ ] T034 [US4] Populate documented instruction entries in OPCODE_TABLE in src/opcodes.rs (151 documented opcodes with correct mnemonic, mode, cycles, size)
- [ ] T035 [US4] Populate illegal opcode entries in OPCODE_TABLE in src/opcodes.rs (105 illegal opcodes marked with "???" mnemonic, 0 cycles, size 1, implemented false)
- [ ] T036 [US4] Update src/lib.rs to re-export OPCODE_TABLE and OpcodeMetadata
- [ ] T037 [US4] Create tests/opcode_table_test.rs with test verifying table has 256 entries
- [ ] T038 [P] [US4] Add test in tests/opcode_table_test.rs verifying all mnemonics are non-empty
- [ ] T039 [P] [US4] Add test in tests/opcode_table_test.rs verifying size_bytes matches addressing_mode per schema
- [ ] T040 [P] [US4] Add test in tests/opcode_table_test.rs verifying documented instructions have non-zero cycles
- [ ] T041 [P] [US4] Add test in tests/opcode_table_test.rs verifying all implemented flags are false
- [ ] T042 [US4] Add doc comments to OPCODE_TABLE and OpcodeMetadata in src/opcodes.rs per API contract

**Checkpoint**: Opcode table is complete, all validation tests pass, table is queryable for decode logic

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, examples, final validation, and quality improvements

- [ ] T043 [P] Create examples/simple_ram.rs demonstrating CPU initialization, memory setup, and basic execution
- [ ] T044 [P] Add module-level doc comments to src/lib.rs with crate overview and links to quickstart
- [ ] T045 [P] Add module-level doc comments to src/cpu.rs explaining CPU state and execution model
- [ ] T046 [P] Add module-level doc comments to src/memory.rs explaining MemoryBus abstraction
- [ ] T047 [P] Add module-level doc comments to src/opcodes.rs explaining opcode table structure
- [ ] T048 Verify all tests pass with `cargo test`
- [ ] T049 Verify WASM compilation succeeds with `cargo build --target wasm32-unknown-unknown`
- [ ] T050 Run `cargo clippy` and address any warnings
- [ ] T051 Run `cargo doc --open` and verify documentation renders correctly
- [ ] T052 Verify test coverage reaches at least 80% of defined structures and initialization code (per SC-010)
- [ ] T053 Execute quickstart.md validation steps to ensure all documented workflows work

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - User Story 1 (P1) can start after Foundational
  - User Story 2 (P2) can start after Foundational (but CPU instantiation from US1 needed for integration)
  - User Story 3 (P3) depends on US1 (CPU struct) and US2 (MemoryBus) being complete
  - User Story 4 (P4) can start after Foundational, but is referenced by US3 (execute loop)
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Depends on Foundational (Phase 2) - Defines CPU struct and state
- **User Story 2 (P2)**: Depends on Foundational (Phase 2) - Defines memory abstraction
- **User Story 3 (P3)**: Depends on US1 (CPU struct), US2 (MemoryBus), and US4 (OPCODE_TABLE)
- **User Story 4 (P4)**: Depends on Foundational (Phase 2), referenced by US3

### Suggested Implementation Order

Given the dependencies, the optimal implementation order is:

1. Phase 1 (Setup)
2. Phase 2 (Foundational)
3. Phase 4 (User Story 1 - CPU struct) - Core state structure
4. Phase 5 (User Story 2 - MemoryBus) - Memory abstraction
5. Phase 6 (User Story 4 - Opcode Table) - Metadata needed by execute loop
6. Phase 7 (User Story 3 - Execute Loop) - Ties everything together
7. Phase 8 (Polish) - Documentation and validation

### Parallel Opportunities

- **Phase 1 (Setup)**: T001-T006 can run sequentially as each builds on previous
- **Phase 2 (Foundational)**: T007-T010 can run in parallel (different type definitions)
- **User Story 1**: T013 and T014 can run in parallel (different getter methods)
- **User Story 4**: T038, T039, T040, T041 can run in parallel (different validation tests)
- **Polish Phase**: T043, T044, T045, T046, T047 can run in parallel (different documentation files)

---

## Parallel Example: Foundational Phase

```bash
# Launch all foundational type definitions together:
Task: "Define AddressingMode enum in src/addressing.rs"
Task: "Define OpcodeMetadata struct in src/opcodes.rs"
Task: "Define ExecutionError enum in src/lib.rs"
```

## Parallel Example: User Story 4 Tests

```bash
# Launch all opcode table validation tests together:
Task: "Add test verifying all mnemonics are non-empty"
Task: "Add test verifying size_bytes matches addressing_mode"
Task: "Add test verifying documented instructions have non-zero cycles"
Task: "Add test verifying all implemented flags are false"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (CPU struct and initialization)
4. **STOP and VALIDATE**: Test CPU instantiation independently
5. This provides a minimal but functional CPU core with state inspection

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add User Story 1 → CPU struct complete → Can instantiate and inspect state
3. Add User Story 2 → Memory abstraction complete → CPU can interact with memory
4. Add User Story 4 → Opcode table complete → Metadata available for decode
5. Add User Story 3 → Execute loop complete → CPU can fetch-decode-execute (reports unimplemented)
6. Polish → Documentation and validation complete → Feature ready for next instructions

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (CPU struct)
   - Developer B: User Story 2 (MemoryBus)
   - Developer C: User Story 4 (Opcode Table) - can start in parallel
3. After US1, US2, US4 complete:
   - Developer A: User Story 3 (Execute Loop) - integrates all previous work
4. All developers: Polish phase in parallel

---

## Notes

- [P] tasks = different files, no dependencies, can run in parallel
- [Story] label maps task to specific user story for traceability
- Each user story should be independently testable where possible (US3 depends on US1, US2, US4)
- All tasks have specific file paths for clarity
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Tests are structural validation tests (per FR-011, SC-003, SC-010)
- No instruction implementations in this feature - all opcodes return UnimplementedOpcode
- WASM compatibility is verified at setup and polish phases (per FR-014, SC-002)

---

## Task Count Summary

- **Total Tasks**: 53
- **Phase 1 (Setup)**: 6 tasks
- **Phase 2 (Foundational)**: 4 tasks
- **Phase 3 (User Story 1)**: 7 tasks
- **Phase 4 (User Story 2)**: 7 tasks
- **Phase 5 (User Story 3)**: 8 tasks
- **Phase 6 (User Story 4)**: 10 tasks
- **Phase 7 (Polish)**: 11 tasks

### Tasks per User Story

- **US1 (Project Initialization)**: 7 tasks - CPU struct, getters, initialization
- **US2 (Memory Bus Abstraction)**: 7 tasks - MemoryBus trait, FlatMemory implementation
- **US3 (Fetch-Decode-Execute Loop)**: 8 tasks - step(), run_for_cycles(), execution tests
- **US4 (Opcode Metadata Table)**: 10 tasks - OPCODE_TABLE population and validation

### Parallel Opportunities

- **Foundational Phase**: 4 tasks can run in parallel (different type definitions)
- **User Story 1**: 2 tasks can run in parallel (T013, T014 - different getter groups)
- **User Story 4**: 4 tests can run in parallel (T038-T041 - independent validations)
- **Polish Phase**: 5 documentation tasks can run in parallel (T043-T047)

**Total Parallelizable Tasks**: ~15 tasks

### Suggested MVP Scope

**Minimum Viable Product** (can be delivered incrementally):

- Phase 1 (Setup) + Phase 2 (Foundational) + Phase 3 (User Story 1)
- This provides: Buildable Rust project with CPU struct that can be instantiated and inspected
- Estimated: 17 tasks for minimal CPU core with state

**Full Feature Scope**:

- All phases (1-7) including all user stories
- This provides: Complete CPU foundation with memory abstraction, execute loop, opcode table
- Estimated: 53 tasks for complete foundational architecture

---

## Success Criteria Mapping

This task list maps to the following success criteria from spec.md:

- **SC-001**: T005, T050 - Project compiles successfully
- **SC-002**: T006, T049 - WASM compilation succeeds
- **SC-003**: T048 - All tests pass
- **SC-004**: T011-T015 - CPU instantiation and register inspection
- **SC-005**: T025-T032 - Execute loop functionality
- **SC-006**: T033-T042 - Opcode table completeness
- **SC-007**: T028, T031 - Cycle counter functionality
- **SC-008**: T018-T024 - MemoryBus trait implementation ease
- **SC-009**: T043-T047, T051 - Documentation coverage
- **SC-010**: T052 - Test coverage requirement
