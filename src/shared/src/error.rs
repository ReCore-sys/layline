use thiserror::Error;

#[derive(Error, Debug)]
pub enum SharedError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid game location: {0}")]
    InvalidGameLocation(std::path::PathBuf),
    #[error("Invalid user directory: {0}")]
    InvalidUserDir(std::path::PathBuf),
}