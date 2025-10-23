use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Commit {
    pub tree: String,
    pub parent: Option<String>,
    pub author: Author,
    pub timestamp: DateTime<Utc>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub email: String,
}

impl Commit {
    pub fn new(tree: String, parent: Option<String>, author: Author, message: String) -> Self {
        Commit {
            tree,
            parent,
            author,
            timestamp: Utc::now(),
            message,
        }
    }
}
