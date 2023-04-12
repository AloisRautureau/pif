//! Lexer module
//! `Lexeme::lexer(source)` takes a string slice as input, then creates a lazily evaluated
//! iterator of lexemes parsed from said source

use logos::Logos;

/// Valid lexemes for Pif
#[derive(Logos, Eq, PartialEq, Clone, Debug)]
pub enum Lexeme {
    #[token("=>")]
    Implies,
    #[token(r"(")]
    OpeningParentheses,
    #[token(r")")]
    ClosingParentheses,
    #[token(",")]
    Comma,
    #[token("/\\")]
    And,
    #[token(".")]
    Stop,

    #[regex(r"[A-Z][a-zA-Z0-9_']*", |lex| lex.slice().to_string())]
    Variable(String),
    #[regex(r"[a-z][a-zA-Z0-9_']*", |lex| lex.slice().to_string())]
    Constant(String),
    #[regex(r"[0-9_]+", |lex| lex.slice().parse::<usize>().unwrap())]
    Integer(usize),

    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    #[regex(r"#[^\n]*\n", logos::skip)]
    UnrecognizedLexeme,
}
logos_nom_bridge::token_parser!(token: Lexeme);
