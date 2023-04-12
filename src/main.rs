use std::env;
use logos_nom_bridge::Tokens;
use pif::{Parser, Sniffer};

fn main() {
    let args = env::args().collect::<Vec<_>>();

    // One argument (the file path) runs the REPL with basic axioms loaded in
    // Otherwise we just run the REPL with no axioms
    let mut sniffer = if let Some(file) = args.get(1) {
        Sniffer::new(file)
    } else {
        Sniffer::default()
    };

    let mut rl = rustyline::DefaultEditor::new().expect("failed to open repl");
    while let Ok(line) = rl.readline("sniffer >> ") {
        let query = Parser::parse_query(Tokens::new(&line));
        if let Ok(derivation_tree) = sniffer.find(&query) {
            ptree::print_tree(&derivation_tree).unwrap();
        } else {
            println!("no result found for query")
        }
    }
}