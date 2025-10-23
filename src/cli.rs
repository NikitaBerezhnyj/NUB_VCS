use crate::commands::{self};
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
    Init,
    Add {
        files: Vec<String>,
    },
    Commit {
        #[arg(short, long)]
        message: String,
    },
    Status,
    Reset,
    // Log,
    // Diff { #[arg(long)] staged: bool },
    // Checkout { target: String },
    // Branch { name: Option<String> },
    // Merge { branch: String },
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        match self.command {
            Commands::Init => commands::init::execute(),
            Commands::Add { files } => commands::add::execute(files),
            Commands::Commit { message } => commands::commit::execute(message),
            Commands::Status => commands::status::execute(),
            Commands::Reset => commands::reset::execute(),
        }
    }
}
