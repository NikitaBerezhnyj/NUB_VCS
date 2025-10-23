use crate::error::NubError;
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

const NUB_DIR: &str = ".nub-vcs";
const OBJECTS_DIR: &str = "objects";
const COMMITS_DIR: &str = "commits";
const REFS_DIR: &str = "refs";
const HEADS_DIR: &str = "heads";
const HEAD_FILE: &str = "HEAD";
const INDEX_FILE: &str = "index";
const CONFIG_FILE: &str = "config";

pub struct Repository {
    pub root: PathBuf,
    pub nub_dir: PathBuf,
}

impl Repository {
    /// Create a new repository structure
    pub fn init(path: &Path) -> Result<Self> {
        let nub_dir = path.join(NUB_DIR);

        // Check if repository already exists
        if nub_dir.exists() {
            return Err(NubError::RepositoryAlreadyExists.into());
        }

        // Create directory structure
        fs::create_dir(&nub_dir)?;
        fs::create_dir(nub_dir.join(OBJECTS_DIR))?;
        fs::create_dir(nub_dir.join(COMMITS_DIR))?;
        fs::create_dir(nub_dir.join(REFS_DIR))?;
        fs::create_dir(nub_dir.join(REFS_DIR).join(HEADS_DIR))?;

        let repo = Repository {
            root: path.to_path_buf(),
            nub_dir,
        };

        // Initialize HEAD file
        repo.init_head()?;

        // Initialize empty index
        repo.init_index()?;

        // Initialize config
        repo.init_config()?;

        Ok(repo)
    }

    /// Find repository in current or parent directories
    pub fn find() -> Result<Self> {
        let mut current = std::env::current_dir()?;

        loop {
            let nub_dir = current.join(NUB_DIR);
            if nub_dir.exists() && nub_dir.is_dir() {
                return Ok(Repository {
                    root: current,
                    nub_dir,
                });
            }

            if !current.pop() {
                return Err(NubError::RepositoryNotFound.into());
            }
        }
    }

    fn init_head(&self) -> Result<()> {
        let head_path = self.nub_dir.join(HEAD_FILE);
        fs::write(head_path, "ref: refs/heads/main")?;
        Ok(())
    }

    fn init_index(&self) -> Result<()> {
        let index_path = self.nub_dir.join(INDEX_FILE);
        // Create empty JSON array for index
        fs::write(index_path, "[]")?;
        Ok(())
    }

    fn init_config(&self) -> Result<()> {
        let config_path = self.nub_dir.join(CONFIG_FILE);
        let default_config = serde_json::json!({
            "user": {
                "name": "NUB User",
                "email": "user@nub.local"
            }
        });
        fs::write(config_path, serde_json::to_string_pretty(&default_config)?)?;
        Ok(())
    }

    pub fn objects_dir(&self) -> PathBuf {
        self.nub_dir.join(OBJECTS_DIR)
    }

    pub fn commits_dir(&self) -> PathBuf {
        self.nub_dir.join(COMMITS_DIR)
    }

    pub fn refs_dir(&self) -> PathBuf {
        self.nub_dir.join(REFS_DIR)
    }

    pub fn heads_dir(&self) -> PathBuf {
        self.refs_dir().join(HEADS_DIR)
    }

    pub fn head_path(&self) -> PathBuf {
        self.nub_dir.join(HEAD_FILE)
    }

    pub fn index_path(&self) -> PathBuf {
        self.nub_dir.join(INDEX_FILE)
    }

    pub fn config_path(&self) -> PathBuf {
        self.nub_dir.join(CONFIG_FILE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_creates_structure() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path()).unwrap();

        assert!(repo.nub_dir.exists());
        assert!(repo.objects_dir().exists());
        assert!(repo.commits_dir().exists());
        assert!(repo.heads_dir().exists());
        assert!(repo.head_path().exists());
        assert!(repo.index_path().exists());
        assert!(repo.config_path().exists());
    }

    #[test]
    fn test_init_twice_fails() {
        let temp = TempDir::new().unwrap();
        Repository::init(temp.path()).unwrap();
        let result = Repository::init(temp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_head_content() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path()).unwrap();
        let head_content = fs::read_to_string(repo.head_path()).unwrap();
        assert_eq!(head_content, "ref: refs/heads/main");
    }
}
