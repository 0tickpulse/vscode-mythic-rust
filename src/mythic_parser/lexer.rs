use crate::errors::error_registry::SyntaxError;
use crate::utilities::positions_and_ranges::{CustomPosition, CustomRange};

/// All types of tokens for the Mythic parser.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Hash, Copy)]
pub enum TokenType {
    LeftSquareBracket,
    RightSquareBracket,
    LeftBrace,
    RightBrace,
    Semicolon,
    Equal,
    Dash,
    At,
    Tilde,
    Question,
    Exclamation,
    Colon,
    LessThan,
    GreaterThan,
    Dot,
    Percent,
    Identifier,
    String,
    Number,
    Space,
    Eof,
}

fn max_length(values: &[&str]) -> usize {
    values.iter().map(|&s| s.len()).max().unwrap_or(0)
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub struct MythicToken {
    pub source: String,
    pub type_: TokenType,
    pub lexeme: Option<String>,
    pub literal: Option<String>,
    pub line: u32,
    pub start: u32,
    pub current: u32,
}

impl MythicToken {
    pub fn new(
        source: String,
        type_: TokenType,
        lexeme: Option<String>,
        literal: Option<String>,
        line: u32,
        start: u32,
        current: u32,
    ) -> Self {
        Self {
            source,
            type_,
            lexeme,
            literal,
            line,
            start,
            current,
        }
    }

    pub fn length(&self) -> usize {
        self.lexeme.as_ref().map(|s| s.len()).unwrap_or(0)
    }

    pub fn get_range(&self) -> CustomRange {
        CustomRange::new(
            CustomPosition::from_offset(self.start, &self.source),
            CustomPosition::from_offset(self.current, &self.source),
        )
    }
}

pub struct MythicScanner {
    source: String,
    tokens: Vec<MythicToken>,
    start: u32,
    current: u32,
    line: u32,
}

impl MythicScanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<MythicToken>, SyntaxError> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }
        self.tokens.push(MythicToken::new(
            self.source.clone(),
            TokenType::Eof,
            None,
            None,
            self.line,
            self.start,
            self.current,
        ));
        Ok(self.tokens.clone())
    }

    fn scan_token(&mut self) -> Result<(), SyntaxError> {
        let c = self.advance();
        match c {
            '[' => self.add_token(TokenType::LeftSquareBracket, None),
            ']' => self.add_token(TokenType::RightSquareBracket, None),
            '{' => self.add_token(TokenType::LeftBrace, None),
            '}' => self.add_token(TokenType::RightBrace, None),
            ';' => self.add_token(TokenType::Semicolon, None),
            '=' => self.add_token(TokenType::Equal, None),
            '-' => self.add_token(TokenType::Dash, None),
            '@' => self.add_token(TokenType::At, None),
            '~' => self.add_token(TokenType::Tilde, None),
            '?' => self.add_token(TokenType::Question, None),
            '!' => self.add_token(TokenType::Exclamation, None),
            ':' => self.add_token(TokenType::Colon, None),
            '<' => self.add_token(TokenType::LessThan, None),
            '>' => self.add_token(TokenType::GreaterThan, None),
            '.' => self.add_token(TokenType::Dot, None),
            '%' => self.add_token(TokenType::Percent, None),
            ' ' => self.add_token(TokenType::Space, None),
            '\r' => (),
            '\t' => (),
            '\n' => self.line += 1,
            '"' => {
                self.string('"')?;
            }
            '\'' => {
                self.string('\'')?;
            }
            _ => {
                if c.is_ascii_digit() {
                    self.number()?;
                } else if c.is_alphabetic() || c == '_' {
                    self.identifier()?;
                } else {
                    return Err(SyntaxError::new(
                        self.get_range(),
                        format!("Unexpected character: {}", c),
                    ));
                }

                return Err(SyntaxError::new(
                    self.get_range(),
                    format!("Unexpected character: {}", c),
                ));
            }
        };
        Ok(())
    }

    fn number(&mut self) -> Result<(), SyntaxError> {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        let value = self.source[self.start as usize..self.current as usize].to_string();
        self.add_token(TokenType::Number, Some(&value));
        Ok(())
    }

    fn identifier(&mut self) -> Result<(), SyntaxError> {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }
        let value = self.source[self.start as usize..self.current as usize].to_string();
        self.add_token(TokenType::Identifier, Some(&value));
        Ok(())
    }

    fn string(&mut self, end: char) -> Result<(), SyntaxError> {
        while self.peek() != end && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        self.advance();
        if self.is_at_end() {
            return Err(SyntaxError::new(
                self.get_range(),
                "Unterminated string.".to_string(),
            ));
        }
        let value = self.source[(self.start + 1) as usize..(self.current - 1) as usize].to_string();
        self.add_token(TokenType::String, Some(&value));
        Ok(())
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source
                .chars()
                .nth(self.current as usize)
                .unwrap_or('\0')
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() as u32 {
            '\0'
        } else {
            self.source
                .chars()
                .nth((self.current + 1) as usize)
                .unwrap_or('\0')
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len() as u32
    }

    fn add_token(&mut self, type_: TokenType, literal: Option<&str>) {
        let lexeme = self.source[self.start as usize..self.current as usize].to_string();
        let literal = literal.map(|s| s.to_string());
        self.tokens.push(MythicToken::new(
            self.source.clone(),
            type_,
            Some(lexeme),
            literal,
            self.line,
            self.start,
            self.current,
        ));
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source
            .chars()
            .nth((self.current - 1) as usize)
            .unwrap()
    }

    fn get_position(&self) -> CustomPosition {
        CustomPosition::from_offset(self.current, &self.source)
    }

    fn get_range(&self) -> CustomRange {
        CustomRange::new(
            CustomPosition::from_offset(self.start, &self.source),
            CustomPosition::from_offset(self.current, &self.source),
        )
    }
}
