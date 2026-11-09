static INCLUDES_DIR: include_dir::Dir =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/includes");

pub fn load_layline_lua(lua: &mut mlua::Lua) -> Result<(), mlua::Error> {
    for lua_file in INCLUDES_DIR.find("**/*.lua").unwrap() {
        if let include_dir::DirEntry::File(file) = lua_file {
            let lua_code = file.contents_utf8().unwrap();
            lua.load(lua_code).exec()?;
        }
    }
    Ok(())
}
