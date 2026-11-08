# Login and world loading sequence

**Protocol reference:** Project Zomboid 42.21.0 (revision `4a0e9546ec`).
Packet IDs, layouts and behavior may differ in other versions.

This page describes the normal non-Steam dedicated-server join path for the
primary local player. It is an overview of the main exchanges, not a complete
catalogue of every packet sent while loading. Server options, existing character
data, loading progress and other players affect the traffic. Packets may be
sent concurrently, and packets with different reliability or ordering settings
should not be assumed to arrive in the order listed here.

The packet IDs below identify Zomboid application packets carried inside RakNet
messages. See [Networking](index.md) for the application header and
[Network data types](data-types.md) for field encodings.

## 1. Connect and authenticate

1. The client establishes a RakNet connection, then sends
   [Login](login.md) (`0x0001`) with its username, password, client version
   and authentication type.
2. The server checks the version, authenticates the account and checks
   conditions such as capacity and duplicate usernames. A rejection can
   produce [AccessDenied](rejected.md) (`0x0064`) followed by disconnection.
   A second-factor challenge is a separate conditional path.
3. On acceptance, the server reserves a connection slot and prepares the
   player's online ID. No in-world player has been registered yet.
4. The server sends zero or more `SpawnRegion` (`0x0073`) packets containing an
   index and a serialized Lua table, followed by connection details carried in
   `RequestData` (`0x0029`). These contain the slot, role, map, mods, spawn
   location, server and sandbox options, game time and other world data.
   The client processes the details and checks required mods before proceeding.

The server issues spawn-region packets before connection details, but both use
plain reliable delivery rather than reliable ordered delivery, so this is not
a cross-packet arrival-order guarantee. A server-list [Ping](ping.md) is a
separate connection path and does not perform login.

## 2. Load or choose a character

After processing connection details, the normal client checks for a saved
character. `LoadPlayerProfile` (`0x0004`) exchanges a player index for a profile
response. A response says whether the profile exists; if it does, it also
contains position, world version, dead status, username and serialized
character data. The client requests subsequent indexes while profiles exist,
until a response says no profile exists.

An existing live primary character can be used for loading. Otherwise, the
client presents spawn and character-creation screens. Profile data may already
be cached by the client, so an additional profile request is not necessarily
sent every time the client checks it.

## 3. Enter the loading queue

The client sends `LoginQueueRequest` (`0x0002`) with no payload when entering
the loading queue state. The server replies using the same packet ID:

| Response type | Byte | Meaning |
| --- | --- | --- |
| `ConnectionImmediate` | `1` | The client may begin game loading. |
| `PlaceInQueue` | `2` | The client must wait; the response includes its position and server information. |

Even when the configurable queue is disabled, the immediate response is part
of this exchange. When queuing is enabled, the server can send position updates
and later admit the client. The server records that the connection entered the
loading queue; other loading requests check this state.

## 4. Download loading data

During `GameLoadingState`, the client requests the following datasets in order,
waiting for each response before requesting the next:

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
big-endian integers, followed by the part bytes. Parts are at most 1,024 bytes;
the client acknowledges after receiving 204,800 bytes of parts, allowing the
server to send the next window. This application-level transfer is separate
from RakNet fragmentation and acknowledgements.

## 5. Check content and load the world

If `DoLuaChecksum` is enabled, world initialization performs a `Checksum`
(`0x0000`) exchange. The initial request contains a short subtype `1` and Lua,
script and animation checksum strings. The server replies with the same subtype
and three booleans. A mismatch can lead to further group/file checksum
exchanges and rejection; the server can allow a role to bypass the check.
With the option disabled, the client skips this checksum exchange.

The client also loads map chunks. It sends `RequestZipList` (`0x0027`) with
chunk request numbers, coordinates and checksums. The server can respond with
`SentChunk` (`0x0026`) parts, `NotRequiredInZip` (`0x0028`) for requests that
do not need a download, or `ChunkNotReady` (`0x002a`) when a chunk is not
ready. The client can also use `NotRequiredInZip` to cancel requests. Chunk
streaming continues beyond initial loading; it is not one single response.

For a new character, the client sends `CreatePlayer` (`0x0005`) during world
initialization, including the local player index, spawn region, character
descriptor, visuals and traits. The server chooses a spawn point, creates and
saves the character, and responds with the index and serialized player data.
For an existing character, the client instead loads the saved profile; it
does not need this new-character exchange.

## 6. Finish loading and connect the player

1. Once game loading completes, the client sends `LoginQueueDone` (`0x0003`)
   with the loading time as a long. The server releases the queue position and
   replies with the same packet ID and a zero long. The client's loading state
   completes on that response.
2. On entering the in-game state, the client sends `PlayerConnect` (`0x0006`):
   a player index, chunk-grid width and extra-info flags, each one byte. It
   also sends other initial traffic, including `TimeSync` (`0x0062`).
3. The server loads the saved character, associates it with the connection,
   assigns its online ID and marks the connection fully connected. It sends
   `ConnectedPlayer` (`0x0007`) to the joining client and informs other
   clients. The joining client's own response contains a `-1` short marker,
   its player index and online ID, game time and additional player data.
4. The client waits for its player to receive an online ID from that response.
   The server and client also exchange initial world, player, weather, mod and
   other state; those packets are not fully enumerated here.

Completing the login packet alone is therefore not sufficient to enter the
world: connection details, loading data, character handling, chunk streaming
and the final player-connect exchange all have distinct responsibilities.
