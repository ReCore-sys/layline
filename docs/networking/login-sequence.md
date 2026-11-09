# Login and world loading sequence

**Game version:** Project Zomboid 42.21.0 (commit `4a0e9546ec`).
Packet IDs, layouts and behavior may differ in other versions.

A typical non-Steam join to a dedicated server looks like this. Server
settings, existing characters and loading progress affect the exact traffic.
Packets can overlap, so the steps below don't guarantee their arrival order.

The packet IDs below identify Zomboid application packets carried inside RakNet
messages. See [Networking](index.md) for the application header and
[Network data types](data-types.md) for field encodings.

## 1. Connect and authenticate

1. The client connects over RakNet and sends
   [Login](packets/login.md) (`0x0001`) with its username, password, version
   and authentication type.
2. The server checks the version, authenticates the account and checks
   conditions such as capacity and duplicate usernames. A rejection can
   produce [AccessDenied](packets/access-denied.md) (`0x0064`) before
   disconnection. Some accounts also require a second-factor challenge.
3. On acceptance, the server reserves a connection slot and prepares the
   player's online ID. The player isn't in the world yet.
4. The server sends zero or more [SpawnRegion](packets/spawn-region.md)
   (`0x0073`) packets containing an index and a serialized Lua table, followed
   by connection details carried in [RequestData](packets/request-data.md)
   (`0x0029`). The connection details include the slot, role, map, mods, spawn
   location, server and sandbox options, game time and other world data. The
   client checks required mods before continuing.

The server sends spawn regions before connection details, but neither packet
type guarantees arrival first. A server-list [Ping](packets/ping.md) is a
separate path and doesn't log in.

## 2. Load or choose a character

After reading the connection details, the client checks for saved characters
with [LoadPlayerProfile](packets/load-player-profile.md) (`0x0004`). It asks
about a local-player index; the server reports whether a profile exists and
includes character data when it does. The client checks later indexes until
the server reports no profile.

The client can load an existing live primary character. If there isn't one, it
offers character creation and spawn selection. Cached profile data can also
mean no request is needed.

## 3. Enter the loading queue

The client sends `LoginQueueRequest` (`0x0002`) with no payload when entering
the loading queue state. The server replies using the same packet ID:

| Response type | `u8` | Meaning |
| --- | --- | --- |
| `ConnectionImmediate` | `1` | The client may begin game loading. |
| `PlaceInQueue` | `2` | The client must wait; the response includes its position and server information. |

The server sends the immediate response even when its queue is disabled. With
queuing enabled, it can send position updates before admitting the client.
Other loading requests check that the client entered this state.

## 4. Download loading data

During the Java client's `GameLoadingState`, it requests these datasets in
order, waiting for each response before moving to the next:

1. `ZombieOutfitDescriptors`
2. `PlayerZombieDescriptors`
3. `RadioData`
4. `WorldMap`

These are all `RequestData` (`0x0029`) exchanges, not four separate packet
IDs. The server's earlier connection details also use this mechanism. The
client can request other datasets, such as `PlayerVisited`, outside this
initial chain.

`RequestData` starts with a one-byte request type and a one-byte data ID:

| Request type | Value | Use |
| --- | --- | --- |
| `Request` | `1` | Client requests a dataset. |
| `FullData` | `2` | Server sends the complete dataset. |
| `PartData` | `3` | Server sends a portion of the dataset. |
| `PartDataACK` | `4` | Client acknowledges a window of received parts. |

| Data ID | Value |
| --- | --- |
| `ConnectionDetails` | `0` |
| `ZombieOutfitDescriptors` | `1` |
| `PlayerZombieDescriptors` | `2` |
| `RadioData` | `3` |
| `WorldMap` | `4` |
| `PlayerVisited` | `5` |

For data smaller than 1,024 bytes, the server sends `FullData` followed by the
data. Otherwise it sends `PartData`: total size, offset and part size as
big-endian `i32` values, followed by the part bytes. Parts are at most 1,024 bytes;
the client acknowledges after receiving 204,800 bytes of parts, allowing the
server to send the next window. This application-level transfer is separate
from RakNet fragmentation and acknowledgements.

## 5. Check content and load the world

If `DoLuaChecksum` is enabled, world initialization performs a `Checksum`
(`0x0000`) exchange. The initial request contains an `i16` subtype `1` and Lua,
script and animation checksum strings. The server replies with the same subtype
and three `bool` values. A mismatch can lead to further group/file checksum
exchanges and rejection; the server can allow a role to bypass the check.
With the option disabled, the client skips this checksum exchange.

The client also loads map chunks. It sends `RequestZipList` (`0x0027`) with
chunk request numbers, coordinates and checksums. The server can respond with
`SentChunk` (`0x0026`) parts, `NotRequiredInZip` (`0x0028`) for requests that
do not need a download, or `ChunkNotReady` (`0x002a`) when a chunk is not
ready. The client can also use `NotRequiredInZip` to cancel requests. Chunk
streaming continues beyond initial loading; it is not one single response.

For a new character, the client sends `CreatePlayer` (`0x0005`) with the local
player index, spawn region, descriptor, visuals and traits. The server chooses
a spawn point, creates and saves the character, then replies with the index
and serialized player data. Existing characters use their saved profile
instead.

## 6. Finish loading and connect the player

1. Once game loading completes, the client sends `LoginQueueDone` (`0x0003`)
   with the loading time as an `i64`. The server releases the queue position
   and replies with the same packet ID and a zero `i64`. That reply completes
   the client's loading state.
2. On entering the in-game state, the client sends `PlayerConnect` (`0x0006`):
   a player index, chunk-grid width and extra-info flags, each one byte. It
   also sends other initial traffic, including `TimeSync` (`0x0062`).
3. The server loads the saved character, associates it with the connection,
   assigns its online ID and marks the connection fully connected. It sends
   `ConnectedPlayer` (`0x0007`) to the joining client and informs other
   clients. The joining client's own response contains an `i16` marker set to `-1`,
   its player index and online ID, game time and additional player data.
4. The client waits for its player to receive an online ID from that response.
   The server and client also exchange initial world, player, weather, mod and
   other state; those packets are not fully enumerated here.
