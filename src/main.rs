use std::{path::PathBuf, fs::File};
use clap::Parser;
use game::{GameSpecification, GraphGenerator};
use anyhow::Result;

mod bidirectional_list;
mod game;

#[derive(Parser)]
struct Opts {
    filename: PathBuf,
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    let game_spec: GameSpecification = serde_json::from_reader(File::open(opts.filename)?)?;
    let mut graph_generator = GraphGenerator::new(game_spec);

    let (_, edges) = graph_generator.generate()?;

    for edge in edges {
        println!("{},{}", edge.0, edge.1);
    }

    Ok(())
}
