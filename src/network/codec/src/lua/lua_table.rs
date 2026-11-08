use crate::{NetDecode, NetEncode};
use bytes::{Buf, BufMut, Bytes, BytesMut};

const STRING_TAG: u8 = 0;
const NUMBER_TAG: u8 = 1;
const TABLE_TAG: u8 = 2;
const MAX_TABLE_DEPTH: usize = 64;

/// A Lua table in Kahlua's native wire format, not a `TableNetworkUtils` or
/// `PZNetKahluaTable` value.
#[derive(Debug)]
pub struct NetworkLuaTable {
    entries: Vec<(NetworkLuaKey, NetworkLuaValue)>,
}

#[derive(Debug)]
enum NetworkLuaKey {
    String(String),
    Number(f64),
}

#[derive(Debug)]
enum NetworkLuaValue {
    String(String),
    Number(f64),
    Table(NetworkLuaTable),
}

impl NetworkLuaTable {
    /// Copies a Lua table into the network representation.
    pub fn from_lua_table(table: &mlua::Table) -> mlua::Result<Self> {
        Self::from_lua_table_at_depth(table, 0)
    }

    /// Creates a Lua table in the caller's runtime.
    pub fn to_lua_table(&self, lua: &mlua::Lua) -> mlua::Result<mlua::Table> {
        self.to_lua_table_at_depth(lua, 0)
    }

    fn from_lua_table_at_depth(table: &mlua::Table, depth: usize) -> mlua::Result<Self> {
        if depth > MAX_TABLE_DEPTH {
            return Err(mlua::Error::RuntimeError(
                "Lua table nesting is too deep".to_string(),
            ));
        }

        let mut entries = Vec::new();
        for pair in table.pairs::<mlua::Value, mlua::Value>() {
            let (key, value) = pair?;
            entries.push((
                NetworkLuaKey::from_lua_value(key)?,
                NetworkLuaValue::from_lua_value(value, depth)?,
            ));
        }
        Ok(Self { entries })
    }

    fn to_lua_table_at_depth(&self, lua: &mlua::Lua, depth: usize) -> mlua::Result<mlua::Table> {
        if depth > MAX_TABLE_DEPTH {
            return Err(mlua::Error::RuntimeError(
                "Lua table nesting is too deep".to_string(),
            ));
        }

        let table = lua.create_table()?;
        for (key, value) in &self.entries {
            table.raw_set(
                key.to_lua_value(lua)?,
                value.to_lua_value(lua, depth)?,
            )?;
        }
        Ok(table)
    }
}

impl NetworkLuaKey {
    fn from_lua_value(value: mlua::Value) -> mlua::Result<Self> {
        match value {
            mlua::Value::String(value) => Ok(Self::String(value.to_str()?.to_string())),
            mlua::Value::Integer(value) => Ok(Self::Number(exact_integer(value)?)),
            mlua::Value::Number(value) => Ok(Self::Number(value)),
            value => Err(unsupported_type("key", &value)),
        }
    }

    fn to_lua_value(&self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        match self {
            Self::String(value) => Ok(mlua::Value::String(lua.create_string(value)?)),
            Self::Number(value) => Ok(mlua::Value::Number(*value)),
        }
    }
}

impl NetworkLuaValue {
    fn from_lua_value(value: mlua::Value, depth: usize) -> mlua::Result<Self> {
        match value {
            mlua::Value::String(value) => Ok(Self::String(value.to_str()?.to_string())),
            mlua::Value::Integer(value) => Ok(Self::Number(exact_integer(value)?)),
            mlua::Value::Number(value) => Ok(Self::Number(value)),
            mlua::Value::Table(value) => Ok(Self::Table(
                NetworkLuaTable::from_lua_table_at_depth(&value, depth + 1)?,
            )),
            value => Err(unsupported_type("value", &value)),
        }
    }

    fn to_lua_value(&self, lua: &mlua::Lua, depth: usize) -> mlua::Result<mlua::Value> {
        match self {
            Self::String(value) => Ok(mlua::Value::String(lua.create_string(value)?)),
            Self::Number(value) => Ok(mlua::Value::Number(*value)),
            Self::Table(value) => Ok(mlua::Value::Table(
                value.to_lua_table_at_depth(lua, depth + 1)?,
            )),
        }
    }
}

impl TryFrom<mlua::Table> for NetworkLuaTable {
    type Error = mlua::Error;

    fn try_from(table: mlua::Table) -> Result<Self, Self::Error> {
        Self::from_lua_table(&table)
    }
}

impl NetEncode for NetworkLuaTable {
    fn encode(&self) -> Bytes {
        let mut bytes = BytesMut::new();
        encode_table(self, &mut bytes, 0);
        bytes.freeze()
    }
}

impl NetDecode for NetworkLuaTable {
    fn decode(bytes: &mut Bytes) -> Option<Self> {
        let mut remaining = bytes.clone();
        let table = decode_table(&mut remaining, 0)?;
        *bytes = remaining;
        Some(table)
    }
}

fn encode_table(table: &NetworkLuaTable, bytes: &mut BytesMut, depth: usize) {
    assert!(depth <= MAX_TABLE_DEPTH, "Lua table nesting is too deep");

    let count = u32::try_from(table.entries.len()).expect("Lua table has too many entries");
    bytes.put_u32(count);
    for (key, value) in &table.entries {
        match key {
            NetworkLuaKey::String(value) => encode_string(value, bytes),
            NetworkLuaKey::Number(value) => encode_number(*value, bytes),
        }
        encode_value(value, bytes, depth);
    }
}

fn encode_value(value: &NetworkLuaValue, bytes: &mut BytesMut, depth: usize) {
    match value {
        NetworkLuaValue::String(value) => encode_string(value, bytes),
        NetworkLuaValue::Number(value) => encode_number(*value, bytes),
        NetworkLuaValue::Table(value) => {
            bytes.put_u8(TABLE_TAG);
            encode_table(value, bytes, depth + 1);
        }
    }
}

fn encode_string(value: &str, bytes: &mut BytesMut) {
    bytes.put_u8(STRING_TAG);
    bytes.extend(value.encode());
}

fn encode_number(value: f64, bytes: &mut BytesMut) {
    bytes.put_u8(NUMBER_TAG);
    bytes.extend(value.encode());
}

fn exact_integer(value: i64) -> mlua::Result<f64> {
    let number = value as f64;
    if number as i128 != i128::from(value) {
        return Err(mlua::Error::RuntimeError(
            "Lua integer cannot be represented exactly as a Kahlua number".to_string(),
        ));
    }
    Ok(number)
}

fn unsupported_type(position: &str, value: &mlua::Value) -> mlua::Error {
    mlua::Error::RuntimeError(format!(
        "unsupported Kahlua table {position} type: {}",
        value.type_name()
    ))
}

fn decode_table(bytes: &mut Bytes, depth: usize) -> Option<NetworkLuaTable> {
    if depth > MAX_TABLE_DEPTH {
        return None;
    }

    let count = usize::try_from(u32::decode(bytes)?).ok()?;
    if count > bytes.remaining() / 6 {
        return None;
    }

    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        entries.push((decode_key(bytes)?, decode_value(bytes, depth)?));
    }
    Some(NetworkLuaTable { entries })
}

fn decode_key(bytes: &mut Bytes) -> Option<NetworkLuaKey> {
    match bytes.try_get_u8().ok()? {
        STRING_TAG => Some(NetworkLuaKey::String(String::decode(bytes)?)),
        NUMBER_TAG => Some(NetworkLuaKey::Number(f64::decode(bytes)?)),
        _ => None,
    }
}

fn decode_value(bytes: &mut Bytes, depth: usize) -> Option<NetworkLuaValue> {
    match bytes.try_get_u8().ok()? {
        STRING_TAG => Some(NetworkLuaValue::String(String::decode(bytes)?)),
        NUMBER_TAG => Some(NetworkLuaValue::Number(f64::decode(bytes)?)),
        TABLE_TAG => Some(NetworkLuaValue::Table(decode_table(bytes, depth + 1)?)),
        _ => None,
    }
}
