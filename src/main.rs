use std::env;
use pif::Sniffer;

fn main() {
    let args = env::args().collect::<Vec<_>>();

    // One argument (the file path) runs the REPL with basic axioms loaded in
    // Otherwise we just run the REPL with no axioms
    if let Some(file) = args.get(1) {
        Sniffer::new(file);
        todo!()
    } else {
        todo!()
    }
}