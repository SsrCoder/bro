use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(version, about, long_about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    OCR(OCR),
}

#[derive(Parser, Debug)]
struct OCR {
    #[arg(short, long)]
    url: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    println!("cli: {cli:?}");
}
