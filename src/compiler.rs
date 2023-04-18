use crate::scanner::{self, TokenType};

pub fn compile(source: String) {
    let mut scanner = scanner::Scanner::init(source);

    // let mut line = 0;

    loop {
        let token = scanner.scan_token();

        println!("{:?} {} {}", token.token_type, token.lexeme, token.line);

        if token.token_type == TokenType::Eof {
            break;
        }
    }
}
