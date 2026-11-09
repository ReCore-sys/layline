# SpawnRegion

**Game version:** Project Zomboid 42.21.0 (commit `4a0e9546ec`).

For each region offered during login, the server sends a `SpawnRegion` packet
with its index and serialized table. It doesn't send the
`SpawnRegions()` Lua file itself; see [Spawn points](../../world/spawn-points.md)
for that data.

| Property | Value |
| --- | --- |
| Packet ID | `0x0073` (115) |
| Direction | Server to client |
| State | After login is accepted, during connection setup |
| RakNet priority | High (1) |
| Reliability | `Reliable` |
| Ordering channel | `0` |

## Packet fields

| Field | Type | Description |
| --- | --- | --- |
| Marker | `u8` | `0x86` |
| Packet ID | `i16` | `0x0073` |
| Region index | `i32` | 1-based index used to identify the region |
| Region table | Kahlua table | Serialized region data |

The Java server uses Kahlua's native table serializer here. The Java client
reads the index, then loads the rest of the packet with world version `249`.
This is not [`TableNetworkUtils`](../lua/table-serialization.md#tablenetworkutils)
or [`PZNetKahluaTableImpl`](../lua/table-serialization.md#pznetkahlua-table);
see [Kahlua tables](../lua/table-serialization.md#kahlua-tables).

## Region table

A sanitized 42.21.0 capture showed these top-level keys:

| Key | Type | Purpose |
| --- | --- | --- |
| `name` | `String` | Region name shown in the spawn-region menu |
| `file` | `String` | Path to the region's spawn-points Lua file |
| `points` | Table | Profession-keyed spawn-point lists |

Each profession maps to a 1-based array of point tables. Captured points used
numeric `posX`, `posY` and `posZ` coordinates; the source format allows `posZ`
to be omitted. In Rust-like notation, the captured structure is:

```text
{
    name: String,
    file: String,
    points: {
        [profession: String]: {
            [point_index: f64]: {
                posX: f64,
                posY: f64,
                posZ?: f64,
            },
        },
    },
}
```

These are observed table values, not additional fixed packet fields. Available
professions and coordinates come from the region's
[points file](../../world/spawn-points.md).

## Ordering

The server sends regions with indexes from `1` through the region count.
Reliable delivery does not preserve their order, and neither does it guarantee
arrival before the `RequestData` connection-details packet. The client uses
the indexes to rebuild the list. The chosen region is sent later during new
player creation.

In the decompiled Java source, `GameServer` writes the index and calls the
Kahlua runtime's `KahluaTable.save`; `GameClient` reads it and calls
`KahluaTable.load(buffer, 249)`. The serializer implementation isn't included
here. The capture confirms `String`, `f64` and nested-table values; other
Kahlua tags remain unknown.
