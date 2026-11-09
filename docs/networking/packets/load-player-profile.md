# LoadPlayerProfile

**Game version:** Project Zomboid 42.21.0 (commit `4a0e9546ec`).

`LoadPlayerProfile` checks whether a client has a saved character for a local
player index. The request and response use the same packet ID.

| Property         | Value                                                            |
|------------------|------------------------------------------------------------------|
| Packet ID        | `0x0004` (4)                                                     |
| Direction        | Client to server and server to client                            |
| State            | After connection details, while selecting or loading a character |
| RakNet priority  | High (1)                                                         |
| Reliability      | `ReliableOrdered`                                                |
| Ordering channel | `0`                                                              |

## Packet fields

| Field                           | Type       | Description                                                           |
|---------------------------------|------------|-----------------------------------------------------------------------|
| Request: player index           | `u8`       | Identifies the local player                                           |
| Response: profile status        | `bool`     | `false` means no saved profile; `true` is followed by profile fields  |
| Response: player index          | `u8`       | Echoes the requested local-player index                               |
| Response: position              | `f32 × 3`  | `x`, `y`, and `z`; present only when a profile exists                 |
| Response: world version         | `i32`      | Present only when a profile exists                                    |
| Response: dead status           | `bool`     | Present only when a profile exists                                    |
| Response: username              | UTF string | Present only when a profile exists                                    |
| Response: character data length | `i32`      | Byte length of the serialized profile; present only when it exists    |
| Response: character data        | `u8[]`     | Serialized profile bytes; present only when a profile exists          |

When a profile exists, the response includes information such as the character's
position, world version, dead status, username and serialized character data.
The client checks subsequent player indexes while profiles exist; a response
without a profile ends that search.

For a server that does not persist characters, reply with `false` and the
requested player index. This is a two-byte response (`00 <playerIndex>`), with
no profile fields; it tells the client that no saved character is available.

An existing live primary character can be used. Otherwise, the client proceeds
to character creation and spawn selection. Profile data may be cached, so this
exchange does not necessarily happen on every login.

Profile lookup happens before the loading queue. See the
[login and world loading sequence](../login-sequence.md).
