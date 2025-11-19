//! 6502 Assembler Module
//!
//! Converts assembly language source code into binary machine code.

pub mod encoder;
pub mod lexer;
pub mod parser;
pub mod source_map;
pub mod symbol_table;

// Re-export lexer types for public API
pub use lexer::{tokenize, Token, TokenType, TokenStream};

use crate::opcodes;

// Addressing mode value range constants
const IMMEDIATE_MODE_MAX: u16 = 0xFF;
const ZERO_PAGE_MAX: u16 = 0xFF;
const BRANCH_OFFSET_MIN: i32 = -128;
const BRANCH_OFFSET_MAX: i32 = 127;
const BRANCH_INSTRUCTION_SIZE: u16 = 2;

/// A segment of code at a specific address
///
/// Segments are created each time the assembler encounters a `.org` directive
/// or at the start of assembly. They track where code is located in memory.
#[derive(Debug, Clone, PartialEq)]
pub struct CodeSegment {
    /// Starting address of this segment in memory
    pub address: u16,

    /// Number of bytes in this segment
    pub length: u16,
}

/// Complete output from assembling source code
#[derive(Debug, Clone)]
pub struct AssemblerOutput {
    /// Assembled machine code bytes (flat array, no gaps)
    pub bytes: Vec<u8>,

    /// Symbol table with all defined labels
    pub symbol_table: Vec<Symbol>,

    /// Source map for debugging
    pub source_map: source_map::SourceMap,

    /// Non-fatal warnings encountered during assembly
    pub warnings: Vec<AssemblerWarning>,

    /// Code segments tracking address layout
    ///
    /// Each segment represents a contiguous block of code at a specific address.
    /// Segments are created when `.org` directives change the assembly address.
    pub segments: Vec<CodeSegment>,
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

    /// Convert flat bytes to a ROM image with proper address layout
    ///
    /// This method creates a contiguous ROM image from the assembled bytes,
    /// filling gaps between segments with the specified fill byte.
    ///
    /// # Arguments
    ///
    /// * `fill_byte` - Value to use for uninitialized memory (typically 0xFF or 0x00)
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` containing the full ROM image from the lowest to highest address.
    /// The returned vector starts at the address of the first segment and ends at
    /// the last byte of the last segment.
    ///
    /// # Examples
    ///
    /// ```
    /// use lib6502::assembler::assemble;
    ///
    /// let source = r#"
    /// .org $8000
    /// START:
    ///     LDA #$42
    ///
    /// .org $FFFC
    /// .word START
    /// "#;
    ///
    /// let output = assemble(source).unwrap();
    /// let rom = output.to_rom_image(0xFF);
    ///
    /// // ROM starts at $8000 and ends at $FFFD (inclusive)
    /// // Length is $FFFD - $8000 + 1 = $7FFE = 32766 bytes
    /// assert_eq!(rom.len(), 0x7FFE);
    ///
    /// // First bytes are the program
    /// assert_eq!(rom[0], 0xA9); // LDA #$42 opcode
    /// assert_eq!(rom[1], 0x42);
    ///
    /// // Gap is filled with 0xFF
    /// assert_eq!(rom[2], 0xFF);
    ///
    /// // Reset vector at the end
    /// assert_eq!(rom[0x7FFC], 0x00); // Low byte of $8000
    /// assert_eq!(rom[0x7FFD], 0x80); // High byte of $8000
    /// ```
    pub fn to_rom_image(&self, fill_byte: u8) -> Vec<u8> {
        if self.segments.is_empty() {
            return Vec::new();
        }

        // Find min and max addresses
        let min_address = self.segments.first().unwrap().address;
        let last_segment = self.segments.last().unwrap();

        // Calculate max_address safely (last byte of last segment)
        // Note: address + length gives us one past the last byte
        let max_address = last_segment
            .address
            .wrapping_add(last_segment.length)
            .wrapping_sub(1);

        // Calculate ROM size - handle wraparound case
        let rom_size = if max_address < min_address {
            // Wraparound case (e.g., ends at $FFFF)
            (0x10000 - min_address as usize) + (max_address as usize + 1)
        } else {
            (max_address as usize - min_address as usize) + 1
        };

        // Create ROM buffer filled with fill_byte
        let mut rom = vec![fill_byte; rom_size];

        // Copy each segment to its proper location
        let mut byte_offset = 0;
        for segment in &self.segments {
            let rom_offset = (segment.address - min_address) as usize;
            let segment_bytes = &self.bytes[byte_offset..byte_offset + segment.length as usize];
            rom[rom_offset..rom_offset + segment.length as usize].copy_from_slice(segment_bytes);
            byte_offset += segment.length as usize;
        }

        rom
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

/// Lexical analysis errors (FR-007: separate from syntactic errors)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexerError {
    /// Invalid hex digit in hex number (e.g., '$ZZ')
    InvalidHexDigit {
        ch: char,
        line: usize,
        column: usize,
    },

    /// Invalid binary digit in binary number (e.g., '%222')
    InvalidBinaryDigit {
        ch: char,
        line: usize,
        column: usize,
    },

    /// Missing digits after hex prefix (e.g., '$' followed by non-hex)
    MissingHexDigits {
        line: usize,
        column: usize,
    },

    /// Missing digits after binary prefix (e.g., '%' followed by non-binary)
    MissingBinaryDigits {
        line: usize,
        column: usize,
    },

    /// Number too large (overflow u16 range)
    NumberTooLarge {
        value: String,
        max: u16,
        line: usize,
        column: usize,
    },

    /// Unexpected character (invalid token start)
    UnexpectedCharacter {
        ch: char,
        line: usize,
        column: usize,
    },
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerError::InvalidHexDigit { ch, line, column } => write!(
                f,
                "Line {}, Column {}: Invalid hex digit '{}' in hex number",
                line, column, ch
            ),
            LexerError::InvalidBinaryDigit { ch, line, column } => write!(
                f,
                "Line {}, Column {}: Invalid binary digit '{}' in binary number",
                line, column, ch
            ),
            LexerError::MissingHexDigits { line, column } => write!(
                f,
                "Line {}, Column {}: Expected hex digits after '$' prefix",
                line, column
            ),
            LexerError::MissingBinaryDigits { line, column } => write!(
                f,
                "Line {}, Column {}: Expected binary digits after '%' prefix",
                line, column
            ),
            LexerError::NumberTooLarge {
                value,
                max,
                line,
                column,
            } => write!(
                f,
                "Line {}, Column {}: Number '{}' exceeds maximum value {} (u16 range)",
                line, column, value, max
            ),
            LexerError::UnexpectedCharacter { ch, line, column } => write!(
                f,
                "Line {}, Column {}: Unexpected character '{}'",
                line, column, ch
            ),
        }
    }
}

impl std::error::Error for LexerError {}

/// Classification of assembly errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorType {
    /// Lexical analysis error (invalid token, malformed number, etc.)
    LexicalError(LexerError),

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
            match &self.error_type {
                ErrorType::LexicalError(_) => "Lexical Error",
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

/// Value in a directive argument (either a literal or a symbol reference)
#[derive(Debug, Clone, PartialEq)]
pub enum DirectiveValue {
    /// Literal numeric value
    Literal(u16),

    /// Symbol reference (label or constant name)
    Symbol(String),
}

/// Assembler directive types
#[derive(Debug, Clone, PartialEq)]
pub enum AssemblerDirective {
    /// Set origin address (.org $XXXX)
    Origin { address: u16 },

    /// Insert literal bytes (.byte $XX, $YY, ...)
    /// Values can be literals (0-255) or symbol references
    Byte { values: Vec<DirectiveValue> },

    /// Insert literal 16-bit words (.word $XXXX, $YYYY, ...)
    /// Values can be literals (0-65535) or symbol references
    Word { values: Vec<DirectiveValue> },
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

    // Tokenize the entire source
    let tokens = match tokenize(source) {
        Ok(tokens) => tokens,
        Err(lexer_errors) => {
            // Convert lexer errors to assembler errors
            for lex_err in lexer_errors {
                let (line, column) = match &lex_err {
                    LexerError::InvalidHexDigit { line, column, .. } => (*line, *column),
                    LexerError::InvalidBinaryDigit { line, column, .. } => (*line, *column),
                    LexerError::MissingHexDigits { line, column } => (*line, *column),
                    LexerError::MissingBinaryDigits { line, column } => (*line, *column),
                    LexerError::NumberTooLarge { line, column, .. } => (*line, *column),
                    LexerError::UnexpectedCharacter { line, column, .. } => (*line, *column),
                };
                errors.push(AssemblerError {
                    error_type: ErrorType::LexicalError(lex_err.clone()),
                    line,
                    column,
                    span: (column, column + 1),
                    message: lex_err.to_string(),
                });
            }
            return Err(errors);
        }
    };

    // Group tokens by line
    let mut token_lines: Vec<Vec<Token>> = Vec::new();
    let mut current_line_tokens = Vec::new();

    for token in tokens {
        match &token.token_type {
            TokenType::Newline => {
                // End of line - save current line tokens
                token_lines.push(current_line_tokens);
                current_line_tokens = Vec::new();
            }
            TokenType::Eof => {
                // End of file - save last line if not empty
                if !current_line_tokens.is_empty() {
                    token_lines.push(current_line_tokens);
                }
                break;
            }
            _ => {
                // Add token to current line
                current_line_tokens.push(token);
            }
        }
    }

    // Parse all lines using token-based parser
    let parsed_lines: Vec<_> = token_lines
        .iter()
        .enumerate()
        .filter_map(|(idx, line_tokens)| {
            parser::parse_token_line(line_tokens, idx + 1)
        })
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
                    label,
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
                            name,
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
    let mut segments: Vec<CodeSegment> = Vec::new();
    current_address = 0u16;

    // Track current segment
    let mut current_segment_start = 0u16;
    let mut current_segment_length = 0u16;

    for line in &parsed_lines {
        // Handle directives
        if let Some(ref directive) = line.directive {
            match directive {
                AssemblerDirective::Origin { address } => {
                    // Finalize previous segment if it has any bytes
                    if current_segment_length > 0 {
                        segments.push(CodeSegment {
                            address: current_segment_start,
                            length: current_segment_length,
                        });
                    }

                    // Start new segment
                    current_address = *address;
                    current_segment_start = *address;
                    current_segment_length = 0;
                }
                AssemblerDirective::Byte { values } => {
                    let instruction_start_address = current_address;

                    // Resolve values (handle both literals and symbol references)
                    let mut resolved_bytes = Vec::new();
                    for val in values {
                        let resolved = match val {
                            DirectiveValue::Literal(lit) => *lit,
                            DirectiveValue::Symbol(name) => {
                                // Look up symbol in symbol table
                                match symbol_table.lookup_symbol_ignore_case(name) {
                                    Some(sym) => sym.value,
                                    None => {
                                        errors.push(AssemblerError {
                                            error_type: ErrorType::UndefinedLabel,
                                            line: line.line_number,
                                            column: 0,
                                            span: line.span,
                                            message: format!(
                                                "Undefined symbol '{}' in .byte directive",
                                                name
                                            ),
                                        });
                                        continue;
                                    }
                                }
                            }
                        };

                        // Validate it fits in a byte
                        if resolved > 0xFF {
                            errors.push(AssemblerError {
                                error_type: ErrorType::RangeError,
                                line: line.line_number,
                                column: 0,
                                span: line.span,
                                message: format!(
                                    "Value ${:04X} in .byte directive exceeds 8-bit range (max $FF)",
                                    resolved
                                ),
                            });
                            continue;
                        }

                        resolved_bytes.push(resolved as u8);
                    }

                    // Add bytes to output
                    bytes.extend(&resolved_bytes);

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
                            end: instruction_start_address
                                .wrapping_add(resolved_bytes.len() as u16),
                        },
                    );

                    current_address = current_address.wrapping_add(resolved_bytes.len() as u16);
                    current_segment_length =
                        current_segment_length.wrapping_add(resolved_bytes.len() as u16);
                }
                AssemblerDirective::Word { values } => {
                    let instruction_start_address = current_address;

                    // Resolve values (handle both literals and symbol references)
                    let mut resolved_words = Vec::new();
                    for val in values {
                        let resolved = match val {
                            DirectiveValue::Literal(lit) => *lit,
                            DirectiveValue::Symbol(name) => {
                                // Look up symbol in symbol table
                                match symbol_table.lookup_symbol_ignore_case(name) {
                                    Some(sym) => sym.value,
                                    None => {
                                        errors.push(AssemblerError {
                                            error_type: ErrorType::UndefinedLabel,
                                            line: line.line_number,
                                            column: 0,
                                            span: line.span,
                                            message: format!(
                                                "Undefined symbol '{}' in .word directive",
                                                name
                                            ),
                                        });
                                        continue;
                                    }
                                }
                            }
                        };
                        resolved_words.push(resolved);
                    }

                    // Add words in little-endian format
                    for word in resolved_words.iter() {
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
                            end: instruction_start_address
                                .wrapping_add((resolved_words.len() * 2) as u16),
                        },
                    );

                    current_address =
                        current_address.wrapping_add((resolved_words.len() * 2) as u16);
                    current_segment_length =
                        current_segment_length.wrapping_add((resolved_words.len() * 2) as u16);
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
            match resolve_operand(
                operand,
                &symbol_table,
                instruction_start_address,
                mnemonic,
                (line.line_number, 0, line.span),
            ) {
                Ok((m, v)) => (m, v),
                Err(e) => {
                    errors.push(e);
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
                current_segment_length = current_segment_length.wrapping_add(instruction_size);
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

    // Finalize the last segment if it has any bytes
    if current_segment_length > 0 {
        segments.push(CodeSegment {
            address: current_segment_start,
            length: current_segment_length,
        });
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
        segments,
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

/// Validate that a value fits in immediate mode (0-255)
fn validate_immediate_value(value: u16, name: &str, kind: SymbolKind) -> Result<u16, String> {
    if value > IMMEDIATE_MODE_MAX {
        let kind_str = match kind {
            SymbolKind::Constant => "Constant",
            SymbolKind::Label => "Label",
        };
        Err(format!(
            "{} '{}' value ${:04X} exceeds 8-bit range (expected $00-${:02X}) for immediate mode",
            kind_str, name, value, IMMEDIATE_MODE_MAX
        ))
    } else {
        Ok(value)
    }
}

/// Resolve an operand, handling both numeric values and label references
///
/// # Arguments
///
/// * `operand` - The operand text to resolve
/// * `symbol_table` - Symbol table for looking up labels and constants
/// * `current_address` - Current assembly address for relative branches
/// * `mnemonic` - Instruction mnemonic for context
/// * `line_info` - Line number, column, and span for error reporting
fn resolve_operand(
    operand: &str,
    symbol_table: &symbol_table::SymbolTable,
    current_address: u16,
    mnemonic: &str,
    line_info: (usize, usize, (usize, usize)),
) -> Result<(crate::addressing::AddressingMode, u16), AssemblerError> {
    let (line, column, span) = line_info;
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
            // Look up constant in symbol table (case-insensitive, no allocation)
            if let Some(symbol) = symbol_table.lookup_symbol_ignore_case(inner) {
                // Validate immediate value is in range (works for both constants and labels)
                let value = validate_immediate_value(symbol.value, &symbol.name, symbol.kind)
                    .map_err(|msg| AssemblerError {
                        error_type: ErrorType::RangeError,
                        line,
                        column,
                        span,
                        message: msg,
                    })?;
                return Ok((crate::addressing::AddressingMode::Immediate, value));
            } else {
                // Symbol not found - return undefined error
                return Err(AssemblerError {
                    error_type: ErrorType::UndefinedLabel,
                    line,
                    column,
                    span,
                    message: format!("Undefined symbol '{}'", inner.to_uppercase()),
                });
            }
        }

        // Parse as normal immediate value
        return parser::detect_addressing_mode(operand).map_err(|e| AssemblerError {
            error_type: ErrorType::InvalidOperand,
            line,
            column,
            span,
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
                    line,
                    column,
                    span,
                    message: format!("Invalid branch target '{}': {}", operand, e),
                })?;

            // Calculate relative offset
            let next_instruction_address = current_address + BRANCH_INSTRUCTION_SIZE;
            let offset = target_address as i32 - next_instruction_address as i32;

            // Check if offset is in range
            if !(BRANCH_OFFSET_MIN..=BRANCH_OFFSET_MAX).contains(&offset) {
                return Err(AssemblerError {
                    error_type: ErrorType::RangeError,
                    line,
                    column,
                    span,
                    message: format!(
                        "Branch to ${:04X} is out of range (offset: {}, expected {} to {})",
                        target_address, offset, BRANCH_OFFSET_MIN, BRANCH_OFFSET_MAX
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
            line,
            column,
            span,
            message: format!("Invalid operand '{}': {}", operand, e),
        });
    }

    // Check for indexed addressing with symbol (e.g., "IO_BASE,X" or "BUFFER,Y")
    // Use split_once for clarity and to reject malformed input
    if let Some((base, index)) = operand_trimmed.split_once(',') {
        let base_part = base.trim();
        let index_part = index.trim();

        // Check if base is a symbol reference (not a number)
        if !base_part.is_empty()
            && !base_part.starts_with('$')
            && !base_part.starts_with('%')
            && !base_part.chars().next().is_some_and(|c| c.is_ascii_digit())
        {
            if let Some(symbol) = symbol_table.lookup_symbol_ignore_case(base_part) {
                let value = symbol.value;

                // Determine indexed addressing mode based on index register (case-insensitive)
                if index_part.eq_ignore_ascii_case("X") {
                    if value <= ZERO_PAGE_MAX {
                        return Ok((crate::addressing::AddressingMode::ZeroPageX, value));
                    } else {
                        return Ok((crate::addressing::AddressingMode::AbsoluteX, value));
                    }
                } else if index_part.eq_ignore_ascii_case("Y") {
                    if value <= ZERO_PAGE_MAX {
                        return Ok((crate::addressing::AddressingMode::ZeroPageY, value));
                    } else {
                        return Ok((crate::addressing::AddressingMode::AbsoluteY, value));
                    }
                } else {
                    return Err(AssemblerError {
                        error_type: ErrorType::InvalidOperand,
                        line,
                        column,
                        span,
                        message: format!(
                            "Invalid index register '{}' (must be X or Y)",
                            index_part
                        ),
                    });
                }
            }
            // Symbol not found - fall through to error handling below
        }

        // Has comma but not a valid symbol,index format - parse normally
        return parser::detect_addressing_mode(operand).map_err(|e| AssemblerError {
            error_type: ErrorType::InvalidOperand,
            line,
            column,
            span,
            message: format!("Invalid operand '{}': {}", operand, e),
        });
    }

    // Must be a label/constant reference - look it up (case-insensitive, no allocation)
    if let Some(symbol) = symbol_table.lookup_symbol_ignore_case(operand_trimmed) {
        match symbol.kind {
            SymbolKind::Constant => {
                // For constants, use value directly and choose addressing mode based on value
                let value = symbol.value;

                // Determine addressing mode based on value range
                // If value fits in zero page (0-255), use ZeroPage, otherwise Absolute
                if value <= ZERO_PAGE_MAX {
                    Ok((crate::addressing::AddressingMode::ZeroPage, value))
                } else {
                    Ok((crate::addressing::AddressingMode::Absolute, value))
                }
            }
            SymbolKind::Label => {
                // For labels, use value as memory address
                let target_address = symbol.value;

                // Determine if this is a branch instruction (needs relative addressing)
                if is_branch {
                    // Calculate relative offset
                    // Branch offset is relative to the address of the next instruction
                    let next_instruction_address = current_address + BRANCH_INSTRUCTION_SIZE;
                    let offset = target_address as i32 - next_instruction_address as i32;

                    // Check if offset is in range
                    if !(BRANCH_OFFSET_MIN..=BRANCH_OFFSET_MAX).contains(&offset) {
                        return Err(AssemblerError {
                            error_type: ErrorType::RangeError,
                            line,
                            column,
                            span,
                            message: format!(
                                "Branch to '{}' is out of range (offset: {}, expected {} to {})",
                                operand_trimmed, offset, BRANCH_OFFSET_MIN, BRANCH_OFFSET_MAX
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
            line,
            column,
            span,
            message: format!("Undefined symbol '{}'", operand_trimmed.to_uppercase()),
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
