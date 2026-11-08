# Login

**Protocol reference:** Project Zomboid 42.21.0 (revision `4a0e9546ec`).
Packet IDs and behavior may differ in other versions.

The client sends `Login` after establishing a RakNet connection for a normal
multiplayer join. A server-list [Ping](ping.md) uses a separate path and
sends `Ping` instead of `Login`.

| Property | Value |
| --- | --- |
| Packet ID | `0x0001` (1) |
| Direction | Client to server |
| State | Connected, before login |
| Priority | `1` (`HIGH`) |
| Reliability | `3` (`RELIABLE_ORDERED`) |
| Ordering channel | `0` |

## Packet format

Fields appear in the order shown. Types and byte order are described in
[Network data types](data-types.md). The table includes the application header
but excludes RakNet transport framing.

| Field | Type | Description |
| --- | --- | --- |
| Marker | Byte | `0x86` |
| Packet ID | Short | `0x0001` |
| Username | String | Client account username |
| Password | String | Client account password value |
| Client version | String | Client game and build version |
| Authentication type | Int | Client authentication type |

The server reads each string with `getUTF` and applies Java `String.trim()`
before processing it. The password field is sensitive and should not be logged.
The authentication type is a separate integer; its interpretation depends on
the authentication implementation.

## Server processing

The server checks the client version and authenticates the account, then applies
additional checks including duplicate usernames, capacity and ping limits.
Rejected clients can receive an [AccessDenied](rejected.md) packet and are
disconnected.
Some accounts may require a second authentication factor.

After an accepted login without a second-factor request, the server reserves a
connection slot, sends configured spawn regions, and sends connection details
in a `RequestData` packet. Those details include map, mod, server-option and
world data. This does not yet create the in-world player.

See [Login and world loading sequence](login-sequence.md) for the subsequent
loading and player-connect exchanges.
