//! Parser module
//! A parser takes a vector of lexemes as input, and creates an AST
//! by walking though said vector

use logos_nom_bridge::Tokens;
use logos::{Lexer, Logos};
use nom::branch::alt;
use nom::bytes::complete::is_a;
use nom::multi::{many0, separated_list0};
use nom::Parser as NomParser;
use nom::sequence::{delimited, pair};
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
    pub fn parse(source: &str) -> Vec<Rule<String>> {
        Parser::parse_rules(Tokens::new(source)).unwrap().1
    }

    fn parse_rules(input: Input) -> nom::IResult<Input, Vec<Rule<String>>> {
        many0(Self::parse_rule)(input)
    }
    fn parse_rule(input: Input) -> nom::IResult<Input, Rule<String>> {
        let (rest, premisses) = Self::parse_atoms(input)?;

        let continuation = Lexeme::Implies.parse(rest.clone());
        let (rest, rule) = if let Ok((rest, _)) = continuation {
            let (rest, conclusion) = Self::parse_atom(rest)?;
            (rest, Rule {
                premisses,
                conclusion
            })
        } else {
            (rest, Rule {
                premisses: vec![],
                conclusion: premisses[0].clone()
            })
        };

        println!("rule: {rule:?}");

        let result = Lexeme::Comma.parse(rest);
        if let Ok((rest, _)) = result {
            Ok((rest, rule))
        } else {
            panic!("aled")
        }
    }

    fn parse_atoms(input: Input) -> nom::IResult<Input, Vec<Atom<String>>> {
        separated_list0(Lexeme::And, Self::parse_atom)(input)
    }
    fn parse_atom(input: Input) -> nom::IResult<Input, Atom<String>> {
        println!("before atom: {input:?}");
        let (rest, (symbol, terms)) = pair(Self::parse_constant, delimited(Lexeme::OpeningParentheses, Self::parse_terms, Lexeme::ClosingParentheses))(input)?;
        println!("after atom: {rest:?}");
        println!("read: {symbol}, {terms:?}");
        Ok((rest, Atom {
            symbol,
            terms
        }))
    }

    fn parse_terms(input: Input) -> nom::IResult<Input, Vec<Term<String>>> {
        println!("before terms: {input:?}");
        let res = separated_list0(Lexeme::Comma, Self::parse_term)(input);
        println!("after terms: {:?}", res.as_ref().unwrap().0);
        res
    }
    fn parse_term(input: Input) -> nom::IResult<Input, Term<String>> {
        alt((Self::parse_variable, Self::parse_application, Self::parse_integer))(input)
    }

    logos_nom_bridge::data_variant_parser! {
        fn parse_variable(input) -> Result<Term<String>>;
        pattern = Lexeme::Variable(value) => Term::Variable { value };
    }
    logos_nom_bridge::data_variant_parser! {
        fn parse_application(input) -> Result<Term<String>>;
        pattern = Lexeme::Constant(symbol) => Term::Application {
            symbol,
            terms: vec![]
        };
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