# Lua values on the network

The game's decompiled Java code uses several Lua-table serializers. Their
formats are different, so a table's shape alone doesn't tell you how to decode it:

| Encoding | Used for | Notes |
| --- | --- | --- |
| Java `KahluaTable.save` / `KahluaTable.load` | Packets such as [SpawnRegion](../packets/spawn-region.md) | Runtime serializer; its implementation is not in the decompiled source |
| [`TableNetworkUtils`](table-serialization.md#tablenetworkutils) | Mod data and other ordinary table payloads | Small, tagged set of supported key and value types |
| [`PZNetKahluaTableImpl`](table-serialization.md#pznetkahlua-table) | Network actions and game-aware Lua values | Tagged values include references to live game objects |

See [Table serialization](table-serialization.md) for layouts and supported
values. Check the packet's Java reader or writer to identify the right format.
