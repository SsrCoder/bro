pub mod subcommands;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(version, about, long_about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Ocr(subcommands::Ocr),
}

impl Command {
    fn run(&self) {
        match self {
            Self::Ocr(ocr) => ocr.run(),
        }
    }
}

fn main() {
    let cli = Cli::parse();
    cli.command.run();
    // println!("cli: {cli:?}");
}
