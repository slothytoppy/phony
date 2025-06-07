use std::path::PathBuf;

use clap::Parser;
use lexer::Lexer;
use tracing::info;

mod lexer;
mod parser;

#[derive(clap::Parser, Debug)]
struct Args {
    input: PathBuf,
}

fn main() {
    // let args = Args::parse();

    // let file = std::fs::read_to_string(args.input).expect("failed to read input file");
    let data = r#"= >= <= == > < 12345"#;

    let tokens = Lexer::new(data).lex().unwrap();

    let ast = parser::Parser::default()
        .parse(tokens)
        .expect("failed to parse input file");

    info!(?ast);
}
