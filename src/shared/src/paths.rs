use crate::error::SharedError;
use std::path::PathBuf;

/// Gets the Zomboid installation directory, either from the `ZOMBOID_SERVER_INSTALL_PATH` environment variable or from the current working directory.
/// It's assumed the layline executable is alongside the regular zomboid server files, so likely
/// `C:\Program Files (x86)\Steam\steamapps\common\Project Zomboid Dedicated Server` on Windows or `~/.steam/steam/steamapps/common/Project Zomboid Dedicated Server` on Linux.
pub fn get_zomboid_install_dir() -> Result<PathBuf, SharedError> {
    let base_path = if let Some(path) = std::env::var_os("ZOMBOID_SERVER_INSTALL_PATH") {
        path.into()
    } else if let Ok(path) = std::env::current_dir() {
        path
    } else {
        return Err(SharedError::InvalidGameLocation(
            std::env::current_dir().unwrap_or_default(),
        ));
    };
    if ![
        base_path.join("media"),
        base_path.join("java"),
        base_path.join("jre64"),
    ]
    .iter()
    .all(|p| p.exists())
    {
        return Err(SharedError::InvalidGameLocation(base_path));
    };
    if !base_path.join("media").join("maps").exists() {
        return Err(SharedError::InvalidGameLocation(base_path));
    }
    Ok(base_path)
}

/// Gets the Zomboid user directory, either from the `ZOMBOID_USER_DIR` environment variable or from the user's home directory.
/// Is usually `C:\Users\<username>\Zomboid` on Windows or `~/Zomboid` on Linux.
pub fn get_user_dir() -> Result<PathBuf, SharedError> {
    let user_dir = if let Some(path) = std::env::var_os("ZOMBOID_USER_DIR") {
        path.into()
    } else if let Some(path) = std::env::home_dir() {
        path.join("Zomboid")
    } else {
        return Err(SharedError::InvalidUserDir(
            std::env::current_dir().unwrap_or_default(),
        ));
    };
    if !user_dir.join("Lua").exists() {
        return Err(SharedError::InvalidUserDir(user_dir));
    }
    Ok(user_dir)
}
