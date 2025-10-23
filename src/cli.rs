use crate::commands;
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "nub")]
#[command(about = "A simple version control system for learning purposes", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new repository
    Init,
    // Заготовки для майбутніх команд
    // Add { files: Vec<String> },
    // Commit { #[arg(short, long)] message: String },
    // Log,
    // Status,
    // Diff { #[arg(long)] staged: bool },
    // Checkout { target: String },
    // Branch { name: Option<String> },
    // Merge { branch: String },
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        match self.command {
            Commands::Init => commands::init::execute(),
        }
    }
}
