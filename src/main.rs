use clap::Parser;
use sine_mml::cli::args::{Cli, Command};
use sine_mml::cli::handlers::{
    clear_history_handler, export_handler, history_handler, play_handler,
};
use sine_mml::cli::output;

#[cfg(feature = "midi-output")]
use sine_mml::cli::handlers::midi_handler;

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Play(args) => play_handler(args),
        Command::History => history_handler(),
        Command::Export(args) => export_handler(args),
        Command::ClearHistory => clear_history_handler(),
        #[cfg(feature = "midi-output")]
        Command::Midi(args) => midi_handler(args),
    };

    if let Err(e) = result {
        output::error(&format!("{e:#}"));
        std::process::exit(1);
    }
}
