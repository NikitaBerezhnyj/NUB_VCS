use crate::error::NubError;
use crate::objects::Blob;
use crate::repository::Repository;
use anyhow::Result;
use colored::Colorize;
use fs::DirEntry;
use serde_json::Value;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

fn collect_files(path: &Path, repo_root: &Path, collected: &mut Vec<PathBuf>) -> Result<()> {
    if path
        .file_name()
        .map(|n: &OsStr| n == ".nub-vcs")
        .unwrap_or(false)
    {
        return Ok(());
    }

    let abs_path: PathBuf = if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    };

    if abs_path.is_file() {
        let rel_path: PathBuf = abs_path
            .strip_prefix(repo_root)
            .unwrap_or(&abs_path)
            .to_path_buf();
        collected.push(rel_path);
    } else if abs_path.is_dir() {
        for entry in fs::read_dir(&abs_path)? {
            let entry: DirEntry = entry?;
            collect_files(&entry.path(), repo_root, collected)?;
        }
    }

    Ok(())
}

pub fn execute(files: Vec<String>) -> Result<()> {
    let repo: Repository = Repository::find()?;
    let index_path: PathBuf = repo.index_path();

    let head_path: PathBuf = repo.head_path();
    let head_ref: String = fs::read_to_string(&head_path)?;
    let head_ref_path: &str = head_ref.trim_start_matches("ref: ").trim();
    let branch_path: PathBuf = repo.root.join(".nub-vcs").join(head_ref_path);

    let committed_tree: HashMap<String, String> = if branch_path.exists() {
        let commit_hash: String = fs::read_to_string(&branch_path)?.trim().to_string();
        let commit_path: PathBuf = repo.commits_dir().join(commit_hash);
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
                        .filter_map(|e: &Value| {
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

    let index: Vec<Value> = if index_path.exists() {
        let data: String = fs::read_to_string(&index_path)?;
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    };

    let mut index_map: HashMap<String, String> = index
        .iter()
        .filter_map(|v: &Value| {
            Some((
                v.get("path")?.as_str()?.replace("\\", "/"),
                v.get("hash")?.as_str()?.to_string(),
            ))
        })
        .collect();

    let mut all_files: Vec<PathBuf> = Vec::new();
    for f in files {
        let path: PathBuf = PathBuf::from(&f);
        if !path.exists() {
            return Err(NubError::FileNotFound(f.clone()).into());
        }
        collect_files(&path, &repo.root, &mut all_files)?;
    }

    all_files.sort();
    all_files.dedup();

    for file_path in all_files {
        let full_path: PathBuf = repo.root.join(&file_path);
        let content: Vec<u8> = fs::read(&full_path).map_err(|e| NubError::IoError(e))?;

        let blob: Blob = Blob::new(content);
        let object_path: PathBuf = repo.objects_dir().join(&blob.hash);
        if !object_path.exists() {
            fs::write(&object_path, &blob.content)?;
        }

        let relative_path: String = file_path.to_string_lossy().replace("\\", "/");

        if let Some(commit_hash) = committed_tree.get(&relative_path) {
            if commit_hash == &blob.hash {
                continue;
            }
        }

        if let Some(existing_hash) = index_map.get(&relative_path) {
            if existing_hash == &blob.hash {
                continue;
            }
        }

        index_map.insert(relative_path.clone(), blob.hash.clone());
        println!(
            "{} added {} ({})",
            "✓".green().bold(),
            relative_path.cyan(),
            "blob".yellow()
        );
    }

    let new_index: Vec<Value> = index_map
        .into_iter()
        .map(|(path, hash)| serde_json::json!({"path": path, "hash": hash}))
        .collect();

    fs::write(index_path, serde_json::to_string_pretty(&new_index)?)?;

    Ok(())
}
