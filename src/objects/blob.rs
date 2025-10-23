use sha2::{Digest, Sha256};

pub struct Blob {
    pub content: Vec<u8>,
    pub hash: String,
}

impl Blob {
    pub fn new(content: Vec<u8>) -> Self {
        let hash: String = Self::calculate_hash(&content);
        Blob { content, hash }
    }

    fn calculate_hash(content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }
}
