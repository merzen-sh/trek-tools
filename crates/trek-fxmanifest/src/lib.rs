//! Parsing of FiveM `fxmanifest.lua` files.
//!
//! The crate is organized as a compiler-like pipeline:
//!
//! `Source` → [`Lexer`] → [`Parser`] → [`Manifest`] (AST)
//!
//! # Example
//!
//! ```
//! use trek_fxmanifest::{Key, parse};
//!
//! let manifest = parse("fx_version 'cerulean'\nversion '1.2.3'\n").unwrap();
//! assert_eq!(manifest.fx_version.as_deref(), Some("cerulean"));
//! assert_eq!(manifest.version(), Some("1.2.3"));
//! assert!(manifest.values(&Key::ClientScripts).is_empty());
//! ```

mod ast;
mod diagnostic;
mod lexer;
mod parser;
mod source;

use std::fmt;

pub use ast::{Game, Key, Manifest, Statement};
pub use diagnostic::{Codeframe, Diagnostic, Severity, Span};
pub use lexer::{LexError, Lexer, Token, TokenKind};
pub use parser::{ParseError, Parser};
pub use source::Source;

/// Any failure raised while running the pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Lex(LexError),
    Parse(ParseError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Lex(e) => write!(f, "{e}"),
            Error::Parse(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Lex(e) => Some(e),
            Error::Parse(e) => Some(e),
        }
    }
}

impl From<LexError> for Error {
    fn from(value: LexError) -> Self {
        Error::Lex(value)
    }
}

impl From<ParseError> for Error {
    fn from(value: ParseError) -> Self {
        Error::Parse(value)
    }
}

/// Runs the full pipeline over raw manifest text and returns the AST.
pub fn parse(input: &str) -> Result<Manifest, Error> {
    let tokens = Lexer::new(input).tokenize()?;
    let manifest = Parser::new(tokens).parse()?;
    Ok(manifest)
}
