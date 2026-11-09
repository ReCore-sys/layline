# SpawnRegion

**Protocol reference:** Project Zomboid 42.21.0 (revision `4a0e9546ec`).
Packet IDs and behavior may differ in other versions.

The server sends one `SpawnRegion` packet for each spawn region offered to a
joining client. The packet transfers the region's index and serialized Lua
table; it is not the `SpawnRegions()` Lua file itself. See
[Spawn points](../../world/spawn-points.md) for the region and point data
formats.

| Property | Value |
| --- | --- |
| Packet ID | `0x0073` (115) |
| Direction | Server to client |
| State | After login is accepted, during connection setup |
| Priority | `1` (`HIGH`) |
| Reliability | `2` (`RELIABLE`) |
| Ordering channel | `0` |

## Packet format

Fields appear in the order shown. Types and byte order are described in
[Network data types](../data-types.md). The table includes the application
header but excludes RakNet transport framing.

| Field | Type | Description |
| --- | --- | --- |
| Marker | Byte | `0x86` |
| Packet ID | Short | `0x0073` |
| Region index | Int | 1-based index used to identify this region |
| Region table | Kahlua table | Serialized region data |

The region table uses the game's Kahlua table serializer, not the custom
[`TableNetworkUtils`](../lua/table-serialization.md#tablenetworkutils) or
[`PZNetKahluaTableImpl`](../lua/table-serialization.md#pznetkahlua-table)
encodings. A sanitized 42.21.0 capture confirms the table format for the
SpawnRegion values; see
[Kahlua tables](../lua/table-serialization.md#kahlua-tables). The server writes
the table after the index; the client reads the index, loads a Kahlua table
from the remaining packet data using world version `249`, and stores the
result under that index.

## Region table fields

The serialized table is a region entry, not just its spawn-point coordinates.
A sanitized 42.21.0 capture showed these top-level keys:

| Key | Value | Purpose |
| --- | --- | --- |
| `name` | String | Region name shown when choosing a spawn region |
| `file` | String | Path to the region's spawn-points Lua file |
| `points` | Table | Profession-keyed spawn-point lists |

Each profession in `points` maps to a 1-based array of point tables. The
captured points used numeric `posX`, `posY`, and `posZ` coordinates; the source
format permits `posZ` to be omitted. The captured structure has this shape:

```text
{
    name: string,
    file: string,
    points: {
        [profession: string]: {
            [point_index: number]: {
                posX: number,
                posY: number,
                posZ?: number,
            },
        },
    },
}
```

The actual coordinates and available professions come from the region's
points file; see [Spawn points](../../world/spawn-points.md). These are
capture-observed table fields, not additional fixed packet-header fields.

## Sequence and ordering

After accepting a login, the server iterates region indexes from `1` through
the region count and sends one packet per index. The index lets the client
reconstruct the region list even if packets arrive out of order. These packets
are reliable but not reliable-ordered, so their send order is not an arrival
order guarantee. The server sends them before the connection-details
`RequestData` packet, but the two packet types likewise have no cross-packet
ordering guarantee.

The client uses the received table to populate the spawn-region choices. The
selected region name is later sent as part of new-player creation; it is not
selected by the `SpawnRegion` packet itself.

## Decompiled implementation

The packet ID and transport settings come from `PacketTypes.SpawnRegion`.
`GameServer` writes the index and calls `KahluaTable.save`; `GameClient`
receives the index and calls `KahluaTable.load(buffer, 249)`. The Kahlua
serializer implementation is external to the decompiled source in this
repository. The capture documents the string, double and nested-table values
used by SpawnRegion; other Kahlua value tags remain unspecified.
