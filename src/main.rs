use anyhow::Result;
use clap::Parser;
use rhyme_checker::{run, Cli};

fn main() -> Result<()> {
    let cli = Cli::parse();
    run(&cli)
}
