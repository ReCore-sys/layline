# Lua values on the network

The decompilation uses more than one way to serialize Lua tables. These
formats are not interchangeable:

| Encoding | Used for | Notes |
| --- | --- | --- |
| Kahlua table `save`/`load` | Packets such as [SpawnRegion](../packets/spawn-region.md) | Runtime serializer; its implementation is external to this decompilation |
| [`TableNetworkUtils`](table-serialization.md#tablenetworkutils) | Mod data and other ordinary table payloads | Small, tagged set of supported key and value types |
| [`PZNetKahluaTableImpl`](table-serialization.md#pznetkahlua-table) | Network actions and game-aware Lua values | Tagged values include references to live game objects |

See [Table serialization](table-serialization.md) for the layouts and
supported value types. Check the packet or call site to determine which
encoding applies; the table shape alone does not identify the wire format.
