# Tasks: Memory Mapping Module with UART Device Support

**Input**: Design documents from `/specs/004-memory-mapping-module/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/device_trait.md, quickstart.md

**Tests**: This implementation does not include explicit test tasks. Integration tests and examples serve as validation per the quickstart.md guide.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Single project structure (repository root):
- **Source**: `src/` (Rust library crate)
- **Tests**: `tests/` (integration tests)
- **Examples**: `examples/` (usage demonstrations)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and module structure

- [ ] T001 Create `src/devices/` module directory
- [ ] T002 Add `pub mod devices;` declaration to `src/lib.rs`
- [ ] T003 Create `src/devices/mod.rs` with module declarations

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core Device trait and MemoryBus implementation that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 Define Device trait in `src/devices/mod.rs` with read/write/size methods
- [ ] T005 Define DeviceMapping struct in `src/devices/mod.rs` with base_addr and boxed device
- [ ] T006 Define MappedMemory struct in `src/devices/mod.rs` with devices Vec and unmapped_value
- [ ] T007 Implement MappedMemory::new() constructor in `src/devices/mod.rs` with default unmapped value 0xFF
- [ ] T008 Implement MappedMemory::add_device() method in `src/devices/mod.rs` with overlap detection
- [ ] T009 Implement MemoryBus trait for MappedMemory in `src/devices/mod.rs` with device routing logic
- [ ] T010 Re-export Device, MappedMemory types in `src/lib.rs` public API

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Basic Memory-Mapped Device Architecture (Priority: P1) 🎯 MVP

**Goal**: Enable developers to create 6502 systems with multiple memory-mapped devices (RAM, ROM) in distinct address ranges without modifying CPU core

**Independent Test**: Create system with 16KB RAM at $0000-$3FFF and 16KB ROM at $C000-$FFFF, run program accessing both, verify correct routing (see quickstart.md Example 1)

### Implementation for User Story 1

- [ ] T011 [P] [US1] Create `src/devices/ram.rs` module file
- [ ] T012 [P] [US1] Implement RamDevice struct in `src/devices/ram.rs` with Vec<u8> data field
- [ ] T013 [P] [US1] Implement RamDevice::new(size: u16) constructor in `src/devices/ram.rs`
- [ ] T014 [P] [US1] Implement RamDevice::load_bytes() method in `src/devices/ram.rs` for initializing contents
- [ ] T015 [US1] Implement Device trait for RamDevice in `src/devices/ram.rs` with read/write/size methods
- [ ] T016 [P] [US1] Create `src/devices/rom.rs` module file
- [ ] T017 [P] [US1] Implement RomDevice struct in `src/devices/rom.rs` with Vec<u8> data field
- [ ] T018 [P] [US1] Implement RomDevice::new(data: Vec<u8>) constructor in `src/devices/rom.rs`
- [ ] T019 [US1] Implement Device trait for RomDevice in `src/devices/rom.rs` with read/write (no-op)/size methods
- [ ] T020 [US1] Add ram and rom submodules to `src/devices/mod.rs`
- [ ] T021 [US1] Re-export RamDevice and RomDevice in `src/devices/mod.rs`
- [ ] T022 [US1] Re-export RamDevice and RomDevice in `src/lib.rs` public API
- [ ] T023 [P] [US1] Create integration test file `tests/memory_mapping_tests.rs`
- [ ] T024 [US1] Add test_ram_device_basic_read_write to `tests/memory_mapping_tests.rs`
- [ ] T025 [US1] Add test_rom_device_read_only to `tests/memory_mapping_tests.rs`
- [ ] T026 [US1] Add test_mapped_memory_routing to `tests/memory_mapping_tests.rs` verifying RAM/ROM address routing
- [ ] T027 [US1] Add test_unmapped_address_returns_ff to `tests/memory_mapping_tests.rs`
- [ ] T028 [US1] Add test_overlapping_devices_rejected to `tests/memory_mapping_tests.rs`
- [ ] T029 [P] [US1] Create example file `examples/memory_mapped_system.rs`
- [ ] T030 [US1] Implement memory_mapped_system example in `examples/memory_mapped_system.rs` with RAM/ROM setup per quickstart.md

**Checkpoint**: At this point, User Story 1 (core memory mapping) should be fully functional and testable independently

---

## Phase 4: User Story 2 - 6551 UART Serial Device Emulation (Priority: P2)

**Goal**: Enable developers to add 6551 UART serial device with memory-mapped registers and callback interface for external terminal integration

**Independent Test**: Map UART to $5000-$5003, write to registers, verify status bits and transmit callback delivery (see quickstart.md Example 2)

### Implementation for User Story 2

- [ ] T031 [P] [US2] Create `src/devices/uart.rs` module file
- [ ] T032 [P] [US2] Import VecDeque from std::collections in `src/devices/uart.rs`
- [ ] T033 [P] [US2] Define Uart6551 struct in `src/devices/uart.rs` with register fields
- [ ] T034 [US2] Add rx_buffer: VecDeque<u8> field to Uart6551 in `src/devices/uart.rs`
- [ ] T035 [US2] Add rx_buffer_capacity: usize field to Uart6551 in `src/devices/uart.rs`
- [ ] T036 [US2] Add on_transmit: Option<Box<dyn Fn(u8)>> callback field to Uart6551 in `src/devices/uart.rs`
- [ ] T037 [US2] Implement Uart6551::new() constructor in `src/devices/uart.rs` initializing all registers and 256-byte buffer
- [ ] T038 [P] [US2] Implement Uart6551::set_transmit_callback() method in `src/devices/uart.rs`
- [ ] T039 [P] [US2] Implement Uart6551::receive_byte() method in `src/devices/uart.rs` with buffer and overflow handling
- [ ] T040 [US2] Implement private read_data_register() method in `src/devices/uart.rs` popping from rx_buffer
- [ ] T041 [US2] Implement private write_data_register() method in `src/devices/uart.rs` invoking transmit callback
- [ ] T042 [US2] Implement private update_status_register() method in `src/devices/uart.rs` setting RDRF/TDRE/overrun bits
- [ ] T043 [US2] Implement Device::read() for Uart6551 in `src/devices/uart.rs` with register dispatch (0=data, 1=status, 2=command, 3=control)
- [ ] T044 [US2] Implement Device::write() for Uart6551 in `src/devices/uart.rs` with register dispatch and status read-only handling
- [ ] T045 [US2] Implement Device::size() for Uart6551 in `src/devices/uart.rs` returning 4
- [ ] T046 [P] [US2] Add public inspection methods to Uart6551 in `src/devices/uart.rs` (status(), rx_buffer_len()) for testing
- [ ] T047 [US2] Add uart submodule to `src/devices/mod.rs`
- [ ] T048 [US2] Re-export Uart6551 in `src/devices/mod.rs`
- [ ] T049 [US2] Re-export Uart6551 in `src/lib.rs` public API
- [ ] T050 [P] [US2] Create integration test file `tests/uart_tests.rs`
- [ ] T051 [US2] Add test_uart_data_register_transmit to `tests/uart_tests.rs` verifying callback invocation
- [ ] T052 [US2] Add test_uart_data_register_receive to `tests/uart_tests.rs` verifying rx buffer and RDRF flag
- [ ] T053 [US2] Add test_uart_status_register_read_only to `tests/uart_tests.rs`
- [ ] T054 [US2] Add test_uart_status_bits to `tests/uart_tests.rs` verifying TDRE/RDRF/overrun bits
- [ ] T055 [US2] Add test_uart_command_control_registers to `tests/uart_tests.rs` verifying read/write persistence
- [ ] T056 [US2] Add test_uart_buffer_overflow to `tests/uart_tests.rs` verifying overflow flag and dropped bytes
- [ ] T057 [US2] Add test_uart_echo_mode to `tests/uart_tests.rs` if echo mode implemented
- [ ] T058 [P] [US2] Create example file `examples/uart_echo.rs`
- [ ] T059 [US2] Implement uart_echo example in `examples/uart_echo.rs` demonstrating UART read/write loop per quickstart.md

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - Browser-Based Serial Terminal Connection (Priority: P3)

**Goal**: Enable users to interact with emulated 6502 system through browser terminal interface with bidirectional character flow

**Independent Test**: Open emulator webpage, type characters, run echo program, verify terminal display (see quickstart.md Example 4)

### Implementation for User Story 3

- [ ] T060 [P] [US3] Create example file `examples/wasm_terminal.rs` for WASM-bindgen integration patterns
- [ ] T061 [US3] Document WASM callback setup in `examples/wasm_terminal.rs` with transmit function pointer example
- [ ] T062 [US3] Document terminal receive_byte() invocation in `examples/wasm_terminal.rs` with onData handler example
- [ ] T063 [US3] Add JavaScript integration example to `examples/wasm_terminal.rs` as code comment showing xterm.js setup
- [ ] T064 [P] [US3] Update quickstart.md Example 4 with complete WASM/xterm.js integration code if needed
- [ ] T065 [US3] Add browser compatibility notes to `examples/wasm_terminal.rs` for Chrome/Firefox/Safari/Edge
- [ ] T066 [P] [US3] Create manual test checklist in `specs/004-memory-mapping-module/browser-test-plan.md`
- [ ] T067 [US3] Document UART buffer behavior in browser context in `specs/004-memory-mapping-module/browser-test-plan.md`

**Checkpoint**: All user stories should now be independently functional

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, verification, and cross-cutting improvements

- [ ] T068 [P] Add comprehensive doc comments to Device trait in `src/devices/mod.rs`
- [ ] T069 [P] Add comprehensive doc comments to MappedMemory in `src/devices/mod.rs`
- [ ] T070 [P] Add doc comments and usage examples to RamDevice in `src/devices/ram.rs`
- [ ] T071 [P] Add doc comments and usage examples to RomDevice in `src/devices/rom.rs`
- [ ] T072 [P] Add comprehensive doc comments to Uart6551 in `src/devices/uart.rs` with register bit descriptions
- [ ] T073 Verify WASM compilation with `cargo build --target wasm32-unknown-unknown`
- [ ] T074 Run all tests with `cargo test` and verify pass
- [ ] T075 Run clippy with `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] T076 Run formatter with `cargo fmt --check`
- [ ] T077 [P] Update CLAUDE.md Active Technologies section if needed
- [ ] T078 Validate quickstart.md examples compile and run

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User Story 1 (P1): Can start after Phase 2 - No dependencies on other stories
  - User Story 2 (P2): Can start after Phase 2 - Functionally independent of US1, but uses same infrastructure
  - User Story 3 (P3): Logically depends on US2 (needs UART device) but examples can be written in parallel
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Independent implementation, integrates with MappedMemory from Phase 2
- **User Story 3 (P3)**: Logically follows US2 (demonstrates UART in browser) but can be developed in parallel if team capacity allows

### Within Each User Story

- **User Story 1**: RamDevice and RomDevice can be implemented in parallel, integration tests depend on both being complete
- **User Story 2**: Struct definition and methods can progress sequentially, inspection methods and examples can proceed in parallel once core is done
- **User Story 3**: Documentation tasks can all proceed in parallel

### Parallel Opportunities

**Phase 1 (Setup)**:
- All tasks sequential (directory creation order matters)

**Phase 2 (Foundational)**:
- T004-T006 define structs/traits - sequential
- T007-T009 implement methods - sequential (each builds on previous)
- T010 re-export - sequential (depends on T004-T009)

**Phase 3 (User Story 1)**:
- T011-T015 (RamDevice) can run in parallel with T016-T019 (RomDevice)
- T023-T028 (integration tests) can run in parallel once T022 is complete
- T029-T030 (example) can run in parallel with tests

**Phase 4 (User Story 2)**:
- T031-T036 (struct definition) sequential
- T038-T039 (public methods) can run in parallel
- T046 (inspection methods) can run in parallel with T047-T049
- T050-T057 (all tests) can run in parallel once T049 is complete
- T058-T059 (example) can run in parallel with tests

**Phase 5 (User Story 3)**:
- T060-T065 (all example/doc tasks) can run in parallel
- T066-T067 (test plan) can run in parallel with examples

**Phase 6 (Polish)**:
- T068-T072 (doc comments) can all run in parallel
- T077-T078 (documentation updates) can run in parallel

---

## Parallel Example: User Story 1

```bash
# After Phase 2 completes, launch RamDevice and RomDevice in parallel:
Task: "Create src/devices/ram.rs module file"
Task: "Implement RamDevice struct in src/devices/ram.rs with Vec<u8> data field"
Task: "Implement RamDevice::new(size: u16) constructor in src/devices/ram.rs"
Task: "Implement RamDevice::load_bytes() method in src/devices/ram.rs for initializing contents"

# At the same time in parallel:
Task: "Create src/devices/rom.rs module file"
Task: "Implement RomDevice struct in src/devices/rom.rs with Vec<u8> data field"
Task: "Implement RomDevice::new(data: Vec<u8>) constructor in src/devices/rom.rs"

# After T022 completes, launch all integration tests in parallel:
Task: "Create integration test file tests/memory_mapping_tests.rs"
Task: "Add test_ram_device_basic_read_write to tests/memory_mapping_tests.rs"
Task: "Add test_rom_device_read_only to tests/memory_mapping_tests.rs"
Task: "Add test_mapped_memory_routing to tests/memory_mapping_tests.rs"
Task: "Add test_unmapped_address_returns_ff to tests/memory_mapping_tests.rs"
Task: "Add test_overlapping_devices_rejected to tests/memory_mapping_tests.rs"

# Launch example in parallel with tests:
Task: "Create example file examples/memory_mapped_system.rs"
Task: "Implement memory_mapped_system example in examples/memory_mapped_system.rs"
```

---

## Parallel Example: User Story 2

```bash
# After struct definition (T031-T036), launch public methods in parallel:
Task: "Implement Uart6551::set_transmit_callback() method in src/devices/uart.rs"
Task: "Implement Uart6551::receive_byte() method in src/devices/uart.rs with buffer and overflow handling"

# After T049 completes, launch all UART tests in parallel:
Task: "Create integration test file tests/uart_tests.rs"
Task: "Add test_uart_data_register_transmit to tests/uart_tests.rs"
Task: "Add test_uart_data_register_receive to tests/uart_tests.rs"
Task: "Add test_uart_status_register_read_only to tests/uart_tests.rs"
Task: "Add test_uart_status_bits to tests/uart_tests.rs"
Task: "Add test_uart_command_control_registers to tests/uart_tests.rs"
Task: "Add test_uart_buffer_overflow to tests/uart_tests.rs"
Task: "Add test_uart_echo_mode to tests/uart_tests.rs"

# Launch example in parallel with tests:
Task: "Create example file examples/uart_echo.rs"
Task: "Implement uart_echo example in examples/uart_echo.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: Foundational (T004-T010) - CRITICAL, blocks all stories
3. Complete Phase 3: User Story 1 (T011-T030)
4. **STOP and VALIDATE**: Run `cargo test`, verify RAM/ROM routing works
5. Run example: `cargo run --example memory_mapped_system`
6. **MVP COMPLETE**: Can now demonstrate multiple memory-mapped devices

### Incremental Delivery

1. **Foundation** (Phases 1-2): Device trait + MappedMemory infrastructure ready
2. **MVP** (Phase 3): Add User Story 1 → Test independently → **Demo RAM/ROM mapping**
3. **Serial I/O** (Phase 4): Add User Story 2 → Test independently → **Demo UART communication**
4. **Browser Experience** (Phase 5): Add User Story 3 → Test independently → **Demo browser terminal**
5. **Production Ready** (Phase 6): Polish → All tests passing → **Release**

Each phase adds value without breaking previous functionality.

### Parallel Team Strategy

With multiple developers:

1. **Foundation First**: Team completes Phases 1-2 together (blocking work)
2. **Parallel Stories** (after Phase 2 complete):
   - Developer A: User Story 1 (T011-T030)
   - Developer B: User Story 2 (T031-T059) - can start even while US1 in progress
   - Developer C: User Story 3 (T060-T067) - documentation/examples can proceed early
3. **Convergence**: Each story completes and integrates independently
4. **Polish Together**: Team runs Phase 6 tasks in parallel

---

## Notes

- **[P] tasks**: Different files, no dependencies - safe to run in parallel
- **[Story] label**: Maps task to specific user story for traceability
- **Independent stories**: Each user story can be tested standalone per quickstart.md
- **No explicit test phase**: Integration tests embedded in each user story phase
- **Examples as tests**: Examples in quickstart.md serve as acceptance criteria
- **WASM compatible**: All tasks maintain no_std compatibility (no OS dependencies)
- **Stop at any checkpoint**: Each phase delivers independently valuable functionality

### Success Criteria Mapping

- **SC-001** (3+ devices): Verified by T030 (memory_mapped_system example)
- **SC-002** (100% data integrity): Verified by T052, T056 (UART receive tests)
- **SC-003** (<100ms TX latency): Manual verification per T067 (browser test plan)
- **SC-004** (<100ms RX latency): Manual verification per T067 (browser test plan)
- **SC-005** (100 bytes/sec): Verified by T056 (buffer overflow test with rapid input)
- **SC-006** (WASM compilation): Verified by T073 (explicit WASM build check)
- **SC-007** (Browser compatibility): Manual verification per T066-T067 (browser test plan)

### Task Count Summary

- **Setup**: 3 tasks
- **Foundational**: 7 tasks
- **User Story 1 (P1)**: 20 tasks
- **User Story 2 (P2)**: 29 tasks
- **User Story 3 (P3)**: 8 tasks
- **Polish**: 11 tasks

**Total**: 78 tasks

### Parallel Task Opportunities

- **Setup**: 0 parallel (sequential directory creation)
- **Foundational**: 0 parallel (foundational layer must be sequential)
- **User Story 1**: 8 parallel opportunities (RAM/ROM in parallel, tests in parallel, example in parallel)
- **User Story 2**: 11 parallel opportunities (methods, tests, examples)
- **User Story 3**: 6 parallel opportunities (all doc/example tasks)
- **Polish**: 7 parallel opportunities (doc comments, validation tasks)

**Total Parallel Opportunities**: 32 tasks can run concurrently (41% of total)
