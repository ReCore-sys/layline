# Login

**Game version:** Project Zomboid 42.21.0 (commit `4a0e9546ec`).

The client sends `Login` after connecting to a server. A server-list [Ping](ping.md)
uses a separate path and does not send login credentials.

| Property | Value |
| --- | --- |
| Packet ID | `0x0001` (1) |
| Direction | Client to server |
| State | Connected, before login |
| RakNet priority | High (1) |
| Reliability | `ReliableOrdered` |
| Ordering channel | `0` |

## Packet fields

After the application header, the packet contains:

| Field | Type | Description |
| --- | --- | --- |
| Marker | `u8` | `0x86` |
| Packet ID | `i16` | `0x0001` |
| Username | `String` | Client account username |
| Password | `String` | Client account password value |
| Client version | `String` | Game and build version |
| Authentication type | `i32` | Authentication method identifier |

The server reads the strings with the Java `ByteBufferReader.getUTF` method
and applies Java's `String.trim()`. Treat the password as sensitive; don't log
it. The authentication type depends on the server's authentication setup.

## What happens next

The server checks the client version, authenticates the account and checks
conditions such as duplicate usernames, capacity and ping limits. A rejected
login can receive [AccessDenied](access-denied.md) before the server disconnects
the client. Some accounts may also need a second authentication factor.

After an accepted login, the server reserves a connection slot and sends the
configured spawn regions and connection details in [RequestData](request-data.md).
The player is not in the world yet; see the
[login and world loading sequence](../login-sequence.md).
