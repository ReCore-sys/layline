use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorldError {
    #[error("Invalid game location: {0}")]
    InvalidGameLocation(PathBuf)
}