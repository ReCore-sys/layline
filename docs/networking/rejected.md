# Rejected

**Protocol reference:** Project Zomboid 42.21.0 (revision `4a0e9546ec`).
Packet IDs and behavior may differ in other versions.

The server can send `AccessDenied` when it rejects a [Login](login.md), for
example because of a version mismatch, a banned account or a full server.
In those login rejection paths, the server then disconnects the client.

| Property | Value |
| --- | --- |
| Packet ID | `0x0064` (100) |
| Direction | Server to client |
| State | Client loading |
| Priority | `0` (`IMMEDIATE`) |
| Reliability | `2` (`RELIABLE`) |
| Ordering channel | `0` |

## Packet format

Fields appear in the order shown. Types and byte order are described in
[Network data types](data-types.md). The table includes the application header
but excludes RakNet transport framing.

| Field | Type | Description |
| --- | --- | --- |
| Marker | Byte | `0x86` |
| Packet ID | Short | `0x0064` |
| Reason | String | Translation key suffix and optional arguments |

The client splits the reason on `##`, looks up the first part with the
`UI_OnConnectFailed_` prefix, and passes up to the next three parts as
translation arguments. It then triggers the Lua `OnConnectFailed` event with
the translated message.

Examples of reasons sent by the server include `ServerFull`, `AlreadyConnected`,
`Banned` and `ClientVersionMismatch##<client version>##<server version>`.

### Example

An `AccessDenied` packet with reason `ServerFull` has these application-level
bytes:

```text
86 00 64 00 0a 53 65 72 76 65 72 46 75 6c 6c
```
