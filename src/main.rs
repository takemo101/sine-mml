use clap::Parser;
use sine_mml::cli::args::{Cli, Command};
use sine_mml::cli::handlers::{export_handler, history_handler, play_handler};
use sine_mml::cli::output;

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Play(args) => play_handler(args),
        Command::History => history_handler(),
        Command::Export(args) => export_handler(args),
    };

    if let Err(e) = result {
        output::error(&format!("{e:#}"));
        std::process::exit(1);
    }
}
