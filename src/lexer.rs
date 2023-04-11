use logos::Logos;

#[derive(Debug)]
pub enum LexerError {
    UnrecognizedLexeme,
    UnexpectedEof,
}

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
    Error,
}
logos_nom_bridge::token_parser!(token: Lexeme);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexer_test() {
        let mut lexer = Lexeme::lexer(
r"# Attacker capabilities

att(X) /\ att(Y) => att(pair(X,Y)).
att(pair(X,Y)) => att(X).
att(pair(X,Y)) => att(Y).

att(X) => att(h(X)).

att(X) /\ att(Y) => att(senc(X,Y)).
att(senc(X,Y)) /\ att(Y) => att(X).

att(X) /\ att(Y) => att(aenc(X,Y)).
att(aenc(X,pub(Y))) /\ att(Y) => att(X).

att(X) => att(pub(X)).

# Test protocol

att(kleak).
att(senc(secret,ksecret)).
att(senc(leak,kleak)).

# att(secret) is not derivable.
# att(leak) is derivable.
");

        assert_eq!(lexer.next(), Some(Ok(Lexeme::Newline)));
    }
}