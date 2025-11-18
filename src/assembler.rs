//! 6502 Assembler Module
//!
//! Converts assembly language source code into binary machine code.

pub mod encoder;
pub mod parser;
pub mod source_map;
pub mod symbol_table;

use crate::opcodes;

/// Complete output from assembling source code
#[derive(Debug, Clone)]
pub struct AssemblerOutput {
    /// Assembled machine code bytes
    pub bytes: Vec<u8>,

    /// Symbol table with all defined labels
    pub symbol_table: Vec<Symbol>,

    /// Source map for debugging
    pub source_map: source_map::SourceMap,

    /// Non-fatal warnings encountered during assembly
    pub warnings: Vec<AssemblerWarning>,
}

impl AssemblerOutput {
    /// Look up a symbol by name
    pub fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        self.symbol_table.iter().find(|s| s.name == name)
    }

    /// Get symbol address by name
    ///
    /// This is a convenience method that returns just the address.
    /// Use `lookup_symbol()` if you need the full symbol information.
    ///
    /// # Returns
    ///
    /// * `Some(address)` if the symbol exists
    /// * `None` if the symbol is not found
    ///
    /// # Examples
    ///
    /// ```
    /// use lib6502::assembler::assemble;
    ///
    /// let source = "START:\n    LDA #$42";
    /// let output = assemble(source).unwrap();
    /// assert_eq!(output.lookup_symbol_addr("START"), Some(0));
    /// ```
    pub fn lookup_symbol_addr(&self, name: &str) -> Option<u16> {
        self.lookup_symbol(name).map(|symbol| symbol.value)
    }

    /// Get source location for a given instruction address
    pub fn get_source_location(&self, address: u16) -> Option<source_map::SourceLocation> {
        self.source_map.get_source_location(address)
    }

    /// Get address range for a given source line
    pub fn get_address_range(&self, line: usize) -> Option<source_map::AddressRange> {
        self.source_map.get_address_range(line)
    }
}

/// Symbol classification: label (memory address) or constant (literal value)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// Memory address (defined with ':' suffix)
    Label,

    /// Literal value (defined with '=' assignment)
    Constant,
}

/// A symbol table entry mapping a name to a value (address or constant)
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    /// Symbol name (case-sensitive after normalization)
    pub name: String,

    /// Value: memory address for labels, literal value for constants
    pub value: u16,

    /// Symbol classification (label or constant)
    pub kind: SymbolKind,

    /// Source line where symbol was defined
    pub defined_at: usize,
}

/// A non-fatal warning from the assembler
#[derive(Debug, Clone)]
pub struct AssemblerWarning {
    /// Line number where warning occurred
    pub line: usize,

    /// Warning message
    pub message: String,
}

/// Error type for label validation failures
#[derive(Debug, Clone, PartialEq)]
pub enum LabelError {
    /// Label starts with invalid character (must start with letter)
    InvalidStart(String),

    /// Label contains invalid characters (only alphanumeric and underscore allowed)
    InvalidCharacters(String),

    /// Label is too long (max 32 characters)
    TooLong(usize),
}

impl std::fmt::Display for LabelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LabelError::InvalidStart(msg) => write!(f, "{}", msg),
            LabelError::InvalidCharacters(msg) => write!(f, "{}", msg),
            LabelError::TooLong(len) => {
                write!(f, "label name too long ({} characters, max 32)", len)
            }
        }
    }
}

impl std::error::Error for LabelError {}

/// An error encountered during assembly
#[derive(Debug, Clone, PartialEq)]
pub struct AssemblerError {
    /// Error type classification
    pub error_type: ErrorType,

    /// Line number where error occurred (1-indexed)
    pub line: usize,

    /// Column number where error starts (0-indexed)
    pub column: usize,

    /// Character span (start, end) in the source line
    pub span: (usize, usize),

    /// Human-readable error message
    pub message: String,
}

/// Classification of assembly errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    /// Syntax error (invalid format, unexpected character)
    SyntaxError,

    /// Undefined label reference
    UndefinedLabel,

    /// Duplicate label definition
    DuplicateLabel,

    /// Invalid label name (too long, starts with digit, etc.)
    InvalidLabel,

    /// Invalid mnemonic (not a recognized instruction)
    InvalidMnemonic,

    /// Invalid operand format for addressing mode
    InvalidOperand,

    /// Operand value out of range (e.g., immediate value > 255, branch too far)
    RangeError,

    /// Invalid directive usage
    InvalidDirective,

    /// Undefined constant reference
    UndefinedConstant,

    /// Duplicate constant definition
    DuplicateConstant,

    /// Name collision (constant and label with same name)
    NameCollision,

    /// Invalid constant value (out of range, not literal)
    InvalidConstantValue,
}

impl std::fmt::Display for AssemblerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Line {}, Column {}: {} - {}",
            self.line,
            self.column,
            match self.error_type {
                ErrorType::SyntaxError => "Syntax Error",
                ErrorType::UndefinedLabel => "Undefined Label",
                ErrorType::DuplicateLabel => "Duplicate Label",
                ErrorType::InvalidLabel => "Invalid Label",
                ErrorType::InvalidMnemonic => "Invalid Mnemonic",
                ErrorType::InvalidOperand => "Invalid Operand",
                ErrorType::RangeError => "Range Error",
                ErrorType::InvalidDirective => "Invalid Directive",
                ErrorType::UndefinedConstant => "Undefined Constant",
                ErrorType::DuplicateConstant => "Duplicate Constant",
                ErrorType::NameCollision => "Name Collision",
                ErrorType::InvalidConstantValue => "Invalid Constant Value",
            },
            self.message
        )
    }
}

/// Assembler directive types
#[derive(Debug, Clone, PartialEq)]
pub enum AssemblerDirective {
    /// Set origin address (.org $XXXX)
    Origin { address: u16 },

    /// Insert literal bytes (.byte $XX, $YY, ...)
    Byte { values: Vec<u8> },

    /// Insert literal 16-bit words (.word $XXXX, $YYYY, ...)
    Word { values: Vec<u16> },
}

/// Helper to detect if a mnemonic is a branch instruction
fn is_branch_mnemonic(mnemonic: &str) -> bool {
    matches!(
        mnemonic,
        "BCC" | "BCS" | "BEQ" | "BMI" | "BNE" | "BPL" | "BVC" | "BVS"
    )
}

/// Assemble source code into machine code
///
/// # Arguments
///
/// * `source` - The assembly source code text
///
/// # Returns
///
/// Ok(AssemblerOutput) on success, Err(Vec<AssemblerError>) on failure
pub fn assemble(source: &str) -> Result<AssemblerOutput, Vec<AssemblerError>> {
    let mut errors = Vec::new();

    // Parse all lines
    let parsed_lines: Vec<_> = source
        .lines()
        .enumerate()
        .filter_map(|(idx, line)| parser::parse_line(line, idx + 1))
        .collect();

    // Pass 1: Build symbol table
    let mut symbol_table = symbol_table::SymbolTable::new();
    let mut current_address = 0u16;

    for line in &parsed_lines {
        // Check for label definition
        if let Some(ref label) = line.label {
            // Validate label name
            if let Err(e) = validate_label(label) {
                errors.push(AssemblerError {
                    error_type: ErrorType::InvalidLabel,
                    line: line.line_number,
                    column: 0,
                    span: line.span,
                    message: e.to_string(),
                });
            } else {
                // Add to symbol table
                if let Err(existing) = symbol_table.add_symbol(
                    label.clone(),
                    current_address,
                    SymbolKind::Label,
                    line.line_number,
                ) {
                    // Check if it's a collision with a different kind or a duplicate label
                    if existing.kind == SymbolKind::Label {
                        errors.push(AssemblerError {
                            error_type: ErrorType::DuplicateLabel,
                            line: line.line_number,
                            column: 0,
                            span: line.span,
                            message: format!(
                                "Duplicate label '{}' (previously defined at line {})",
                                label, existing.defined_at
                            ),
                        });
                    } else {
                        errors.push(AssemblerError {
                            error_type: ErrorType::NameCollision,
                            line: line.line_number,
                            column: 0,
                            span: line.span,
                            message: format!(
                                "Name collision: '{}' is already defined as a constant at line {}",
                                label, existing.defined_at
                            ),
                        });
                    }
                }
            }
        }

        // Check for constant assignment
        if let Some((ref name, ref value_str)) = line.constant {
            // Validate constant name (same rules as labels)
            if let Err(e) = validate_label(name) {
                errors.push(AssemblerError {
                    error_type: ErrorType::InvalidLabel,
                    line: line.line_number,
                    column: 0,
                    span: line.span,
                    message: format!("Invalid constant name: {}", e),
                });
            } else {
                // Parse the constant value
                match parser::parse_number(value_str) {
                    Ok(value) => {
                        // Add to symbol table with Constant kind
                        if let Err(existing) = symbol_table.add_symbol(
                            name.clone(),
                            value,
                            SymbolKind::Constant,
                            line.line_number,
                        ) {
                            // Check if it's a collision with a different kind or a duplicate constant
                            if existing.kind == SymbolKind::Constant {
                                errors.push(AssemblerError {
                                    error_type: ErrorType::DuplicateConstant,
                                    line: line.line_number,
                                    column: 0,
                                    span: line.span,
                                    message: format!(
                                        "Duplicate constant '{}' (previously defined at line {})",
                                        name, existing.defined_at
                                    ),
                                });
                            } else {
                                errors.push(AssemblerError {
                                    error_type: ErrorType::NameCollision,
                                    line: line.line_number,
                                    column: 0,
                                    span: line.span,
                                    message: format!(
                                        "Name collision: '{}' is already defined as a label at line {}",
                                        name, existing.defined_at
                                    ),
                                });
                            }
                        }
                    }
                    Err(e) => {
                        errors.push(AssemblerError {
                            error_type: ErrorType::InvalidConstantValue,
                            line: line.line_number,
                            column: 0,
                            span: line.span,
                            message: format!("Invalid constant value '{}': {}", value_str, e),
                        });
                    }
                }
            }
            // Constants don't consume address space
            continue;
        }

        // Handle directives
        if let Some(ref directive) = line.directive {
            match directive {
                AssemblerDirective::Origin { address } => {
                    current_address = *address;
                }
                AssemblerDirective::Byte { values } => {
                    current_address = current_address.wrapping_add(values.len() as u16);
                }
                AssemblerDirective::Word { values } => {
                    current_address = current_address.wrapping_add((values.len() * 2) as u16);
                }
            }
            continue;
        }

        // Skip lines with only labels or comments
        if line.mnemonic.is_none() {
            continue;
        }

        // Calculate instruction size to advance address
        let mnemonic = line.mnemonic.as_ref().unwrap();

        // Determine addressing mode (without resolving labels yet)
        let mode = if let Some(ref operand) = line.operand {
            match parser::detect_addressing_mode_or_label(operand) {
                Ok(m) => m,
                Err(_) => {
                    // Error will be caught in Pass 2
                    crate::addressing::AddressingMode::Implicit
                }
            }
        } else {
            crate::addressing::AddressingMode::Implicit
        };

        // Look up instruction size
        if let Ok(opcode_meta) = encoder::find_opcode_metadata(mnemonic, mode) {
            current_address = current_address.wrapping_add(opcode_meta.size_bytes as u16);
        } else {
            // Error will be caught in Pass 2
            // Branch instructions are always 2 bytes (opcode + relative offset)
            if is_branch_mnemonic(mnemonic) {
                current_address = current_address.wrapping_add(2);
            } else {
                current_address = current_address.wrapping_add(1);
            }
        }
    }

    // Return early if there were label validation errors
    if !errors.is_empty() {
        return Err(errors);
    }

    // Pass 2: Encode instructions with label resolution
    let mut bytes = Vec::new();
    let mut source_map = source_map::SourceMap::new();
    current_address = 0u16;

    for line in &parsed_lines {
        // Handle directives
        if let Some(ref directive) = line.directive {
            match directive {
                AssemblerDirective::Origin { address } => {
                    current_address = *address;
                }
                AssemblerDirective::Byte { values } => {
                    let instruction_start_address = current_address;

                    // Add bytes directly to output
                    bytes.extend(values);

                    // Add to source map
                    source_map.add_mapping(
                        instruction_start_address,
                        source_map::SourceLocation {
                            line: line.line_number,
                            column: 0,
                            length: line.span.1 - line.span.0,
                        },
                    );
                    source_map.add_line_mapping(
                        line.line_number,
                        source_map::AddressRange {
                            start: instruction_start_address,
                            end: instruction_start_address.wrapping_add(values.len() as u16),
                        },
                    );

                    current_address = current_address.wrapping_add(values.len() as u16);
                }
                AssemblerDirective::Word { values } => {
                    let instruction_start_address = current_address;

                    // Add words in little-endian format
                    for word in values {
                        bytes.push((word & 0xFF) as u8); // Low byte
                        bytes.push(((word >> 8) & 0xFF) as u8); // High byte
                    }

                    // Add to source map
                    source_map.add_mapping(
                        instruction_start_address,
                        source_map::SourceLocation {
                            line: line.line_number,
                            column: 0,
                            length: line.span.1 - line.span.0,
                        },
                    );
                    source_map.add_line_mapping(
                        line.line_number,
                        source_map::AddressRange {
                            start: instruction_start_address,
                            end: instruction_start_address.wrapping_add((values.len() * 2) as u16),
                        },
                    );

                    current_address = current_address.wrapping_add((values.len() * 2) as u16);
                }
            }
            continue;
        }

        // Skip lines with only labels or comments
        if line.mnemonic.is_none() {
            continue;
        }

        let mnemonic = line.mnemonic.as_ref().unwrap();

        // Check for invalid directive (mnemonic starts with .)
        if mnemonic.starts_with('.') {
            errors.push(AssemblerError {
                error_type: ErrorType::InvalidDirective,
                line: line.line_number,
                column: 0,
                span: line.span,
                message: format!("Invalid or unknown directive: {}", mnemonic),
            });
            continue;
        }

        let instruction_start_address = current_address;

        // Detect addressing mode and resolve operand value (including labels)
        let (mode, value) = if let Some(ref operand) = line.operand {
            match resolve_operand(operand, &symbol_table, instruction_start_address, mnemonic) {
                Ok((m, v)) => (m, v),
                Err(e) => {
                    errors.push(AssemblerError {
                        error_type: e.error_type,
                        line: line.line_number,
                        column: 0,
                        span: line.span,
                        message: e.message,
                    });
                    continue; // Error recovery: skip this line
                }
            }
        } else {
            (crate::addressing::AddressingMode::Implicit, 0)
        };

        // Encode the instruction
        match encoder::encode_instruction(mnemonic, mode, value) {
            Ok(instruction_bytes) => {
                let instruction_size = instruction_bytes.len() as u16;

                // Add to source map (address → source location)
                source_map.add_mapping(
                    instruction_start_address,
                    source_map::SourceLocation {
                        line: line.line_number,
                        column: 0,
                        length: line.span.1 - line.span.0,
                    },
                );

                // Add to source map (line → address range)
                source_map.add_line_mapping(
                    line.line_number,
                    source_map::AddressRange {
                        start: instruction_start_address,
                        end: instruction_start_address.wrapping_add(instruction_size),
                    },
                );

                bytes.extend(instruction_bytes);
                current_address = current_address.wrapping_add(instruction_size);
            }
            Err(mut e) => {
                // Update error with correct line info
                e.line = line.line_number;
                e.span = line.span;

                // Check if this is an invalid mnemonic
                let is_valid_mnemonic = opcodes::OPCODE_TABLE
                    .iter()
                    .any(|op| op.mnemonic == mnemonic.as_str());

                if !is_valid_mnemonic {
                    e.error_type = ErrorType::InvalidMnemonic;
                    e.message = format!("Invalid mnemonic '{}'", mnemonic);
                }

                errors.push(e);
                // Error recovery: continue to collect more errors
            }
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    // Finalize source map (sort for binary search)
    source_map.finalize();

    Ok(AssemblerOutput {
        bytes,
        symbol_table: symbol_table.symbols().to_vec(),
        source_map,
        warnings: Vec::new(),
    })
}

/// Assemble source code with a specified origin address
///
/// This is a convenience function equivalent to prepending `.org <origin>` to the source.
/// Labels and code addresses will be calculated relative to this origin.
///
/// # Arguments
///
/// * `source` - The assembly source code text
/// * `origin` - Starting address for assembled code (0x0000 to 0xFFFF)
///
/// # Returns
///
/// Ok(AssemblerOutput) on success, Err(Vec<AssemblerError>) on failure
///
/// # Examples
///
/// ```
/// use lib6502::assembler::assemble_with_origin;
///
/// let source = "START:\n    LDA #$42";
/// let output = assemble_with_origin(source, 0x8000).unwrap();
/// assert_eq!(output.symbol_table[0].value, 0x8000);
/// ```
pub fn assemble_with_origin(
    source: &str,
    origin: u16,
) -> Result<AssemblerOutput, Vec<AssemblerError>> {
    // Prepend .org directive to source
    let source_with_origin = format!(".org ${:04X}\n{}", origin, source);
    assemble(&source_with_origin)
}

/// Resolve an operand, handling both numeric values and label references
fn resolve_operand(
    operand: &str,
    symbol_table: &symbol_table::SymbolTable,
    current_address: u16,
    mnemonic: &str,
) -> Result<(crate::addressing::AddressingMode, u16), AssemblerError> {
    // Check if operand is a label reference (no prefix like $, #, (, etc.)
    let operand_trimmed = operand.trim();

    // Check for accumulator mode first (case-insensitive "A")
    if operand_trimmed.eq_ignore_ascii_case("A") {
        return Ok((crate::addressing::AddressingMode::Accumulator, 0));
    }

    // Check if this is a branch instruction
    let is_branch = is_branch_mnemonic(mnemonic);

    // Special handling for immediate mode with constants (#CONSTANT)
    if let Some(stripped) = operand_trimmed.strip_prefix('#') {
        let inner = stripped.trim();

        // Check if it's a constant reference (no $, %, or digit prefix)
        if !inner.starts_with('$')
            && !inner.starts_with('%')
            && !inner.chars().next().is_some_and(|c| c.is_ascii_digit())
        {
            // Look up constant in symbol table
            let constant_name = inner.to_uppercase();
            if let Some(symbol) = symbol_table.lookup_symbol(&constant_name) {
                // Check if it's actually a constant, not a label
                if symbol.kind == SymbolKind::Constant {
                    // Validate immediate value is in range
                    if symbol.value > 0xFF {
                        return Err(AssemblerError {
                            error_type: ErrorType::RangeError,
                            line: 0,
                            column: 0,
                            span: (0, 0),
                            message: format!(
                                "Constant '{}' value ${:04X} exceeds 8-bit range (0-255) for immediate mode",
                                constant_name, symbol.value
                            ),
                        });
                    }
                    return Ok((crate::addressing::AddressingMode::Immediate, symbol.value));
                } else {
                    // Using a label in immediate mode - this is unusual but allowed
                    // Treat label address as immediate value
                    if symbol.value > 0xFF {
                        return Err(AssemblerError {
                            error_type: ErrorType::RangeError,
                            line: 0,
                            column: 0,
                            span: (0, 0),
                            message: format!(
                                "Label '{}' address ${:04X} exceeds 8-bit range (0-255) for immediate mode",
                                constant_name, symbol.value
                            ),
                        });
                    }
                    return Ok((crate::addressing::AddressingMode::Immediate, symbol.value));
                }
            } else {
                // Symbol not found - return undefined error
                return Err(AssemblerError {
                    error_type: ErrorType::UndefinedLabel,
                    line: 0,
                    column: 0,
                    span: (0, 0),
                    message: format!("Undefined symbol '{}'", constant_name),
                });
            }
        }

        // Parse as normal immediate value
        return parser::detect_addressing_mode(operand).map_err(|e| AssemblerError {
            error_type: ErrorType::InvalidOperand,
            line: 0,
            column: 0,
            span: (0, 0),
            message: format!("Invalid operand '{}': {}", operand, e),
        });
    }

    // If it starts with a special character, it's not a plain label/constant
    if operand_trimmed.starts_with('$')
        || operand_trimmed.starts_with('#')
        || operand_trimmed.starts_with('(')
        || operand_trimmed.starts_with('%')
        || operand_trimmed
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit())
    {
        // Special handling for branch instructions with numeric addresses
        // Branch instructions only support relative addressing, so we need to
        // calculate the offset from a numeric target address
        if is_branch && !operand_trimmed.starts_with('#') && !operand_trimmed.starts_with('(') {
            // Parse the target address (supports $hex, %binary, or decimal)
            let target_address =
                parser::parse_number(operand_trimmed).map_err(|e| AssemblerError {
                    error_type: ErrorType::InvalidOperand,
                    line: 0,
                    column: 0,
                    span: (0, 0),
                    message: format!("Invalid branch target '{}': {}", operand, e),
                })?;

            // Calculate relative offset
            let next_instruction_address = current_address + 2; // Branch instructions are 2 bytes
            let offset = target_address as i32 - next_instruction_address as i32;

            // Check if offset is in range (-128 to +127)
            if !(-128..=127).contains(&offset) {
                return Err(AssemblerError {
                    error_type: ErrorType::RangeError,
                    line: 0,
                    column: 0,
                    span: (0, 0),
                    message: format!(
                        "Branch to ${:04X} is out of range (offset: {}, must be -128 to +127)",
                        target_address, offset
                    ),
                });
            }

            // Convert to unsigned byte (two's complement)
            let offset_byte = (offset as i8) as u8;
            return Ok((
                crate::addressing::AddressingMode::Relative,
                offset_byte as u16,
            ));
        }

        // Parse as normal addressing mode (non-branch instructions or immediate/indirect modes)
        return parser::detect_addressing_mode(operand).map_err(|e| AssemblerError {
            error_type: ErrorType::InvalidOperand,
            line: 0,
            column: 0,
            span: (0, 0),
            message: format!("Invalid operand '{}': {}", operand, e),
        });
    }

    // Check for indexed addressing with symbol (e.g., "IO_BASE,X" or "BUFFER,Y")
    if operand_trimmed.contains(',') {
        let parts: Vec<&str> = operand_trimmed.split(',').collect();
        if parts.len() == 2 {
            let base_part = parts[0].trim();
            let index_part = parts[1].trim().to_uppercase();

            // Check if base is a symbol reference (not a number)
            if !base_part.starts_with('$')
                && !base_part.starts_with('%')
                && !base_part
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_digit())
            {
                let symbol_name = base_part.to_uppercase();
                if let Some(symbol) = symbol_table.lookup_symbol(&symbol_name) {
                    let value = symbol.value;

                    // Determine indexed addressing mode based on index register
                    match index_part.as_str() {
                        "X" => {
                            if value <= 0xFF {
                                return Ok((crate::addressing::AddressingMode::ZeroPageX, value));
                            } else {
                                return Ok((crate::addressing::AddressingMode::AbsoluteX, value));
                            }
                        }
                        "Y" => {
                            if value <= 0xFF {
                                return Ok((crate::addressing::AddressingMode::ZeroPageY, value));
                            } else {
                                return Ok((crate::addressing::AddressingMode::AbsoluteY, value));
                            }
                        }
                        _ => {
                            return Err(AssemblerError {
                                error_type: ErrorType::InvalidOperand,
                                line: 0,
                                column: 0,
                                span: (0, 0),
                                message: format!(
                                    "Invalid index register '{}' (must be X or Y)",
                                    index_part
                                ),
                            });
                        }
                    }
                }
                // Symbol not found - fall through to error handling below
            }
        }
        // Has comma but not a valid symbol,index format - parse normally
        return parser::detect_addressing_mode(operand).map_err(|e| AssemblerError {
            error_type: ErrorType::InvalidOperand,
            line: 0,
            column: 0,
            span: (0, 0),
            message: format!("Invalid operand '{}': {}", operand, e),
        });
    }

    // Must be a label/constant reference - look it up
    // Symbols are stored in uppercase, so convert operand to uppercase for lookup
    let symbol_name = operand_trimmed.to_uppercase();
    if let Some(symbol) = symbol_table.lookup_symbol(&symbol_name) {
        match symbol.kind {
            SymbolKind::Constant => {
                // For constants, use value directly and choose addressing mode based on value
                let value = symbol.value;

                // Determine addressing mode based on value range
                // If value fits in zero page (0-255), use ZeroPage, otherwise Absolute
                if value <= 0xFF {
                    Ok((crate::addressing::AddressingMode::ZeroPage, value))
                } else {
                    Ok((crate::addressing::AddressingMode::Absolute, value))
                }
            }
            SymbolKind::Label => {
                // For labels, use value as memory address
                let target_address = symbol.value;

                // Determine if this is a branch instruction (needs relative addressing)
                let is_branch = matches!(
                    mnemonic,
                    "BCC" | "BCS" | "BEQ" | "BMI" | "BNE" | "BPL" | "BVC" | "BVS"
                );

                if is_branch {
                    // Calculate relative offset
                    // Branch offset is relative to the address of the next instruction
                    let next_instruction_address = current_address + 2; // Branch instructions are 2 bytes
                    let offset = target_address as i32 - next_instruction_address as i32;

                    // Check if offset is in range (-128 to +127)
                    if !(-128..=127).contains(&offset) {
                        return Err(AssemblerError {
                            error_type: ErrorType::RangeError,
                            line: 0,
                            column: 0,
                            span: (0, 0),
                            message: format!(
                                "Branch to '{}' is out of range (offset: {}, must be -128 to +127)",
                                operand_trimmed, offset
                            ),
                        });
                    }

                    // Convert to unsigned byte (two's complement)
                    let offset_byte = (offset as i8) as u8;
                    Ok((
                        crate::addressing::AddressingMode::Relative,
                        offset_byte as u16,
                    ))
                } else {
                    // Absolute addressing for JMP, JSR, etc.
                    Ok((crate::addressing::AddressingMode::Absolute, target_address))
                }
            }
        }
    } else {
        // Undefined symbol
        Err(AssemblerError {
            error_type: ErrorType::UndefinedLabel,
            line: 0,
            column: 0,
            span: (0, 0),
            message: format!("Undefined symbol '{}'", symbol_name),
        })
    }
}

/// Validate a label name according to 6502 conventions
///
/// Labels must:
/// - Start with a letter [a-zA-Z]
/// - Contain only alphanumeric characters and underscores
/// - Not exceed 32 characters in length
///
/// # Returns
///
/// * `Ok(())` if the label is valid
/// * `Err(LabelError)` with specific validation failure reason
///
/// # Examples
///
/// ```
/// use lib6502::assembler::validate_label;
///
/// assert!(validate_label("START").is_ok());
/// assert!(validate_label("loop_1").is_ok());
/// assert!(validate_label("_invalid").is_err()); // starts with underscore
/// assert!(validate_label("1invalid").is_err()); // starts with digit
/// ```
pub fn validate_label(name: &str) -> Result<(), LabelError> {
    if name.is_empty() {
        return Err(LabelError::InvalidStart(
            "label name cannot be empty".to_string(),
        ));
    }

    if name.len() > 32 {
        return Err(LabelError::TooLong(name.len()));
    }

    let mut chars = name.chars();
    let first = chars.next().unwrap();

    if !first.is_ascii_alphabetic() {
        return Err(LabelError::InvalidStart(format!(
            "label must start with a letter, not '{}'",
            first
        )));
    }

    for ch in chars {
        if !ch.is_ascii_alphanumeric() && ch != '_' {
            return Err(LabelError::InvalidCharacters(format!(
                "label contains invalid character '{}' (only letters, digits, and underscores allowed)",
                ch
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_label_valid() {
        assert!(validate_label("START").is_ok());
        assert!(validate_label("loop_1").is_ok());
        assert!(validate_label("MyLabel").is_ok());
        assert!(validate_label("A").is_ok());
    }

    #[test]
    fn test_validate_label_invalid() {
        assert!(validate_label("").is_err());
        assert!(validate_label("1START").is_err());
        assert!(validate_label("MY-LABEL").is_err());
        assert!(validate_label("LABEL!").is_err());
        assert!(validate_label(&"A".repeat(33)).is_err());
    }

    #[test]
    fn test_validate_label_error_types() {
        // Test InvalidStart errors
        match validate_label("") {
            Err(LabelError::InvalidStart(_)) => {}
            _ => panic!("Expected InvalidStart error for empty label"),
        }

        match validate_label("1START") {
            Err(LabelError::InvalidStart(_)) => {}
            _ => panic!("Expected InvalidStart error for label starting with digit"),
        }

        match validate_label("_invalid") {
            Err(LabelError::InvalidStart(_)) => {}
            _ => panic!("Expected InvalidStart error for label starting with underscore"),
        }

        // Test InvalidCharacters errors
        match validate_label("MY-LABEL") {
            Err(LabelError::InvalidCharacters(_)) => {}
            _ => panic!("Expected InvalidCharacters error for label with hyphen"),
        }

        match validate_label("LABEL!") {
            Err(LabelError::InvalidCharacters(_)) => {}
            _ => panic!("Expected InvalidCharacters error for label with exclamation"),
        }

        // Test TooLong error
        match validate_label(&"A".repeat(33)) {
            Err(LabelError::TooLong(33)) => {}
            _ => panic!("Expected TooLong error for 33-character label"),
        }
    }

    #[test]
    fn test_assemble_with_origin_basic() {
        let source = "START:\n    LDA #$42";
        let output = assemble_with_origin(source, 0x8000).unwrap();

        // Check that the label has the correct address
        assert_eq!(output.symbol_table.len(), 1);
        assert_eq!(output.symbol_table[0].name, "START");
        assert_eq!(output.symbol_table[0].value, 0x8000);

        // Check assembled bytes (LDA immediate)
        assert_eq!(output.bytes.len(), 2);
        assert_eq!(output.bytes[0], 0xA9); // LDA immediate opcode
        assert_eq!(output.bytes[1], 0x42);
    }

    #[test]
    fn test_assemble_with_origin_multiple_labels() {
        let source = "START:\n    NOP\nLOOP:\n    NOP\n    JMP LOOP";
        let output = assemble_with_origin(source, 0x1000).unwrap();

        // Check that labels have correct addresses
        assert_eq!(output.symbol_table.len(), 2);
        let start = output.lookup_symbol("START").unwrap();
        let loop_label = output.lookup_symbol("LOOP").unwrap();

        assert_eq!(start.value, 0x1000);
        assert_eq!(loop_label.value, 0x1001); // After one NOP
    }

    #[test]
    fn test_assemble_with_origin_preserves_errors() {
        let source = "INVALID SYNTAX HERE";
        let result = assemble_with_origin(source, 0x8000);
        assert!(result.is_err());
    }

    #[test]
    fn test_lookup_symbol_addr() {
        let source = "START:\n    LDA #$42\nEND:\n    RTS";
        let output = assemble(source).unwrap();

        // Test existing symbol
        assert_eq!(output.lookup_symbol_addr("START"), Some(0));
        assert_eq!(output.lookup_symbol_addr("END"), Some(2));

        // Test non-existent symbol
        assert_eq!(output.lookup_symbol_addr("NONEXISTENT"), None);
    }

    #[test]
    fn test_lookup_symbol_addr_with_origin() {
        let source = "START:\n    NOP\nEND:\n    NOP";
        let output = assemble_with_origin(source, 0x2000).unwrap();

        assert_eq!(output.lookup_symbol_addr("START"), Some(0x2000));
        assert_eq!(output.lookup_symbol_addr("END"), Some(0x2001));
    }

    #[test]
    fn test_lookup_symbol_still_works() {
        // Ensure the richer lookup_symbol method still works alongside lookup_symbol_addr
        let source = "START:\n    LDA #$42";
        let output = assemble(source).unwrap();

        let symbol = output.lookup_symbol("START");
        assert!(symbol.is_some());
        let symbol = symbol.unwrap();
        assert_eq!(symbol.name, "START");
        assert_eq!(symbol.value, 0);
        assert_eq!(symbol.defined_at, 1);
    }
}
