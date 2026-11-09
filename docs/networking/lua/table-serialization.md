# Lua table serialization

**Game version:** Project Zomboid 42.21.0 (commit `4a0e9546ec`).

The decompiled Java source shows three serializers for Lua tables. They use
different layouts and type tags, so choose the decoder from the packet's
reader or writer—not from the table's shape.

## Kahlua tables

The Java Kahlua runtime provides `KahluaTable.save` and `KahluaTable.load`.
`SpawnRegion` uses them to write and read a region table. Kahlua's serializer
implementation isn't included in the decompiled source, but a sanitized
42.21.0 connection capture shows this layout:

- A table starts with a four-byte big-endian entry count, followed by that
  many key/value pairs.
- Each key and value starts with a type tag. Tag `0` is a string encoded with
  the two-byte-length UTF-8 string format; tag `1` is an eight-byte
  big-endian `f64`; tag `2` is a nested table, serialized recursively after
  the tag.
- Captured keys were `String` or `f64`; values were `String`, `f64` or nested
  tables.

All four captured region tables match this layout. Other Kahlua tags weren't
present in the capture, so the Rust `NetworkLuaTable` decoder only accepts the
observed types. It stores tables in an owned representation, preserving
numeric keys without requiring a Lua runtime; `to_lua_table` can materialize
that representation in a caller-supplied runtime.

This isn't the same encoding as `TableNetworkUtils` or
`PZNetKahluaTableImpl`.

## `TableNetworkUtils`

In Java, `TableNetworkUtils.save` writes a four-byte entry count, followed by
that many key/value pairs. Each key and value starts with a one-byte type tag
and its payload. The count includes only entries supported by this codec.

| Tag | Supported as | Payload |
| --- | --- | --- |
| `0` | Key or value: `String` | Java `ByteBufferWriter.putUTF` encoding |
| `1` | Key or value: `f64` | 8-byte double |
| `2` | Value: nested Kahlua table | Another `TableNetworkUtils` table |
| `3` | Value: `bool` | One-byte boolean |
| `4` | Value: inventory item | `InventoryItem.saveWithSize` payload |
| `5` | Value: `IsoDirections` | Enum payload |
| `6` | Value: dead body | Object ID payload |

Keys can be `String` or `f64`. Values can also be `bool`, nested Kahlua tables,
inventory items, directions or dead bodies. The Java writer skips unsupported
entries, including integer keys. `saveSome` uses the same format but only
includes keys from the supplied set.

Tags mean different things depending on whether they're used for a key or a
value. For example, tag `2` is only used for a nested table value. Strings use
Java `ByteBufferWriter.putUTF` here; see
[Network data types](../data-types.md) for that encoding.

## PZNetKahlua table

The Java `PZNetKahluaTableImpl.save` method also writes a four-byte entry
count. It encodes each key and value with a type tag and a type-specific
payload. Key tags are `0` for `i32`, `1` for `String` and `2` for `f64`.
Unsupported entries are omitted.

The value tags are a separate mapping:

| Tag | Value type |
| --- | --- |
| `0` | `i32` |
| `1` | `String` |
| `2` | `f64` |
| `3` | Ordinary Kahlua table |
| `4` | Nested `PZNetKahluaTable` |
| `5` | `bool` |
| `6` | Inventory item |
| `7` | `IsoObject` reference |
| `8` | `IsoPlayer` reference |
| `9` | `ItemContainer` reference |
| `10` | `BodyPart` reference |
| `11` | `VehiclePart` reference |
| `12` | `BaseVehicle` reference |
| `13` | `IsoGridSquare` coordinates |
| `14` | Recipe reference |
| `15` | `BloodBodyPartType` |
| `16` | Dead-body object ID |
| `17` | `IsoAnimal` reference |
| `18` | Sprite object-info reference |
| `19` | Resource reference |
| `20` | Hutch nest-box index |
| `22` | Evolved-recipe reference |
| `23` | Fluid-container reference |
| `24` | Vehicle-window reference |
| `25` | Craft-recipe reference |
| `26` | `ArrayList` |
| `27` | Explicit null sentinel |
| `28` | `f32` |
| `29` | `i64` |
| `30` | `i16` |
| `31` | `i8` |
| `32` | `IsoDirections` |
| `33` | State reference |
| `34` | Climb-through-window positioning parameters |
| `35` | `IsoZombie` reference |
| `36` | Craft-bench component reference |
| `37` | State stage |
| `38` | `Variables` |

Tag `21` is unused in this version. Primitive values use their fixed-width
representations; strings use the Java `GameWindow.WriteString` helper. Other
values use game-specific serializers or identifiers. For example, a player,
vehicle or square reference is resolved from an ID or coordinates. Tag `3`
hands nested ordinary tables to Kahlua's serializer.

An `ArrayList` writes its length followed by tagged elements. The writer uses
the first element's type to encode every element, so lists should be
homogeneous and use a supported type. Tag `27` has no payload.
This is an explicit `PZNetKahluaNull` sentinel; ordinary Lua `nil` is not a
storable table value.

This format is for network data, not portable saves. The Java
`PZNetKahluaTableImpl.load(ByteBuffer, worldVersion)` method isn't implemented;
loading requires a live connection to resolve game-object references.

## Choosing the codec

Use the serializer called by the Java packet handler:

- `GameServer` sends `SpawnRegion` tables with Kahlua's `save`.
- The implemented `NetworkLuaTable` codec covers the `String`, `f64` and
  nested-table forms observed in the SpawnRegion capture.
- `TableNetworkUtils` handles ordinary mod data and other table payloads.
- Network action arguments use `PZNetKahluaTableImpl` when values may include
  game objects or other non-primitive types.

These details come from the decompiled Java files
`decomp/network/TableNetworkUtils.java`,
`decomp/network/PZNetKahluaTableImpl.java`,
`decomp/network/GameServer.java` and `decomp/network/GameClient.java`.
