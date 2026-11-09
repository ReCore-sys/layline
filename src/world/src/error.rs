use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorldError {
    #[error("Shared error: {0}")]
    SharedError(#[from] layline_shared::error::SharedError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Lua error: {0}")]
    LuaError(#[from] mlua::Error),
}