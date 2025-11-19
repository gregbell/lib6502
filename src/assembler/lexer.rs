//! Lexical analysis for 6502 assembly source
//!
//! The lexer converts assembly source text into a stream of typed tokens with source
//! location information. This is the first phase of assembly, separating character-level
//! tokenization from syntactic analysis.

/// Classification of lexical tokens in 6502 assembly
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Identifiers and keywords
    /// Identifiers: mnemonics, labels, symbol references (uppercase normalized)
    Identifier(String),

    // Numbers (parsed values)
    /// Decimal number literal (0-65535)
    DecimalNumber(u16),
    /// Hexadecimal number literal with $ prefix (0x0000-0xFFFF, parsed)
    HexNumber(u16),
    /// Binary number literal with % prefix (0-65535, parsed)
    BinaryNumber(u16),

    // Operators and punctuation
    /// Colon `:` - label definition suffix
    Colon,
    /// Comma `,` - operand separator, indexed addressing
    Comma,
    /// Hash `#` - immediate mode prefix
    Hash,
    /// Dollar `$` - hex number prefix (standalone, not part of HexNumber)
    Dollar,
    /// Percent `%` - binary number prefix (standalone, not part of BinaryNumber)
    Percent,
    /// Equal `=` - constant assignment operator
    Equal,
    /// Left parenthesis `(` - indirect addressing open
    LParen,
    /// Right parenthesis `)` - indirect addressing close
    RParen,
    /// Dot `.` - directive prefix
    Dot,

    // Structural
    /// Whitespace (spaces/tabs, preserved for formatters)
    Whitespace,
    /// Line terminator (CRLF or LF normalized to single token)
    Newline,
    /// Comment text after semicolon (excluding `;` itself)
    Comment(String),
    /// End of file marker
    Eof,
}

/// A single lexical token with type, value, and source location
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// Token classification and optional parsed value
    pub token_type: TokenType,

    /// Source line number (1-indexed for user display)
    pub line: usize,

    /// Column offset within line (0-indexed)
    pub column: usize,

    /// Character span (for error highlighting)
    pub length: usize,
}

/// Lexer state for converting source text into tokens
pub struct Lexer<'a> {
    /// Reference to original source text (lifetime-bound)
    source: &'a str,

    /// Iterator over (byte_offset, char) pairs
    chars: std::str::CharIndices<'a>,

    /// Current character being examined
    current: Option<(usize, char)>,

    /// Current line number (starts at 1)
    line: usize,

    /// Byte offset where current line begins
    line_start: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source text
    pub fn new(source: &'a str) -> Self {
        let mut chars = source.char_indices();
        let current = chars.next();
        Lexer {
            source,
            chars,
            current,
            line: 1,
            line_start: 0,
        }
    }

    /// Advance to the next character in the source
    fn advance(&mut self) {
        self.current = self.chars.next();
    }

    /// Peek at the current character without consuming it
    fn peek(&self) -> Option<char> {
        self.current.map(|(_, ch)| ch)
    }

    /// Calculate the current column offset (0-indexed)
    fn column(&self) -> usize {
        match self.current {
            Some((pos, _)) => pos - self.line_start,
            None => self.source.len() - self.line_start,
        }
    }

    /// Scan an identifier: [a-zA-Z][a-zA-Z0-9_]* (uppercase normalized)
    fn scan_identifier(&mut self, _start_pos: usize, start_col: usize) -> Token {
        let mut identifier = String::new();

        // Collect identifier characters
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphabetic() || ch.is_ascii_digit() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        Token {
            token_type: TokenType::Identifier(identifier.to_uppercase()),
            line: self.line,
            column: start_col,
            length: identifier.len(),
        }
    }

    /// Scan a hexadecimal number: $[0-9A-Fa-f]+
    fn scan_hex_number(&mut self, start_col: usize) -> Result<Token, super::LexerError> {
        let mut hex_str = String::new();

        // Skip the $ prefix (already consumed by caller)

        // Collect hex digits
        while let Some(ch) = self.peek() {
            if ch.is_ascii_hexdigit() {
                hex_str.push(ch);
                self.advance();
            } else if ch.is_ascii_alphanumeric() {
                // Invalid hex digit (alphanumeric but not hex)
                return Err(super::LexerError::InvalidHexDigit {
                    ch,
                    line: self.line,
                    column: self.column(),
                });
            } else {
                break;
            }
        }

        // Ensure we got at least one hex digit
        if hex_str.is_empty() {
            return Err(super::LexerError::InvalidHexDigit {
                ch: ' ', // placeholder
                line: self.line,
                column: self.column(),
            });
        }

        // Parse hex value
        let value = u16::from_str_radix(&hex_str, 16)
            .map_err(|_| super::LexerError::NumberTooLarge {
                value: format!("${}", hex_str),
                max: u16::MAX,
                line: self.line,
                column: start_col,
            })?;

        Ok(Token {
            token_type: TokenType::HexNumber(value),
            line: self.line,
            column: start_col,
            length: hex_str.len() + 1, // +1 for $
        })
    }

    /// Scan a binary number: %[01]+
    fn scan_binary_number(&mut self, start_col: usize) -> Result<Token, super::LexerError> {
        let mut bin_str = String::new();

        // Skip the % prefix (already consumed by caller)

        // Collect binary digits
        while let Some(ch) = self.peek() {
            if ch == '0' || ch == '1' {
                bin_str.push(ch);
                self.advance();
            } else if ch.is_ascii_digit() {
                // Invalid binary digit
                return Err(super::LexerError::InvalidBinaryDigit {
                    ch,
                    line: self.line,
                    column: self.column(),
                });
            } else {
                break;
            }
        }

        // Parse binary value
        let value = u16::from_str_radix(&bin_str, 2)
            .map_err(|_| super::LexerError::NumberTooLarge {
                value: format!("%{}", bin_str),
                max: u16::MAX,
                line: self.line,
                column: start_col,
            })?;

        Ok(Token {
            token_type: TokenType::BinaryNumber(value),
            line: self.line,
            column: start_col,
            length: bin_str.len() + 1, // +1 for %
        })
    }

    /// Scan a decimal number: [0-9]+
    fn scan_decimal_number(&mut self, _start_pos: usize, start_col: usize) -> Result<Token, super::LexerError> {
        let mut num_str = String::new();

        // Collect decimal digits
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Parse decimal value
        let value: u16 = num_str.parse()
            .map_err(|_| super::LexerError::NumberTooLarge {
                value: num_str.clone(),
                max: u16::MAX,
                line: self.line,
                column: start_col,
            })?;

        Ok(Token {
            token_type: TokenType::DecimalNumber(value),
            line: self.line,
            column: start_col,
            length: num_str.len(),
        })
    }

    /// Scan a comment: ;.* until newline
    fn scan_comment(&mut self, start_col: usize) -> Token {
        let mut comment = String::new();

        // Skip the ; prefix (already consumed by caller)

        // Collect comment text until newline or EOF
        while let Some(ch) = self.peek() {
            if ch == '\n' || ch == '\r' {
                break;
            }
            comment.push(ch);
            self.advance();
        }

        let length = comment.len() + 1; // +1 for ;

        Token {
            token_type: TokenType::Comment(comment),
            line: self.line,
            column: start_col,
            length,
        }
    }

    /// Scan a single-character token (operators, punctuation)
    fn scan_single_char_token(&mut self, ch: char, start_col: usize) -> Token {
        let token_type = match ch {
            ':' => TokenType::Colon,
            ',' => TokenType::Comma,
            '#' => TokenType::Hash,
            '$' => TokenType::Dollar,
            '%' => TokenType::Percent,
            '=' => TokenType::Equal,
            '(' => TokenType::LParen,
            ')' => TokenType::RParen,
            '.' => TokenType::Dot,
            _ => unreachable!("scan_single_char_token called with invalid character"),
        };

        self.advance();

        Token {
            token_type,
            line: self.line,
            column: start_col,
            length: 1,
        }
    }

    /// Get the next token from the source
    fn next_token(&mut self) -> Result<Option<Token>, super::LexerError> {
        let Some(ch) = self.peek() else {
            // End of file
            return Ok(None);
        };

        let start_col = self.column();
        let start_pos = self.current.map(|(pos, _)| pos).unwrap_or(0);

        match ch {
            // Whitespace (spaces and tabs)
            ' ' | '\t' => {
                let mut whitespace_len = 0;
                while let Some(ch) = self.peek() {
                    if ch == ' ' || ch == '\t' {
                        whitespace_len += 1;
                        self.advance();
                    } else {
                        break;
                    }
                }
                Ok(Some(Token {
                    token_type: TokenType::Whitespace,
                    line: self.line,
                    column: start_col,
                    length: whitespace_len,
                }))
            }

            // Newline (handle both CRLF and LF)
            '\n' | '\r' => {
                let mut length = 1;
                self.advance();

                // Handle CRLF
                if ch == '\r' && self.peek() == Some('\n') {
                    self.advance();
                    length = 2;
                }

                let token = Token {
                    token_type: TokenType::Newline,
                    line: self.line,
                    column: start_col,
                    length,
                };

                // Update line tracking
                self.line += 1;
                self.line_start = self.current.map(|(pos, _)| pos).unwrap_or(self.source.len());

                Ok(Some(token))
            }

            // Comment
            ';' => {
                self.advance(); // consume ;
                Ok(Some(self.scan_comment(start_col)))
            }

            // Hex number or dollar sign
            '$' => {
                self.advance(); // consume $
                // Check if followed by alphanumeric (hex number expected)
                if let Some(next_ch) = self.peek() {
                    if next_ch.is_ascii_alphanumeric() {
                        // Commit to hex number - will error if not valid hex
                        return Ok(Some(self.scan_hex_number(start_col)?));
                    }
                }
                // Standalone $ (for use in addressing modes like $(addr),Y)
                Ok(Some(Token {
                    token_type: TokenType::Dollar,
                    line: self.line,
                    column: start_col,
                    length: 1,
                }))
            }

            // Binary number or percent sign
            '%' => {
                self.advance(); // consume %
                // Check if followed by digit (binary number expected)
                if let Some(next_ch) = self.peek() {
                    if next_ch.is_ascii_digit() {
                        // Commit to binary number - will error if not 0 or 1
                        return Ok(Some(self.scan_binary_number(start_col)?));
                    }
                }
                // Standalone % (though not typically used alone)
                Ok(Some(Token {
                    token_type: TokenType::Percent,
                    line: self.line,
                    column: start_col,
                    length: 1,
                }))
            }

            // Decimal number
            '0'..='9' => Ok(Some(self.scan_decimal_number(start_pos, start_col)?)),

            // Identifier (mnemonic, label, symbol)
            'a'..='z' | 'A'..='Z' => {
                Ok(Some(self.scan_identifier(start_pos, start_col)))
            }

            // Single-character operators
            ':' | ',' | '#' | '=' | '(' | ')' | '.' => {
                Ok(Some(self.scan_single_char_token(ch, start_col)))
            }

            // Unexpected character
            _ => Err(super::LexerError::UnexpectedCharacter {
                ch,
                line: self.line,
                column: start_col,
            }),
        }
    }
}

/// Tokenize assembly source text into a vector of tokens
///
/// This is the main entry point for lexical analysis. It converts source text
/// into a stream of typed tokens with source location information.
///
/// # Arguments
/// * `source` - Assembly source text to tokenize
///
/// # Returns
/// * `Ok(Vec<Token>)` - Successfully tokenized source
/// * `Err(Vec<LexerError>)` - Lexical errors encountered during tokenization
///
/// # Examples
/// ```
/// use lib6502::assembler::lexer::{tokenize, TokenType};
///
/// let tokens = tokenize("LDA #$42").unwrap();
/// assert_eq!(tokens.len(), 4); // LDA, whitespace, #, $42
/// ```
pub fn tokenize(source: &str) -> Result<Vec<Token>, Vec<super::LexerError>> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();
    let mut errors = Vec::new();

    loop {
        match lexer.next_token() {
            Ok(Some(token)) => tokens.push(token),
            Ok(None) => {
                // Add EOF token
                tokens.push(Token {
                    token_type: TokenType::Eof,
                    line: lexer.line,
                    column: lexer.column(),
                    length: 0,
                });
                break;
            }
            Err(err) => {
                errors.push(err);
                // Try to recover by skipping the problematic character
                lexer.advance();
            }
        }
    }

    if errors.is_empty() {
        Ok(tokens)
    } else {
        Err(errors)
    }
}

/// Token stream with lookahead capability for parser consumption
pub struct TokenStream {
    /// Complete token sequence (pre-parsed by lexer)
    tokens: Vec<Token>,

    /// Current read position (index into tokens vec)
    position: usize,
}

impl TokenStream {
    /// Create a new token stream from a vector of tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        TokenStream {
            tokens,
            position: 0,
        }
    }

    /// Peek at the current token without consuming it
    ///
    /// Returns None if at end of token stream.
    ///
    /// # Examples
    /// ```
    /// use lib6502::assembler::{tokenize, TokenStream, TokenType};
    ///
    /// let tokens = tokenize("LDA #$42").unwrap();
    /// let mut stream = TokenStream::new(tokens);
    /// if let Some(token) = stream.peek() {
    ///     assert!(matches!(token.token_type, TokenType::Identifier(_)));
    /// }
    /// ```
    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    /// Peek ahead n tokens without consuming them
    ///
    /// Returns None if n tokens ahead would be past end of stream.
    /// peek_n(0) is equivalent to peek().
    ///
    /// # Examples
    /// ```
    /// use lib6502::assembler::{tokenize, TokenStream, TokenType};
    ///
    /// let tokens = tokenize("LDA #$42").unwrap();
    /// let stream = TokenStream::new(tokens);
    /// // Look ahead to see the # token (skipping whitespace)
    /// if let Some(token) = stream.peek_n(2) {
    ///     assert_eq!(token.token_type, TokenType::Hash);
    /// }
    /// ```
    pub fn peek_n(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.position + n)
    }

    /// Consume and return the current token, advancing the stream
    ///
    /// Returns None if at end of token stream.
    ///
    /// # Examples
    /// ```
    /// use lib6502::assembler::{tokenize, TokenStream, TokenType};
    ///
    /// let tokens = tokenize("LDA").unwrap();
    /// let mut stream = TokenStream::new(tokens);
    /// let token = stream.consume().unwrap();
    /// assert!(matches!(token.token_type, TokenType::Identifier(_)));
    /// ```
    pub fn consume(&mut self) -> Option<Token> {
        if self.position < self.tokens.len() {
            let token = self.tokens[self.position].clone();
            self.position += 1;
            Some(token)
        } else {
            None
        }
    }

    /// Skip all whitespace and newline tokens
    ///
    /// Advances the stream position past any Whitespace or Newline tokens.
    /// Stops at the first non-whitespace token or EOF.
    ///
    /// # Examples
    /// ```
    /// use lib6502::assembler::{tokenize, TokenStream, TokenType};
    ///
    /// let tokens = tokenize("LDA   \n  #$42").unwrap();
    /// let mut stream = TokenStream::new(tokens);
    /// stream.consume(); // consume LDA
    /// stream.skip_whitespace(); // skip spaces and newline
    /// let token = stream.peek().unwrap();
    /// assert_eq!(token.token_type, TokenType::Hash);
    /// ```
    pub fn skip_whitespace(&mut self) {
        while let Some(token) = self.peek() {
            match token.token_type {
                TokenType::Whitespace | TokenType::Newline => {
                    self.position += 1;
                }
                _ => break,
            }
        }
    }

    /// Check if the stream is at end of file
    ///
    /// Returns true if current position is at EOF token or past end of stream.
    ///
    /// # Examples
    /// ```
    /// use lib6502::assembler::{tokenize, TokenStream};
    ///
    /// let tokens = tokenize("LDA").unwrap();
    /// let mut stream = TokenStream::new(tokens);
    /// assert!(!stream.is_eof()); // at LDA
    /// stream.consume(); // consume LDA
    /// assert!(stream.is_eof()); // at EOF
    /// ```
    pub fn is_eof(&self) -> bool {
        match self.peek() {
            Some(token) => matches!(token.token_type, TokenType::Eof),
            None => true,
        }
    }

    /// Get the current token's source location for error reporting
    ///
    /// Returns (line, column) tuple. If at EOF or past end, returns the
    /// location of the EOF token or (0, 0) if no tokens exist.
    ///
    /// # Examples
    /// ```
    /// use lib6502::assembler::{tokenize, TokenStream};
    ///
    /// let tokens = tokenize("LDA").unwrap();
    /// let stream = TokenStream::new(tokens);
    /// let (line, column) = stream.current_location();
    /// assert_eq!(line, 1);
    /// assert_eq!(column, 0);
    /// ```
    pub fn current_location(&self) -> (usize, usize) {
        match self.peek() {
            Some(token) => (token.line, token.column),
            None => {
                // If past end, try to get EOF token location
                if let Some(eof) = self.tokens.last() {
                    (eof.line, eof.column)
                } else {
                    (0, 0)
                }
            }
        }
    }
}
