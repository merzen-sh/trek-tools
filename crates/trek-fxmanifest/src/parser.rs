use std::fmt;

use crate::ast::{Key, Manifest, Statement};
use crate::lexer::{Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "parse error at {}:{}: {}",
            self.line, self.col, self.message
        )
    }
}

impl std::error::Error for ParseError {}

/// Stage 3 of the pipeline: applies grammar rules to the token stream.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(mut self) -> Result<Manifest, ParseError> {
        let mut statements = Vec::new();

        loop {
            self.skip_newlines();
            match self.peek().kind.clone() {
                TokenKind::Eof => break,
                TokenKind::Ident(name) => {
                    let key = Key::parse(&name);
                    self.next();
                    statements.push(self.parse_statement(key)?);
                }
                other => {
                    return Err(ParseError {
                        message: format!("expected a declaration key, found {}", describe(&other)),
                        line: self.peek().line,
                        col: self.peek().col,
                    });
                }
            }
        }

        Ok(Manifest::from_statements(statements))
    }

    fn parse_statement(&mut self, key: Key) -> Result<Statement, ParseError> {
        let statement = match self.peek().kind.clone() {
            TokenKind::LParen => {
                self.next();
                let values = self.parse_call_args()?;
                if values.len() == 1 {
                    Statement::Scalar {
                        key,
                        value: values.into_iter().next().expect("exactly one value"),
                    }
                } else {
                    Statement::Table { key, values }
                }
            }
            TokenKind::Equals => {
                self.next();
                match self.peek().kind.clone() {
                    TokenKind::LBrace => {
                        self.next();
                        self.parse_table_body(key)?
                    }
                    TokenKind::Str(value) | TokenKind::Ident(value) => {
                        self.next();
                        Statement::Scalar { key, value }
                    }
                    other => {
                        return Err(ParseError {
                            message: format!(
                                "expected a value or '{{' after '=', found {}",
                                describe(&other)
                            ),
                            line: self.peek().line,
                            col: self.peek().col,
                        });
                    }
                }
            }
            TokenKind::LBrace => {
                self.next();
                self.parse_table_body(key)?
            }
            TokenKind::Str(value) | TokenKind::Ident(value) => {
                self.next();
                Statement::Scalar { key, value }
            }
            other => {
                return Err(ParseError {
                    message: format!(
                        "declaration '{}' has no value, found {}",
                        key,
                        describe(&other)
                    ),
                    line: self.peek().line,
                    col: self.peek().col,
                });
            }
        };

        match self.peek().kind {
            TokenKind::Newline | TokenKind::Eof => Ok(statement),
            _ => Err(ParseError {
                message: format!(
                    "expected end of line, found {}",
                    describe(&self.peek().kind)
                ),
                line: self.peek().line,
                col: self.peek().col,
            }),
        }
    }

    /// Parses `( 'a', 'b', ... )` after the opening parenthesis.
    fn parse_call_args(&mut self) -> Result<Vec<String>, ParseError> {
        let mut values = Vec::new();

        loop {
            match self.peek().kind.clone() {
                TokenKind::RParen => {
                    self.next();
                    return Ok(values);
                }
                TokenKind::Str(value) | TokenKind::Ident(value) => {
                    self.next();
                    values.push(value);
                    match self.peek().kind {
                        TokenKind::Comma => {
                            self.next();
                        }
                        TokenKind::RParen => {}
                        _ => {
                            return Err(ParseError {
                                message: format!(
                                    "expected ',' or ')' in argument list, found {}",
                                    describe(&self.peek().kind)
                                ),
                                line: self.peek().line,
                                col: self.peek().col,
                            });
                        }
                    }
                }
                other => {
                    return Err(ParseError {
                        message: format!("expected a string argument, found {}", describe(&other)),
                        line: self.peek().line,
                        col: self.peek().col,
                    });
                }
            }
        }
    }

    /// Parses `{ 'a', 'b', ... }` up to and including the closing brace;
    /// newlines and trailing commas are allowed inside the table.
    fn parse_table_body(&mut self, key: Key) -> Result<Statement, ParseError> {
        let mut values = Vec::new();

        loop {
            self.skip_newlines();
            match self.peek().kind.clone() {
                TokenKind::RBrace => {
                    self.next();
                    return Ok(Statement::Table { key, values });
                }
                TokenKind::Str(value) | TokenKind::Ident(value) => {
                    self.next();
                    values.push(value);
                    self.skip_newlines();
                    if matches!(self.peek().kind, TokenKind::Comma) {
                        self.next();
                    } else if !matches!(self.peek().kind, TokenKind::RBrace) {
                        return Err(ParseError {
                            message: format!(
                                "expected ',' or '}}' in table '{}', found {}",
                                key,
                                describe(&self.peek().kind)
                            ),
                            line: self.peek().line,
                            col: self.peek().col,
                        });
                    }
                }
                other => {
                    return Err(ParseError {
                        message: format!(
                            "expected a string in table '{}', found {}",
                            key,
                            describe(&other)
                        ),
                        line: self.peek().line,
                        col: self.peek().col,
                    });
                }
            }
        }
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek().kind, TokenKind::Newline) {
            self.next();
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos).cloned()?;
        self.pos += 1;
        Some(token)
    }
}

fn describe(kind: &TokenKind) -> String {
    Token {
        kind: kind.clone(),
        line: 0,
        col: 0,
        offset: 0,
    }
    .describe()
}
