# Spawn points

This documents the spawn-region and spawn-point Lua data used by the
Project Zomboid 42.21.0 decompilation in this repository. It describes the
game's current conventions, not a version-independent format.

## Region definitions

A region is a selectable spawn location group. The region list is returned by
`SpawnRegions()` and contains entries that point to a spawn-points file:

```lua
function SpawnRegions()
    return {
        {
            name = "Example Map",
            file = "media/maps/ExampleMap/spawnpoints.lua",
        },
        {
            name = "Server-only Region",
            serverfile = "path/to/server-spawnpoints.lua",
        },
    }
end
```

Each entry needs a `name` and either `file` or `serverfile`. The default server
region list is built from map directories containing `spawnpoints.lua`; the
map directory name becomes the region name and the file path is
`media/maps/<map-directory>/spawnpoints.lua`.

The region list is separate from the points file. On connection, the server
sends each region's index and serialized Lua table to the client.

Layline reads `Server/<save_name>_spawnregions.lua` from the Zomboid user
directory and loads only the point files listed by its `SpawnRegions()`
function. A `file` path is resolved from the game installation directory; a
`serverfile` path is resolved from the `Server` directory. This supports
save-specific files such as `<save_name>_spawnpoints.lua` while keeping the
region's configured name and file reference in the packet.

## Spawn-points files

A points file defines `SpawnPoints()` and returns a table keyed by profession.
Each profession maps to an array of points:

```lua
function SpawnPoints()
    return {
        unemployed = {
            { posX = 12067, posY = 6801, posZ = 0 },
            { posX = 12072, posY = 6804 },
        },
        firefighter = {
            { posX = 12080, posY = 6810, posZ = 0 },
        },
    }
end
```

Point coordinates can be specified in either of these forms:

| Fields | Meaning |
| --- | --- |
| `posX`, `posY`, optional `posZ` | Absolute world-square coordinates |
| `worldX`, `worldY`, `posX`, `posY`, optional `posZ` | Cell coordinates plus a position within the cell |

For the second form, each cell is 300 world squares wide and high:

```text
worldSquareX = worldX * 300 + posX
worldSquareY = worldY * 300 + posY
```

`posZ` defaults to `0`. When using cell-relative coordinates, both `worldX`
and `worldY` and both local coordinates are required. The decompiled parser
ignores points that do not provide the required numeric fields.

## Choosing a point

For a new character, the server looks up the selected region, then looks for a
point list matching the character's profession. If there is no list for that
profession, it tries `unemployed`; it chooses a point from the resulting list.
The server's configured `SpawnPoint=x,y,z` can override region selection.
Respawning in an allowed safehouse can override the position afterward.

The decompiled runtime also recognizes `position = "center"` while collecting
spawn locations: it chooses a point near the center of the map's metadata
bounds. This is separate from the ordinary numeric point shape used by
character creation, so prefer explicit coordinates for region points.

## Implementation notes

These are executable Lua files, not plain JSON or a declarative text format.
The game calls `SpawnRegions()` and `SpawnPoints()` to obtain Lua tables.
Supporting arbitrary spawn files therefore requires running compatible Lua;
supporting just the table shapes is not enough if files contain computed values
or helper calls.

The format and selection behavior here are based on
`decomp/network/SpawnRegions.java`, `decomp/iso/SpawnPoints.java`, and
`decomp/network/packets/character/CreatePlayerPacket.java`.
