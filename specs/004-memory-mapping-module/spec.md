# Feature Specification: Memory Mapping Module with UART Device Support

**Feature Branch**: `004-memory-mapping-module`
**Created**: 2025-11-17
**Status**: Draft
**Input**: User description: "A new memory mapping module/trait so that we can wire up multiple different emulated hardware devices to the read/write memory interface and one particular: a serial device via a UART. Ultimately, I'd like to be able to use iterm.js in the browser to have a terminal/serial connection to the machine. I think we should emulate the same thing that Ben Eater does in his RS232 interface with the 6551 UART video https://www.youtube.com/watch?v=zsERDRM1oy8"

## Clarifications

### Session 2025-11-17

- Q: What byte value should be returned when the CPU reads from an unmapped memory address? → A: Return $FF (255) for all unmapped reads

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Basic Memory-Mapped Device Architecture (Priority: P1)

An emulator developer wants to create a 6502 system with multiple memory-mapped hardware devices (RAM, ROM, I/O chips) where each device occupies specific address ranges. They need to configure which device handles reads and writes for different memory regions without modifying CPU core code.

**Why this priority**: This is the foundational capability. Without a working memory mapping system, no other features (UART, multiple devices, etc.) can function. This represents the minimum viable product.

**Independent Test**: Can be fully tested by creating a system with a 16KB RAM device mapped to $0000-$3FFF and a 16KB ROM device mapped to $C000-$FFFF, running a program that accesses both regions, and verifying correct routing.

**Acceptance Scenarios**:

1. **Given** a system configured with RAM at $0000-$3FFF and ROM at $C000-$FFFF, **When** the CPU reads from address $1234, **Then** the request is routed to the RAM device
2. **Given** a system configured with multiple devices, **When** the CPU writes to address $DEAD, **Then** the request is routed to the device responsible for that address range
3. **Given** a system with unmapped address regions, **When** the CPU reads from an unmapped address, **Then** the system returns $FF

---

### User Story 2 - 6551 UART Serial Device Emulation (Priority: P2)

An emulator developer wants to add a 6551 UART serial communication device to their 6502 system. They need the UART to be accessible via memory-mapped registers (data, status, command, control) at a configurable base address, and to send/receive bytes through a callback interface.

**Why this priority**: Builds on P1's memory mapping to add the first concrete I/O device. Represents a significant value add (serial communication) but depends on P1 being functional.

**Independent Test**: Can be tested by mapping a UART device to $5000-$5003, writing configuration bytes to control register, writing a data byte to transmit, polling status register for ready status, and verifying bytes are delivered through the callback interface.

**Acceptance Scenarios**:

1. **Given** a UART mapped to base address $5000, **When** 6502 code writes a byte to $5000 (data register), **Then** that byte is available for transmission through the serial output callback
2. **Given** a UART with data available from serial input, **When** 6502 code reads the status register at $5001, **Then** bit 3 (receiver data register full) is set to 1
3. **Given** a UART configured with specific baud rate and word format via control register ($5003), **When** 6502 code reads back the control register, **Then** the configured settings are preserved
4. **Given** a UART that just transmitted a byte, **When** 6502 code reads the status register at $5001, **Then** bit 4 (transmitter data register empty) is set to 1

---

### User Story 3 - Browser-Based Serial Terminal Connection (Priority: P3)

A user running the emulator in a web browser wants to interact with the emulated 6502 system through a terminal interface (like xterm.js or similar). Characters they type in the browser terminal should be received by the UART, and bytes transmitted by the UART should appear in the terminal display.

**Why this priority**: This is the end-user facing feature that makes serial communication useful, but it depends on both P1 (memory mapping) and P2 (UART device) being complete. It represents the full user experience goal.

**Independent Test**: Can be tested by opening the emulator webpage, typing characters in the browser terminal, running 6502 code that echoes received bytes back through UART transmit, and verifying the characters appear in the terminal display.

**Acceptance Scenarios**:

1. **Given** an emulator webpage with terminal display connected to UART, **When** user types the letter 'A' in the terminal, **Then** the UART receiver buffer contains byte $41 (ASCII 'A')
2. **Given** 6502 code that transmits the byte $48 (ASCII 'H') through the UART, **When** the byte is transmitted, **Then** the letter 'H' appears in the browser terminal display
3. **Given** a user typing rapidly in the terminal, **When** multiple characters are typed before 6502 code can read them, **Then** the characters are buffered and available for reading in order (FIFO behavior)

---

### Edge Cases

- **Unmapped memory reads**: When 6502 code reads from an address with no registered device, the system returns $FF (mimicking classic 6502 floating bus behavior). Writes to unmapped addresses are silently ignored.
- How does the system handle overlapping memory regions (two devices claiming the same address)?
- What happens when UART transmit buffer is full and 6502 code tries to write another byte?
- What happens when UART receive buffer is empty and 6502 code tries to read a byte?
- How does the system behave if the terminal connection is disconnected while the emulator is running?
- Can the memory mapping configuration be changed dynamically while the emulator is running, or must it be set once at initialization?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a mechanism to register multiple hardware device implementations with the memory bus
- **FR-002**: System MUST support configuring address range mappings (start address and size) for each registered device
- **FR-003**: System MUST route read operations to the appropriate device based on the target address
- **FR-004**: System MUST route write operations to the appropriate device based on the target address
- **FR-005**: System MUST return $FF when reading from unmapped memory addresses
- **FR-006**: System MUST support a 6551 UART device implementation with four memory-mapped registers (data, status, command, control)
- **FR-007**: UART data register (offset +0) MUST allow writing bytes for transmission and reading received bytes
- **FR-008**: UART status register (offset +1) MUST indicate transmitter ready status (bit 4) and receiver data available status (bit 3)
- **FR-009**: UART command register (offset +2) MUST allow configuring parity mode, echo mode, and interrupt enables
- **FR-010**: UART control register (offset +3) MUST allow configuring baud rate, word length, and stop bits
- **FR-011**: UART MUST provide a callback or event interface for delivering transmitted bytes to external consumers
- **FR-012**: UART MUST provide a method for external sources to inject received bytes into the receiver buffer
- **FR-013**: System MUST buffer received bytes when they arrive faster than 6502 code can process them
- **FR-014**: System MUST indicate buffer overflow conditions through status register flags
- **FR-015**: All device implementations MUST operate without OS-level I/O dependencies to maintain WebAssembly portability

### Key Entities

- **Memory-Mapped Device**: A hardware component (RAM, ROM, UART, etc.) that responds to read/write operations within a specific address range. Has configurable base address and size. Implements read and write behaviors specific to the device type.

- **Address Mapping**: A configuration that associates an address range (start address + size) with a specific device instance. Determines which device handles operations for addresses in that range.

- **UART Device**: A specific type of memory-mapped device implementing 6551 ACIA serial communication. Contains four registers (data, status, command, control), transmit buffer, receive buffer, and configuration state (baud rate, parity, word length).

- **Serial Data Buffer**: A queue structure for holding bytes awaiting transmission or bytes received but not yet read. Implements FIFO (first-in-first-out) ordering. Has maximum capacity and overflow detection.

- **Serial Terminal Bridge**: A connection layer that routes bytes between the UART device and a browser-based terminal interface. Handles bidirectional data flow and connection lifecycle events.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Developer can configure a 6502 system with at least 3 different memory-mapped devices (e.g., RAM, ROM, UART) each occupying distinct address ranges
- **SC-002**: 6502 code can successfully transmit and receive bytes through the UART device with 100% data integrity (no byte corruption or loss under normal operation)
- **SC-003**: Browser terminal interface displays transmitted characters within 100ms of 6502 transmission
- **SC-004**: User typing in browser terminal has characters available to 6502 code within 100ms of keystroke
- **SC-005**: System can handle serial data rates of at least 100 bytes per second in each direction without data loss
- **SC-006**: All device implementations compile to WebAssembly without requiring OS-level I/O APIs
- **SC-007**: Emulator with UART and terminal connection runs in modern web browsers (Chrome, Firefox, Safari, Edge) without requiring native plugins or extensions

## Assumptions

- **A-001**: The terminal interface in the browser will use a library like xterm.js or similar (user mentioned "iterm.js" which likely refers to a terminal emulation library)
- **A-002**: Serial communication will operate at emulated baud rates but actual data transfer between UART and browser terminal will be asynchronous and not bound by real-time constraints
- **A-003**: The 6551 UART emulation will focus on core functionality (data/status/command/control registers) and may omit advanced features like hardware flow control if not critical for initial use case
- **A-004**: The memory mapping system will be configured at system initialization time, not dynamically reconfigured during emulation (dynamic reconfiguration can be added later if needed)
- **A-005**: The system will follow the same 6551 ACIA register layout and behavior as demonstrated in Ben Eater's video (base address + 0/1/2/3 offsets for data/status/command/control)
- **A-006**: Received bytes will be buffered with a reasonable default buffer size (e.g., 256 bytes) to handle bursty input from the terminal
- **A-007**: The CPU core already provides the MemoryBus trait that this feature will extend or layer on top of

## Dependencies

- **D-001**: Existing CPU core implementation with MemoryBus trait (src/cpu.rs, src/memory.rs)
- **D-002**: WebAssembly build configuration and WASM-compatible development environment
- **D-003**: Browser-based terminal library (xterm.js or similar) for P3 browser terminal integration
- **D-004**: Project constitution principles must be maintained (Modularity, WASM Portability, Cycle Accuracy, Clarity & Hackability)

## Out of Scope

- **OS-001**: Actual hardware UART communication with physical serial ports (this is WebAssembly-focused emulation only)
- **OS-002**: Network-based serial connections (telnet, websocket pass-through to real serial devices)
- **OS-003**: Full 6551 ACIA hardware emulation including undocumented registers or manufacturer-specific quirks
- **OS-004**: Performance optimization for extremely high data rates (>1000 bytes/sec)
- **OS-005**: Other memory-mapped I/O devices beyond UART (graphics, sound, etc.) - those are future features
- **OS-006**: Dynamic memory map reconfiguration during emulation runtime
- **OS-007**: Interrupt-driven serial I/O (polling mode is sufficient for initial implementation)
