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

| Field                             | Type          | Description                                                                |
|-----------------------------------|---------------|----------------------------------------------------------------------------|
| Request: player index             | ¯\\\_(ツ)\_/¯ | Identifies the local player                                                |
| Response: profile status and data | ¯\\\_(ツ)\_/¯ | Reports whether a saved profile exists; includes profile data when it does |

When a profile exists, the response includes information such as the character's
position, world version, dead status, username and serialized character data.
The client checks subsequent player indexes while profiles exist; a response
without a profile ends that search.

An existing live primary character can be used. Otherwise, the client proceeds
to character creation and spawn selection. Profile data may be cached, so this
exchange does not necessarily happen on every login.

Profile lookup happens before the loading queue. See the
[login and world loading sequence](../login-sequence.md).
