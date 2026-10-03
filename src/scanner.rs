use crate::{
    interpreter::Interpreter,
    token::{Literal, Token},
    token_type::{self, TokenType},
};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 0,
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    pub fn scan_tokens(&mut self, interpreter: &mut Interpreter) -> &[Token] {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token(interpreter);
        }
        self.tokens
            .push(Token::new(TokenType::Eof, String::new(), None, self.line));

        &self.tokens
    }

    fn scan_token(&mut self, interpreter: &mut Interpreter) {
        let ch = self.advance();

        match ch {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),

            '!' => {
                let token_type = if self.match_char('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(token_type);
            }
            '=' => {
                let token_type = if self.match_char('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(token_type);
            }
            '<' => {
                let token_type = if self.match_char('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(token_type);
            }
            '>' => {
                let token_type = if self.match_char('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(token_type);
            }

            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }

            ' ' | '\r' | '\t' => {
                // ignore whitespace
            }
            '\n' => self.line += 1,

            _ => {
                interpreter.error(self.line, format!("Unexpected character '{ch}'"));
            }
        }
    }

    fn advance(&mut self) -> char {
        let ch = self.source[self.current..].chars().next().unwrap();
        self.current += ch.len_utf8();
        ch
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_with_literal(token_type, None);
    }

    fn add_token_with_literal(&mut self, token_type: TokenType, literal: Option<Literal>) {
        let lexeme = self.source[self.start..self.current].to_owned();

        self.tokens.push(Token {
            token_type,
            lexeme,
            literal,
            line: self.line,
        });
    }

    fn peek(&self) -> char {
        self.source[self.current..].chars().next().unwrap_or('\0')
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.peek() != expected {
            return false;
        }

        self.advance();
        true
    }
}

#[test]
fn scans_single_character_tokens() {
    let mut scanner = Scanner::new("()+*".to_string());
    let mut interpreter = Interpreter { had_error: false };

    let tokens = scanner.scan_tokens(&mut interpreter);

    assert_eq!(tokens.len(), 5); // 4 tokens + EOF
    assert_eq!(tokens[0].token_type, TokenType::LeftParen);
    assert_eq!(tokens[1].token_type, TokenType::RightParen);
    assert_eq!(tokens[2].token_type, TokenType::Plus);
    assert_eq!(tokens[3].token_type, TokenType::Star);
}
