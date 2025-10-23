use crate::repository::Repository;
use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

pub fn execute() -> Result<()> {
    match Repository::find() {
        Ok(repo) => {
            let index_path: PathBuf = repo.index_path();
            fs::write(&index_path, "[]")?;

            println!("{} Reset successful.", "✓".green().bold());

            Ok(())
        }
        Err(e) => {
            eprintln!("{} {}", "✗".red().bold(), e);
            Err(e)
        }
    }
}
