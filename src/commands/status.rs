use crate::repository::Repository;
use anyhow::Result;
use colored::Colorize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;

pub fn execute() -> Result<()> {
    let repo = Repository::find()?;
    let index_path = repo.index_path();
    let head_path = repo.head_path();

    let index_data = fs::read_to_string(&index_path).unwrap_or_else(|_| "[]".to_string());
    let index: Vec<Value> = serde_json::from_str(&index_data).unwrap_or_default();

    let index_map: HashMap<String, String> = index
        .iter()
        .filter_map(|v| {
            Some((
                v.get("path")?.as_str()?.replace("\\", "/"),
                v.get("hash")?.as_str()?.to_string(),
            ))
        })
        .collect();

    let head_ref = fs::read_to_string(&head_path)?;
    let head_ref_path = head_ref.trim_start_matches("ref: ").trim();

    let branch_path = repo.nub_dir.join(head_ref_path);

    let last_commit_hash = if branch_path.exists() {
        Some(fs::read_to_string(&branch_path)?.trim().to_string())
    } else {
        None
    };

    let committed_tree: HashMap<String, String> = if let Some(commit_hash) = &last_commit_hash {
        let commit_path = repo.commits_dir().join(commit_hash);
        if commit_path.exists() {
            let commit_data = fs::read_to_string(commit_path)?;
            let commit_json: Value = serde_json::from_str(&commit_data)?;

            if let Some(tree_hash) = commit_json.get("tree").and_then(|v| v.as_str()) {
                let tree_path = repo.objects_dir().join(tree_hash);
                if tree_path.exists() {
                    let tree_data = fs::read_to_string(tree_path)?;
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

    let mut all_paths: HashSet<String> = HashSet::new();
    all_paths.extend(index_map.keys().cloned());
    all_paths.extend(committed_tree.keys().cloned());

    let mut working_files: HashMap<String, String> = HashMap::new();

    for path_str in &all_paths {
        let full_path = repo.root.join(path_str);
        if full_path.exists() && full_path.is_file() {
            if let Ok(data) = fs::read(&full_path) {
                let mut hasher = Sha256::new();
                hasher.update(&data);
                let hash = format!("{:x}", hasher.finalize());
                working_files.insert(path_str.clone(), hash);
            }
        }
    }

    for entry in walkdir::WalkDir::new(&repo.root)
        .into_iter()
        .filter_entry(|e| {
            !e.path()
                .strip_prefix(&repo.root)
                .unwrap()
                .starts_with(".nub-vcs")
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let rel = path.strip_prefix(&repo.root).unwrap();
        let rel_str = rel.to_string_lossy().replace("\\", "/");

        if !all_paths.contains(&rel_str) {
            if let Ok(data) = fs::read(path) {
                let mut hasher = Sha256::new();
                hasher.update(&data);
                let hash = format!("{:x}", hasher.finalize());
                working_files.insert(rel_str, hash);
            }
        }
    }

    let mut staged: Vec<String> = vec![];
    let mut modified: Vec<String> = vec![];
    let mut untracked: Vec<String> = vec![];

    for (path, index_hash) in &index_map {
        match committed_tree.get(path) {
            Some(commit_hash) if commit_hash != index_hash => {
                staged.push(path.clone());
            }
            None => {
                staged.push(path.clone());
            }
            _ => {}
        }
    }

    for (path, work_hash) in &working_files {
        if let Some(index_hash) = index_map.get(path) {
            if index_hash != work_hash {
                modified.push(path.clone());
            }
        } else {
            match committed_tree.get(path) {
                Some(commit_hash) => {
                    if commit_hash != work_hash {
                        modified.push(path.clone());
                    }
                }
                None => {
                    untracked.push(path.clone());
                }
            }
        }
    }

    println!("{}", "On branch:".bold());
    println!(
        " {}",
        head_ref_path.split('/').last().unwrap_or("unknown").cyan()
    );
    println!();

    if !staged.is_empty() {
        println!("{}", "Staged for commit:".green().bold());
        for file in &staged {
            println!("  {}", file.green());
        }
        println!();
    }

    if !modified.is_empty() {
        println!("{}", "Modified:".yellow().bold());
        for file in &modified {
            println!("  {}", file.yellow());
        }
        println!();
    }

    if !untracked.is_empty() {
        println!("{}", "Untracked files:".red().bold());
        for file in &untracked {
            println!("  {}", file.red());
        }
        println!();
    }

    if staged.is_empty() && modified.is_empty() && untracked.is_empty() {
        println!("{}", "✓ Working directory clean".green().bold());
    }

    Ok(())
}
