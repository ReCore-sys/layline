# RequestData

**Protocol reference:** Project Zomboid 42.21.0 (revision `4a0e9546ec`).
Packet IDs and behavior may differ in other versions.

`RequestData` is a shared bidirectional packet for requesting and transferring several
connection and world datasets. It carries connection details after login, and
later carries datasets requested during world loading.

| Property         | Value                              |
|------------------|------------------------------------|
| Packet ID        | `0x0029` (41)                      |
| Direction        | Both                               |
| State            | Connection setup and world loading |
| Priority         | `1` (`HIGH`)                       |
| Reliability      | `2` (`RELIABLE`)                   |
| Ordering channel | `4`                                |

## Common packet format

Every message starts with the normal application header, followed by a request
type and data ID. Types and byte order are described in
[Network data types](../data-types.md).

| Order | Wire type | Field             | Details                     |
|------:|-----------|-------------------|-----------------------------|
|     1 | `byte`    | Marker            | `0x86`                      |
|     2 | `int16`   | Packet ID         | `0x0029`                    |
|     3 | `byte`    | Request type      | Selects the operation below |
|     4 | `byte`    | Data ID           | Selects the dataset         |
|     5 | Variable  | Operation payload | Depends on the request type |

## Request types

| Type | Name          | Direction        | Payload                                                                                       |
|------|---------------|------------------|-----------------------------------------------------------------------------------------------|
| `1`  | `Request`     | Client to server | Requests the selected dataset                                                                 |
| `2`  | `FullData`    | Server to client | Complete dataset bytes follow the two control bytes                                           |
| `3`  | `PartData`    | Server to client | Total size, offset and part size as 32-bit big-endian integers, followed by that part's bytes |
| `4`  | `PartDataACK` | Client to server | Acknowledges a window; there are no fields after the two control bytes                        |

For a dataset smaller than 1,024 bytes, the server uses `FullData`. Larger
datasets are sent in `PartData` chunks of at most 1,024 bytes. The client
acknowledges a window after receiving 204,800 bytes of parts. This chunking is
an application-level transfer and is separate from RakNet's transport
fragmentation and acknowledgements.

## Data IDs

| ID  | Name                      | Use                                                                             |
|-----|---------------------------|---------------------------------------------------------------------------------|
| `0` | `ConnectionDetails`       | Server options, map, mods, spawn location and other connection/world setup data |
| `1` | `ZombieOutfitDescriptors` | World-loading dataset                                                           |
| `2` | `PlayerZombieDescriptors` | World-loading dataset                                                           |
| `3` | `RadioData`               | World-loading dataset                                                           |
| `4` | `WorldMap`                | World-loading dataset                                                           |
| `5` | `PlayerVisited`           | Player-visited data                                                             |

These IDs are one-byte enum ordinals in the order shown. Request types are
likewise one-byte enum ordinals. The server handles `Request` and
`PartDataACK`; the client handles `FullData` and `PartData`.

## Examples

The examples show the complete control fields and, for dataset bodies, an
explicit placeholder rather than invented bytes.

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

The connection-details body carries world setup such as the map, mods,
server/sandbox options, spawn location and game time. It is structured binary
data in the field order documented below, not JSON or additional
`RequestData` fields.

### Split a dataset into parts

After the two control bytes, `PartData` has three 32-bit big-endian integers:
the total dataset size, the offset of this part, and the part size. The
captured join sent `ConnectionDetails` as 131 parts for a 133,448-byte dataset.
The first part's envelope was:

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

The ACK for the `ConnectionDetails` transfer is exactly:

```text
86 00 29 04 00
```

It identifies the dataset but carries no offset or byte count. The sender
tracks the transfer per connection and sends the next window (up to 204,800
data bytes) after receiving the ACK.

## ConnectionDetails body

The `FullData`/`PartData` envelope is followed by the `ConnectionDetails`
dataset itself. Its fields are written consecutively in this order; there is
no nested packet header or padding inside the dataset. Primitive encodings are
described in [Network data types](../data-types.md).

| Order | Wire type                      | Field                            | Details                                                               |
|------:|--------------------------------|----------------------------------|-----------------------------------------------------------------------|
|     1 | `boolean`                      | Co-op host                       | `isCoopHost`                                                          |
|     2 | `int32`                        | Maximum players                  | Server max-player setting                                             |
|     3 | `boolean`                      | Co-op Steam host details present | If true, fields 3a and 3b follow                                      |
|    3a | `int64`                        | Host Steam ID                    | Present only when field 3 is true                                     |
|    3b | `UTF`                          | Server name                      | Present only when field 3 is true                                     |
|     4 | `byte`                         | Player index                     | `connection.playerIds[0] / 4`                                         |
|     5 | Role record                    | Role                             | See the role fields below                                             |
|     6 | `UTF`                          | Game map                         | Map string sent to the client                                         |
|     7 | Conditional list               | Workshop items                   | Present only when Steam mode is enabled; starts with an `int16` count |
|     8 | `int32` count and entries      | Mods                             | Count followed by that many mod records                               |
|     9 | `int32` × 3                    | Start location                   | X, Y, Z; vanilla writes `10745`, `9412`, `0`                          |
|    10 | `int32` count and entries      | Public server options            | Count followed by `UTF` key/value pairs                               |
|    11 | Sandbox-options serialization  | Sandbox options                  | Written by `SandboxOptions.save`                                      |
|    12 | Game-time serialization        | Game time                        | Written by `GameTime.saveToPacket`                                    |
|    13 | Erosion serialization          | Erosion configuration            | Written by `ErosionConfig.save`                                       |
|    14 | Global-object serialization    | Global objects                   | Written by `SGlobalObjects.saveInitialStateForClient`                 |
|    15 | `int32`                        | Reset ID                         | Server world reset ID                                                 |
|    16 | `UTF`                          | Poisonous berry                  | Current configured berry                                              |
|    17 | `UTF`                          | Poisonous mushroom               | Current configured mushroom                                           |
|    18 | World-dictionary serialization | World dictionary                 | Written by `WorldDictionary.saveDataForClient`; variable length       |

The role record (field 5) is:

| Order | Wire type                 | Field            | Details                 |
|------:|---------------------------|------------------|-------------------------|
|     1 | `UTF`                     | Name             |                         |
|     2 | `UTF`                     | Description      |                         |
|     3 | `float32`                 | Color red        | `color.r`               |
|     4 | `float32`                 | Color green      | `color.g`               |
|     5 | `float32`                 | Color blue       | `color.b`               |
|     6 | `float32`                 | Color alpha      | `color.a`               |
|     7 | `int32`                   | Position         |                         |
|     8 | `byte`                    | Capability count |                         |
|     9 | `byte` × capability count | Capabilities     | One byte per capability |
|    10 | `boolean`                 | Read-only        | `is_read_only`          |

When Steam mode is enabled, field 7 is a 16-bit count followed by that many
records, each containing a 64-bit Workshop item ID and a 64-bit timestamp.
Field 8's mod records each contain three `UTF` strings: mod ID, Workshop ID,
and display name. Missing mods are still represented; their Workshop ID is
empty and their display name is the mod ID.

### 11. Sandbox options

`SandboxOptions.save` writes a versioned block:

| Order | Wire type                         | Field                   | Details                                                                                                                             |
|------:|-----------------------------------|-------------------------|-------------------------------------------------------------------------------------------------------------------------------------|
|     1 | `byte` × 4                        | Magic                   | ASCII `SAND` (`53 41 4e 44`)                                                                                                        |
|     2 | `int32`                           | World version           | `249`                                                                                                                               |
|     3 | `int32`                           | Format version          | `6`                                                                                                                                 |
|     4 | `int32`                           | Option count            | Number of option name/value pairs                                                                                                   |
|     5 | Repeated `GameWindow.WriteString` | Option name, then value | Repeated once per option, in the server's `SandboxOptions.options` order; values are strings parsed according to each option's type |
|     6 | `GameWindow.WriteString`          | Preset                  | Selected world preset; follows all option pairs                                                                                     |

`GameWindow.WriteString` is a signed 16-bit byte length followed by UTF-8
bytes; it is not the same helper as `ByteBufferWriter.putUTF`. The option
count and order come from the server's `SandboxOptions.options` list. Values
are serialized as strings and parsed according to each option's type. The
final string is the selected world preset.

### 12. Game time

`GameTime.saveToPacket` writes these fields in order:

| Order | Wire type                         | Field            | Details                                          |
|------:|-----------------------------------|------------------|--------------------------------------------------|
|     1 | `float32`                         | Multiplier       | Game-time multiplier                             |
|     2 | `int32`                           | Nights survived  |                                                  |
|     3 | `int32`                           | Target zombies   |                                                  |
|     4 | `float32`                         | Last time of day |                                                  |
|     5 | `float32`                         | Time of day      |                                                  |
|     6 | `int32`                           | Day              |                                                  |
|     7 | `int32`                           | Month            |                                                  |
|     8 | `int32`                           | Year             |                                                  |
|     9 | `float32`                         | Reserved         | Always `0.0`                                     |
|    10 | `float32`                         | Reserved         | Always `0.0`                                     |
|    11 | `int32`                           | Reserved         | Always `0`                                       |
|    12 | `boolean`                         | Has mod data     | The current server path writes `true`            |
|    13 | Kahlua table, if field 12 is true | Mod data         | Serialized with Kahlua's native table serializer |

The three zero-valued fields are still present on the wire. The optional
mod-data table uses Kahlua's native table serializer (see
[Lua table serialization](../lua/table-serialization.md#kahlua-tables)); while
preparing the packet, the server temporarily removes the `camping`, `farming`
and `trapping` entries from this table. `saveToPacket` first calls
`getModData()`, so the current server path emits the presence flag as true
and serializes a table, even when it has no entries. The client reads this
table using world version `249`. Other fields written by GameTime's save-file
serializer, such as helicopter schedule and climate-manager data, are not
part of this packet form.

### 13. Erosion configuration

`ErosionConfig.save` is a fixed 148-byte block: 24 big-endian 32-bit integers
and 13 big-endian 32-bit floats. Fields occur in this exact order:

| Order | Wire type      | Field                                                                                                                                                                  | Details |
|------:|----------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------|---------|
|   1–3 | `int32` × 3    | `seeds.seedMain0`, `seeds.seedMain1`, `seeds.seedMain2`                                                                                                                |         |
|   4–6 | `int32` × 3    | `seeds.seedMoisture0`, `seeds.seedMoisture1`, `seeds.seedMoisture2`                                                                                                    |         |
|   7–9 | `int32` × 3    | `seeds.seedMinerals0`, `seeds.seedMinerals1`, `seeds.seedMinerals2`                                                                                                    |         |
| 10–12 | `int32` × 3    | `seeds.seedKudzu0`, `seeds.seedKudzu1`, `seeds.seedKudzu2`                                                                                                             |         |
| 13–16 | `int32` × 4    | `time.tickunit`, `time.ticks`, `time.eticks`, `time.epoch`                                                                                                             |         |
| 17–21 | `int32` × 5    | `season.lat`, `season.tempMax`, `season.tempMin`, `season.tempDiff`, `seasonLag`                                                                                       |         |
|    22 | `float32`      | `season.noon`                                                                                                                                                          |         |
| 23–25 | `int32` × 3    | `season.seedA`, `season.seedB`, `season.seedC`                                                                                                                         |         |
| 26–37 | `float32` × 12 | `season.jan`, `season.feb`, `season.mar`, `season.apr`, `season.may`, `season.jun`, `season.jul`, `season.aug`, `season.sep`, `season.oct`, `season.nov`, `season.dec` |         |

The debug settings from `ErosionConfig.Debug` are not written by this
serializer.

### 14. Global-object initial state

`SGlobalObjects.saveInitialStateForClient` writes:

| Order | Wire type                                     | Field             | Details                                          |
|------:|-----------------------------------------------|-------------------|--------------------------------------------------|
|     1 | `byte`                                        | System count      | Number of global-object systems                  |
|     2 | `UTF` (per system)                            | System name       |                                                  |
|     3 | `boolean` (per system)                        | Has initial state | Indicates whether the following table is present |
|     4 | `TableNetworkUtils` table, if field 3 is true | Initial state     | Serialized with `TableNetworkUtils`              |

Before serializing, the server adds an `_objects` table to each system's
initial-state table. `_objects` is a 1-based array; each object entry contains
numeric `x`, `y`, and `z` values plus the keys configured for synchronization
by that global-object system. The optional table uses
[`TableNetworkUtils`](../lua/table-serialization.md#tablenetworkutils), not
Kahlua's native serializer.

### 18. World dictionary

The world-dictionary field is an `int32` byte length followed by that many
bytes. The length includes both the dictionary data and the final length-
prefixed UTF sentinel `DICTIONARY_PACKET_END`. Within that byte block, the
dictionary data has this layout:

| Order | Wire type                                     | Field                  | Details                                                                                 |
|------:|-----------------------------------------------|------------------------|-----------------------------------------------------------------------------------------|
|     1 | `int32`                                       | Dictionary version     | Currently `1`                                                                           |
|     2 | `int16`                                       | Next info ID           |                                                                                         |
|     3 | `byte`                                        | Next object-name ID    |                                                                                         |
|     4 | `int32`                                       | Next sprite-name ID    |                                                                                         |
|     5 | `int32`                                       | Mod ID count           |                                                                                         |
|     6 | `GameWindow.WriteString` (repeated)           | Mod ID                 | Repeated `mod ID count` times                                                           |
|     7 | `int32`                                       | Module count           |                                                                                         |
|     8 | `GameWindow.WriteString` (repeated)           | Module name            | Repeated `module count` times                                                           |
|     9 | `int32`                                       | Item count             |                                                                                         |
|    10 | `ItemInfo` record (repeated)                  | Item info              | Repeated `item count` times                                                             |
|    11 | `int32`                                       | Entity count           |                                                                                         |
|    12 | `EntityInfo` record (repeated)                | Entity info            | Repeated `entity count` times                                                           |
|    13 | `int32`                                       | Object-name count      |                                                                                         |
|    14 | `byte` + `GameWindow.WriteString` (repeated)  | Object ID, object name | Repeated `object-name count` times                                                      |
|    15 | `int32`                                       | Sprite-name count      |                                                                                         |
|    16 | `int32` + `GameWindow.WriteString` (repeated) | Sprite ID, sprite name | Repeated `sprite-name count` times                                                      |
|    17 | `StringDictionary` data                       | String dictionary      | Nested encoding                                                                         |
|    18 | `ScriptsDictionary` data                      | Scripts dictionary     | Nested encoding                                                                         |
|    19 | `UTF`                                         | End sentinel           | Written with `putUTF` as `DICTIONARY_PACKET_END`; included in the enclosing byte length |

`ItemInfo`, `EntityInfo`, `StringDictionary` and `ScriptsDictionary` each have
their own nested encoding. The client first reads the length-prefixed byte
block, loads those dictionary sections, then reads and validates the sentinel.
This block is variable-sized and is commonly a substantial part of
`ConnectionDetails`.

These blocks are concatenated without per-field lengths, except where shown.
Their sizes depend on settings and world state, so a decoder must parse each
one in sequence rather than skip unknown fields by guessing offsets.

## Other dataset bodies

The transfer envelope does not define the body format for the other data IDs.
For example, the server writes zombie outfit descriptors through
`PersistentOutfits.save`, player-zombie descriptors as a 16-bit count followed
by descriptor records, radio data through `RecordedMedia`, and world-map data
through `WorldMapServer`. Decode each according to its corresponding game
serializer rather than treating it as text or JSON.

### Typical loading exchange

After connection setup, a client can request the loading datasets one at a
time. For example, a request for `ZombieOutfitDescriptors` followed by a full
response has this shape:

```text
Client → server: 86 00 29 01 01
Server → client: 86 00 29 02 01 <descriptor data…>
```

The later requests use data IDs `2` (`PlayerZombieDescriptors`), `3`
(`RadioData`) and `4` (`WorldMap`) in their corresponding requests and
responses. Large response bodies instead use `PartData` and the part
acknowledgement exchange.

The server sends `FullData` or `PartData` for `ConnectionDetails` after
accepting login and sending the configured `SpawnRegion` packets. The client
later requests the loading datasets, generally in the order listed above.
See [Login and world loading sequence](../login-sequence.md) for the broader
flow.

Although this packet uses reliable delivery, it is not reliable-ordered.
Reliability ensures retransmission while the connection remains active; it
does not guarantee ordering relative to other packets, including
`SpawnRegion`. The client and server must not treat the send order as a
cross-packet arrival-order guarantee.
