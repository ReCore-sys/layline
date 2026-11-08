# Server-list ping

**Protocol reference:** Project Zomboid 42.21.0 (revision `4a0e9546ec`).
Packet IDs and behavior may differ in other versions.

The server-list ping queries a server's connection count and maximum player
count. It is a Zomboid application packet exchanged over an established RakNet
connection, distinct from RakNet's own ping and latency measurement.

| Property | Value |
| --- | --- |
| Packet ID | `0x0072` (114) |
| Direction | Client to server and server to client |
| State | Connected, before login |
| Priority | `0` (`IMMEDIATE`) |
| Reliability | `0` (`UNRELIABLE`) |
| Ordering channel | `0` |

## Sequence

1. The client establishes a RakNet connection.
2. The client sends `Ping` instead of a login packet. The request contains the
   client's configured server IP string.
3. The server accepts `Ping` before login, marks the connection as pinged and
   replies with the IP string, connection count and maximum player count.
4. The client raises the `ServerPinged` event and disconnects.

The server sends the size of its connection list, including the ping
connection. The client subtracts one from that value when displaying the player
count. This is a connection count, not necessarily a count of individual
players. A RakNet-level connection can still be subject to password or other
connection requirements.

## Packet format

Fields appear in the order shown. Types and byte order are described in
[Network data types](data-types.md). The tables include the
application header but exclude RakNet transport framing.

### Request: client to server

| Field | Type | Description |
| --- | --- | --- |
| Marker | Byte | `0x86` |
| Packet ID | Short | `0x0072` |
| IP | String | Client's configured server IP |

### Response: server to client

| Field | Type | Description |
| --- | --- | --- |
| Marker | Byte | `0x86` |
| Packet ID | Short | `0x0072` |
| IP | String | IP from the request |
| Connections | Int | Size of the server's connection list |
| Maximum players | Int | Configured maximum, capped at 254 |

### Example

A request for `127.0.0.1` and a response with one connection and a maximum of
32 players have the following application-level bytes:

```text
Request:  86 00 72 00 09 31 32 37 2e 30 2e 30 2e 31
Response: 86 00 72 00 09 31 32 37 2e 30 2e 30 2e 31 00 00 00 01 00 00 00 20
```

In this example, the client displays `0/32` after subtracting its own
connection.