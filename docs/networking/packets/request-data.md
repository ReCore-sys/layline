# RequestData

**Game version:** Project Zomboid 42.21.0 (commit `4a0e9546ec`).

`RequestData` has two jobs: it carries connection details after login, then
transfers the datasets the client needs while loading.

Method names below refer to the game's decompiled Java source, not Rust APIs.

| Property         | Value                                 |
|------------------|---------------------------------------|
| Packet ID        | `0x0029` (41)                         |
| Direction        | Client to server and server to client |
| State            | Connection setup and world loading    |
| RakNet priority  | High (1)                              |
| Reliability      | `Reliable`                            |
| Ordering channel | `4`                                   |

## Packet fields

Each message starts with the application header, then a request type and data
ID. The payload depends on the request type. See
[Network data types](../data-types.md) for primitive encodings and strings.

| Order | Type     | Field             | Details                     |
|------:|----------|-------------------|-----------------------------|
|     1 | `u8`     | Marker            | `0x86`                      |
|     2 | `i16`    | Packet ID         | `0x0029`                    |
|     3 | `u8`     | Request type      | Selects the operation below |
|     4 | `u8`     | Data ID           | Selects the dataset         |
|     5 | Variable | Operation payload | Depends on the request type |

## Request types

| Value | Name          | Direction        | Payload                                     |
|------:|---------------|------------------|---------------------------------------------|
|   `1` | `Request`     | Client to server | Requests a dataset                          |
|   `2` | `FullData`    | Server to client | Complete dataset follows the control fields |
|   `3` | `PartData`    | Server to client | Three big-endian `i32` values, then a chunk |
|   `4` | `PartDataACK` | Client to server | Acknowledges a window; no additional fields |

For data under 1,024 bytes, the server sends `FullData`. Larger datasets use
`PartData` chunks of up to 1,024 bytes. The client acknowledges a window after
receiving 204,800 bytes. This is application-level chunking, separate from
RakNet fragmentation and acknowledgements.

## Data IDs

| ID  | Name                      | Use                                                                             |
|-----|---------------------------|---------------------------------------------------------------------------------|
| `0` | `ConnectionDetails`       | Server options, map, mods, spawn location and other connection/world setup data |
| `1` | `ZombieOutfitDescriptors` | World-loading dataset                                                           |
| `2` | `PlayerZombieDescriptors` | World-loading dataset                                                           |
| `3` | `RadioData`               | World-loading dataset                                                           |
| `4` | `WorldMap`                | World-loading dataset                                                           |
| `5` | `PlayerVisited`           | Player-visited data                                                             |

The request type and data ID are `u8` enum values. The server handles
`Request` and `PartDataACK`; the client handles `FullData` and `PartData`.

## Examples

Examples show the control fields; unknown payload bytes are omitted.

### Request a dataset

To request `ConnectionDetails` (data ID `0`), the client sends:

```text
86 00 29 01 00
│  │     │  └─ data ID: ConnectionDetails
│  │     └──── request type: Request
│  └────────── packet ID: RequestData
└───────────── application marker
```

Other dataset requests use the same packet and replace the final byte with the
requested data ID. For example, `86 00 29 01 03` requests `RadioData`.

### Send a complete dataset

For `ConnectionDetails`, a `FullData` response has this shape:

```text
86 00 29 02 00 <connection-details bytes…>
         │  │  └─ opaque dataset body
         │  └──── data ID: ConnectionDetails
         └─────── request type: FullData
```

The connection-details body is a binary record containing world setup such as
the map, mods, server and sandbox options, spawn location, and game time. Its
fields are listed below.

### Split a dataset into parts

After the control fields, `PartData` has three big-endian `i32` values: total
size, offset, and chunk size. One captured join split a 133,448-byte
`ConnectionDetails` body into 131 parts. Its first part was:

```text
86 00 29 03 00 00 02 09 48 00 00 00 00 00 00 04 00 <1,024 data bytes>
         │  │  └───────────┬───────────┬───────────┘
         │  │       total: 133448    offset: 0     size: 1024
         │  └─ data ID: ConnectionDetails
         └──── request type: PartData
```

Each subsequent part increments the offset by the preceding part size. The
last captured part had offset `133120` and size `328`:

```text
86 00 29 03 00 00 02 09 48 00 02 08 00 00 00 01 48 <328 data bytes>
```

The client ACKs `ConnectionDetails` with:

```text
86 00 29 04 00
```

The ACK identifies the dataset but carries no offset or byte count. The sender
tracks progress per connection and sends the next window after receiving it.

## Connection-details body

`FullData` carries the whole connection-details record; `PartData` carries a
chunk of it. Once reassembled, the record has the fields below, with no nested
packet header or padding. See [Network data types](../data-types.md) for the
primitive encodings.

| Order | Type                           | Field                            | Details                                                              |
|------:|--------------------------------|----------------------------------|----------------------------------------------------------------------|
|     1 | `bool`                         | Co-op host                       | `isCoopHost`                                                         |
|     2 | `i32`                          | Maximum players                  | Server max-player setting                                            |
|     3 | `bool`                         | Co-op Steam host details present | If true, fields 3a and 3b follow                                     |
|    3a | `i64`                          | Host Steam ID                    | Present only when field 3 is true                                    |
|    3b | `String`                       | Server name                      | Present only when field 3 is true                                    |
|     4 | `u8`                           | Player index                     | `connection.playerIds[0] / 4`                                        |
|     5 | Role record                    | Role                             | See the role fields below                                            |
|     6 | `String`                       | Game map                         | Map string sent to the client                                        |
|     7 | Conditional list               | Workshop items                   | Present only in Steam mode; starts with an `i16` count               |
|     8 | `i32` count and entries        | Mods                             | Count followed by that many mod records                              |
|     9 | `i32` × 3                      | Start location                   | X, Y, Z; vanilla writes `10745`, `9412`, `0`                         |
|    10 | `i32` count and entries        | Public server options            | Count followed by `String` key/value pairs                           |
|    11 | Sandbox-options serialization  | Sandbox options                  | Written by Java `SandboxOptions.save`                                |
|    12 | Game-time serialization        | Game time                        | Written by Java `GameTime.saveToPacket`                              |
|    13 | Erosion serialization          | Erosion configuration            | Written by Java `ErosionConfig.save`                                 |
|    14 | Global-object serialization    | Global objects                   | Written by Java `SGlobalObjects.saveInitialStateForClient`           |
|    15 | `i32`                          | Reset ID                         | Server world reset ID                                                |
|    16 | `String`                       | Poisonous berry                  | Current configured berry                                             |
|    17 | `String`                       | Poisonous mushroom               | Current configured mushroom                                          |
|    18 | World-dictionary serialization | World dictionary                 | Written by Java `WorldDictionary.saveDataForClient`; variable length |

The role record (field 5) is:

| Order | Type                    | Field            | Details                 |
|------:|-------------------------|------------------|-------------------------|
|     1 | `String`                | Name             |                         |
|     2 | `String`                | Description      |                         |
|     3 | `f32`                   | Color red        | `color.r`               |
|     4 | `f32`                   | Color green      | `color.g`               |
|     5 | `f32`                   | Color blue       | `color.b`               |
|     6 | `f32`                   | Color alpha      | `color.a`               |
|     7 | `i32`                   | Position         |                         |
|     8 | `u8`                    | Capability count |                         |
|     9 | `u8` × capability count | Capabilities     | One byte per capability |
|    10 | `bool`                  | Read-only        | `is_read_only`          |

In Steam mode, field 7 starts with an `i16` count and is followed by that many
records, each with an `i64` Workshop item ID and timestamp. Each mod in field 8
has three strings: mod ID, Workshop ID and display name. Missing mods still
appear with an empty Workshop ID and the mod ID as their display name.

### 11. Sandbox options

The Java `SandboxOptions.save` method writes this versioned block:

| Order | Type              | Field                   | Details                                                                       |
|------:|-------------------|-------------------------|-------------------------------------------------------------------------------|
|     1 | `u8` × 4          | Magic                   | ASCII `SAND` (`53 41 4e 44`)                                                  |
|     2 | `i32`             | World version           | `249`                                                                         |
|     3 | `i32`             | Format version          | `6`                                                                           |
|     4 | `i32`             | Option count            | Number of option name/value pairs                                             |
|     5 | Repeated `String` | Option name, then value | Written with Java `GameWindow.WriteString`, in `SandboxOptions.options` order |
|     6 | `String`          | Preset                  | Written with Java `GameWindow.WriteString` after the option pairs             |

The Java `GameWindow.WriteString` helper writes a signed 16-bit byte length
and UTF-8 bytes. Unlike Java `ByteBufferWriter.putUTF`, it handles
non-positive lengths differently. The option count and order come from the
server's `SandboxOptions.options` list; each value is written as a string and
parsed according to its option type.

### 12. Game time

The Java `GameTime.saveToPacket` method writes these fields in order:

| Order | Type                              | Field            | Details                                          |
|------:|-----------------------------------|------------------|--------------------------------------------------|
|     1 | `f32`                             | Multiplier       | Game-time multiplier                             |
|     2 | `i32`                             | Nights survived  |                                                  |
|     3 | `i32`                             | Target zombies   |                                                  |
|     4 | `f32`                             | Last time of day |                                                  |
|     5 | `f32`                             | Time of day      |                                                  |
|     6 | `i32`                             | Day              |                                                  |
|     7 | `i32`                             | Month            |                                                  |
|     8 | `i32`                             | Year             |                                                  |
|     9 | `f32`                             | Reserved         | Always `0.0`                                     |
|    10 | `f32`                             | Reserved         | Always `0.0`                                     |
|    11 | `i32`                             | Reserved         | Always `0`                                       |
|    12 | `bool`                            | Has mod data     | The current server path writes `true`            |
|    13 | Kahlua table, if field 12 is true | Mod data         | Serialized with Kahlua's native table serializer |

Those reserved zeros are still sent. Before serializing mod data, the Java
server removes `camping`, `farming` and `trapping` from the table (see
[Lua table serialization](../lua/table-serialization.md#kahlua-tables)).
`getModData()` returns a table even when it has no entries, so the server sets
the presence flag to `true`. The client loads that table with world version
`249`.

### 13. Erosion configuration

The Java `ErosionConfig.save` method writes 24 big-endian `i32` values and 13
big-endian `f32` values (148 bytes total):

| Order | Type       | Field                                                                                                                                                                  | Details |
|------:|------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------|---------|
|   1–3 | `i32` × 3  | `seeds.seedMain0`, `seeds.seedMain1`, `seeds.seedMain2`                                                                                                                |         |
|   4–6 | `i32` × 3  | `seeds.seedMoisture0`, `seeds.seedMoisture1`, `seeds.seedMoisture2`                                                                                                    |         |
|   7–9 | `i32` × 3  | `seeds.seedMinerals0`, `seeds.seedMinerals1`, `seeds.seedMinerals2`                                                                                                    |         |
| 10–12 | `i32` × 3  | `seeds.seedKudzu0`, `seeds.seedKudzu1`, `seeds.seedKudzu2`                                                                                                             |         |
| 13–16 | `i32` × 4  | `time.tickunit`, `time.ticks`, `time.eticks`, `time.epoch`                                                                                                             |         |
| 17–21 | `i32` × 5  | `season.lat`, `season.tempMax`, `season.tempMin`, `season.tempDiff`, `seasonLag`                                                                                       |         |
|    22 | `f32`      | `season.noon`                                                                                                                                                          |         |
| 23–25 | `i32` × 3  | `season.seedA`, `season.seedB`, `season.seedC`                                                                                                                         |         |
| 26–37 | `f32` × 12 | `season.jan`, `season.feb`, `season.mar`, `season.apr`, `season.may`, `season.jun`, `season.jul`, `season.aug`, `season.sep`, `season.oct`, `season.nov`, `season.dec` |         |

The debug settings from `ErosionConfig.Debug` are not written by this
serializer.

### 14. Global-object initial state

The Java `SGlobalObjects.saveInitialStateForClient` method writes:

| Order | Type                                          | Field             | Details                                          |
|------:|-----------------------------------------------|-------------------|--------------------------------------------------|
|     1 | `u8`                                          | System count      | Number of global-object systems                  |
|     2 | `String` per system                           | System name       |                                                  |
|     3 | `bool` per system                             | Has initial state | Indicates whether the following table is present |
|     4 | `TableNetworkUtils` table, if field 3 is true | Initial state     | Serialized with `TableNetworkUtils`              |

Before serializing, the server adds an `_objects` table to each system's
initial-state table. `_objects` is a 1-based array; each object entry contains
numeric `x`, `y`, and `z` values plus the keys configured for synchronization
by that global-object system. The optional table uses
[`TableNetworkUtils`](../lua/table-serialization.md#tablenetworkutils), not
Kahlua's native serializer.

### 18. World dictionary

The world-dictionary field starts with an `i32` byte length. It covers the
dictionary data and a final length-prefixed `String` sentinel,
`DICTIONARY_PACKET_END`. The dictionary layout is:

| Order | Type                        | Field                  | Details                                                                                                       |
|------:|-----------------------------|------------------------|---------------------------------------------------------------------------------------------------------------|
|     1 | `i32`                       | Dictionary version     | Currently `1`                                                                                                 |
|     2 | `i16`                       | Next info ID           |                                                                                                               |
|     3 | `u8`                        | Next object-name ID    |                                                                                                               |
|     4 | `i32`                       | Next sprite-name ID    |                                                                                                               |
|     5 | `i32`                       | Mod ID count           |                                                                                                               |
|     6 | Repeated `String`           | Mod ID                 | Written with Java `GameWindow.WriteString`                                                                    |
|     7 | `i32`                       | Module count           |                                                                                                               |
|     8 | Repeated `String`           | Module name            | Written with Java `GameWindow.WriteString`                                                                    |
|     9 | `i32`                       | Item count             |                                                                                                               |
|    10 | Repeated `ItemInfo`         | Item info              |                                                                                                               |
|    11 | `i32`                       | Entity count           |                                                                                                               |
|    12 | Repeated `EntityInfo`       | Entity info            |                                                                                                               |
|    13 | `i32`                       | Object-name count      |                                                                                                               |
|    14 | `u8` + `String` (repeated)  | Object ID, object name | Name written with Java `GameWindow.WriteString`                                                               |
|    15 | `i32`                       | Sprite-name count      |                                                                                                               |
|    16 | `i32` + `String` (repeated) | Sprite ID, sprite name | Name written with Java `GameWindow.WriteString`                                                               |
|    17 | `StringDictionary` data     | String dictionary      | Nested encoding                                                                                               |
|    18 | `ScriptsDictionary` data    | Scripts dictionary     | Nested encoding                                                                                               |
|    19 | `String`                    | End sentinel           | Written with Java `ByteBufferWriter.putUTF` as `DICTIONARY_PACKET_END`; included in the enclosing byte length |

`ItemInfo`, `EntityInfo`, `StringDictionary` and `ScriptsDictionary` each have
their own nested encoding. The client reads the length-prefixed block, loads
those sections, then checks the sentinel. Its size varies and it can make up a
large part of `ConnectionDetails`.

The blocks are concatenated; only the lengths shown above are present. Their
sizes depend on server settings and world state, so parse each section in order.

## Current Layline MVP

After sending the configured spawn regions, Layline sends a `FullData`
`ConnectionDetails` response with minimal, non-Steam defaults: no Steam host
details, workshop items, mods, public options, or global-object systems; the
GameTime mod-data flag is false. Steam-mode sessions must enable the conditional
workshop section, even when its item count is zero. The map and maximum-player
count are taken from the server configuration. The default world-dictionary
block is empty, which is enough for the client to finish reading connection
details but is not a valid dictionary for later world initialization.

The packet type can encode requests, full data, parts and part acknowledgments.
The login path currently sends only `FullData` and declines to send a body of
1,024 bytes or more; application-level part transfer and ACK tracking are not
implemented yet.

## Other dataset bodies

The Java server uses separate serializers for the other data IDs: zombie
outfit descriptors go through `PersistentOutfits.save`, player-zombie
descriptors start with an `i16` count, radio data goes through `RecordedMedia`,
and world-map data through `WorldMapServer`.

### Loading example

After connection setup, a client can request the loading datasets one at a
time. For example, a request for `ZombieOutfitDescriptors` followed by a full
response has this shape:

```text
Client → server: 86 00 29 01 01
Server → client: 86 00 29 02 01 <descriptor data…>
```

The client uses data IDs `2`, `3` and `4` for the next three datasets. Large
responses use `PartData` and the same ACK flow.

The server sends `ConnectionDetails` after accepting login and sending the
configured `SpawnRegion` packets. The client requests the other datasets later;
see the [login and world loading sequence](../login-sequence.md).

`RequestData` is reliable but not ordered. The send order therefore doesn't
guarantee arrival order relative to other packets, including `SpawnRegion`.
