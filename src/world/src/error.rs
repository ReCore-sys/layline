use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorldError {
    #[error("Invalid game location: {0}")]
    InvalidGameLocation(PathBuf),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Lua error: {0}")]
    LuaError(#[from] mlua::Error),
}