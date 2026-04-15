use front_end::parser::{CGDSLParser, Rule};
use pest_consume::*;
use std::env;
use std::fs;
use std::io::{self, Read};

fn main() {
    let args: Vec<_> = env::args().collect();

    let input: String = if args.len() > 1 {
        let input_file = &args[1];
        fs::read_to_string(input_file).expect("Failed to read file")
    } else {
        let mut stdin = String::new();
        io::stdin()
            .read_to_string(&mut stdin)
            .expect("Failed to read stdin");
        stdin
    };

    let game = CGDSLParser::parse(Rule::file, &input)
        .expect("Parse failed")
        .single()
        .expect("Expected single");
    let game = CGDSLParser::file(game).expect("Map failed");

    let graph = game.to_lowered_graph();
    let json = serde_json::to_string_pretty(&graph).expect("Serialization failed");

    if args.len() > 2 {
        let output_file = &args[2];
        fs::write(output_file, &json).expect("Failed to write");
    } else {
        println!("{}", json);
    }
}
