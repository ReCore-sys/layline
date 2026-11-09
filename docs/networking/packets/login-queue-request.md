# LoginQueueRequest

**Game version:** Project Zomboid 42.21.0 (commit `4a0e9546ec`).

| Property | Value |
| --- | --- |
| Packet ID | `0x0002` (2) |
| Direction | Client to server and server to client |
| State | After profile lookup, before loading world data |
| RakNet priority | Immediate (0) |
| Reliability | `ReliableOrdered` |
| Ordering channel | `0` |

## Packet fields

The client request has no payload. The server response starts with a message
type:

| Field | Type | Description |
| --- | --- | --- |
| Message type | `u8` | `1` for `ConnectionImmediate`; `2` for `PlaceInQueue` |

`ConnectionImmediate` has no fields after the message type. `PlaceInQueue`
includes the following fields in order:

| Field | Type | Description |
| --- | --- | --- |
| Queue position | `u8` | 1-based position in the login queue |
| Maximum players | `i16` | Server player limit |
| Current players | `i16` | Current player count |
| Current zombies | `i32` | Server zombie count |
| Zombies killed by fire today | `i32` | Daily statistic |
| Zombies killed today | `i32` | Daily statistic |
| Players zombified today | `i32` | Daily statistic |
| Players killed by fire today | `i32` | Daily statistic |
| Players killed by zombies today | `i32` | Daily statistic |
| Players killed by players today | `i32` | Daily statistic |
| Corpses burned today | `i32` | Daily statistic |
| Server-time minutes | `u8` | Current game-time minute |
| Server-time hour | `u8` | Current game-time hour |
| Server-time day | `u8` | Zero-based day |
| Server-time month | `u8` | Zero-based month |
| Server-time year | `i16` | Current game-time year |
| Base temperature | `f32` | Climate data |
| Humidity | `f32` | Climate data |
| Rain intensity | `f32` | Climate data |
| Wind speed (kph) | `f32` | Climate data |
| Wind angle (degrees) | `f32` | Climate data |
| Fog intensity | `f32` | Climate data |
| Season ID | `u8` | Current climate season |
| Moon phase | `u8` | Current moon phase |

When the server admits the client immediately (including when queueing is
disabled), the response is just the `ConnectionImmediate` message type.

## Wire examples

```text
Client → server: 86 00 02
Server → client: 86 00 02 01
```

The client request contains only the application packet header. The response
adds `01`, the `ConnectionImmediate` message type. A queued response begins
with `02` and is followed by the `PlaceInQueue` fields above.
