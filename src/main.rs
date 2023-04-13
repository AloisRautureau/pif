use logos_nom_bridge::Tokens;
use sniffer::{Parser, Sniffer};
use std::env;

fn main() {
    let args = env::args().collect::<Vec<_>>();

    // One argument (the file path) runs the REPL with basic axioms loaded in
    // Otherwise we just run the REPL with no axioms
    let mut sniffer = if let Some(file) = args.get(1) {
        if let Ok(from_file) = Sniffer::new(file) {
            from_file
        } else {
            println!("failed to load file");
            return;
        }
    } else {
        Sniffer::default()
    };

    let mut rl = rustyline::DefaultEditor::new().expect("failed to open repl");
    while let Ok(line) = rl.readline("sniffer >> ") {
        let mut words = line.split_whitespace();
        match words.next() {
            Some("load") => {
                if let Some(file) = words.next() {
                    if let Ok(from_file) = Sniffer::new(file) {
                        sniffer = from_file;
                    } else {
                        println!("failed to load file");
                    }
                }
            }
            Some("dump") => todo!(),

            Some("query") => {
                let query = if let Some(query) = words.next() {
                    query
                } else {
                    println!("missing argument: query <atom>");
                    continue;
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
            }

            // TODO => debug print (for now the same as query)
            Some("query-debug") => {
                let query = if let Some(query) = words.next() {
                    query
                } else {
                    println!("missing argument: query <atom>");
                    continue;
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
            }
            Some("define") => todo!(),
            Some("remove") => todo!(),
            Some("print-sniffer") => {
                println!("Rules:");
                print!("{}", sniffer.rules_to_string());
                println!("Derived from:");
                sniffer.print_derived_from();
            },
            Some("rules") => {
                println!("Rules:");
                print!("{}", sniffer.rules_to_string());
            }
            Some("axioms") => {
                println!("Axioms:");
                print!("{}", sniffer.axioms_to_string());
            }
            Some("derived-from") => {
                println!("Derived from:");
                sniffer.print_derived_from();
            }
            Some("quit") => break,
            _ => println!("unrecognized command"),
        }
    }
}
