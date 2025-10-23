use crate::repository::Repository;
use anyhow::Result;
use colored::Colorize;
use std::env;

pub fn execute() -> Result<()> {
    let current_dir: std::path::PathBuf = env::current_dir()?;

    match Repository::init(&current_dir) {
        Ok(repo) => {
            println!(
                "{} Initialized empty nub repository in {}",
                "✓".green().bold(),
                repo.nub_dir.display().to_string().cyan()
            );
            Ok(())
        }
        Err(e) => {
            eprintln!("{} {}", "✗".red().bold(), e);
            Err(e)
        }
    }
}
