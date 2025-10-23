use crate::error::NubError;
use crate::objects::Blob;
use crate::repository::Repository;
use anyhow::Result;
use colored::Colorize;
use serde_json::Value;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{self, DirEntry};
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
    let repo = Repository::find()?;
    let index_path = repo.index_path();

    let index: Vec<Value> = if index_path.exists() {
        let data = fs::read_to_string(&index_path)?;
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    };

    let mut index_map: HashMap<String, String> = index
        .iter()
        .filter_map(|v| {
            Some((
                v.get("path")?.as_str()?.replace("\\", "/"),
                v.get("hash")?.as_str()?.to_string(),
            ))
        })
        .collect();

    let head_path = repo.head_path();
    let committed_tree: HashMap<String, String> = if head_path.exists() {
        let head_ref = fs::read_to_string(&head_path)?;
        let head_ref_path = head_ref.trim_start_matches("ref: ").trim();
        let branch_path = repo.nub_dir.join(head_ref_path);

        if branch_path.exists() {
            let commit_hash = fs::read_to_string(&branch_path)?.trim().to_string();
            let commit_path = repo.commits_dir().join(&commit_hash);

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
        }
    } else {
        HashMap::new()
    };

    let mut all_files: Vec<PathBuf> = Vec::new();
    for f in files {
        let path = PathBuf::from(&f);
        if !path.exists() {
            return Err(NubError::FileNotFound(f.clone()).into());
        }
        collect_files(&path, &repo.root, &mut all_files)?;
    }
    all_files.sort();
    all_files.dedup();

    let mut changed_count = 0;

    for file_path in all_files {
        let full_path = repo.root.join(&file_path);
        let content = fs::read(&full_path).map_err(NubError::IoError)?;
        let blob = Blob::new(content);
        let relative_path = file_path.to_string_lossy().replace("\\", "/");

        let is_changed = match index_map.get(&relative_path) {
            Some(existing_hash) => existing_hash != &blob.hash,
            None => match committed_tree.get(&relative_path) {
                Some(commit_hash) => commit_hash != &blob.hash,
                None => true,
            },
        };

        if !is_changed {
            continue;
        }

        let object_path = repo.objects_dir().join(&blob.hash);
        if !object_path.exists() {
            fs::write(&object_path, &blob.content)?;
        }

        index_map.insert(relative_path.clone(), blob.hash.clone());
        changed_count += 1;

        println!(
            "{} staged {} ({})",
            "✓".green().bold(),
            relative_path.cyan(),
            "blob".yellow()
        );
    }

    if changed_count == 0 {
        println!("{}", "No changes to stage".dimmed());
        return Ok(());
    }

    let new_index: Vec<Value> = index_map
        .into_iter()
        .map(|(path, hash)| {
            serde_json::json!({
                "path": path,
                "hash": hash
            })
        })
        .collect();

    fs::write(index_path, serde_json::to_string_pretty(&new_index)?)?;

    Ok(())
}
