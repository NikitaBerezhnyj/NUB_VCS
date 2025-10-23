use crate::repository::Repository;
use anyhow::Result;
use colored::Colorize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn execute() -> Result<()> {
    let repo: Repository = Repository::find()?;
    let index_path: PathBuf = repo.index_path();
    let head_path: PathBuf = repo.head_path();

    let index_data: String = fs::read_to_string(&index_path).unwrap_or_else(|_| "[]".to_string());
    let index: Vec<Value> = serde_json::from_str(&index_data).unwrap_or_default();

    let head_ref: String = fs::read_to_string(&head_path)?;
    let head_ref_path: &str = head_ref.trim_start_matches("ref: ").trim();
    let branch_path: PathBuf = repo.root.join(".nub-vcs").join(head_ref_path);

    let last_commit_hash: Option<String> = if branch_path.exists() {
        Some(fs::read_to_string(&branch_path)?.trim().to_string())
    } else {
        None
    };

    let committed_tree: HashMap<String, String> = if let Some(commit_hash) = last_commit_hash {
        let commit_path: PathBuf = repo.commits_dir().join(&commit_hash);
        if commit_path.exists() {
            let commit_data: String = fs::read_to_string(commit_path)?;
            let commit_json: Value = serde_json::from_str(&commit_data)?;

            if let Some(tree_hash) = commit_json.get("tree").and_then(|v| v.as_str()) {
                let tree_path: PathBuf = repo.objects_dir().join(tree_hash);
                if tree_path.exists() {
                    let tree_data: String = fs::read_to_string(tree_path)?;
                    let tree_json: Value = serde_json::from_str(&tree_data)?;

                    tree_json["entries"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|e| {
                            Some((
                                e.get("name")?.as_str()?.replace("\\", "/"),
                                e.get("hash")?.as_str()?.to_string(),
                            ))
                        })
                        .collect()
                } else {
                    HashMap::new()
                }
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        }
    } else {
        HashMap::new()
    };

    let mut working_files: HashMap<String, String> = HashMap::new();
    for entry in walkdir::WalkDir::new(&repo.root)
        .into_iter()
        .filter_map(|e: std::result::Result<walkdir::DirEntry, walkdir::Error>| e.ok())
        .filter(|e: &walkdir::DirEntry| e.file_type().is_file())
    {
        let path: &Path = entry.path();
        let rel: &Path = path.strip_prefix(&repo.root).unwrap();
        if rel.starts_with(".nub-vcs") {
            continue;
        }

        let data: Vec<u8> = fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let hash: String = format!("{:x}", hasher.finalize());
        working_files.insert(rel.to_string_lossy().replace("\\", "/"), hash);
    }

    let mut index_map: HashMap<String, String> = HashMap::new();
    for entry in &index {
        if let (Some(path), Some(hash)) = (entry.get("path"), entry.get("hash")) {
            let path_str: String = path.as_str().unwrap().replace("\\", "/");
            index_map.insert(path_str, hash.as_str().unwrap().to_string());
        }
    }

    let mut added: Vec<String> = vec![];
    let mut modified: Vec<String> = vec![];
    let mut untracked: Vec<String> = vec![];

    for (path, hash) in &working_files {
        match committed_tree.get(path) {
            Some(commit_hash) => {
                if commit_hash != hash {
                    modified.push(path.clone());
                }
            }
            None => untracked.push(path.clone()),
        }
    }

    for (path, _) in &index_map {
        if !working_files.contains_key(path) {
            added.push(path.clone());
        }
    }

    println!("{}", "On branch:".bold());
    println!(
        "  {}",
        head_ref_path.split('/').last().unwrap_or("unknown").cyan()
    );
    println!();

    if !added.is_empty() {
        println!("{}", "Staged for commit:".green().bold());
        for file in &added {
            println!("  {}", file.green());
        }
    }

    if !modified.is_empty() {
        println!("{}", "Modified:".yellow().bold());
        for file in &modified {
            println!("  {}", file.yellow());
        }
    }

    if !untracked.is_empty() {
        println!("{}", "Untracked files:".red().bold());
        for file in &untracked {
            println!("  {}", file.red());
        }
    }

    if added.is_empty() && modified.is_empty() && untracked.is_empty() {
        println!("{}", "✓ Working directory clean".green().bold());
    }

    Ok(())
}
