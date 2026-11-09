# LoadPlayerProfile

**Protocol reference:** Project Zomboid 42.21.0 (revision `4a0e9546ec`).
Packet IDs and behavior may differ in other versions.

`LoadPlayerProfile` checks whether a client has a saved character for a
particular local-player index. It uses the same packet ID for the client request
and server response.

| Property | Value |
| --- | --- |
| Packet ID | `0x0004` (4) |
| Direction | Both |
| State | After connection details, while selecting or loading a character |
| Priority | `1` (`HIGH`) |
| Reliability | `3` (`RELIABLE_ORDERED`) |
| Ordering channel | `0` |

## Exchange

The client sends a request containing a player index. The server responds for
that index, indicating whether a saved profile exists. The exact primitive
field encodings are not specified here; consult the version-matched decompiled
reader and writer before implementing a codec.

When a profile exists, the response includes profile information such as the
character's position, world version, dead status, username and serialized
character data. The client may request subsequent player indexes while
profiles exist; the response indicating no profile ends that search.

The client can use an existing live primary character when one is available.
If there is no saved character, it proceeds to character creation and spawn
selection. Profile data may be cached by the client, so this exchange is not
necessarily repeated on every login.

This exchange is distinct from `LoginQueueRequest`: profile lookup happens
before the client enters the loading queue. See
[Login and world loading sequence](../login-sequence.md).
