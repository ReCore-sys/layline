# Network data types

**Protocol reference:** Project Zomboid 42.21.0 (revision `4a0e9546ec`).

These encodings describe the game's application packets, not RakNet's
transport headers. Multi-byte values are big-endian unless a packet specifies
otherwise. Fields are written consecutively without padding.

## Primitive types

| Type | Size | Encoding | Example |
| --- | --- | --- | --- |
| Boolean | 1 byte | `00` for false, `01` for true; the reader treats any nonzero byte as true | `01` = true |
| Byte | 1 byte | Signed 8-bit integer; some fields interpret the same byte as unsigned | `ff` = -1 signed or 255 unsigned |
| Short | 2 bytes | Signed 16-bit two's-complement integer | `00 72` = 114 |
| Int | 4 bytes | Signed 32-bit two's-complement integer | `00 00 00 20` = 32 |
| Long | 8 bytes | Signed 64-bit two's-complement integer | `00 00 00 00 00 00 00 01` = 1 |
| Float | 4 bytes | IEEE 754 binary32 | `3f 80 00 00` = 1.0 |
| Double | 8 bytes | IEEE 754 binary64 | `3f f0 00 00 00 00 00 00` = 1.0 |
| Char | 2 bytes | Unsigned Java UTF-16 code unit, not necessarily a complete Unicode character | `00 41` = `A` |
| Enum | 1 byte | Enum ordinal, read as an unsigned byte; the enum determines the valid range | `02` = ordinal 2 |

Signedness and interpretation are field-specific: for example, the game packet
marker is read as an unsigned byte even though the underlying Java byte is
signed. Enum ordinals can change when their declaration order changes.

## String

`ByteBufferWriter.putUTF` writes a signed 16-bit byte length followed by that
many UTF-8 bytes. The length counts **bytes**, not characters. Null and empty
strings are both written as `00 00`.

| Value | Bytes |
| --- | --- |
| Empty string | `00 00` |
| `A` | `00 01 41` |
| `é` | `00 02 c3 a9` |
| `127.0.0.1` | `00 09 31 32 37 2e 30 2e 30 2e 31` |

`ByteBufferReader.getUTF` reads the length as a signed short, so a valid
non-negative length is at most 32,767 bytes. It requires the specified bytes
to be present and decodes malformed UTF-8 using replacement characters.
The writer casts the encoded length to a short; lengths above 32,767 therefore
cannot be read as a valid length by this reader.

Not every string in the game necessarily uses this format. In particular,
`GameWindow.ReadString(ByteBuffer)` has different behavior for non-positive
lengths; check the individual packet's reader and writer before assuming the
encoding or error behavior.

## Decoding

RakNet delivers application messages rather than a byte stream; the values
above are read in order from one message. A message can still contain invalid
or incomplete application data. Check that each field, length and enum value
is valid before reading it, and reject malformed packets rather than combining
them with the next message.
