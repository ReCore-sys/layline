# Networking

**Protocol reference:** Project Zomboid 42.21.0 (revision `4a0e9546ec`).
Behavior and packet IDs may differ in other versions.

Project Zomboid uses RakNet for its main multiplayer connection. Game packets
are carried as RakNet messages. Steam connections use the same peer interface
but have Steam-specific connection behavior.

## Packet format

The following header identifies a game packet within a RakNet message. It does
not include RakNet's transport framing.

| Field | Size | Description |
| --- | --- | --- |
| Marker | 1 byte | `0x86`, identifying a game packet |
| Packet ID | 2 bytes | Signed big-endian `PacketType` ordinal |
| Payload | Variable | Packet-specific data |

Each packet type specifies its RakNet priority, reliability and ordering
channel. Packet IDs are derived from the order of the `PacketType` enum, so
they are version-dependent.

## Data types

See [Network data types](data-types.md) for primitive encodings, byte order
and string handling.

## Packets

| Packet | ID | Direction | Description |
| --- | --- | --- | --- |
| [Login](packets/login.md) | `0x0001` | Client to server | Account login and version information |
| [AccessDenied](packets/rejected.md) | `0x0064` | Server to client | Login rejection reason |
| [Ping](packets/ping.md) | `0x0072` | Both | Server-list connection and player-limit query |

See [Login and world loading sequence](login-sequence.md) for the exchanges
involved in joining a server and entering the world.