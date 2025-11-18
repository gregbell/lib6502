# Feature Specification: CPU Interrupt Support

**Feature Branch**: `005-cpu-interrupt-support`
**Created**: 2025-11-18
**Status**: Draft
**Input**: User description: "add true interrupt support to the CPU. I can imagine either a promise based solution where a device passes a promise in when it triggers an interrupt and then resolves it once it's done after being called by the CPU or some type of game/event loop. It would be awesome if you could write a device in JavaScript if required."

## Clarifications

### Session 2025-11-18

- Q: Should the interrupt mechanism mimic real 6502 hardware behavior (level-sensitive IRQ line, no queueing) or implement a modern queued interrupt controller? → A: Level-sensitive like real hardware: Single IRQ line, no queue. Multiple devices can pull IRQ low simultaneously. ISR polls device status registers to identify interrupt sources.
- Q: How should devices be notified that their interrupt is being serviced? → A: Automatic notification: CPU automatically notifies all devices with pending interrupts when entering ISR (simplified but not hardware-accurate).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - External Device Signals CPU (Priority: P1)

An external device (such as a timer, UART serial port, or keyboard controller) needs to notify the CPU that it requires attention. The device signals an interrupt, and the CPU responds by executing the appropriate interrupt handler at the next available opportunity.

**Why this priority**: This is the fundamental capability that enables all interrupt-driven I/O. Without this, the emulator can only support polling-based I/O, which doesn't accurately reflect how real 6502 systems worked.

**Independent Test**: Can be fully tested by creating a simple device that signals an interrupt after a timer expires, and verifying the CPU executes the interrupt vector and delivers value by enabling realistic device emulation.

**Acceptance Scenarios**:

1. **Given** a device is connected to the CPU and an interrupt handler is registered, **When** the device signals an interrupt, **Then** the CPU executes the interrupt handler at the next instruction boundary
2. **Given** the interrupt disable flag (I flag) is set, **When** a device signals an interrupt, **Then** the CPU does not execute the interrupt handler until the I flag is cleared
3. **Given** an interrupt has been signaled, **When** the CPU begins executing the interrupt handler, **Then** the device receives notification that its interrupt is being handled

---

### User Story 2 - Multiple Device Interrupt Coordination (Priority: P2)

Multiple devices connected to the CPU may signal interrupts independently. The system must handle these interrupts in a predictable order and ensure each device's interrupt is processed.

**Why this priority**: Real systems have multiple interrupt sources. This enables emulating realistic systems with timers, UART, keyboard, and other peripherals all operating simultaneously.

**Independent Test**: Can be tested by connecting multiple test devices, having them signal interrupts in a known sequence, and verifying each interrupt is handled correctly.

**Acceptance Scenarios**:

1. **Given** multiple devices assert their interrupt request simultaneously, **When** the CPU processes the interrupt, **Then** the IRQ line remains active until all devices clear their interrupt requests
2. **Given** a device asserts an interrupt while the CPU is handling another interrupt, **When** the first interrupt handler completes and clears the I flag, **Then** the CPU immediately re-enters the interrupt handler if the IRQ line is still active
3. **Given** multiple devices have asserted interrupt requests, **When** the interrupt handler executes, **Then** the handler can poll device status registers to identify which devices triggered interrupts

---

### User Story 3 - Cross-Language Device Support (Priority: P3)

A developer writes a device emulator in JavaScript that needs to signal interrupts to the Rust-based CPU when running in a WASM environment. The device should be able to trigger interrupts and receive notification when they are handled.

**Why this priority**: This enables the web demo to have interactive devices written in JavaScript, making the emulator more accessible and easier to extend for educational purposes.

**Independent Test**: Can be tested by creating a JavaScript-based timer device in the WASM demo that signals periodic interrupts and logs when they are handled.

**Acceptance Scenarios**:

1. **Given** a JavaScript device is connected to the WASM CPU, **When** the device asserts its interrupt request, **Then** the CPU processes the interrupt identically to native devices
2. **Given** the CPU begins handling an interrupt from a JavaScript device, **When** the interrupt handler executes, **Then** the JavaScript device receives acknowledgment notification
3. **Given** a JavaScript device needs to assert an interrupt, **When** the device uses the provided interface, **Then** the interrupt request is registered without blocking JavaScript execution

---

### Edge Cases

- What happens when an interrupt is signaled but no interrupt handler is registered?
- How does the system handle an interrupt signaled during the execution of an interrupt handler (nested interrupts)?
- What happens when a device tries to signal an interrupt after it has been disconnected?
- How does the system handle extremely high interrupt rates that could starve normal program execution?
- What happens when the interrupt vector in memory points to invalid code?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The CPU MUST check for pending interrupts at instruction boundaries (after each instruction completes)
- **FR-002**: Devices MUST be able to signal interrupts to the CPU through a defined interface
- **FR-003**: The CPU MUST respect the interrupt disable flag (I flag) and only process interrupts when I flag is clear
- **FR-004**: The CPU MUST execute interrupt handlers by reading the interrupt vector from memory locations 0xFFFE-0xFFFF (IRQ vector)
- **FR-005**: The CPU MUST automatically notify all devices with active interrupt requests when entering the interrupt service routine
- **FR-006**: The system MUST support multiple independent interrupt sources
- **FR-007**: The system MUST implement a level-sensitive IRQ line that remains active while any device has an unserviced interrupt request (multiple devices share the IRQ line via logical OR)
- **FR-008**: The CPU MUST save processor state (program counter and status flags) on the stack when entering an interrupt handler
- **FR-009**: The CPU MUST set the interrupt disable flag when entering an interrupt handler to prevent nested interrupts (unless explicitly re-enabled)
- **FR-010**: The interrupt mechanism MUST work across language boundaries in WASM (JavaScript devices → Rust CPU)
- **FR-011**: The interrupt system MUST not block normal CPU execution when no interrupts are pending (zero overhead when idle)
- **FR-012**: Devices MUST be able to determine if their interrupt was successfully delivered

### Key Entities

- **IRQ Line**: Level-sensitive signal line shared by all devices; active (low) when any device has an unserviced interrupt request
- **Device Interrupt State**: Each device maintains its own interrupt request flag that contributes to the shared IRQ line state
- **Interrupt Handler**: Code executed by the CPU in response to an active IRQ line, identified by the interrupt vector in memory
- **Device Interface**: Abstract mechanism through which devices assert/clear their interrupt request and receive acknowledgment from the CPU, independent of device implementation language

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Devices can successfully trigger interrupts and the CPU processes them within one instruction cycle after the interrupt is signaled (when I flag is clear)
- **SC-002**: The system correctly handles at least 10 different interrupt sources operating simultaneously
- **SC-003**: JavaScript-based devices in the WASM demo can trigger interrupts with the same reliability as native Rust devices (100% delivery rate)
- **SC-004**: The interrupt overhead when no interrupts are pending is unmeasurable (no performance degradation)
- **SC-005**: All interrupt-driven 6502 test programs execute correctly with cycle-accurate timing

## Assumptions

- Interrupt handling follows the standard 6502 IRQ (Interrupt Request) behavior as documented in the MOS 6502 Programming Manual
- The interrupt vector is read from memory addresses 0xFFFE (low byte) and 0xFFFF (high byte) as per 6502 specification
- The IRQ line is level-sensitive (active low) matching real 6502 hardware; no interrupt priority or queueing mechanism needed
- The BRK instruction's existing interrupt behavior (if implemented) will be preserved and is compatible with hardware interrupts
- Devices are responsible for clearing their own interrupt request flags; the ISR must acknowledge the device to clear the IRQ line
- The system does not need to support NMI (Non-Maskable Interrupt) in this initial implementation

## Out of Scope

- NMI (Non-Maskable Interrupt) support - this feature focuses only on maskable IRQ interrupts
- Interrupt priority levels or queueing - the system uses a simple level-sensitive IRQ line like real hardware
- Interrupt coalescing or batching optimizations
- Hardware-specific interrupt controllers (e.g., VIC, PIA) - only the basic CPU IRQ line mechanism
- Debugging or tracing tools for interrupt behavior
