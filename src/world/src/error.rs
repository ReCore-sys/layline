use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorldError {
    #[error("Shared error: {0}")]
    SharedError(#[from] layline_shared::error::SharedError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Lua error: {0}")]
    LuaError(#[from] mlua::Error),
    #[error("Missing map directory in save: {0}")]
    MissingMapDirectory(String),
    #[error("Mismatched checksum for chunk at ({0}, {1}): expected {2:#X}, got {3:#X}")]
    MismatchedChecksum(i32, i32, i64, i64),
}
