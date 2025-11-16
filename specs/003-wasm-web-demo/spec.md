# Feature Specification: Interactive 6502 Assembly Web Demo

**Feature Branch**: `003-wasm-web-demo`
**Created**: 2025-11-16
**Status**: Draft
**Input**: User description: "Create a spec for a demo website that will show off lib6502. I'd like the homepage to have an area to write assembly code (left side) and the machine will run on the right side with visibility of registers, buttons to step through and/or run it, and other essential tools. We will have to prepare the lib6502 rust project for use in WebAssembly as a precursor and design a simple HTML/CSS/JS website that can be deployed to the github site's for the repo. I want it to be simple, nicely designed, minimal, and easy to use and understand. There will be more expansive demos, so this site isn't supposed to show off everything, but it is supposed to be a place where you can write and run 6502 assembly code."

## User Scenarios & Testing

### User Story 1 - Write and Execute Assembly Code (Priority: P1)

A developer or 6502 enthusiast visits the demo site to write simple 6502 assembly code and immediately see it execute in the emulator. They can type assembly instructions on the left side and watch the CPU state change on the right side as the code runs.

**Why this priority**: This is the core value proposition - allowing users to experience the 6502 emulator without installation. Without this, the demo has no purpose.

**Independent Test**: Can be fully tested by loading the page, typing assembly code (e.g., `LDA #$42`), clicking run, and observing the accumulator register change to $42. Delivers immediate hands-on experience with the emulator.

**Acceptance Scenarios**:

1. **Given** the demo page is loaded, **When** user types valid assembly code in the left editor, **Then** the code is displayed with syntax highlighting
2. **Given** valid assembly code is entered, **When** user clicks the "Run" button, **Then** the code executes and registers update in real-time
3. **Given** code is executing, **When** the program completes, **Then** the final CPU state is visible in the register display

---

### User Story 2 - Step Through Code Execution (Priority: P2)

A user wants to understand how their assembly code executes instruction-by-instruction. They can step through each instruction and observe how registers and flags change at each step, helping them learn or debug their code.

**Why this priority**: Step-through debugging is essential for education and understanding CPU behavior, but the demo is still valuable without it (just less instructive).

**Independent Test**: Can be fully tested by entering a multi-instruction program, clicking "Step" repeatedly, and verifying that only one instruction executes per click while registers update. Delivers educational value for understanding instruction-level execution.

**Acceptance Scenarios**:

1. **Given** assembly code is loaded into the emulator, **When** user clicks "Step" button, **Then** exactly one instruction executes and PC advances accordingly
2. **Given** code is being stepped through, **When** user observes register display, **Then** only the registers affected by the current instruction change
3. **Given** mid-execution, **When** user clicks "Reset", **Then** all registers return to initial state and PC returns to start

---

### User Story 3 - View CPU State in Real-Time (Priority: P1)

A user needs to see the current state of the 6502 CPU while code executes. The right panel displays all registers (A, X, Y, PC, SP), processor flags (N, V, D, I, Z, C), and current cycle count in a clear, readable format.

**Why this priority**: Without visible CPU state, users cannot understand what the emulator is doing. This is fundamental to the demo's educational and demonstration value.

**Independent Test**: Can be fully tested by running any program and verifying that all CPU registers and flags are displayed and update correctly. Delivers transparency into emulator operation.

**Acceptance Scenarios**:

1. **Given** the page is loaded, **When** user views the right panel, **Then** all CPU registers show their initial values (typically $00)
2. **Given** code is executing, **When** an instruction modifies a register, **Then** the register value updates immediately in the display
3. **Given** an instruction affects processor flags, **When** the instruction executes, **Then** affected flags change state (e.g., Z flag sets when result is zero)

---

### User Story 4 - Load Example Programs (Priority: P3)

A new user wants to quickly see the emulator in action without writing code from scratch. The demo provides 2-3 simple example programs (e.g., "Hello Counter", "Fibonacci") that can be loaded with one click.

**Why this priority**: Examples improve initial user experience and reduce friction, but the demo is still functional without them. Users can write their own code.

**Independent Test**: Can be fully tested by clicking an example program button and verifying the editor populates with example code that runs successfully. Delivers quick-start capability for new users.

**Acceptance Scenarios**:

1. **Given** the demo page is loaded, **When** user clicks an example program button, **Then** the editor populates with the example's assembly code
2. **Given** an example is loaded, **When** user clicks "Run", **Then** the example executes successfully and produces expected output
3. **Given** an example is loaded, **When** user modifies the code, **Then** the modified version runs instead of the original

---

### User Story 5 - Inspect Memory Contents (Priority: P2)

A user wants to see what data is stored in memory as their program executes. They can view memory contents in a dedicated panel, navigate to specific addresses, and observe how instructions modify memory (e.g., stores, stack operations).

**Why this priority**: Memory visibility is crucial for understanding program behavior, especially for learning about stack operations, data storage, and pointer arithmetic. However, basic code execution is still valuable without it.

**Independent Test**: Can be fully tested by running a program that stores values to memory (e.g., `STA $1000`), opening the memory viewer, navigating to the address, and verifying the value is displayed correctly. Delivers insight into memory state and program data flow.

**Acceptance Scenarios**:

1. **Given** the demo page is loaded, **When** user views the memory panel, **Then** memory contents are displayed in a readable hexadecimal format
2. **Given** a program writes to memory (e.g., `STA $1000`), **When** the instruction executes, **Then** the memory viewer updates to show the new value at that address
3. **Given** user wants to inspect a specific address, **When** user navigates to or jumps to that address in the memory viewer, **Then** the viewer scrolls to and highlights that memory location

---

### User Story 6 - Access the Demo via GitHub Pages (Priority: P1)

A potential user or contributor finds the lib6502 GitHub repository and wants to try the emulator. They click a demo link that takes them to a GitHub Pages site where the interactive demo loads instantly without installation or setup.

**Why this priority**: Distribution is critical - without GitHub Pages deployment, the demo cannot reach users. This is infrastructure, not a feature, but it's required for any value delivery.

**Independent Test**: Can be fully tested by accessing the GitHub Pages URL and verifying the page loads with full functionality. Delivers zero-friction access to the demo.

**Acceptance Scenarios**:

1. **Given** a user navigates to the GitHub Pages URL, **When** the page loads, **Then** the complete demo interface is visible and functional
2. **Given** the demo is hosted on GitHub Pages, **When** a user accesses it from any modern browser, **Then** the WebAssembly module loads and executes correctly
3. **Given** the repository is updated, **When** changes are pushed to the designated branch, **Then** GitHub Pages automatically rebuilds and deploys the updated demo

---

### Edge Cases

- What happens when user enters invalid assembly syntax (e.g., `LDA #$XYZ`)? System should display a clear error message without crashing.
- What happens when a program enters an infinite loop? System should provide a "Stop" button to halt execution and a cycle counter to detect runaway programs.
- What happens when user tries to execute an empty program? System should either prevent execution or display a message indicating no code to run.
- What happens when user rapidly clicks "Step" many times? System should queue or debounce inputs to prevent UI freezing.
- What happens when the browser doesn't support WebAssembly? System should display a clear message that the browser is unsupported.
- What happens when user tries to access memory outside the 64KB address space? The emulator wraps addresses (standard 6502 behavior).
- What happens when user wants to inspect a specific memory address? System should provide a memory viewer that displays memory contents in a readable format.
- What happens when memory values change during execution? The memory viewer should update to reflect changes, potentially highlighting modified bytes.

## Requirements

### Functional Requirements

#### Core Emulation Features

- **FR-001**: System MUST compile the lib6502 Rust library to WebAssembly and expose a JavaScript API for CPU control (step, run, reset, read registers)
- **FR-002**: System MUST provide a text editor on the left side of the interface where users can type 6502 assembly code
- **FR-003**: System MUST assemble user-provided assembly code into 6502 machine code before execution
- **FR-004**: System MUST display all CPU registers (A, X, Y, PC, SP) with their current hexadecimal values
- **FR-005**: System MUST display all processor status flags (N, V, D, I, Z, C) as binary or boolean indicators
- **FR-006**: System MUST display the current cycle count during program execution
- **FR-007**: System MUST provide a "Run" button that executes the entire program until completion or halt
- **FR-008**: System MUST provide a "Step" button that executes exactly one instruction and updates display
- **FR-009**: System MUST provide a "Reset" button that restores CPU to initial state and reloads the program
- **FR-010**: System MUST provide a "Stop" button to halt a running program mid-execution
- **FR-010a**: System MUST provide selectable execution speeds including authentic 6502 clock rates (0.5 MHz, 1 MHz, 1.79 MHz, 2 MHz, and unlimited)
- **FR-010b**: System MUST default to 1 MHz execution speed (authentic 6502 timing)
- **FR-010c**: System MUST maintain accurate timing for selected clock speed using cycle-accurate execution

#### User Interface Requirements

- **FR-011**: Interface MUST use a split-panel layout with assembly editor on the left and CPU state display on the right
- **FR-012**: Assembly editor MUST provide basic syntax highlighting for 6502 mnemonics and operands
- **FR-013**: CPU register display MUST update in real-time as instructions execute
- **FR-014**: Interface MUST be responsive and functional on desktop browsers (mobile support optional)
- **FR-015**: Design MUST be minimal and clean, prioritizing readability and ease of use over visual complexity
- **FR-016**: System MUST provide 2-3 example programs that users can load with one click
- **FR-017**: System MUST provide a memory viewer that displays memory contents in hexadecimal format
- **FR-018**: Memory viewer MUST allow users to navigate to specific memory addresses
- **FR-019**: Memory viewer MUST update in real-time when memory contents change during execution
- **FR-020**: Memory viewer SHOULD highlight or indicate recently modified memory locations

#### Deployment Requirements

- **FR-021**: Website MUST be deployable to GitHub Pages as static HTML/CSS/JavaScript files
- **FR-022**: System MUST load and initialize without requiring any external server or backend services
- **FR-023**: WebAssembly module MUST be bundled with the static site assets
- **FR-024**: System MUST display a clear error message if the browser does not support WebAssembly

#### Error Handling

- **FR-025**: System MUST validate assembly code syntax before execution and display clear error messages for invalid syntax
- **FR-026**: System MUST handle and display errors gracefully without crashing the browser tab
- **FR-027**: System MUST detect and handle infinite loops (e.g., via iteration limit or stop button)

### Key Entities

- **Assembly Program**: User-written 6502 assembly code as text, includes labels, mnemonics, operands, and comments
- **CPU State**: Current values of all registers (A, X, Y, PC, SP), flags (N, V, D, I, Z, C), and cycle count
- **Example Program**: Pre-written assembly code demonstrating common patterns (loops, arithmetic, etc.)
- **WebAssembly Module**: Compiled lib6502 emulator with JavaScript bindings for CPU control and state inspection

## Success Criteria

### Measurable Outcomes

- **SC-001**: Users can write, assemble, and execute a simple 6502 program (e.g., load a value into A register) in under 30 seconds from page load
- **SC-002**: Step-through execution advances exactly one instruction per click with visible register updates in under 100ms
- **SC-003**: The demo site loads completely and becomes interactive in under 3 seconds on a standard broadband connection
- **SC-004**: Example programs execute correctly and produce expected register states 100% of the time
- **SC-005**: The demo works correctly in the 3 major browser engines (Chromium, Firefox, Safari/WebKit)
- **SC-006**: Users can halt a running infinite loop program within 1 second of clicking "Stop"
- **SC-007**: Assembly syntax errors are detected and reported to the user before execution begins
- **SC-008**: The demo provides a clear, understandable view of CPU state that requires no additional documentation to interpret
- **SC-009**: Users can navigate to any memory address and view its contents within 2 seconds
- **SC-010**: Memory changes during execution are visible in the memory viewer with clear indication of modified locations
- **SC-011**: At 1 MHz speed setting, the emulator executes 1,000,000 cycles per second with less than 5% timing variance
- **SC-012**: Users can switch between speed presets during execution without disrupting program state

## Scope

### In Scope

- WebAssembly compilation of lib6502 core emulator
- Basic assembler functionality to convert assembly text to machine code
- Interactive web interface with editor and CPU state display
- Step and run execution modes
- Reset and stop controls
- 2-3 example programs
- GitHub Pages deployment configuration
- Basic syntax highlighting in the editor
- Error handling for invalid assembly and execution errors
- Memory viewer with navigation and real-time updates
- Visual indication of modified memory locations

### Out of Scope

- Advanced debugging features (breakpoints, watchpoints, conditional breaks) - deferred to future expansive demos
- Disassembly view showing machine code as assembly
- Persistent storage of user programs (localStorage, cloud save)
- Sharing programs via URL or export
- Advanced editor features (autocomplete, intellisense, themes)
- Mobile-optimized layout and touch controls
- Multiple execution speed controls or animation of execution
- Comprehensive tutorial or documentation embedded in the demo
- Support for assembler directives beyond basic labels
- Integration with external tools or APIs

## Assumptions

- Modern browser support: Target browsers support WebAssembly (released 2017+)
- Single-user experience: No collaboration or multi-user features needed
- Educational context: Users have basic familiarity with assembly language concepts
- The existing lib6502 assembler/disassembler (feature 002) will be available for integration
- Static hosting is sufficient: No server-side execution or storage required
- Example programs will be under 50 lines each to maintain simplicity
- Initial memory state is all zeros (standard 6502 reset state)
- Programs are loaded at address $0600 (standard monitor convention)

## Dependencies

- Completion of assembler/disassembler feature (002-assembler-disassembler) for assembly-to-machine-code conversion
- Rust toolchain with `wasm32-unknown-unknown` target support
- `wasm-bindgen` or similar tool for generating JavaScript bindings (dev dependency only)
- GitHub Pages enabled on the repository
- Modern text editor component for assembly code input (could be textarea or lightweight library)

## Resolved Questions

### Execution Speed (RESOLVED)

**Decision**: Run mode executes at authentic 6502 clock speeds with user-selectable speed presets.

**Default Speed**: 1 MHz (1,000,000 cycles/second)
- Matches original Apple II, Commodore 64, Atari 2600
- Provides authentic retro computing experience
- At 60fps: ~16,667 cycles per frame

**Selectable Speed Presets**:
1. **0.5 MHz** - "Slow" (educational, easier to observe)
2. **1.0 MHz** - "1 MHz (Authentic)" (default, original 6502 speed)
3. **1.79 MHz** - "1.79 MHz (NES/C64 PAL)" (PAL region systems)
4. **2.0 MHz** - "2 MHz (Apple IIgs)" (faster variant)
5. **Unlimited** - "Maximum" (run as fast as browser allows)

**Implementation**: Speed selector dropdown in UI controls. JavaScript calculates cycles per frame based on selected speed and maintains timing via requestAnimationFrame.

**Rationale**:
- Authentic speeds provide educational value (experience real 6502 timing)
- Slower speeds help users observe execution step-by-step
- Unlimited speed useful for quickly running longer programs
- Multiple presets accommodate different learning styles and use cases

---

## Open Questions

- Program size limits: What is the maximum reasonable program size for the editor? (Assumption: limit to ~200 lines to maintain simplicity)
- Memory viewer format: Should memory be displayed as a scrollable hex dump, paged view, or with configurable address ranges? (Current plan: scrollable hex dump with virtual scrolling)
