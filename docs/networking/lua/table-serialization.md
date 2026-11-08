# Lua table serialization

**Protocol reference:** Project Zomboid 42.21.0 (revision `4a0e9546ec`).
Encodings and supported values may differ in other versions.

The decompiled code has three table serialization paths. Their layouts and
type tags differ, so a decoder must know which serializer the enclosing packet
or call site uses.

## Kahlua tables

The Kahlua runtime provides `KahluaTable.save` and `KahluaTable.load`. The
`SpawnRegion` packet writes a region table with `save` and reads it with
`load(buffer, 249)`. The Kahlua implementation is not included in the
decompiled source, but a sanitized Project Zomboid 42.21.0 connection capture
shows the following layout for SpawnRegion tables:

- A table starts with a four-byte big-endian entry count, followed by that
  many key/value pairs.
- Each key and value starts with a type tag. Tag `0` is a string encoded with
  the two-byte-length UTF-8 string format; tag `1` is an eight-byte
  big-endian double; tag `2` is a nested table, serialized recursively after
  the tag.
- Captured keys were strings or doubles; captured values were strings,
  doubles or nested tables.

The four captured region tables parsed completely with this layout. Other
Kahlua value tags are not established by that capture and are not supported by
the codec implementation. The decoder accepts only these observed types.
`NetworkLuaTable` uses an owned representation so decoding does not require a
Lua runtime; `to_lua_table` materializes it in a caller-supplied runtime. The
owned representation preserves numeric keys rather than coercing them into
string-keyed JSON objects.

Do not assume this encoding is the same as either `TableNetworkUtils` or
`PZNetKahluaTableImpl`. In particular, the SpawnRegion table is not serialized
with `TableNetworkUtils`.

## `TableNetworkUtils`

`TableNetworkUtils.save` writes a table as a four-byte entry count followed by
that many key/value pairs. Each key and value starts with a one-byte type tag,
then its payload. The count includes only entries whose key and value types
are supported by this codec.

| Tag | Supported as | Payload |
| --- | --- | --- |
| `0` | Key or value: string | `ByteBufferWriter.putUTF` string |
| `1` | Key or value: double | 8-byte double |
| `2` | Value: nested Kahlua table | Another `TableNetworkUtils` table |
| `3` | Value: boolean | Boolean byte |
| `4` | Value: inventory item | `InventoryItem.saveWithSize` payload |
| `5` | Value: `IsoDirections` | Enum payload |
| `6` | Value: dead body | Object ID payload |

Keys are restricted to strings and doubles. Values may be strings, doubles,
booleans, nested ordinary Kahlua tables, inventory items, directions or dead
bodies. Other entries are skipped; this includes integer keys and unsupported
values. `saveSome` applies the same encoding but also filters entries to a
provided key set.

The same tags identify keys and values, but they do not have the same valid
types in each position. For example, tag `2` is only used for a nested table
value, not a table key. Strings use `putUTF` here; see
[Network data types](../data-types.md) for that encoding.

## PZNetKahlua table

`PZNetKahluaTableImpl.save` also writes a four-byte entry count and then each
key and value as a type tag followed by a type-specific payload. Its key tags
are `0` for integer, `1` for string and `2` for double.
Entries with unsupported key or value types are omitted.

The value tags are a separate mapping:

| Tag | Value type |
| --- | --- |
| `0` | Integer |
| `1` | String |
| `2` | Double |
| `3` | Ordinary Kahlua table |
| `4` | Nested `PZNetKahluaTable` |
| `5` | Boolean |
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
| `28` | Float |
| `29` | Long |
| `30` | Short |
| `31` | Byte |
| `32` | `IsoDirections` |
| `33` | State reference |
| `34` | Climb-through-window positioning parameters |
| `35` | `IsoZombie` reference |
| `36` | Craft-bench component reference |
| `37` | State stage |
| `38` | `Variables` |

Tag `21` is not assigned in this version. The primitive values use their
corresponding fixed-width representations; strings use `GameWindow.WriteString`.
Other values use game-specific serializers or identifiers. The exact payload
for an object reference depends on its type—for example, a player, vehicle or
square is represented by an identifier or coordinates and resolved against
game state. Tag `3` delegates nested ordinary tables to Kahlua's native
serializer.

An `ArrayList` writes its length followed by tagged elements. The writer uses
the first element's type to encode every element, so lists should be
homogeneous and use a supported type. Tag `27` has no payload.
This is an explicit `PZNetKahluaNull` sentinel; ordinary Lua `nil` is not a
storable table value.

This codec is for network-aware data, not a portable save format.
`PZNetKahluaTableImpl.load(ByteBuffer, worldVersion)` is not implemented; its
network loader requires a connection so game-object references can be
resolved.

## Choosing the codec

Use the call site as the source of truth:

- `GameServer` sends `SpawnRegion` tables using Kahlua's native `save`.
- The implemented `NetworkLuaTable` codec covers the string, double and
  nested-table forms observed in the SpawnRegion capture.
- `TableNetworkUtils` is called directly for generic mod-data and other
  table payloads.
- Network action arguments use `PZNetKahluaTableImpl` when values may include
  game objects or other non-primitive types.

The sources examined for this reference are
`decomp/network/TableNetworkUtils.java`,
`decomp/network/PZNetKahluaTableImpl.java`,
`decomp/network/GameServer.java`, and `decomp/network/GameClient.java`.
