use bytes::Bytes;
use layline_codec::{NetDecode, NetEncode, lua::lua_table::NetworkLuaTable};
use mlua::Lua;

fn spawn_region_table(lua: &Lua) -> NetworkLuaTable {
    let table = lua.create_table().unwrap();
    let points = lua.create_table().unwrap();
    let chef = lua.create_table().unwrap();
    let point = lua.create_table().unwrap();

    point.set("posX", 1.0).unwrap();
    chef.set(1, point).unwrap();
    points.set("chef", chef).unwrap();
    table.set("points", points).unwrap();

    NetworkLuaTable::from_lua_table(&table).unwrap()
}

fn expected_spawn_region_table() -> Bytes {
    Bytes::from_static(
        b"\x00\x00\x00\x01\
          \x00\x00\x06points\
          \x02\x00\x00\x00\x01\
          \x00\x00\x04chef\
          \x02\x00\x00\x00\x01\
          \x01\x3f\xf0\x00\x00\x00\x00\x00\x00\
          \x02\x00\x00\x00\x01\
          \x00\x00\x04posX\
          \x01\x3f\xf0\x00\x00\x00\x00\x00\x00",
    )
}

#[test]
fn encodes_nested_tables_using_the_spawn_region_format() {
    let lua = Lua::new();
    let table = spawn_region_table(&lua);

    assert_eq!(table.encode(), expected_spawn_region_table());
}

#[test]
fn encodes_string_values_with_the_kahlua_string_tag() {
    let lua = Lua::new();
    let table = lua.create_table().unwrap();
    table.set("name", "Muldraugh, KY").unwrap();
    let table = NetworkLuaTable::from_lua_table(&table).unwrap();

    assert_eq!(
        table.encode(),
        Bytes::from_static(
            b"\x00\x00\x00\x01\
              \x00\x00\x04name\
              \x00\x00\x0dMuldraugh, KY",
        ),
    );
}

#[test]
fn decodes_nested_tables_into_a_caller_supplied_lua_state() {
    let mut encoded = expected_spawn_region_table().to_vec();
    encoded.push(0xaa);
    let mut bytes = Bytes::from(encoded);
    let table = NetworkLuaTable::decode(&mut bytes).unwrap();
    assert_eq!(bytes, Bytes::from_static(&[0xaa]));

    let lua = Lua::new();
    let table = table.to_lua_table(&lua).unwrap();
    let points: mlua::Table = table.get("points").unwrap();
    let chef: mlua::Table = points.get("chef").unwrap();
    let point: mlua::Table = chef.get(1).unwrap();

    assert_eq!(point.get::<f64>("posX").unwrap(), 1.0);
}

#[test]
fn rejects_unknown_tags_without_consuming_input() {
    let input = Bytes::from_static(b"\x00\x00\x00\x01\x7f\x00\x00\x00\x00\x00");
    let mut bytes = input.clone();

    assert!(NetworkLuaTable::decode(&mut bytes).is_none());
    assert_eq!(bytes, input);
}
