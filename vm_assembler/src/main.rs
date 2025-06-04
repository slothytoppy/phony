use std::path::PathBuf;

use clap::Parser;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(clap::Parser, Debug)]
struct Args {
    input: PathBuf,
}

fn main() {
    let _ = tracing_subscriber::FmtSubscriber::builder()
        .with_ansi(true)
        .with_max_level(LevelFilter::INFO)
        .finish()
        .try_init();

    let args = Args::parse();

    tracing::info!(?args);

    let file = std::fs::read_to_string(args.input).expect("failed to read input file");

    let _ = vm_assembler::Parser::parse(&file).expect("failed to parse input file");
}
