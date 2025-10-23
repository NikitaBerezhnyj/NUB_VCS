use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TreeEntry {
    pub name: String,
    pub hash: String,
    pub entry_type: EntryType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    Blob,
    Tree,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tree {
    pub entries: Vec<TreeEntry>,
}

impl Tree {
    pub fn new() -> Self {
        Tree {
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, name: String, hash: String, entry_type: EntryType) {
        self.entries.push(TreeEntry {
            name,
            hash,
            entry_type,
        });
    }
}
