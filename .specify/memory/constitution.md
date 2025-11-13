<!--
SYNC IMPACT REPORT
==================
Version Change: None → 1.0.0 (Initial Constitution)
Date: 2025-11-13

Principles Established:
- I. Modularity & Separation of Concerns (NEW)
- II. WebAssembly Portability (NEW)
- III. Cycle Accuracy (NEW)
- IV. Clarity & Hackability (NEW)
- V. Table-Driven Design (NEW)

Sections Added:
- Core Principles (5 principles)
- Testing & Validation
- Future Expansion
- Governance

Templates Status:
✅ plan-template.md - Compatible (constitution check section ready)
✅ spec-template.md - Compatible (technology-agnostic requirements align)
✅ tasks-template.md - Compatible (supports modular task organization)

Follow-up TODOs:
- None (all placeholders filled)

Rationale:
- Initial constitution establishing core architectural and quality principles
- Based on project vision for portable, modular 6502 CPU core
- Emphasizes WASM compatibility, cycle accuracy, and developer accessibility
-->

# 6502 CPU Core Constitution

## Core Principles

### I. Modularity & Separation of Concerns

The CPU core MUST be fully decoupled from any specific machine implementation. All memory access MUST go through a trait-based bus abstraction with zero hardware assumptions baked into the core. This enables the CPU to be embedded in any fantasy console, emulator, or computing platform without modification.

**Rationale**: A generic, reusable CPU core maximizes project utility and enables diverse embeddings—from flat 64KB RAM to complex NES-style memory maps—without touching CPU logic. This separation also enables debugging layers, logging, and breakpoints to be injected at the bus level.

**Rules**:
- No direct memory arrays or buffers in CPU implementation
- All reads/writes through generic trait interface
- CPU state structure contains only registers, flags, PC, SP, cycle counter, interrupt state
- No OS-level features or platform-specific code in core module

### II. WebAssembly Portability

The CPU core MUST be designed from the ground up for WebAssembly compatibility. Pure Rust implementation with no OS dependencies, heavy runtime requirements, or platform-specific features. The core relies exclusively on simple, deterministic computation that compiles cleanly to WASM.

**Rationale**: Browser-native execution is a first-class target, enabling fantasy console experiences that run anywhere without installation. WASM portability also ensures the core remains lightweight and portable to embedded systems, mobile platforms, and native desktop applications.

**Rules**:
- No `std::fs`, `std::net`, `std::process`, or OS syscalls in core
- No threading or async runtime dependencies in core
- All core dependencies must be `no_std` compatible or WASM-proven
- Deterministic execution with no reliance on system time or randomness

### III. Cycle Accuracy

The CPU MUST implement cycle-accurate timing where it matters: correct cycle totals, page-crossing penalties, branch timing, and interrupt service costs. The implementation MUST match the timing behavior expected by real 6502 software while remaining straightforward to understand and maintain.

**Rationale**: Timing-sensitive programs (games, demos, music players) depend on accurate cycle counts. Fantasy consoles benefit from predictable frame timing. Cycle accuracy is non-negotiable for authenticity, but implementation complexity must be justified—prefer instruction-level accuracy over micro-op models unless timing demands it.

**Rules**:
- Every instruction executes with documented cycle cost
- Page-crossing penalties accounted for (indexed addressing, indirect indexed)
- Branch taken/not-taken timing correct
- Interrupt service cycle costs match hardware behavior
- Flexible clocking: support running CPU for fixed cycle budget per frame
- Document any deviations from hardware timing with rationale

### IV. Clarity & Hackability

The codebase MUST prioritize readability and hackability for Rust developers. Code should be self-explanatory, well-documented, and easy to extend. Avoid clever optimizations that obscure logic. Prefer clear structure over minimal lines of code.

**Rationale**: The project serves educational purposes, hobbyist development, and creative coding. Developers should be able to read the CPU implementation, understand how the 6502 works, and confidently add features or fix bugs. Accessibility matters more than squeezing out the last 5% of performance.

**Rules**:
- Public APIs have clear doc comments with examples
- Internal implementation uses descriptive names and inline comments for non-obvious behavior
- Prefer explicit code over implicit magic (avoid excessive macros or metaprogramming)
- When in doubt, optimize for maintainability over raw performance
- Code review must verify newcomers can understand changes

### V. Table-Driven Design

The instruction decoder MUST use a table-driven approach mapping all 256 opcodes to mnemonic, addressing mode, cycle cost, and instruction size. Avoid duplication of decode logic across the implementation.

**Rationale**: The 6502 has 256 opcodes with significant overlap in addressing modes. Naive switch-statement decoders lead to massive duplication and maintenance burden. A table-driven approach centralizes opcode metadata, making it easier to audit completeness, verify cycle accuracy, and extend the implementation (e.g., adding undocumented opcodes).

**Rules**:
- Single source of truth for opcode metadata (table or const data structure)
- Decode logic references table, does not duplicate mode/cycle/size information
- Table must cover all 256 opcodes (document illegal/undocumented opcodes explicitly)
- Adding new instructions requires updating table only, not scattered match arms

## Testing & Validation

All CPU core changes MUST include verification that instruction execution remains correct. Test coverage expectations:

- **Instruction tests**: Each opcode must have at least one test case verifying correct register/flag/memory state after execution
- **Cycle accuracy tests**: Critical timing-sensitive instruction sequences must verify cycle counts
- **Integration tests**: Full programs (e.g., Klaus Dormann's 6502 functional test) must pass to validate end-to-end behavior
- **Regression tests**: Bugs fixed must have corresponding test cases to prevent reoccurrence

Test-first development is RECOMMENDED but not mandatory. Tests must be deterministic and run in WASM environment.

## Future Expansion

The CPU core is intended as the foundation for a larger fantasy console ecosystem. Anticipated future components:

- Memory-mapped graphics subsystem
- Audio synthesis/chip emulation
- Controller/keyboard input handling
- Assembler and debugging tools
- Sprite editors and asset pipelines
- Browser UI and runtime environment

All future additions MUST preserve CPU core modularity. The CPU should remain independently usable and testable without requiring graphics, audio, or other subsystems.

## Governance

This constitution establishes the architectural and quality standards for the 6502 CPU core project. All code contributions, design decisions, and feature additions must align with these principles.

**Amendment Process**:
- Proposed changes to constitution must include rationale and impact analysis
- Breaking changes to core principles require project maintainer approval
- Constitution updates increment version per semantic versioning (see below)

**Compliance Review**:
- All PRs must verify alignment with principles (especially Modularity, WASM Portability, Cycle Accuracy)
- Complexity introduced must be justified against Clarity & Hackability principle
- Violations flagged in review must be resolved or explicitly justified

**Versioning Policy**:
- MAJOR: Backward incompatible principle changes or removals
- MINOR: New principles added or expanded guidance
- PATCH: Clarifications, wording improvements, non-semantic fixes

**Version**: 1.0.0 | **Ratified**: 2025-11-13 | **Last Amended**: 2025-11-13
