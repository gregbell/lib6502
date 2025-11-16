# Feature Specification: Assembler & Disassembler

**Feature Branch**: `002-assembler-disassembler`
**Created**: 2025-11-14
**Status**: Draft
**Input**: User description: "I want to spec out an assembler / disassembler for this project. Again, it is something that must compile nicely to web assembly and can be used as a library. Make sure to research all the code we already have for the assembler too"

## Clarifications

### Session 2025-11-14

- Q: When assembling "$80" with a mnemonic that supports both zero-page and absolute addressing, which mode should be preferred? → A: Prefer zero-page for "$80" when both modes available; require "$0080" for explicit absolute
- Q: What format should be used for illegal/undocumented opcodes during disassembly: ".byte $XX" or "??? $XX"? → A: Always use ".byte $XX" for illegal opcodes (re-assemblable, standard directive format)
- Q: What are the validation rules for label names (character set, length limits, format)? → A: Alphanumeric + underscore, must start with letter, max 32 chars
- Q: What number formats should be supported in assembly operands (hex, decimal, binary, octal)? → A: Hex ($XX), decimal, binary (%XXXXXXXX) only - common in modern 6502 assemblers like ca65
- Q: What granularity should the source-to-binary mapping use (byte-level, instruction-level, or line-level)? → A: Instruction-level mapping (each instruction maps to source line/column range) - balanced for debugging needs

## User Scenarios & Testing _(mandatory)_

### User Story 1 - Disassemble Binary to Assembly (Priority: P1)

As a developer debugging a 6502 program, I need to convert raw binary machine code into human-readable assembly mnemonics with operands so that I can understand what instructions are executing.

**Why this priority**: Disassembly is the most fundamental debugging tool. Without it, developers cannot inspect what their CPU is executing. This is essential for any emulator or debugging workflow and delivers immediate value independently.

**Independent Test**: Can be fully tested by providing a byte array containing known 6502 opcodes and verifying the disassembler outputs the correct mnemonic and operand representation for each instruction.

**Acceptance Scenarios**:

1. **Given** a byte sequence containing `0xA9 0x42` (LDA #$42), **When** disassembling, **Then** output is "LDA #$42"
2. **Given** a byte sequence containing `0x8D 0x00 0x80` (STA $8000), **When** disassembling, **Then** output is "STA $8000"
3. **Given** a byte sequence containing multiple instructions, **When** disassembling, **Then** each instruction is correctly decoded with proper operand formatting
4. **Given** a byte sequence containing an illegal opcode, **When** disassembling, **Then** output uses ".byte $XX" format to indicate the unknown opcode
5. **Given** a starting address offset, **When** disassembling, **Then** addresses in output reflect the correct memory locations

---

### User Story 2 - Assemble Text to Binary (Priority: P2)

As a developer writing 6502 programs, I need to convert assembly language source code (mnemonics and operands) into executable binary machine code so that I can load and run programs on the emulator.

**Why this priority**: Assembling is required to create executable programs from human-readable source. While essential, it depends on having established the disassembly format and instruction encoding rules first. Still independently testable and valuable.

**Independent Test**: Can be fully tested by providing assembly source text and verifying the assembler produces the exact byte sequence that would execute correctly on the CPU.

**Acceptance Scenarios**:

1. **Given** assembly source "LDA #$42", **When** assembling, **Then** output bytes are `[0xA9, 0x42]`
2. **Given** assembly source "STA $8000", **When** assembling, **Then** output bytes are `[0x8D, 0x00, 0x80]`
3. **Given** multi-line assembly source, **When** assembling, **Then** all instructions are correctly encoded in sequence
4. **Given** assembly with whitespace and capitalization variations, **When** assembling, **Then** parsing is case-insensitive and whitespace-tolerant
5. **Given** assembly using different number formats (e.g., "LDA #$42", "LDA #66", "LDA #%01000010"), **When** assembling, **Then** all three assemble to identical output bytes `[0xA9, 0x42]`
6. **Given** assembly with an invalid mnemonic, **When** assembling, **Then** a clear error is reported with line and column information
7. **Given** assembly with multiple syntax errors, **When** assembling, **Then** all errors are collected and reported (not just the first error)

---

### User Story 3 - Support Symbolic Labels (Priority: P3)

As a developer writing assembly programs, I need to use symbolic labels for addresses (e.g., "LOOP:", "START:") so that I can write maintainable code without hard-coding memory addresses.

**Why this priority**: Labels dramatically improve assembly code readability and maintainability. They're a standard feature of any practical assembler. However, basic instruction assembly must work first.

**Independent Test**: Can be fully tested by assembling code containing label definitions and references, verifying the assembler correctly resolves label addresses and encodes branch/jump targets.

**Acceptance Scenarios**:

1. **Given** assembly with a label definition "START:" and reference "JMP START", **When** assembling, **Then** the JMP instruction correctly encodes the address of the START label
2. **Given** assembly with a forward reference (label used before defined), **When** assembling, **Then** the assembler performs multi-pass resolution and correctly encodes the address
3. **Given** assembly with relative branch to label (e.g., "BEQ LOOP"), **When** assembling, **Then** the correct signed offset is calculated and encoded
4. **Given** assembly with an undefined label reference, **When** assembling, **Then** an error is reported indicating the undefined label
5. **Given** assembly with duplicate label definitions, **When** assembling, **Then** an error is reported indicating the duplicate
6. **Given** assembly with an invalid label (e.g., starts with digit, contains invalid characters, exceeds 32 chars), **When** assembling, **Then** an error is reported with specific validation failure details

---

### User Story 4 - Hexadecimal Dump Formatting (Priority: P4)

As a developer inspecting memory contents, I need to view disassembled code alongside hexadecimal byte representations and memory addresses so that I can correlate assembly instructions with their binary encoding.

**Why this priority**: Hex dump formatting enhances debugging by showing the relationship between assembly and machine code. It's a quality-of-life improvement over bare disassembly but not strictly necessary for core functionality.

**Independent Test**: Can be fully tested by disassembling a known byte sequence and verifying the output includes formatted addresses, hex bytes, and assembly mnemonics in aligned columns.

**Acceptance Scenarios**:

1. **Given** a disassembly starting at address $8000, **When** formatting as hex dump, **Then** output shows "8000: A9 42 LDA #$42"
2. **Given** instructions of varying byte lengths, **When** formatting as hex dump, **Then** hex bytes are left-aligned and mnemonics align in a consistent column
3. **Given** a multi-line disassembly, **When** formatting as hex dump, **Then** addresses increment correctly for each instruction

---

### User Story 5 - Comments and Directives (Priority: P5)

As a developer writing assembly programs, I need to include comments (starting with `;`) and assembler directives (e.g., `.org` for origin address) so that I can document my code and control assembly behavior.

**Why this priority**: Comments and directives are standard assembler features that improve developer experience. They're lower priority because core assembly functionality must work first.

**Independent Test**: Can be fully tested by assembling code containing comments and directives, verifying comments are ignored and directives correctly affect assembly behavior (e.g., `.org $8000` sets the starting address).

**Acceptance Scenarios**:

1. **Given** assembly source with `;` comments, **When** assembling, **Then** comments are ignored and don't affect output
2. **Given** assembly with `.org $8000` directive, **When** assembling, **Then** subsequent instructions are encoded as if starting at address $8000
3. **Given** assembly with `.byte $01, $02` directive, **When** assembling, **Then** literal bytes $01, $02 are inserted into output
4. **Given** an invalid directive, **When** assembling, **Then** a clear error is reported

---

### User Story 6 - Structured Output for IDE Integration (Priority: P2)

As a web IDE developer, I need the assembler and disassembler to return structured data with source mappings and comprehensive error information so that I can build rich editing features like syntax highlighting, error squiggles, and debugging support.

**Why this priority**: This ensures the library is suitable for building a web-based IDE from the start, rather than requiring a rewrite later. It's P2 because it must be designed in alongside basic assembly (P2) to avoid one-way door decisions.

**Independent Test**: Can be fully tested by assembling code with errors and verifying the returned data structure contains all errors with line/column/span information, plus source maps linking bytes to source locations.

**Acceptance Scenarios**:

1. **Given** assembly source with multiple errors, **When** assembling, **Then** all errors are returned with line, column, and character span information
2. **Given** successfully assembled code, **When** querying the source map by instruction address, **Then** the corresponding source line and column range is returned
3. **Given** successfully assembled code, **When** querying by source line, **Then** the corresponding byte address range for all instructions on that line is returned (for breakpoint support)
4. **Given** successfully assembled code, **When** accessing the symbol table, **Then** all label names and resolved addresses are available for queries
5. **Given** disassembled output, **When** accessed programmatically, **Then** structured data (not just formatted text) is available for each instruction

---

### Edge Cases

- What happens when disassembling at an address that doesn't align with instruction boundaries? (May disassemble incorrectly; document as expected behavior)
- What happens when assembling a branch instruction with a target too far for 8-bit relative addressing? (Report error indicating branch out of range)
- What happens when assembling a program that exceeds 64KB? (Report error indicating address overflow)
- What happens when disassembling illegal/undocumented opcodes? (Display as ".byte $XX" with the raw byte value, maintaining re-assemblability since `.byte` is a supported directive)
- What happens with ambiguous addressing mode syntax (e.g., "$80" could be zero-page or absolute)? (Always prefer zero-page mode for "$80" when the mnemonic supports both zero-page and absolute addressing; require explicit "$0080" syntax to force absolute mode. This matches common assembler conventions and optimizes for smaller, faster zero-page instructions.)
- What happens when a label is too long or contains invalid characters? (Report error indicating validation failure; labels must contain only alphanumeric characters and underscores, must start with a letter [a-zA-Z], and cannot exceed 32 characters in length)

## Requirements _(mandatory)_

### Functional Requirements

- **FR-001**: System MUST provide a disassembler function that accepts a byte slice and starting address, returning structured data containing both the disassembled instructions and metadata (address, bytes, mnemonic, operands)
- **FR-002**: Disassembler MUST correctly decode all documented NMOS 6502 opcodes using the existing OPCODE_TABLE metadata
- **FR-003**: Disassembler MUST format operands according to addressing mode (e.g., "#$XX" for immediate, "$XXXX" for absolute, "($XX,X)" for indexed indirect)
- **FR-004**: Disassembler MUST handle illegal opcodes gracefully by representing them as ".byte $XX" directives (where XX is the hex value of the opcode), ensuring re-assemblability rather than failing
- **FR-005**: System MUST provide an assembler function that accepts assembly source text and returns either assembled output with metadata or a collection of structured errors
- **FR-006**: Assembler MUST parse standard 6502 assembly syntax including mnemonic, operands with various addressing modes, and whitespace, supporting hexadecimal ($XX), decimal (no prefix), and binary (%XXXXXXXX) number formats
- **FR-007**: Assembler MUST encode instructions correctly according to the OPCODE_TABLE, selecting the correct opcode byte for each mnemonic/addressing-mode combination
- **FR-008**: Assembler MUST support label definitions (identifier followed by colon) and label references in operands, validating that labels contain only alphanumeric characters and underscores, start with a letter [a-zA-Z], and do not exceed 32 characters in length
- **FR-009**: Assembler MUST perform multi-pass assembly to resolve forward label references
- **FR-010**: Assembler MUST calculate correct relative offsets for branch instructions when using label targets
- **FR-011**: Assembler MUST validate addressing mode operands (e.g., immediate values fit in 8 bits, zero-page addresses are $00-$FF) and when a mnemonic supports both zero-page and absolute modes with ambiguous syntax like "$80", MUST prefer zero-page mode (requiring explicit "$0080" syntax for absolute mode)
- **FR-012**: Assembler MUST collect and report ALL errors found during assembly (not halt on first error), with each error containing line number, column number, error span (start/end positions), error type, and descriptive message
- **FR-013**: Disassembler MUST support optional formatting as hex dump with addresses, hex bytes, and mnemonics
- **FR-014**: Assembler MUST support single-line comments starting with semicolon (`;`)
- **FR-015**: Assembler MUST support `.org` directive to set the starting address for assembled code
- **FR-016**: Assembler MUST support `.byte` directive to insert literal bytes into the output
- **FR-017**: System MUST be `no_std` compatible and compile to WebAssembly without OS dependencies
- **FR-018**: All public APIs MUST be usable as a library from other Rust code (not just command-line tools)
- **FR-019**: Assembler MUST track and provide source-to-binary mapping at instruction-level granularity, recording which source line and column range produced each assembled instruction (mapping from starting byte address of each instruction to its source location)
- **FR-020**: Assembler MUST provide access to the symbol table after assembly, allowing queries of label names and their resolved addresses
- **FR-021**: Assembler MUST support querying the address range for a given source line (for breakpoint support)
- **FR-022**: Disassembler MUST return structured instruction data that can be queried programmatically, not just formatted text strings

### Key Entities

- **Instruction**: Represents a single disassembled instruction containing the mnemonic, addressing mode, operands, byte length, and original address
- **AssemblyLine**: Represents a parsed line of assembly source containing the label (if any), mnemonic, operands, and comments
- **Symbol**: Represents a label name and its resolved address for symbol table management
- **AssemblerError**: Contains error type (syntax, undefined label, range error), line number, column number, span (start/end positions), and descriptive message
- **DisassemblyOptions**: Configuration for disassembly behavior (e.g., starting address, hex dump formatting)
- **AssemblerDirective**: Represents special assembler commands like `.org`, `.byte`, `.word`
- **SourceMap**: Maps assembled instruction addresses to source locations (line, column, span) at instruction-level granularity for debugger integration, allowing bidirectional queries between byte addresses and source positions
- **AssemblerOutput**: Contains assembled bytes, symbol table, source map, and any warnings or errors encountered

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: Disassembler correctly decodes 100% of documented 6502 opcodes from the OPCODE_TABLE
- **SC-002**: Assembler successfully assembles and round-trips (assemble → disassemble → re-assemble) produce identical byte output for all valid assembly code
- **SC-003**: Assembler correctly resolves forward label references in multi-pass assembly
- **SC-004**: Library compiles to WebAssembly with `wasm32-unknown-unknown` target without errors
- **SC-005**: All assembler errors include line numbers, column numbers, character spans, and human-readable descriptions
- **SC-006**: Disassembler performance allows processing at least 10,000 bytes per millisecond on typical hardware (fast enough for real-time debugging)
- **SC-007**: Assembler handles programs of at least 8KB without performance degradation
- **SC-008**: Documentation includes examples of using both assembler and disassembler as library APIs
- **SC-009**: Assembler collects and reports all errors in a single pass (not just the first error)
- **SC-010**: Source map enables bidirectional mapping between source lines and assembled byte addresses

## Assumptions _(mandatory)_

- **A-001**: Assembly syntax follows standard 6502 conventions (uppercase mnemonics, `$` prefix for hexadecimal, `%` prefix for binary, no prefix for decimal, `#` for immediate addressing mode, etc.)
- **A-002**: Disassembly returns structured data suitable for both programmatic use and human-readable text formatting
- **A-003**: The assembler is a multi-pass assembler (sufficient for forward label resolution without complex expression evaluation)
- **A-004**: Undocumented/illegal opcodes are not required to assemble, only disassemble
- **A-005**: Case-insensitive parsing is acceptable for mnemonics and labels (convert to uppercase internally)
- **A-006**: The assembler does not need to support macros, conditional assembly, or complex expressions in operands (future enhancement)
- **A-007**: Assembler performs error recovery to collect multiple errors in a single pass, continuing to parse after encountering errors when possible
- **A-008**: Disassembly of self-modifying code or non-instruction data will produce best-effort output (may be incorrect but won't crash)

## Dependencies _(optional)_

- **DEP-001**: Existing OPCODE_TABLE from `src/opcodes.rs` provides the single source of truth for instruction encoding/decoding
- **DEP-002**: Existing AddressingMode enum from `src/addressing.rs` defines all supported addressing modes

## Scope Boundaries _(optional)_

### In Scope

- Text-based assembly and disassembly
- Standard 6502 documented opcodes
- Labels and basic directives (`.org`, `.byte`)
- Comments in assembly source
- Error reporting with line numbers
- WebAssembly compatibility
- Library API for programmatic use

### Out of Scope

- Assembler macros or conditional assembly
- Complex expression evaluation in operands (e.g., `LDA #(CONSTANT+5)`)
- Linking multiple object files
- Undocumented/illegal opcode assembly (only disassembly)
- Integrated development environment (IDE) or syntax highlighting (but assembler provides structured data to support external IDE integration)
- Advanced debugger features like watch expressions or conditional breakpoints (but symbol table and source mapping support basic debugging)
- Binary file format support (raw binary only; no .prg, .nes, .obj formats)

## Architecture Alignment _(optional)_

This feature aligns with the project constitution:

- **Modularity**: Assembler and disassembler are independent library modules with clean APIs, usable separately or together
- **WebAssembly Portability**: No OS dependencies, pure Rust string/byte processing, compiles to WASM
- **Clarity & Hackability**: Assembly parsing and instruction encoding logic is straightforward, well-documented, and easy to extend
- **Table-Driven Design**: Leverages existing OPCODE_TABLE for encoding/decoding, avoiding duplicate instruction metadata

The assembler/disassembler does not directly impact cycle accuracy but supports development and testing of the CPU core by providing tools to write and inspect programs.
