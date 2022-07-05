use std::{path::PathBuf, fs::File};
use clap::Parser;
use game::GameSpecification;
use anyhow::Result;

mod game;

#[derive(Parser)]
struct Opts {
    filename: PathBuf,
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    let game_spec: GameSpecification = serde_json::from_reader(File::open(opts.filename)?)?;

    game_spec.solve()?;

    Ok(())
}
