use crate::error::NubError;
use crate::objects::commit::{Author, Commit};
use crate::objects::tree::{EntryType, Tree};
use crate::repository::Repository;
use anyhow::Result;
use colored::Colorize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

pub fn execute(message: String) -> Result<()> {
    let repo: Repository = Repository::find()?;

    let index_path: PathBuf = repo.index_path();
    if !index_path.exists() {
        return Err(NubError::InvalidRepository.into());
    }

    let index_data: String = fs::read_to_string(&index_path)?;
    let index: Vec<Value> = serde_json::from_str(&index_data).unwrap_or_default();

    if index.is_empty() {
        eprintln!("{} Nothing to commit", "✗".red().bold());
        return Ok(());
    }

    let mut tree: Tree = Tree::new();
    for entry in &index {
        if let (Some(path), Some(hash)) = (entry.get("path"), entry.get("hash")) {
            tree.add_entry(
                path.as_str().unwrap().to_string(),
                hash.as_str().unwrap().to_string(),
                EntryType::Blob,
            );
        }
    }

    let tree_json: String = serde_json::to_string(&tree)?;
    let mut tree_hasher = Sha256::new();
    tree_hasher.update(tree_json.as_bytes());
    let tree_hash: String = format!("{:x}", tree_hasher.finalize());

    let tree_path: PathBuf = repo.objects_dir().join(&tree_hash);
    fs::write(tree_path, &tree_json)?;

    let head_ref: String = fs::read_to_string(repo.head_path())?;
    let head_ref_path: &str = head_ref.trim_start_matches("ref: ").trim();
    let current_branch: PathBuf = repo.root.join(".nub-vcs").join(head_ref_path);

    let parent_hash: Option<String> = if current_branch.exists() {
        Some(fs::read_to_string(&current_branch)?.trim().to_string())
    } else {
        None
    };

    let config_data: String = fs::read_to_string(repo.config_path())?;
    let config_json: Value = serde_json::from_str(&config_data)?;
    let user: &Value = config_json.get("user").unwrap();
    let author_name: String = user.get("name").unwrap().as_str().unwrap().to_string();
    let author_email: String = user.get("email").unwrap().as_str().unwrap().to_string();

    let author = Author {
        name: author_name,
        email: author_email,
    };

    let commit: Commit = Commit::new(tree_hash.clone(), parent_hash, author, message.clone());
    let commit_json: String = serde_json::to_string_pretty(&commit)?;

    let mut hasher = Sha256::new();
    hasher.update(commit_json.as_bytes());
    let commit_hash: String = format!("{:x}", hasher.finalize());

    let commit_path = repo.commits_dir().join(&commit_hash);
    fs::write(&commit_path, &commit_json)?;

    fs::write(&current_branch, &commit_hash)?;

    fs::write(index_path, "[]")?;

    println!(
        "{} Created commit {}",
        "✓".green().bold(),
        commit_hash[..8].cyan()
    );
    println!("    {}", message);

    Ok(())
}
