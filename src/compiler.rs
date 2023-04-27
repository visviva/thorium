use std::collections::HashMap;

use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    chunk::{Chunk, OpCode},
    scanner::{self, Token, TokenType},
    value::Value,
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

type ParseFn = fn(&mut Parser, can_assign: bool);
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
                        prefix: Some(Self::unary),
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::Bangequal,
                    ParseRule {
                        prefix: None,
                        infix: Some(Self::binary),
                        precedence: Precedence::Equality,
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
                        infix: Some(Self::binary),
                        precedence: Precedence::Equality,
                    },
                ),
                (
                    TokenType::Greater,
                    ParseRule {
                        prefix: None,
                        infix: Some(Self::binary),
                        precedence: Precedence::Comparison,
                    },
                ),
                (
                    TokenType::Greaterequal,
                    ParseRule {
                        prefix: None,
                        infix: Some(Self::binary),
                        precedence: Precedence::Comparison,
                    },
                ),
                (
                    TokenType::Less,
                    ParseRule {
                        prefix: None,
                        infix: Some(Self::binary),
                        precedence: Precedence::Comparison,
                    },
                ),
                (
                    TokenType::Lessequal,
                    ParseRule {
                        prefix: None,
                        infix: Some(Self::binary),
                        precedence: Precedence::Comparison,
                    },
                ),
                (
                    TokenType::Identifier,
                    ParseRule {
                        prefix: Some(Self::variable),
                        infix: None,
                        precedence: Precedence::None,
                    },
                ),
                (
                    TokenType::String,
                    ParseRule {
                        prefix: Some(Self::string),
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
                        prefix: Some(Self::literal),
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
                        prefix: Some(Self::literal),
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
                        prefix: Some(Self::literal),
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

    fn number(&mut self, _can_assign: bool) {
        if let Ok(value) = self.previous.lexeme.parse() {
            let value = Value::Number(value);
            self.emit_constant(value);
        }
    }

    fn emit_constant(&mut self, value: Value) {
        let op: u8 = OpCode::Constant.into();
        let index: u8 = self.make_constant(value);
        self.emit_bytes(&[op, index])
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let index = self.current_chunk().add_constant(value);
        if index > u8::MAX as usize {
            self.error("Too many constants in one chunk");
            return 0;
        }

        index as u8
    }

    fn grouping(&mut self, _can_assign: bool) {
        self.expression();
        self.consume(TokenType::Rightparen, "Expect ')' after expression.");
    }

    fn unary(&mut self, _can_assign: bool) {
        let op_type = self.previous.token_type.clone();

        self.parse_precedence(Precedence::Unary);

        match op_type {
            TokenType::Minus => self.emit_byte(OpCode::Negate.into()),
            TokenType::Bang => self.emit_byte(OpCode::Not.into()),
            _ => unreachable!(),
        }
    }

    fn binary(&mut self, _can_assign: bool) {
        let op_type = self.previous.token_type.clone();
        let rule = self.get_rule(&op_type);
        self.parse_precedence(Precedence::higher_precedence(rule.precedence.clone()));

        match op_type {
            TokenType::Plus => self.emit_byte(OpCode::Add.into()),
            TokenType::Minus => self.emit_byte(OpCode::Subtract.into()),
            TokenType::Star => self.emit_byte(OpCode::Multiply.into()),
            TokenType::Slash => self.emit_byte(OpCode::Divide.into()),
            TokenType::Bangequal => self.emit_bytes(&[OpCode::Equal.into(), OpCode::Not.into()]),
            TokenType::Equalequal => self.emit_byte(OpCode::Equal.into()),
            TokenType::Greater => self.emit_byte(OpCode::Greater.into()),
            TokenType::Greaterequal => self.emit_bytes(&[OpCode::Less.into(), OpCode::Not.into()]),
            TokenType::Less => self.emit_byte(OpCode::Less.into()),
            TokenType::Lessequal => self.emit_bytes(&[OpCode::Greater.into(), OpCode::Not.into()]),
            _ => unreachable!(),
        }
    }

    fn literal(&mut self, _can_assign: bool) {
        match self.previous.token_type {
            TokenType::False => self.emit_byte(OpCode::False.into()),
            TokenType::True => self.emit_byte(OpCode::True.into()),
            TokenType::Nil => self.emit_byte(OpCode::Nil.into()),
            _ => unreachable!(),
        }
    }

    fn string(&mut self, _can_assign: bool) {
        let v = Value::DynamicString(self.previous.lexeme.to_string());
        self.emit_constant(v);
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();

        let precedence: u8 = precedence.into();
        let can_assign: bool = precedence <= Precedence::Assignment.into();

        let prefix_rule = self.get_rule(&self.previous.token_type).prefix;
        match prefix_rule {
            Some(rule) => rule(self, can_assign),
            None => {
                self.error("Expect expression.");
                return;
            }
        }

        while precedence
            <= self
                .get_rule(&self.current.token_type)
                .precedence
                .clone()
                .into()
        {
            self.advance();
            let infix_rule = self.get_rule(&self.previous.token_type).infix;
            if let Some(rule) = infix_rule {
                rule(self, can_assign);
            }
        }

        if can_assign && self.match_token(TokenType::Equal) {
            self.error("Invalid assignment target");
        }
    }

    fn get_rule(&self, op_type: &TokenType) -> &ParseRule {
        self.parse_rules.get(op_type).unwrap()
    }

    fn match_token(&mut self, t: TokenType) -> bool {
        if !self.check(t) {
            false
        } else {
            self.advance();
            true
        }
    }

    fn declaration(&mut self) {
        if self.match_token(TokenType::Var) {
            self.variable_declaration();
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn statement(&mut self) {
        if self.match_token(TokenType::Print) {
            self.print_statement();
        } else {
            self.expression_statement();
        }
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_byte(OpCode::Print.into());
    }

    fn check(&self, t: TokenType) -> bool {
        self.current.token_type == t
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emit_byte(OpCode::Pop.into());
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;

        while self.current.token_type != TokenType::Eof {
            if self.previous.token_type == TokenType::Semicolon {
                return;
            }

            match self.current.token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => self.advance(),
            }
        }
    }

    fn variable_declaration(&mut self) {
        let global = self.parse_variable("Expect a variable name.");

        if self.match_token(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil.into());
        }

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration",
        );

        self.define_variable(global);
    }

    fn parse_variable(&mut self, error: &str) -> u8 {
        self.consume(TokenType::Identifier, error);
        self.identifier_constant(&self.previous.clone())
    }

    fn define_variable(&mut self, global: u8) {
        self.emit_bytes(&[OpCode::DefineGlobal.into(), global]);
    }

    fn identifier_constant(&mut self, t: &Token) -> u8 {
        self.make_constant(Value::DynamicString(t.lexeme.to_string()))
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.previous.clone(), can_assign);
    }

    fn named_variable(&mut self, name: Token, can_assign: bool) {
        let arg = self.identifier_constant(&name);
        if can_assign && self.match_token(TokenType::Equal) {
            self.expression();
            self.emit_bytes(&[OpCode::SetGlobal.into(), arg]);
        } else {
            self.emit_bytes(&[OpCode::GetGlobal.into(), arg]);
        }
    }
}

pub fn compile(source: String) -> Result<Chunk, InterpretError> {
    let mut parser = Parser::init(source);
    parser.advance();

    while !parser.match_token(TokenType::Eof) {
        parser.declaration();
    }

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
