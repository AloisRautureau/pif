use std::fs::File;
use std::io::Read;
use crate::parser::Parser;

mod ast;
mod lexer;
mod parser;

pub fn run() {
    let mut content = String::new();
    File::open("examples/example.pif").unwrap().read_to_string(&mut content).unwrap();

    let rules = Parser::parse(&content);
    println!("{rules:?}")
}