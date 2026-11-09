# Ping

**Game version:** Project Zomboid 42.21.0 (commit `4a0e9546ec`).

This application packet queries a server's connection count and player limit.
It is separate from RakNet's own latency ping, although it uses an established
RakNet connection.

| Property | Value |
| --- | --- |
| Packet ID | `0x0072` (114) |
| Direction | Client to server and server to client |
| State | Connected, before login |
| RakNet priority | Immediate (0) |
| Reliability | `Unreliable` |
| Ordering channel | `0` |

## Exchange

The client sends `Ping` instead of `Login`, including its configured server IP.
The server replies with that IP, the size of its connection list and the
configured player limit. The client raises `ServerPinged` and disconnects.

The count includes the ping connection, so the client subtracts one before
displaying it. A connection isn't always the same as one player.

## Packet fields

Fields use the encodings in [Network data types](../data-types.md).

### Request: client to server

| Field | Type | Description |
| --- | --- | --- |
| Marker | `u8` | `0x86` |
| Packet ID | `i16` | `0x0072` |
| IP | `String` | Client's configured server IP |

### Response: server to client

| Field | Type | Description |
| --- | --- | --- |
| Marker | `u8` | `0x86` |
| Packet ID | `i16` | `0x0072` |
| IP | `String` | IP from the request |
| Connections | `i32` | Size of the server's connection list |
| Maximum players | `i32` | Configured limit, capped at 254 |

### Example

A request for `127.0.0.1` and a response with one connection and a 32-player
limit have these application-level bytes:

```text
Request:  86 00 72 00 09 31 32 37 2e 30 2e 30 2e 31
Response: 86 00 72 00 09 31 32 37 2e 30 2e 30 2e 31 00 00 00 01 00 00 00 20
```

The client displays `0/32` after subtracting its own connection.
