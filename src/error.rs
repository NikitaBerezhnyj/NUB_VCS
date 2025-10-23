use std::fmt;

#[derive(Debug)]
pub enum NubError {
    RepositoryAlreadyExists,
    RepositoryNotFound,
    InvalidRepository,
    FileNotFound(String),
    IoError(std::io::Error),
    SerializationError(String),
}

impl fmt::Display for NubError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NubError::RepositoryAlreadyExists => {
                write!(f, "Repository already exists in this directory")
            }
            NubError::RepositoryNotFound => {
                write!(f, "Not a nub repository (or any of the parent directories)")
            }
            NubError::InvalidRepository => {
                write!(f, "Invalid repository structure")
            }
            NubError::FileNotFound(path) => {
                write!(f, "File not found: {}", path)
            }
            NubError::IoError(err) => {
                write!(f, "IO error: {}", err)
            }
            NubError::SerializationError(msg) => {
                write!(f, "Serialization error: {}", msg)
            }
        }
    }
}

impl std::error::Error for NubError {}

impl From<std::io::Error> for NubError {
    fn from(err: std::io::Error) -> Self {
        NubError::IoError(err)
    }
}

impl From<serde_json::Error> for NubError {
    fn from(err: serde_json::Error) -> Self {
        NubError::SerializationError(err.to_string())
    }
}
