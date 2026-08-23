use std::fmt;
use std::str::Chars;

/// A single lexical unit produced by the lexer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Ident(String),
    Str(String),
    LBrace,
    RBrace,
    LParen,
    RParen,
    Equals,
    Comma,
    Newline,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

impl Token {
    fn new(kind: TokenKind, line: usize, col: usize) -> Self {
        Self { kind, line, col }
    }

    pub fn describe(&self) -> String {
        match &self.kind {
            TokenKind::Ident(s) => format!("identifier '{s}'"),
            TokenKind::Str(s) => format!("string \"{s}\""),
            TokenKind::LBrace => "'{'".into(),
            TokenKind::RBrace => "'}'".into(),
            TokenKind::LParen => "'('".into(),
            TokenKind::RParen => "')'".into(),
            TokenKind::Equals => "'='".into(),
            TokenKind::Comma => "','".into(),
            TokenKind::Newline => "end of line".into(),
            TokenKind::Eof => "end of file".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "lex error at {}:{}: {}",
            self.line, self.col, self.message
        )
    }
}

impl std::error::Error for LexError {}

#[derive(Debug, Clone)]
struct Cursor<'a> {
    chars: std::iter::Peekable<Chars<'a>>,
    line: usize,
    col: usize,
}

impl<'a> Cursor<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
            line: 1,
            col: 1,
        }
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }
}

/// Stage 2 of the pipeline: converts raw source text into a token stream.
#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    cursor: Cursor<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            cursor: Cursor::new(input),
        }
    }

    /// Tokenizes the entire input, collapsing runs of blank lines.
    /// The stream is guaranteed to end with exactly one `TokenKind::Eof`.
    pub fn tokenize(mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens: Vec<Token> = Vec::new();

        loop {
            let (line, col) = (self.cursor.line, self.cursor.col);
            match self.cursor.peek() {
                None => break,
                Some('\n') => {
                    self.cursor.bump();
                    if !matches!(tokens.last(), Some(t) if t.kind == TokenKind::Newline) {
                        tokens.push(Token::new(TokenKind::Newline, line, col));
                    }
                }
                Some(c) if c == ' ' || c == '\t' || c == '\r' => {
                    self.cursor.bump();
                }
                Some('-') => {
                    self.cursor.bump();
                    if self.cursor.peek() == Some('-') {
                        while let Some(c) = self.cursor.peek() {
                            if c == '\n' {
                                break;
                            }
                            self.cursor.bump();
                        }
                    } else {
                        return Err(LexError {
                            message: "unexpected character '-'".into(),
                            line,
                            col,
                        });
                    }
                }
                Some(c @ ('{' | '}' | '(' | ')' | '=' | ',')) => {
                    self.cursor.bump();
                    let kind = match c {
                        '{' => TokenKind::LBrace,
                        '}' => TokenKind::RBrace,
                        '(' => TokenKind::LParen,
                        ')' => TokenKind::RParen,
                        '=' => TokenKind::Equals,
                        _ => TokenKind::Comma,
                    };
                    tokens.push(Token::new(kind, line, col));
                }
                Some(quote @ ('\'' | '"')) => {
                    let s = self.read_string(quote)?;
                    tokens.push(Token::new(TokenKind::Str(s), line, col));
                }
                Some(c) if is_ident_start(c) => {
                    let ident = self.read_ident();
                    tokens.push(Token::new(TokenKind::Ident(ident), line, col));
                }
                Some(c) => {
                    return Err(LexError {
                        message: format!("unexpected character '{c}'"),
                        line,
                        col,
                    });
                }
            }
        }

        tokens.push(Token::new(
            TokenKind::Eof,
            self.cursor.line,
            self.cursor.col,
        ));
        Ok(tokens)
    }

    fn read_string(&mut self, quote: char) -> Result<String, LexError> {
        let (start_line, start_col) = (self.cursor.line, self.cursor.col);
        self.cursor.bump();

        let mut out = String::new();
        loop {
            match self.cursor.peek() {
                None | Some('\n') => {
                    return Err(LexError {
                        message: format!(
                            "unterminated string starting at {start_line}:{start_col}"
                        ),
                        line: start_line,
                        col: start_col,
                    });
                }
                Some(c) if c == quote => {
                    self.cursor.bump();
                    return Ok(out);
                }
                Some(_) => out.push(self.cursor.bump().expect("peeked char")),
            }
        }
    }

    fn read_ident(&mut self) -> String {
        let mut out = String::new();
        loop {
            match self.cursor.peek() {
                Some(c) if is_ident_continue(c) => {
                    out.push(self.cursor.bump().expect("peeked char"))
                }
                _ => return out,
            }
        }
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '@'
}

fn is_ident_continue(c: char) -> bool {
    is_ident_start(c) || matches!(c, '/' | '.' | '-')
}
