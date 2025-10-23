mod cli;
mod commands;
mod error;
mod objects;
mod repository;

use anyhow::Result;
use clap::Parser;
use cli::Cli;

fn main() -> Result<()> {
    let cli: Cli = Cli::parse();
    cli.execute()
}
