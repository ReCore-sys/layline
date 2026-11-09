# AccessDenied

**Game version:** Project Zomboid 42.21.0 (commit `4a0e9546ec`).

The server sends `AccessDenied` when it rejects a login for reasons such as a
version mismatch, a banned account or a full server. It then disconnects the
client.

| Property | Value |
| --- | --- |
| Packet ID | `0x0064` (100) |
| Direction | Server to client |
| State | Client loading |
| RakNet priority | Immediate (0) |
| Reliability | `Reliable` |
| Ordering channel | `0` |

## Packet fields

| Field | Type | Description |
| --- | --- | --- |
| Marker | `u8` | `0x86` |
| Packet ID | `i16` | `0x0064` |
| Reason | `String` | Translation key suffix and optional arguments |

The client splits the reason on `##`, looks up the first part with the
`UI_OnConnectFailed_` prefix, and passes up to three following parts as
translation arguments. It then raises the Lua `OnConnectFailed` event with the
translated message.

Common reasons include `ServerFull`, `AlreadyConnected`, `Banned` and
`ClientVersionMismatch##<client version>##<server version>`.

### Example

`ServerFull` is encoded as:

```text
86 00 64 00 0a 53 65 72 76 65 72 46 75 6c 6c
```
