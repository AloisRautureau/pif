use logos_nom_bridge::Tokens;
use pif::{Parser, Sniffer};
use std::env;

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
        let mut words = line.split_whitespace();
        match words.next() {
            Some("open") => {
                if let Some(file) = words.next() {
                    sniffer = Sniffer::new(file)
                }
            },
            Some("dump") => todo!(),

            Some("query") => {
                let query = if let Some(query) = words.next() {
                    query
                } else {
                    println!("missing argument: query <atom>");
                    continue
                };

                let query = if let Ok(query) = Parser::parse_query(Tokens::new(query)) {
                    query
                } else {
                    println!("failed to parse query");
                    continue;
                };

                if let Ok(derivation_tree) = sniffer.find(&query) {
                    ptree::print_tree(&derivation_tree).unwrap();
                } else {
                    println!("no result found for query")
                }
            },
            Some("define") => todo!(),
            Some("remove") => todo!(),
            Some("rules") => todo!(),

            Some("quit") => break,
            _ => println!("unrecognized command")
        }
    }
}
