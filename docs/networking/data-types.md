# Network data types

**Game version:** Project Zomboid 42.21.0 (commit `4a0e9546ec`).

These types describe the game's application packets, not RakNet's transport
headers. Multi-byte numbers are big-endian unless noted; fields have no padding.

## Primitive types

| Rust-style type | Size | Encoding | Example |
| --- | --- | --- | --- |
| `bool` | 1 byte | `00` is false and `01` is true; readers treat any nonzero value as true | `01` = true |
| `u8` | 1 byte | Unsigned 8-bit integer | `86` = 134 |
| `i8` | 1 byte | Signed 8-bit integer | `ff` = -1 |
| `i16` | 2 bytes | Signed 16-bit two's-complement integer | `00 72` = 114 |
| `i32` | 4 bytes | Signed 32-bit two's-complement integer | `00 00 00 20` = 32 |
| `i64` | 8 bytes | Signed 64-bit two's-complement integer | `00 00 00 00 00 00 00 01` = 1 |
| `f32` | 4 bytes | IEEE 754 binary32 | `3f 80 00 00` = 1.0 |
| `f64` | 8 bytes | IEEE 754 binary64 | `3f f0 00 00 00 00 00 00` = 1.0 |
| `u16` | 2 bytes | Unsigned Java UTF-16 code unit; not necessarily a full character | `00 41` = `A` |
| `enum` | Usually 1 byte | Ordinal encoded using the enum's underlying integer type | `02` = ordinal 2 |

Use the signedness that matches the field. The application marker is `u8`,
although the Java code stores it in a signed `byte`. Enum ordinals can change
when the Java enum's declaration order changes.

## `String`

`String` is a logical type; its wire encoding depends on the writer. The common
`ByteBufferWriter.putUTF` method in the game's Java code writes a signed `i16`
byte length, then that many UTF-8 bytes. The length counts bytes, not characters.
Null and empty strings are both written as `00 00`.

| Value | Bytes |
| --- | --- |
| Empty string | `00 00` |
| `A` | `00 01 41` |
| `é` | `00 02 c3 a9` |
| `127.0.0.1` | `00 09 31 32 37 2e 30 2e 30 2e 31` |

The matching Java method, `ByteBufferReader.getUTF`, reads the length as an
`i16`, so it accepts at most 32,767 bytes. It requires that many bytes to be
present and replaces malformed UTF-8 with replacement characters. The writer
casts its length to `i16`; a longer string won't decode as a valid length.

The Java helper `GameWindow.WriteString` also writes a signed 16-bit length and
UTF-8 bytes, but handles non-positive lengths differently. These method names
come from the game's Java implementation; they are not Rust APIs. Check which
writer a packet uses before choosing a string decoder.

## Decoding

RakNet delivers one application message at a time, not a continuous byte
stream. Decode fields within that message and check lengths, values and enum
ordinals before using them.
