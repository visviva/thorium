use std::collections::HashMap;

use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    chunk::{Chunk, OpCode},
    scanner::{self, Token, TokenType},
    vm::InterpretError,
};

#[derive(Debug, TryFromPrimitive, IntoPrimitive, Clone)]
#[repr(u8)]
enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Precedence {
    pub fn higher_precedence(p: Precedence) -> Precedence {
        match <Precedence as TryInto<u8>>::try_into(p) {
            Ok(value) => Precedence::try_from_primitive(value + 1).unwrap(),
            Err(_) => panic!("Cannot detect precedence."),
        }
    }
}

struct Parser {
    pub chunk: Chunk,
    pub current: Token,
    pub previous: Token,
    pub scanner: scanner::Scanner,
    pub had_error: bool,
    pub panic_mode: bool,
    pub parse_rules: HashMap<TokenType, ParseRule>,
}

type ParseFn = fn(&mut Parser);
struct ParseRule {
    prefix: Option<ParseFn>,
    infix: Option<ParseFn>,
    precedence: Precedence,
}

impl Parser {
    pub fn init(source: String) -> Self {
        Parser {
            chunk: Chunk::init(),
            current: Token::make_token(TokenType::Eof, "", 0),
            previous: Token::make_token(TokenType::Eof, "", 0),
            scanner: scanner::Scanner::init(source),
            had_error: false,
            panic_mode: false,
            parse_rules: HashMap::from([
                (
                    TokenType::Leftparen,
                    ParseRule {
                        prefix: Some(Self::grouping),
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Rightparen,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Leftbrace,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Rightbrace,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Comma,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Dot,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Minus,
                    ParseRule {
                        prefix: Some(Self::unary),
                        infix: Some(Self::binary),
                        precedence: Precedence::Term,
                    },
                ),
                (
                    TokenType::Plus,
                    ParseRule {
                        prefix: None,
                        infix: Some(Self::binary),
                        precedence: Precedence::Term,
                    },
                ),
                (
                    TokenType::Semicolon,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Slash,
                    ParseRule {
                        prefix: None,
                        infix: Some(Self::binary),
                        precedence: Precedence::Factor,
                    },
                ),
                (
                    TokenType::Star,
                    ParseRule {
                        prefix: None,
                        infix: Some(Self::binary),
                        precedence: Precedence::Factor,
                    },
                ),
                (
                    TokenType::Bang,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Bangequal,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Equal,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Equalequal,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Greater,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Greaterequal,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Less,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Lessequal,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Identifier,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::String,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Number,
                    ParseRule {
                        prefix: Some(Self::number),
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::And,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Class,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Else,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::False,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::For,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Fun,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::If,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Nil,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Or,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Print,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Return,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Super,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::This,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::True,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Var,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::While,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Error,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Eof,
                    ParseRule {
                        prefix: None,
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
            ]),
        }
    }

    fn advance(&mut self) {
        std::mem::swap(&mut self.previous, &mut self.current);

        loop {
            self.current = self.scanner.scan_token();
            if self.current.token_type != TokenType::Error {
                break;
            }

            let error = self.current.lexeme.to_string();
            self.error_at_current(error.as_str());
        }
    }

    fn error_at_current(&mut self, message: &str) {
        self.had_error = true;
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;

        let token: &Token = &self.current;
        self.error_at(token, message);
    }

    fn error(&mut self, message: &str) {
        self.had_error = true;
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;

        let token: &Token = &self.previous;
        self.error_at(token, message);
    }

    fn error_at(&self, token: &Token, message: &str) {
        let error_loc = match token.token_type {
            TokenType::Eof => "at end".to_string(),
            TokenType::Error => "".to_string(),
            _ => format!("at '{}'", token.lexeme),
        };

        eprintln!("[line {}] Error {}: {}", token.line, error_loc, message);
    }

    fn consume(&mut self, tt: TokenType, msg: &str) {
        if self.current.token_type == tt {
            self.advance();
        } else {
            self.error_at_current(msg);
        }
    }

    fn emit_byte(&mut self, byte: u8) {
        let line = self.previous.line;
        self.current_chunk().write(byte, line);
    }

    // Usage emit_bytes(&[1,2,3,4]);
    fn emit_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.emit_byte(*byte);
        }
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.chunk
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn end_compilation(&mut self) {
        self.emit_return();
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return.into());
    }

    fn number(&mut self) {
        if let Ok(value) = self.previous.lexeme.parse::<f32>() {
            self.emit_constant(value);
        }
    }

    fn emit_constant(&mut self, value: f32) {
        let op: u8 = OpCode::Constant.into();
        let index: u8 = self.make_constant(value);
        self.emit_bytes(&[op, index])
    }

    fn make_constant(&mut self, value: f32) -> u8 {
        let index = self.current_chunk().add_constant(value);
        if index > u8::MAX as usize {
            self.error("Too many constants in one chunk");
            return 0;
        }

        index as u8
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::Rightparen, "Expect ')' after expression.");
    }

    fn unary(&mut self) {
        let op_type = self.previous.token_type.clone();

        self.parse_precedence(Precedence::Unary);

        match op_type {
            TokenType::Minus => self.emit_byte(OpCode::Negate.into()),
            _ => return,
        }
    }

    fn binary(&mut self) {
        let op_type = self.previous.token_type.clone();
        let rule = self.get_rule(&op_type);
        self.parse_precedence(Precedence::higher_precedence(rule.precedence.clone()));

        match op_type {
            TokenType::Plus => self.emit_byte(OpCode::Add.into()),
            TokenType::Minus => self.emit_byte(OpCode::Subtract.into()),
            TokenType::Star => self.emit_byte(OpCode::Multiply.into()),
            TokenType::Slash => self.emit_byte(OpCode::Divide.into()),
            _ => (),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = self.get_rule(&self.previous.token_type).prefix;
        match prefix_rule {
            Some(rule) => rule(self),
            None => {
                self.error("Expect expression.");
                return;
            }
        }

        let precedence: u8 = precedence.into();
        while precedence
            <= self
                .get_rule(&self.current.token_type)
                .precedence
                .clone()
                .into()
        {
            self.advance();
            let infix_rule = self.get_rule(&self.previous.token_type).infix;
            if infix_rule.is_some() {
                infix_rule.unwrap()(self);
            }
        }
    }

    fn get_rule(&self, op_type: &TokenType) -> &ParseRule {
        self.parse_rules.get(op_type).unwrap()
    }
}

pub fn compile(source: String) -> Result<Chunk, InterpretError> {
    let mut parser = Parser::init(source);
    parser.advance();
    parser.expression();
    parser.consume(TokenType::Eof, "Expect end of expression.");
    parser.end_compilation();
    if parser.had_error {
        Err(InterpretError::CompileError)
    } else {
        if cfg!(debug_assertions) {
            parser.chunk.disassemble("code");
        }

        Ok(parser.chunk)
    }
}
