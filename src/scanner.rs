use std::{collections::HashMap, thread::current};

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
    keywords: HashMap<&'static str, TokenType>,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            keywords: Self::populate_hashmap(),
        }
    }

    fn populate_hashmap() -> HashMap<&'static str, TokenType> {
        let mut keywords = HashMap::new();
        keywords.insert("and", TokenType::And);
        keywords.insert("class", TokenType::Class);
        keywords.insert("else", TokenType::Else);
        keywords.insert("false", TokenType::False);
        keywords.insert("for", TokenType::For);
        keywords.insert("fun", TokenType::Fun);
        keywords.insert("if", TokenType::If);
        keywords.insert("nil", TokenType::Nil);
        keywords.insert("or", TokenType::Or);
        keywords.insert("print", TokenType::Print);
        keywords.insert("return", TokenType::Return);
        keywords.insert("super", TokenType::Super);
        keywords.insert("this", TokenType::This);
        keywords.insert("true", TokenType::True);
        keywords.insert("var", TokenType::Var);
        keywords.insert("while", TokenType::While);
        keywords
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
                    TokenType::Less
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

            '"' => self.string(interpreter),

            _ => {
                if ch.is_ascii_digit() {
                    self.number();
                } else if ch.is_ascii_alphabetic() || ch == '_' {
                    self.identifier();
                } else {
                    interpreter.error(self.line, format!("Unexpected character '{ch}'"));
                }
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

    fn peek_next(&self) -> char {
        self.source[self.current..].chars().nth(1).unwrap_or('\0')
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.peek() != expected {
            return false;
        }

        self.advance();
        true
    }

    fn string(&mut self, interpreter: &mut Interpreter) {
        while self.peek() != '"' && !self.is_at_end() {
            let _ = self.advance();
        }

        if self.is_at_end() {
            interpreter.error(self.line, format!("Incomplete string literal"));
            return;
        }

        let _ = self.advance();

        let value = self.source[(self.start + 1)..(self.current - 1)].to_owned();

        self.add_token_with_literal(TokenType::String, Some(Literal::String(value)));
    }

    fn number(&mut self) {
        loop {
            if self.peek().is_ascii_digit() {
                self.advance();
            } else if self.peek() == '.' && self.peek_next().is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }

        self.add_token_with_literal(
            TokenType::Number,
            Some(Literal::Number(
                self.source[self.start..self.current].parse().unwrap(),
            )),
        );
    }

    fn identifier(&mut self) {
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text = &self.source[self.start..self.current];
        let token_type = self
            .keywords
            .get(text)
            .copied()
            .unwrap_or(TokenType::Identifier);

        self.add_token(token_type);
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
