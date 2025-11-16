# Implementation Tasks: Interactive 6502 Assembly Web Demo

**Feature Branch**: `003-wasm-web-demo`
**Generated**: 2025-11-16
**Based on**: [spec.md](./spec.md), [plan.md](./plan.md)

## Overview

This task list implements the interactive 6502 assembly web demo in dependency order, organized by user story priorities. Each phase represents a complete, independently testable increment of functionality.

## Task Organization

- **[P]**: Parallelizable tasks (no dependencies on incomplete work)
- **[US#]**: User Story label (maps to priorities in spec.md)
- Tasks are listed in execution order within each phase

## Phase 1: Setup & Infrastructure

**Goal**: Establish project structure, WASM toolchain, and deployment pipeline

- [X] T001 Install WASM toolchain: `rustup target add wasm32-unknown-unknown` and `cargo install wasm-pack`
- [X] T002 Add wasm-bindgen dependency to Cargo.toml: `wasm-bindgen = "0.2"` (optional feature)
- [X] T003 Create WASM module directory structure: `src/wasm/` with mod.rs and api.rs
- [X] T004 [P] Create demo directory structure: `demo/` with subdirectories `components/`, `examples/`, `lib6502_wasm/` (gitignore lib6502_wasm/)
- [X] T005 [P] Create base HTML file: `demo/index.html` with split-panel layout and component containers
- [X] T006 [P] Create base CSS file: `demo/styles.css` with Oxide-inspired dark theme, Sixtyfour + JetBrains Mono fonts
- [X] T007 [P] Create GitHub Actions workflow: `.github/workflows/deploy-demo.yml` for automated WASM build and GitHub Pages deployment
- [X] T008 Verify WASM build: run `wasm-pack build --target web --out-dir demo/lib6502_wasm` and check output

**Phase Complete When**: WASM toolchain installed, directory structure created, build pipeline verified

---

## Phase 2: Foundational WASM API (Blocking Prerequisites)

**Goal**: Implement core WASM bindings exposing CPU control and state access

**Independent Test**: Can create Emulator6502 instance, execute step(), and read registers from JavaScript console

### WASM Module Structure

- [X] T009 Implement src/wasm/mod.rs: Module exports and public re-exports
- [X] T010 WASM uses FlatMemory directly (no separate wrapper needed)

### Core WASM API (src/wasm/api.rs)

- [X] T011 Implement Emulator6502 struct with constructor wrapping CPU<FlatMemory>
- [X] T012 [P] Implement step() method returning Result<(), JsError>
- [X] T013 [P] Implement run_for_cycles(cycles: u32) returning Result<u32, JsError>
- [X] T014 [P] Implement reset() method
- [X] T015 [P] Implement register getters: get_a(), get_x(), get_y(), get_pc(), get_sp(), get_cycles()
- [X] T016 [P] Implement flag getters: get_flag_n(), get_flag_v(), get_flag_d(), get_flag_i(), get_flag_z(), get_flag_c()
- [X] T017 [P] Implement memory access methods: read_memory(addr), write_memory(addr, value), get_memory_page(page)
- [X] T018 [P] Implement load_program(program: &[u8], start_addr: u16)

### Assembly & Disassembly Integration

- [X] T019 Implement assemble(source: String, start_addr: u16) returning AssemblyResult (uses feature 002 assembler)
- [X] T020 Implement assemble_and_load(source: String, start_addr: u16) convenience method
- [X] T021 [P] Implement disassemble(start_addr: u16, num_instructions: u32) returning Vec<DisassemblyLine>

### Validation

- [X] T022 Build WASM module: `wasm-pack build --target web --out-dir demo/lib6502_wasm`
- [ ] T023 Create manual test HTML: Verify WASM API from browser console (create emulator, load program, step, read registers)

**Phase Complete When**: WASM API contract fully implemented, browser console testing confirms all methods work

---

## Phase 3: User Story 6 - GitHub Pages Deployment (P1)

**Goal**: Enable users to access demo via GitHub Pages URL

**Why First**: Infrastructure requirement - enables testing of subsequent user stories in deployed environment

**Independent Test**: Navigate to GitHub Pages URL, verify page loads with basic UI structure visible

### Deployment Configuration

- [ ] T024 [US6] Configure GitHub Pages in repository settings: Enable Pages, set source to main branch /demo directory
- [ ] T025 [US6] Update .github/workflows/deploy-demo.yml: Add WASM build step and Pages deployment step
- [ ] T026 [US6] Create demo/index.html skeleton: Basic structure with title, empty panels for editor and CPU state
- [ ] T027 [US6] Add WASM module loading to demo/index.html: Import init and Emulator6502 from lib6502_wasm.js
- [ ] T028 [US6] Implement basic WASM initialization in demo/app.js: await init(), create emulator instance
- [ ] T029 [US6] Test local deployment: Run `python3 -m http.server 8000 -d demo/` and verify page loads
- [ ] T030 [US6] Push to GitHub and verify Pages deployment: Check workflow runs successfully

**Phase Complete When**: GitHub Pages URL loads demo page, WASM module initializes without errors

**Acceptance Criteria (US6)**:
- ✅ Page loads at GitHub Pages URL with full interface visible
- ✅ WebAssembly module loads correctly in modern browsers
- ✅ GitHub Actions automatically rebuilds and deploys on push

---

## Phase 4: User Story 3 - View CPU State (P1)

**Goal**: Display real-time CPU registers, flags, and cycle count

**Why Second**: Core visibility feature - users need to see CPU state to understand execution

**Independent Test**: Open page, observe initial CPU state (all zeros), manually call `emu.step()` from console, verify UI updates

### UI Components

- [ ] T031 [P] [US3] Implement RegisterDisplay component in demo/components/registers.js: Display A, X, Y, PC, SP with 2/4-digit hex formatting
- [ ] T032 [P] [US3] Implement FlagsDisplay component in demo/components/flags.js: Display N, V, D, I, Z, C as boolean indicators
- [ ] T033 [US3] Add cycle counter display to RegisterDisplay: Show cycles with thousands separators
- [ ] T034 [US3] Create CSS styling for registers and flags in demo/styles.css: Monospace font, clear labels, visual indicators for flags

### Integration

- [ ] T035 [US3] Add register/flag containers to demo/index.html: Right panel with divs for registers and flags
- [ ] T036 [US3] Implement updateCPUDisplay() in demo/app.js: Fetch all registers/flags from WASM, call component update methods
- [ ] T037 [US3] Add requestAnimationFrame update loop in demo/app.js: Call updateCPUDisplay() at 60fps
- [ ] T038 [US3] Test CPU state display: Load simple program, step through, verify registers update correctly

**Phase Complete When**: CPU state displays show initial values, update in real-time when program executes

**Acceptance Criteria (US3)**:
- ✅ All CPU registers display initial values on page load
- ✅ Register values update immediately when instructions execute
- ✅ Processor flags change state when affected by instructions

---

## Phase 5: User Story 1 - Write and Execute Assembly Code (P1)

**Goal**: Users can write assembly code, assemble it, and run it to see CPU state changes

**Why Third**: Core value proposition - complete write → assemble → execute → observe workflow

**Independent Test**: Type assembly code in editor, click Assemble (program loads), click Run (executes), verify registers show expected values

### Editor Component

- [ ] T039 [P] [US1] Implement CodeEditor component in demo/components/editor.js: Textarea with getValue() and setValue() methods
- [ ] T040 [P] [US1] Implement Asm6502Highlighter in demo/components/editor.js: Regex-based syntax highlighting for mnemonics, hex, comments, labels
- [ ] T041 [US1] Add syntax highlighting CSS to demo/styles.css: Colors for instructions, operands, comments, labels
- [ ] T042 [US1] Integrate editor into demo/index.html: Left panel with textarea and highlighted pre element

### Control Panel Component

- [ ] T043 [P] [US1] Implement ControlPanel component in demo/components/controls.js: Assemble, Run, Step, Stop, Reset buttons
- [ ] T044 [P] [US1] Implement button state management in ControlPanel: setMode() to enable/disable based on execution state
- [ ] T045 [US1] Add control panel to demo/index.html: Below editor with button group
- [ ] T046 [US1] Style control panel in demo/styles.css: Clear buttons with hover states

### Assembly Workflow

- [ ] T047 [US1] Implement handleAssemble() in demo/app.js: Get code from editor, call emu.assemble_and_load(), handle errors
- [ ] T048 [P] [US1] Implement ErrorDisplay component in demo/components/error.js: Show assembly errors with line numbers
- [ ] T049 [US1] Implement assembled program persistence: Store assembled state, prevent re-assembly until code changes or reset
- [ ] T050 [US1] Wire Assemble button event: Listen for 'assemble-clicked', call handleAssemble()

### Execution Workflow

- [ ] T051 [US1] Implement handleRun() in demo/app.js: Set mode to 'running', call emu.run_for_cycles() in animation loop
- [ ] T052 [US1] Implement program completion detection: Check if PC reached end of code range or BRK instruction
- [ ] T053 [US1] Wire Run button event: Listen for 'run-clicked', call handleRun()
- [ ] T054 [US1] Update UI during execution: Call updateCPUDisplay() every frame while running

### Validation

- [ ] T055 [US1] Test write-assemble-run workflow: Type `LDA #$42`, click Assemble, click Run, verify A=0x42
- [ ] T056 [US1] Test assembly error handling: Type invalid code, click Assemble, verify error displays with line number
- [ ] T057 [US1] Test program persistence: Assemble once, click Run multiple times, verify no re-assembly

**Phase Complete When**: Complete workflow from typing code → assembling → running → observing state works end-to-end

**Acceptance Criteria (US1)**:
- ✅ User can type assembly code with syntax highlighting
- ✅ Clicking Assemble loads program into memory without executing
- ✅ Clicking Run after Assemble executes code and updates registers
- ✅ Program completion is detected and execution stops

---

## Phase 6: User Story 2 - Step Through Code (P2)

**Goal**: Users can execute code instruction-by-instruction and observe register changes per step

**Independent Test**: Assemble multi-instruction program, click Step repeatedly, verify exactly one instruction executes per click and PC advances

### Step Execution

- [ ] T058 [US2] Implement handleStep() in demo/app.js: Call emu.step() once, update display, handle errors
- [ ] T059 [US2] Wire Step button event: Listen for 'step-clicked', call handleStep()
- [ ] T060 [US2] Implement step debouncing in ControlPanel: Prevent rapid clicks during step execution (<100ms)
- [ ] T061 [US2] Update button states during step: Disable Step briefly during execution, re-enable when complete

### Reset Functionality

- [ ] T062 [US2] Implement handleReset() in demo/app.js: Call emu.reset(), set PC to program start, update display
- [ ] T063 [US2] Wire Reset button event: Listen for 'reset-clicked', call handleReset()
- [ ] T064 [US2] Test reset behavior: Step through program, click Reset, verify all registers return to initial state and PC returns to start

### Validation

- [ ] T065 [US2] Test step-through execution: Load multi-instruction program (5+ instructions), step through completely, verify PC increments correctly
- [ ] T066 [US2] Test per-instruction register changes: Use program that modifies different registers, step and observe only affected register changes

**Phase Complete When**: Step execution works reliably, exactly one instruction per click, Reset restores initial state

**Acceptance Criteria (US2)**:
- ✅ Clicking Step executes exactly one instruction and advances PC
- ✅ Only registers affected by current instruction change
- ✅ Reset button restores all registers to initial state and PC to start

---

## Phase 7: User Story 5 - Inspect Memory Contents (P2)

**Goal**: Users can view memory contents in hex dump format, navigate to addresses, and see real-time updates

**Independent Test**: Run program that stores value to memory (e.g., `STA $1000`), open memory viewer, navigate to $1000, verify value displayed

### Memory Viewer Component

- [ ] T067 [P] [US5] Implement MemoryViewer component in demo/components/memory.js: Virtual scrolling with 16 bytes/row, 4096 rows total
- [ ] T068 [P] [US5] Implement virtual scroll rendering in MemoryViewer: Render only visible rows (~25), update on scroll
- [ ] T069 [P] [US5] Implement memory row rendering: Address (4-digit hex) + 16 hex bytes + ASCII column
- [ ] T070 [P] [US5] Add memory viewer container to demo/index.html: Right panel, stacked below registers/flags or side-by-side
- [ ] T071 [P] [US5] Style memory viewer in demo/styles.css: Monospace, clear columns, scrollable with fixed height

### Memory Navigation

- [ ] T072 [US5] Implement jumpToAddress(addr) in MemoryViewer: Scroll to row containing address, highlight briefly
- [ ] T073 [US5] Add address jump input to memory viewer UI: Text input for hex address, button to jump
- [ ] T074 [US5] Wire address jump: Listen for input, parse hex, call jumpToAddress()

### Real-Time Updates

- [ ] T075 [US5] Implement dirty byte tracking in MemoryViewer: Compare new memory with cached, mark changed bytes
- [ ] T076 [US5] Implement updateMemory() in demo/app.js: Fetch visible memory pages from WASM, pass to MemoryViewer
- [ ] T077 [US5] Add memory viewer to update loop: Call updateMemory() every frame (60fps), only fetch visible pages
- [ ] T078 [US5] Implement changed byte highlighting: CSS flash animation on dirty bytes, 1s fade-out
- [ ] T079 [US5] Style dirty byte highlighting in demo/styles.css: Red/orange background flash

### Validation

- [ ] T080 [US5] Test memory display: Load program at $0600, verify hex dump shows program bytes
- [ ] T081 [US5] Test memory updates: Run `STA $1000` instruction, verify $1000 updates and highlights
- [ ] T082 [US5] Test address navigation: Jump to $0200, $FFFF, verify scroll and highlight work
- [ ] T083 [US5] Test virtual scrolling performance: Scroll through entire 64KB range, verify 60fps maintained

**Phase Complete When**: Memory viewer displays all 64KB, updates in real-time, navigation works, performance smooth

**Acceptance Criteria (US5)**:
- ✅ Memory contents displayed in readable hexadecimal format
- ✅ Memory viewer updates when instruction writes to memory
- ✅ User can navigate to specific addresses via input
- ✅ Virtual scrolling maintains 60fps performance

---

## Phase 8: User Story 4 - Load Example Programs (P3)

**Goal**: Users can load pre-written example programs with one click for quick start

**Independent Test**: Click example program button, verify editor populates with example code, click Run, verify example executes successfully

### Example Programs

- [ ] T084 [P] [US4] Create counter example: `demo/examples/counter.asm` - Simple counter from 0 to 255 in accumulator
- [ ] T085 [P] [US4] Create Fibonacci example: `demo/examples/fibonacci.asm` - Calculate Fibonacci sequence
- [ ] T086 [P] [US4] Create stack demo example: `demo/examples/stack-demo.asm` - Demonstrate PHA/PLA stack operations
- [ ] T087 [P] [US4] Add example program metadata: Define EXAMPLES array in demo/app.js with id, name, description, source

### Example Selector Component

- [ ] T088 [US4] Implement ExampleSelector component in demo/components/examples.js: Dropdown with example programs
- [ ] T089 [US4] Add example selector to demo/index.html: Above or within editor panel
- [ ] T090 [US4] Wire example loading: Listen for 'example-loaded' event, call editor.setValue(), reset emulator
- [ ] T091 [US4] Test example loading: Click each example, verify code loads into editor

### Validation

- [ ] T092 [US4] Test counter example execution: Load counter, run for 1000 cycles, verify A increments
- [ ] T093 [US4] Test Fibonacci example execution: Load Fibonacci, step through, verify correct sequence in registers
- [ ] T094 [US4] Test stack demo execution: Load stack demo, step through, observe SP changes and memory in stack page

**Phase Complete When**: All example programs load correctly and execute successfully

**Acceptance Criteria (US4)**:
- ✅ Clicking example button populates editor with example code
- ✅ Examples execute successfully and produce expected output
- ✅ User can modify example code and run modified version

---

## Phase 9: Execution Speed Controls & Timing

**Goal**: Provide selectable execution speeds including authentic 6502 clock rates

**Independent Test**: Change speed dropdown to different settings (0.5 MHz, 1 MHz, 2 MHz, unlimited), run program, verify timing differences visible

### Speed Selector

- [ ] T095 Add speed selector dropdown to ControlPanel: Options for 0.5, 1.0, 1.79, 2.0 MHz, unlimited
- [ ] T096 Implement getSpeed() and setSpeed() in ControlPanel: Return current speed, update dropdown
- [ ] T097 Wire speed change event: Listen for 'speed-changed', calculate cycles per frame
- [ ] T098 Implement cycle-accurate timing in handleRun(): Calculate target cycles based on speed and 60fps, call run_for_cycles()

### Timing Implementation

- [ ] T099 Add calculateCyclesPerFrame() utility: Convert MHz to cycles per frame at 60fps
- [ ] T100 Default to 1 MHz on page load: Set initial speed selector to 1 MHz
- [ ] T101 Allow speed changes during execution: Update cycles per frame without disrupting program state
- [ ] T102 Test timing accuracy: Run at 1 MHz for 1 second, verify ~1,000,000 cycles executed (±5%)

### Validation

- [ ] T103 Test 0.5 MHz (slow): Verify execution is visibly slower than 1 MHz
- [ ] T104 Test 1 MHz (default): Verify authentic timing matches spec (16,667 cycles/frame at 60fps)
- [ ] T105 Test unlimited speed: Verify program executes as fast as possible (100,000+ cycles/frame)
- [ ] T106 Test speed switching during execution: Run program, change speed mid-execution, verify no crash

**Phase Complete When**: All speed settings work correctly, timing accuracy meets ±5% tolerance

**Acceptance Criteria (SC-010a, SC-010b, SC-010c, SC-011, SC-012)**:
- ✅ System provides speed presets (0.5, 1, 1.79, 2 MHz, unlimited)
- ✅ Default speed is 1 MHz
- ✅ Timing accuracy within 5% variance at 1 MHz
- ✅ User can switch speeds during execution without disrupting program state

---

## Phase 10: Polish & Cross-Cutting Concerns

**Goal**: Final polish, error handling, edge cases, performance optimization

### Error Handling

- [ ] T107 Implement runtime error display: Catch execution errors, show error modal with PC and opcode
- [ ] T108 Handle empty program execution: Disable Run/Step if no program assembled, show message
- [ ] T109 Handle browser compatibility: Display clear message if WebAssembly not supported
- [ ] T110 Test unimplemented opcode error: Execute unimplemented instruction, verify error displays

### Performance Optimization

- [ ] T111 Optimize memory viewer updates: Only re-render changed bytes, not entire visible window
- [ ] T112 Implement Stop button functionality: Halt run loop, return to idle mode
- [ ] T113 Add FPS counter (debug mode): Display actual frame rate for performance monitoring
- [ ] T114 Profile performance: Use browser DevTools, ensure 60fps maintained during Run mode

### UI Polish

- [ ] T115 Add keyboard shortcuts: Space=Step, R=Run, S=Stop, Esc=Reset
- [ ] T116 Improve visual feedback: Button press animations, loading states
- [ ] T117 Add help/documentation panel: Brief instructions for using the demo
- [ ] T118 Responsive layout adjustments: Ensure panels stack appropriately on smaller screens (optional mobile support)

### Documentation

- [ ] T119 Add code comments: Document all components, explain non-obvious logic
- [ ] T120 Create README for demo/: Explain how to build and deploy
- [ ] T121 Update main project README: Add link to live demo, screenshot

### Final Validation

- [ ] T122 Cross-browser testing: Verify demo works in Chrome, Firefox, Safari
- [ ] T123 Load performance test: Verify demo loads in <3s on broadband
- [ ] T124 Stress test: Run infinite loop program, verify Stop button halts within 1s
- [ ] T125 Full end-to-end test: Complete all user stories' acceptance scenarios

**Phase Complete When**: All success criteria met, demo is production-ready

**Acceptance Criteria (All Success Criteria)**:
- ✅ SC-001: Write and execute simple program in <30s from page load
- ✅ SC-002: Step execution advances in <100ms with visible updates
- ✅ SC-003: Demo loads completely in <3s on broadband
- ✅ SC-004: Example programs execute correctly 100% of the time
- ✅ SC-005: Demo works in Chrome, Firefox, Safari
- ✅ SC-006: Stop button halts infinite loop within 1s
- ✅ SC-007: Assembly errors detected and reported before execution
- ✅ SC-008: CPU state view is clear without additional documentation
- ✅ SC-009: Navigate to any memory address within 2s
- ✅ SC-010: Memory changes visible with indication of modified locations

---

## Dependency Graph

```
Phase 1 (Setup)
  ↓
Phase 2 (WASM API) ← blocking prerequisite for all user stories
  ↓
Phase 3 (US6: Deployment) ← infrastructure for testing subsequent phases
  ↓
Phase 4 (US3: CPU State Display) ← needed to observe US1 execution
  ↓
Phase 5 (US1: Write & Execute) ← core workflow
  ↓
Phase 6 (US2: Step Through) ← builds on US1
  ↓
Phase 7 (US5: Memory Viewer) ← independent of US2
  ↓
Phase 8 (US4: Examples) ← independent, can be done anytime after US1
  ↓
Phase 9 (Speed Controls) ← enhancement to US1 Run mode
  ↓
Phase 10 (Polish) ← final refinements
```

**User Story Completion Order**: US6 (Deploy) → US3 (State) → US1 (Execute) → US2 (Step) → US5 (Memory) → US4 (Examples)

**Priority Delivery**: After Phase 5 completes, US1 (P1) core value proposition is fully delivered

---

## Parallel Execution Opportunities

### Within Phase 2 (WASM API):
- T012-T021 can be implemented in parallel (different API method groups)

### Within Phase 3 (Deployment):
- T026 (HTML), T027 (WASM import), T028 (JS init) can be done in parallel

### Within Phase 4 (CPU State):
- T031 (RegisterDisplay) and T032 (FlagsDisplay) can be built in parallel

### Within Phase 5 (Write & Execute):
- T039-T042 (Editor), T043-T046 (Controls), T048 (ErrorDisplay) can be built in parallel
- T040 (Highlighter) independent of T039 (Editor base)

### Within Phase 7 (Memory Viewer):
- T067-T071 (MemoryViewer component) can be built in parallel with other phases

### Within Phase 8 (Examples):
- T084-T086 (all example programs) can be written in parallel

---

## Implementation Strategy

### MVP Scope (Minimum Viable Product)
**Phases 1-5** deliver the core value proposition:
- Users can write assembly code
- Assemble and execute programs
- Observe CPU state changes in real-time
- Deploy to GitHub Pages for public access

**Estimated MVP Timeline**: ~3-5 days for experienced developer

### Full Feature Scope
**Phases 1-10** deliver complete specification:
- All P1, P2, P3 user stories
- Example programs
- Speed controls
- Memory inspection
- Full polish and error handling

**Estimated Full Timeline**: ~7-10 days for experienced developer

---

## Task Summary

**Total Tasks**: 125
- Phase 1 (Setup): 8 tasks
- Phase 2 (WASM API): 15 tasks
- Phase 3 (US6 Deployment): 7 tasks
- Phase 4 (US3 CPU State): 8 tasks
- Phase 5 (US1 Execute): 19 tasks
- Phase 6 (US2 Step): 9 tasks
- Phase 7 (US5 Memory): 17 tasks
- Phase 8 (US4 Examples): 11 tasks
- Phase 9 (Speed Controls): 12 tasks
- Phase 10 (Polish): 19 tasks

**Parallelizable Tasks**: 41 marked with [P]

**User Story Coverage**:
- US1 (Write & Execute): 19 tasks
- US2 (Step Through): 9 tasks
- US3 (View CPU State): 8 tasks
- US4 (Load Examples): 11 tasks
- US5 (Inspect Memory): 17 tasks
- US6 (GitHub Pages): 7 tasks
- Infrastructure/Polish: 54 tasks

**Estimated Completion**: 7-10 days (1 developer), 4-6 days (2 developers with parallelization)
