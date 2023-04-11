//! Parser module
//! A parser takes a vector of lexemes as input, and creates an AST
//! by walking though said vector

use logos_nom_bridge::Tokens;
use nom::branch::alt;
use nom::multi::{many0, separated_list0};
use nom::sequence::{delimited, pair, terminated};
use crate::ast::*;
use crate::lexer::Lexeme;

#[derive(Debug)]
pub enum ParserError {
    ExpectedTerm,
    ExpectedAtom,
    ExpectedRule,
    UnexpectedEof,
    ExpectedEof
}

type Input<'source> = Tokens<'source, Lexeme>;

pub struct Parser;
impl Parser {
    pub fn parse_query(input: Input) -> Atom<String> {
        terminated(Self::parse_atom, Lexeme::Stop)(input).unwrap().1
    }

    pub fn parse_rules(input: Input) -> Vec<Rule<String>> {
        many0(Self::parse_rule)(input).unwrap().1
    }
    fn parse_rule(input: Input) -> nom::IResult<Input, Rule<String>> {
        if let Ok((rest, premisses)) = terminated(Self::parse_atoms, Lexeme::Implies)(input.clone()) {
            let (rest, conclusion) = terminated(Self::parse_atom, Lexeme::Stop)(rest)?;
            Ok((rest, Rule {
                premisses,
                conclusion
            }))
        } else {
            let (rest, conclusion) = terminated(Self::parse_atom, Lexeme::Stop)(input)?;
            Ok((rest, Rule {
                premisses: vec![],
                conclusion
            }))
        }
    }

    fn parse_atoms(input: Input) -> nom::IResult<Input, Vec<Atom<String>>> {
        separated_list0(Lexeme::And, Self::parse_atom)(input)
    }
    fn parse_atom(input: Input) -> nom::IResult<Input, Atom<String>> {
        let (rest, (symbol, terms)) = pair(Self::parse_constant, delimited(Lexeme::OpeningParentheses, Self::parse_terms, Lexeme::ClosingParentheses))(input)?;
        Ok((rest, Atom {
            symbol,
            terms
        }))
    }

    fn parse_terms(input: Input) -> nom::IResult<Input, Vec<Term<String>>> {
        let res = separated_list0(Lexeme::Comma, Self::parse_term)(input);
        res
    }
    fn parse_term(input: Input) -> nom::IResult<Input, Term<String>> {
        alt((Self::parse_variable, Self::parse_application, Self::parse_integer))(input)
    }
    fn parse_application(input: Input) -> nom::IResult<Input, Term<String>> {
        let (rest, symbol) = Self::parse_constant(input)?;
        if let Ok((rest, terms)) = delimited(Lexeme::OpeningParentheses, Self::parse_terms, Lexeme::ClosingParentheses)(rest.clone()) {
            Ok((rest, Term::Application {
                symbol,
                terms,
            }))
        } else {
            Ok((rest, Term::Application {
                symbol,
                terms: vec![],
            }))
        }
    }

    logos_nom_bridge::data_variant_parser! {
        fn parse_variable(input) -> Result<Term<String>>;
        pattern = Lexeme::Variable(value) => Term::Variable { value };
    }
    logos_nom_bridge::data_variant_parser! {
        fn parse_integer(input) -> Result<Term<String>>;
        pattern = Lexeme::Integer(_i) => Term::Variable { value: String::from("todo") };
    }
    logos_nom_bridge::data_variant_parser! {
        fn parse_constant(input) -> Result<String>;
        pattern = Lexeme::Constant(symbol) => symbol;
    }
}