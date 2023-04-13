use itertools::Itertools;
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
            println!("failed to parse file");
            Sniffer::default()
        }
    } else {
        Sniffer::default()
    };

    let mut rl = rustyline::DefaultEditor::new().expect("failed to open repl");
    while let Ok(line) = rl.readline("sniffer >> ") {
        let _ = rl.add_history_entry(line.clone());
        let mut words = line.split_whitespace();
        let command = words.next().unwrap();
        let query = words.join(" ");

        match handle_command(command, &query, &mut sniffer) {
            CommandResult::ParsingError => eprintln!("parsing error"),
            CommandResult::UnknownCommand => eprintln!("unknown command"),
            CommandResult::FileError => eprintln!("failed to open/write to file"),
            CommandResult::NotFoundQuery => eprintln!("no result for query"),
            CommandResult::Quit => break,
            _ => (),
        }
    }
}

enum CommandResult {
    OkCommand,
    ParsingError,
    UnknownCommand,
    Quit,
    FileError,
    NotFoundQuery,
}
fn handle_command(command: &str, query: &str, sniffer: &mut Sniffer) -> CommandResult {
    match command {
        "load" => {
            if let Ok(from_file) = Sniffer::new(query) {
                *sniffer = from_file;
                CommandResult::OkCommand
            } else {
                CommandResult::FileError
            }
        }
        "dump" => CommandResult::OkCommand, //TODO

        "query" => {
            let query = if let Ok(query) = Parser::parse_query(Tokens::new(query)) {
                query
            } else {
                return CommandResult::NotFoundQuery;
            };

            if let Ok(derivation_tree) = sniffer.find(&query) {
                ptree::print_tree(&derivation_tree).unwrap();
                CommandResult::OkCommand
            } else {
                CommandResult::NotFoundQuery
            }
        }
        // TODO => debug print (for now the same as query)
        "query-debug" => {
            let query = if let Ok(query) = Parser::parse_query(Tokens::new(query)) {
                query
            } else {
                return CommandResult::ParsingError;
            };

            if let Ok(derivation_tree) = sniffer.find(&query) {
                ptree::print_tree(&derivation_tree).unwrap();
                CommandResult::OkCommand
            } else {
                CommandResult::NotFoundQuery
            }
        }
        "define" => CommandResult::OkCommand, //TODO
        "remove" => CommandResult::OkCommand, //TODO

        "rules" => {
            println!("{}", sniffer.rules_to_string());
            CommandResult::OkCommand
        }
        "derivation" => {
            let rules = if let Ok(rules) = Parser::parse_rules(Tokens::new(query)) {
                rules
            } else {
                return CommandResult::ParsingError;
            };

            for tree in rules
                .into_iter()
                .filter_map(|r| sniffer.derivation_tree(&r))
            {
                ptree::print_tree(&tree).unwrap()
            }
            CommandResult::OkCommand
        }

        "quit" => CommandResult::Quit,
        _ => CommandResult::UnknownCommand,
    }
}
