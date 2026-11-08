use std::{collections::HashMap, thread::current};

use crate::{
    interpreter::{self, Interpreter},
    token::{Literal, Token},
    token_type::TokenType,
};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    start_line: usize,
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
            start_line: 1,
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
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
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

#[cfg(test)]
mod test {
    use crate::{interpreter, scanner, token};

    use super::*;

    fn token_types(sources: &str) -> Vec<TokenType> {
        let mut scanner = Scanner::new(sources.to_string());
        let mut interpreter = Interpreter::new();

        scanner
            .scan_tokens(&mut interpreter)
            .iter()
            .map(|token| token.token_type)
            .collect()
    }

    fn return_lines(sources: &str, n: usize) -> usize {
        let mut scanner = Scanner::new(sources.to_string());
        let mut interpreter = Interpreter::new();

        scanner
            .scan_tokens(&mut interpreter)
            .iter()
            .nth(n)
            .unwrap()
            .line
    }

    #[test]
    fn scan_operators() {
        assert_eq!(
            token_types("( ) { } ; . , + - * / ! = < >"),
            vec![
                TokenType::LeftParen,
                TokenType::RightParen,
                TokenType::LeftBrace,
                TokenType::RightBrace,
                TokenType::Semicolon,
                TokenType::Dot,
                TokenType::Comma,
                TokenType::Plus,
                TokenType::Minus,
                TokenType::Star,
                TokenType::Slash,
                TokenType::Bang,
                TokenType::Equal,
                TokenType::Less,
                TokenType::Greater,
                TokenType::Eof
            ]
        );
    }

    #[test]
    fn scans_two_character_operator() {
        assert_eq!(
            token_types("!= == >= <="),
            vec![
                TokenType::BangEqual,
                TokenType::EqualEqual,
                TokenType::GreaterEqual,
                TokenType::LessEqual,
                TokenType::Eof
            ]
        );
    }

    #[test]
    fn verify_comments() {
        assert_eq!(token_types("// + ="), vec![TokenType::Eof]);
        assert_eq!(
            token_types("// \n +"),
            vec![TokenType::Plus, TokenType::Eof]
        );
    }

    #[test]
    fn scan_keywords_and_identifiers() {
        assert_eq!(token_types("and"), vec![TokenType::And, TokenType::Eof]);
        assert_eq!(token_types("class"), vec![TokenType::Class, TokenType::Eof]);
        assert_eq!(token_types("else"), vec![TokenType::Else, TokenType::Eof]);
        assert_eq!(token_types("false"), vec![TokenType::False, TokenType::Eof]);
        assert_eq!(token_types("for"), vec![TokenType::For, TokenType::Eof]);
        assert_eq!(token_types("fun"), vec![TokenType::Fun, TokenType::Eof]);
        assert_eq!(token_types("if"), vec![TokenType::If, TokenType::Eof]);
        assert_eq!(token_types("nil"), vec![TokenType::Nil, TokenType::Eof]);
        assert_eq!(token_types("or"), vec![TokenType::Or, TokenType::Eof]);
        assert_eq!(token_types("print"), vec![TokenType::Print, TokenType::Eof]);
        assert_eq!(
            token_types("return"),
            vec![TokenType::Return, TokenType::Eof]
        );
        assert_eq!(token_types("super"), vec![TokenType::Super, TokenType::Eof]);
        assert_eq!(token_types("this"), vec![TokenType::This, TokenType::Eof]);
        assert_eq!(token_types("true"), vec![TokenType::True, TokenType::Eof]);
        assert_eq!(token_types("var"), vec![TokenType::Var, TokenType::Eof]);
        assert_eq!(token_types("while"), vec![TokenType::While, TokenType::Eof]);
    }

    #[test]
    fn scan_numbers() {
        assert_eq!(
            token_types("9 10"),
            vec![TokenType::Number, TokenType::Number, TokenType::Eof]
        );

        assert_eq!(
            token_types("9."),
            vec![TokenType::Number, TokenType::Dot, TokenType::Eof]
        );

        assert_eq!(
            token_types("9.9.9 10.11",),
            vec![
                TokenType::Number,
                TokenType::Dot,
                TokenType::Number,
                TokenType::Number,
                TokenType::Eof
            ]
        );
    }

    #[test]
    fn scan_strings() {
        assert_eq!(token_types(""), vec![TokenType::Eof]);
        assert_eq!(
            token_types(r#""""#),
            vec![TokenType::String, TokenType::Eof]
        );
        assert_eq!(
            token_types(r#""and or""#),
            vec![TokenType::String, TokenType::Eof]
        );
        assert_eq!(
            token_types(r#""9 or" and blabla"#),
            vec![
                TokenType::String,
                TokenType::And,
                TokenType::Identifier,
                TokenType::Eof
            ]
        );
    }

    #[test]
    fn test_lines() {
        assert_eq!(return_lines("and \n ! *", 0), 1);
        assert_eq!(return_lines("and \n ! *", 1), 2);
        assert_eq!(return_lines("and \n ! *", 2), 2);
    }
}
