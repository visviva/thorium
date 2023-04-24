use arcstr::{ArcStr, Substr};
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub struct Scanner {
    pub source: ArcStr,
    start: usize,
    current: usize,
    line: usize,
}

#[derive(Debug, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive, Hash)]
#[repr(u8)]
pub enum TokenType {
    // Single-character tokens.
    Leftparen,
    Rightparen,
    Leftbrace,
    Rightbrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // One or two character tokens.
    Bang,
    Bangequal,
    Equal,
    Equalequal,
    Greater,
    Greaterequal,
    Less,
    Lessequal,
    // Literals.
    Identifier,
    String,
    Number,
    // Keywords.
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    // Special
    Error,
    Eof,
}
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: Substr,
    pub line: usize,
}

impl Token {
    pub fn make_token(tt: TokenType, lexeme: &str, line: usize) -> Self {
        Token {
            token_type: tt,
            lexeme: Substr::from(lexeme),
            line,
        }
    }

    pub fn make_error_token(error: &str, line: usize) -> Self {
        Token {
            token_type: TokenType::Error,
            lexeme: Substr::from(error),
            line,
        }
    }
}

impl Scanner {
    pub fn init(source: String) -> Self {
        Scanner {
            source: ArcStr::from(source),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        let c = self.advance();

        if c.is_alphabetic() {
            return self.parse_identifier();
        }

        if c.is_ascii_digit() {
            return self.parse_number();
        }

        match c {
            '(' => self.make_token(TokenType::Leftparen),
            ')' => self.make_token(TokenType::Rightparen),
            '{' => self.make_token(TokenType::Leftbrace),
            '}' => self.make_token(TokenType::Rightbrace),
            ';' => self.make_token(TokenType::Semicolon),
            ',' => self.make_token(TokenType::Comma),
            '.' => self.make_token(TokenType::Dot),
            '-' => self.make_token(TokenType::Minus),
            '+' => self.make_token(TokenType::Plus),
            '/' => self.make_token(TokenType::Slash),
            '*' => self.make_token(TokenType::Star),

            '!' => {
                let matched = self.match_char('=');
                self.make_token(if matched {
                    TokenType::Bangequal
                } else {
                    TokenType::Bang
                })
            }

            '=' => {
                let matched = self.match_char('=');
                self.make_token(if matched {
                    TokenType::Equalequal
                } else {
                    TokenType::Equal
                })
            }

            '<' => {
                let matched = self.match_char('=');
                self.make_token(if matched {
                    TokenType::Lessequal
                } else {
                    TokenType::Less
                })
            }

            '>' => {
                let matched = self.match_char('=');
                self.make_token(if matched {
                    TokenType::Greaterequal
                } else {
                    TokenType::Greater
                })
            }

            '"' => self.parse_string(),
            '\0' => self.make_token(TokenType::Eof),

            _ => Token::make_error_token("Unexpected character.", self.line),
        }
    }

    fn make_token(&mut self, t: TokenType) -> Token {
        let lexeme = &self.source[self.start..self.current];
        Token::make_token(t, lexeme, self.line)
    }

    fn is_at_end(&self) -> bool {
        self.current == (self.source.len())
    }

    fn advance(&mut self) -> char {
        let c = self.source.as_bytes()[self.current] as char;
        self.current += 1;
        c
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        };

        let c = self.source.as_bytes()[self.current] as char;
        if c != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            let c = self.peek();
            match c {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '/' => {
                    if self.is_at_end() {
                        return;
                    }
                    if self.peek_next() == '/' {
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        }
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.as_bytes()[self.current] as char
    }

    fn peek_next(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.as_bytes()[self.current + 1] as char
    }

    fn parse_string(&mut self) -> Token {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            Token::make_error_token("Unterminated string.", self.line)
        } else {
            self.advance();
            self.make_token(TokenType::String)
        }
    }

    fn parse_number(&mut self) -> Token {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
        }

        while self.peek().is_ascii_digit() {
            self.advance();
        }

        self.make_token(TokenType::Number)
    }

    fn parse_identifier(&mut self) -> Token {
        loop {
            let c = self.peek();
            if c.is_alphanumeric() {
                let _ = self.advance();
            } else {
                break;
            }
        }

        let token_type = self.get_identifier_type();
        self.make_token(token_type)
    }

    fn get_identifier_type(&self) -> TokenType {
        let c = self.source.as_bytes()[self.start] as char;

        match c {
            'a' => self.check_keyword(1, "nd", TokenType::And),
            'c' => self.check_keyword(1, "lass", TokenType::Class),
            'e' => self.check_keyword(1, "lse", TokenType::Else),
            'f' => {
                if (self.current - self.start) > 1 {
                    let c = self.source.as_bytes()[self.start + 1] as char;
                    match c {
                        'a' => self.check_keyword(2, "lse", TokenType::False),
                        'o' => self.check_keyword(2, "r", TokenType::For),
                        'u' => self.check_keyword(2, "n", TokenType::Fun),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'i' => self.check_keyword(1, "f", TokenType::If),
            'n' => self.check_keyword(1, "il", TokenType::Nil),
            'o' => self.check_keyword(1, "r", TokenType::Or),
            'p' => self.check_keyword(1, "rint", TokenType::Print),
            'r' => self.check_keyword(1, "eturn", TokenType::Return),
            's' => self.check_keyword(1, "uper", TokenType::Super),
            't' => {
                if (self.current - self.start) > 1 {
                    let c = self.source.as_bytes()[self.start + 1] as char;
                    match c {
                        'h' => self.check_keyword(2, "is", TokenType::This),
                        'r' => self.check_keyword(2, "ue", TokenType::True),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'v' => self.check_keyword(1, "ar", TokenType::Var),
            'w' => self.check_keyword(1, "hile", TokenType::While),
            _ => TokenType::Identifier,
        }
    }

    fn check_keyword(&self, start: usize, rest: &str, token_type: TokenType) -> TokenType {
        let length = rest.len();
        let substring_start = self.start + start;
        let to_be_matched = &self.source[substring_start..(substring_start + length)];
        if ((self.current - self.start) == (start + length)) && to_be_matched == rest {
            token_type
        } else {
            TokenType::Identifier
        }
    }
}
