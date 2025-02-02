use std::process::exit;

use crossterm::terminal;

use clap::Parser;

use tetris_tui::{
    config::Config, Args, Result, CELL_WIDTH, CONFIG, DISTANCE, HELP_MESSAGE, MAX_LEVEL,
    PLAY_HEIGHT, PLAY_WIDTH, STATS_WIDTH,
};

fn main() -> Result<()> {
    let args = Args::parse();
    if args.number_of_lines_already_filled > 10 {
        eprintln!("The number of lines already filled must be less than or equal 10.");
        exit(1);
    }

    if args.level > MAX_LEVEL {
        eprintln!("Level must be between 0 and {}.", MAX_LEVEL);
        exit(1);
    }

    let (term_width, term_height) = terminal::size()?;
    let play_width = PLAY_WIDTH * CELL_WIDTH + 2;
    let required_width = (STATS_WIDTH + 2 + DISTANCE) * 2 + play_width;
    let required_height = PLAY_HEIGHT + 2;
    if term_width < required_width as u16 || term_height < required_height as u16 {
        eprintln!(
            "The terminal is too small: {}x{}.\nRequired dimensions are  : {}x{}.",
            term_width, term_height, required_width, required_height
        );
        exit(1);
    }

    CONFIG
        .set(Config::get())
        .expect("Failed to set global Config static");

    // create the help message and leak it
    // by converting every String into a Box<'static str> and leaking it
    let help_message: Vec<&'static str> = CONFIG
        .get()
        .unwrap()
        .create_help_message()
        .iter()
        .map(|s| Box::leak(s.clone().into_boxed_str()) as &str)
        .collect();

    HELP_MESSAGE
        .set(help_message)
        .expect("Could not set Help Message");

    tetris_tui::start(&args, term_width, term_height)?;

    Ok(())
}
