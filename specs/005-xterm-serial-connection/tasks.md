# Tasks: xterm.js Serial Terminal Integration

**Input**: Design documents from `/specs/005-xterm-serial-connection/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Manual browser testing per spec.md - no automated test framework required for initial implementation

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- Rust WASM: `src/wasm/`, `src/devices/`
- Demo frontend: `demo/`, `demo/components/`, `demo/examples/`
- All paths relative to repository root (`/home/user/6502/`)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish baseline and verify current state

- [ ] T001 Build WASM module to verify current state: `wasm-pack build --target web --out-dir demo/lib6502_wasm`
- [ ] T002 Serve demo locally and verify existing functionality works: `cd demo && python3 -m http.server 8000`
- [ ] T003 Review research.md decisions: xterm.js 5.5.0, UART at $A000, component architecture

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 Add xterm.js 5.5.0 CSS CDN link to demo/index.html `<head>` section
- [ ] T005 [P] Add xterm.js 5.5.0 and addon-fit 0.10.0 script tags before closing `</body>` in demo/index.html
- [ ] T006 [P] Add terminal panel HTML container with ID `terminal-container` to demo/index.html right panel
- [ ] T007 [P] Add terminal panel base styling to demo/styles.css (`.terminal-panel`, `#terminal-container`)

**Checkpoint**: Foundation ready - HTML structure and CDN links in place, user story implementation can now begin

---

## Phase 3: User Story 1 - Interactive Serial I/O for Assembly Programs (Priority: P1) 🎯 MVP

**Goal**: Enable bidirectional communication between 6502 programs and terminal - users can type in terminal and see echo output

**Independent Test**: Load echo program, type "Hello" in terminal, verify "Hello" appears as output. This is the CORE functionality.

### Implementation for User Story 1

#### WASM Backend Modifications

- [ ] T008 [P] [US1] Modify Emulator6502 constructor in src/wasm/api.rs to accept `on_transmit: js_sys::Function` parameter
- [ ] T009 [US1] Replace FlatMemory with MappedMemory initialization in Emulator6502::new() in src/wasm/api.rs
- [ ] T010 [US1] Add RamDevice at $0000 (32KB) to MappedMemory in src/wasm/api.rs
- [ ] T011 [US1] Create Uart6551 instance with transmit callback in Emulator6502::new() in src/wasm/api.rs
- [ ] T012 [US1] Add Uart6551 to MappedMemory at $A000 in src/wasm/api.rs
- [ ] T013 [US1] Add RomDevice at $C000 (16KB) with reset vector ($FFFC→$0600) to MappedMemory in src/wasm/api.rs
- [ ] T014 [US1] Store `Rc<RefCell<Uart6551>>` reference as field in Emulator6502 struct in src/wasm/api.rs
- [ ] T015 [US1] Add `receive_char(byte: u8)` method to Emulator6502 in src/wasm/api.rs that calls `uart.borrow_mut().receive_byte(byte)`
- [ ] T016 [US1] Rebuild WASM module: `wasm-pack build --target web --out-dir demo/lib6502_wasm`

#### Frontend Terminal Component

- [ ] T017 [P] [US1] Create demo/components/terminal.js with Terminal class skeleton (constructor, write, clear, fit methods)
- [ ] T018 [US1] Implement Terminal constructor in demo/components/terminal.js: create xterm.js instance with config per contracts/javascript-api.md
- [ ] T019 [US1] Implement Terminal.setupEventListeners() in demo/components/terminal.js: handle onData → dispatch 'terminal-data' CustomEvent
- [ ] T020 [US1] Implement Terminal.setupEventListeners() in demo/components/terminal.js: handle window resize → fitAddon.fit()
- [ ] T021 [US1] Implement Terminal.write(text) method in demo/components/terminal.js: call this.term.write(text)
- [ ] T022 [US1] Add welcome message to Terminal constructor in demo/components/terminal.js: "6502 Serial Terminal Ready\\r\\nUART: $A000-$A003\\r\\n\\r\\n"

#### App Integration

- [ ] T023 [US1] Add `import { Terminal } from './components/terminal.js'` to demo/app.js
- [ ] T024 [US1] Add `this.terminal = null` to App constructor in demo/app.js
- [ ] T025 [US1] Create terminal instance BEFORE emulator in App.init() in demo/app.js: `this.terminal = new Terminal('terminal-container')`
- [ ] T026 [US1] Modify Emulator6502 constructor call in App.init() in demo/app.js to pass transmit callback: `(char) => this.terminal.write(char)`
- [ ] T027 [US1] Add 'terminal-data' event listener in App.setupEventListeners() in demo/app.js
- [ ] T028 [US1] Implement App.handleTerminalInput(data) method in demo/app.js: loop through characters, call emulator.receive_char(byte) for each
- [ ] T029 [US1] Modify App.handleReset() in demo/app.js to clear terminal and display "CPU Reset\\r\\n\\r\\n"

#### Manual Testing

- [ ] T030 [US1] Load demo in browser, verify terminal appears with welcome message
- [ ] T031 [US1] Open browser console, manually test: `app.emulator.receive_char(0x41)` (should not crash)
- [ ] T032 [US1] Write simple inline echo test: assemble "LDA #$48; STA $A000; BRK", run, verify 'H' appears in terminal
- [ ] T033 [US1] Test typing in terminal triggers terminal-data events (check console logs)

**Checkpoint**: At this point, User Story 1 should be fully functional - terminal accepts input, UART communication works bidirectionally

---

## Phase 4: User Story 4 - Example Programs for Learning (Priority: P2)

**Goal**: Provide pre-built example programs that demonstrate UART patterns, making it easy to test and learn

**Independent Test**: Click "UART Echo" example, assemble, run, type "Test" in terminal, verify "Test" echoes back

### Implementation for User Story 4

#### Example Assembly Programs

- [ ] T034 [P] [US4] Create demo/examples/uart-echo.asm with polling loop: check RDRF, read $A000, write $A000, repeat
- [ ] T035 [P] [US4] Create demo/examples/uart-hello.asm with message output: load string "Hello, 6502!\\r\\n", write bytes to $A000
- [ ] T036 [P] [US4] Create demo/examples/uart-polling.asm demonstrating status register polling technique

#### Example Selector Integration

- [ ] T037 [US4] Add 'uart-echo' example object to getExamples() array in demo/components/examples.js
- [ ] T038 [P] [US4] Add 'uart-hello' example object to getExamples() array in demo/components/examples.js
- [ ] T039 [P] [US4] Add 'uart-polling' example object to getExamples() array in demo/components/examples.js

#### Manual Testing

- [ ] T040 [US4] Load demo, select "UART Echo" example, assemble, run, type characters, verify echo behavior
- [ ] T041 [P] [US4] Load demo, select "Hello World" example, assemble, run, verify "Hello, 6502!" appears in terminal
- [ ] T042 [P] [US4] Load demo, select "UART Polling" example, assemble, run, verify polling loop works correctly

**Checkpoint**: At this point, User Stories 1 AND 4 both work - users can load examples and see immediate UART functionality

---

## Phase 5: User Story 2 - Terminal State Visibility (Priority: P2)

**Goal**: Users can see terminal ready state and UART status to debug serial programs

**Independent Test**: Load page, verify terminal shows ready message. Load echo program, type char, check memory viewer at $A001 shows RDRF flag set.

### Implementation for User Story 2

#### Visual Indicators

- [ ] T043 [US2] Verify terminal welcome message indicates ready state (already implemented in T022)
- [ ] T044 [US2] Test memory viewer shows UART status register at $A001 with correct flags (RDRF, TDRE) - verify existing functionality
- [ ] T045 [US2] Verify transmitted characters appear immediately in terminal (already implemented in T026 callback)

#### Documentation

- [ ] T046 [US2] Add comment in demo/components/terminal.js documenting ready state indication
- [ ] T047 [US2] Add inline comment in demo/app.js transmit callback explaining immediate display

**Checkpoint**: User Story 2 complete - visibility features verified, mostly leveraging existing US1 implementation

---

## Phase 6: User Story 3 - Terminal Control and Configuration (Priority: P3)

**Goal**: Users can control terminal behavior - clear display, copy output, handle resize

**Independent Test**: Run program with output, click hypothetical clear button (or call terminal.clear()), verify display clears. Copy text from terminal, paste elsewhere.

### Implementation for User Story 3

#### Clear Functionality

- [ ] T048 [US3] Implement Terminal.clear() method in demo/components/terminal.js: call this.term.clear()
- [ ] T049 [US3] Verify clear() is called in App.handleReset() (already implemented in T029)
- [ ] T050 [US3] (Optional) Add clear button to terminal panel in demo/index.html
- [ ] T051 [US3] (Optional) Wire clear button click to call app.terminal.clear() in demo/app.js

#### Copy/Paste Support

- [ ] T052 [US3] Test copy/paste in browser - xterm.js supports this by default, verify it works
- [ ] T053 [US3] Document copy/paste support in terminal component comments

#### Resize Handling

- [ ] T054 [US3] Verify resize handler in Terminal.setupEventListeners() (already implemented in T020)
- [ ] T055 [US3] Test browser window resize, verify terminal adjusts correctly

**Checkpoint**: All user stories complete - full terminal control functionality available

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories or overall quality

- [ ] T056 [P] Add error handling for terminal container not found in demo/components/terminal.js constructor
- [ ] T057 [P] Add error handling for WASM initialization failure in demo/app.js
- [ ] T058 [P] Test edge case: type 256+ characters rapidly, verify buffer overflow handling (OVRN flag)
- [ ] T059 [P] Test edge case: backspace, enter, special characters in terminal
- [ ] T060 [P] Test edge case: CPU reset clears UART buffer and terminal
- [ ] T061 Verify responsive design on 1024px+ screens per SC-007
- [ ] T062 Performance test: verify <100ms echo latency per SC-001
- [ ] T063 Performance test: verify 1 MHz CPU speed maintained with terminal active per SC-005
- [ ] T064 Run through quickstart.md validation steps
- [ ] T065 Update demo README if needed with UART examples and terminal usage

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational phase completion - MVP CORE
- **User Story 4 (Phase 4)**: Depends on US1 completion (needs working terminal for examples)
- **User Story 2 (Phase 5)**: Depends on US1 completion (verifies US1 behavior)
- **User Story 3 (Phase 6)**: Depends on US1 completion (enhances US1 terminal)
- **Polish (Phase 7)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories - **THIS IS THE MVP**
- **User Story 4 (P2)**: Depends on US1 (needs working terminal/UART communication)
- **User Story 2 (P2)**: Depends on US1 (verifies visibility of US1 features)
- **User Story 3 (P3)**: Depends on US1 (enhances US1 terminal controls)

### Within Each User Story

**User Story 1**:
- WASM backend changes (T008-T016) can proceed in sequence, then rebuild
- Frontend terminal component (T017-T022) can proceed in parallel with WASM after T016 rebuild
- App integration (T023-T029) depends on both WASM rebuild and terminal component
- Manual testing (T030-T033) verifies everything

**User Story 4**:
- Example .asm files (T034-T036) can be created in parallel
- examples.js updates (T037-T039) can be done in parallel
- Testing (T040-T042) verifies examples work

### Parallel Opportunities

- **Phase 1**: All setup tasks can run in sequence (baseline verification)
- **Phase 2**: Tasks T005, T006, T007 can run in parallel (different files)
- **User Story 1**:
  - After T016 (WASM rebuild), tasks T017-T022 (terminal component) can proceed in parallel
  - T008-T015 must run in sequence (modify same file)
- **User Story 4**:
  - T034, T035, T036 can run in parallel (different files)
  - T038, T039 can run in parallel (same file, different sections)
  - T041, T042 can run in parallel (independent tests)
- **User Story 2**: Tasks T044, T045 can run in parallel (verification tasks)
- **User Story 3**: Tasks T052, T053 can run in parallel (documentation)
- **Phase 7**: Most polish tasks can run in parallel (T056-T060 marked [P])

---

## Parallel Example: User Story 1

```bash
# Sequential WASM work:
Task T008: Modify constructor signature
Task T009: Replace FlatMemory with MappedMemory
Task T010-T015: Build memory map, add devices
Task T016: Rebuild WASM

# After T016 rebuild, launch in parallel:
Task T017-T022: Create terminal component (different file from app.js)
Task T008-T015: (already done)

# After terminal component done:
Task T023-T029: App integration (sequential, modifies app.js)
Task T030-T033: Manual testing (sequential verification)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (verify baseline)
2. Complete Phase 2: Foundational (add xterm.js CDN, HTML, CSS)
3. Complete Phase 3: User Story 1 (WASM + terminal + app integration)
4. **STOP and VALIDATE**: Test echo manually per T030-T033
5. **DECISION POINT**: MVP is done! Ship this or continue to examples?

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently (T030-T033) → **DEPLOY MVP** ✅
3. Add User Story 4 → Test independently (T040-T042) → Deploy with examples
4. Add User Story 2 → Test independently (T043-T045) → Deploy with visibility
5. Add User Story 3 → Test independently (T048-T055) → Deploy with controls
6. Polish (Phase 7) → Final release

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (CORE - highest priority)
   - After US1 complete:
     - Developer B: User Story 4 (examples)
     - Developer C: User Story 2 (visibility verification)
     - Developer D: User Story 3 (terminal controls)
3. Stories integrate cleanly since US4, US2, US3 all build on US1

---

## Task Summary

**Total Tasks**: 65
- Setup: 3 tasks
- Foundational: 4 tasks (CRITICAL - blocks all stories)
- User Story 1 (P1 - MVP): 26 tasks
- User Story 4 (P2): 9 tasks
- User Story 2 (P2): 5 tasks
- User Story 3 (P3): 8 tasks
- Polish: 10 tasks

**Parallel Opportunities**: 18 tasks marked [P]

**MVP Scope**: Phase 1 + Phase 2 + Phase 3 (User Story 1) = 33 tasks

**Suggested First Delivery**: Complete through T033 (User Story 1 manual testing), validate, then decide whether to continue

---

## Notes

- [P] tasks = different files, no dependencies, can run in parallel
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Stop at any checkpoint to validate story independently
- US1 is the MVP - everything else enhances it
- Existing UART device (src/devices/uart.rs) requires NO changes
- Memory map: $0000-$7FFF RAM, $A000-$A003 UART, $C000-$FFFF ROM
- xterm.js 5.5.0 loaded via CDN, no build tools required
- Manual browser testing per spec - no automated test framework needed
