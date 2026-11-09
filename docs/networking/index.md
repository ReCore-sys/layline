# Networking

**Game version:** Project Zomboid 42.21.0 (commit `4a0e9546ec`).
Behavior and packet IDs may differ in other versions.

Project Zomboid carries its multiplayer packets in RakNet messages. Steam
connections use the same peer interface, with some Steam-specific behavior.

## Packet format

The following header identifies a game packet within a RakNet message. It does
not include RakNet's transport framing.

| Field | Size | Description |
| --- | --- | --- |
| Marker | `u8` | `0x86`, identifying a game packet |
| Packet ID | `i16` | Signed big-endian ordinal from the Java `PacketType` enum |
| Payload | Variable | Packet-specific data |

Each Java packet type specifies its RakNet priority, reliability and ordering
channel. IDs follow the Java `PacketType` enum's declaration order, so they
can change between game versions.

## Data types

See [Network data types](data-types.md) for primitive encodings, byte order
and string handling.

## Lua tables

Lua tables use different encodings depending on the packet and value types.
See [Lua table serialization](lua/index.md) before implementing a table
decoder.

## Packets

| Packet | ID | Direction | Description |
| --- | --- | --- | --- |
| [Login](packets/login.md) | `0x0001` | Client to server | Account login and version information |
| [LoginQueueRequest](packets/login-queue-request.md) | `0x0002` | Both | Request loading admission or report queue status |
| [LoadPlayerProfile](packets/load-player-profile.md) | `0x0004` | Both | Request a player profile or report its availability |
| [RequestData](packets/request-data.md) | `0x0029` | Both | Request or transfer connection and world data |
| [AccessDenied](packets/access-denied.md) | `0x0064` | Server to client | Login rejection reason |
| [Ping](packets/ping.md) | `0x0072` | Both | Server-list connection and player-limit query |
| [SpawnRegion](packets/spawn-region.md) | `0x0073` | Server to client | Spawn-region index and serialized Lua table |

See [Login and world loading sequence](login-sequence.md) for the exchanges
involved in joining a server and entering the world.