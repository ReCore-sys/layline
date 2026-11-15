# Layline
![GitHub License](https://img.shields.io/github/license/ReCore-sys/layline?color=teal)
![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/ReCore-sys/layline?color=teal)
![Static Badge](https://img.shields.io/badge/Zombies-A_Lot-teal)
![Static Badge](https://img.shields.io/badge/Discord-TBA-blue?logo=discord&color=teal)




Project zomboid server in rust. Very early dev, don't expect anything to work yet.

## Why

I'm aware that the world is already flooded with unneeded rust rewrites of stuff, and I'm not going to pretend this
piece of software is the key to world peace. But project zomboid does have considerable performance issues and is built
on some very old code, and I think it would be fun to try and rewrite it in rust. Dragging the game kicking and
screaming into the 21st century would have some major upsides for the community, and being able to run a server on a
fraction of the hardware is hard to dislike.

## Features

- Selecting your spawn location and character
- Validate some parts of saved chunks

## Goals

### What we are trying to achieve

- Extreme performance improvements over the current server.
- Compatibility with the existing client. You won't need any special client or mods to connect to a layline server,
  it'll just work.
- Modern codebase that is easier to maintain and extend.
- Full lua mod support.
- "Good enough" parity with vanilla behavior.
- Better management tools. Web dashboards and TUI mod pickers could be examples of stuff we might add.

### What we are not trying to achieve

- Java modding. That'd be virtually impossible to do in rust, and a vast majority of mods are written in lua anyway.
- Perfect parity with vanilla behavior. It'll be the same game with all the same features and gameplay, but some things
  will be slightly different. For example zombies might have slightly different pathfinding or loot might not have
  identical distributions. We'll match what makes sense, but there isn't much point getting seed-identical behavior if
  it won't affect gameplay in any meaningful way. That being said, if some niche feature being 1:1 with vanilla is vital
  to your plans, come have a chat, and we'll see what we can do.
- Client improvements. This is a server project, and we won't be touching the client. ie, you won't be getting better
  FPS, but the server you are playing on will be getting 5 trillion TPS on a raspbery pi. Maybe.

## Contributing

To get started, clone the repo and run `cargo build`. You will need to have rust installed, grab it
from [here](https://rustup.rs/).

You will also need project zomboid and the server files, which you can get from steam. During development, set the
`ZOMBOID_SERVER_INSTALL_PATH` to where the project zomboid server files are installed. This is usually something like
`C:/Program Files (x86)/Steam/steamapps/common/Project Zomboid Dedicated Server` on Windows. I'll assume you know what
you are doing on linux.

To get a decompiled version of the vanilla game, there's a python script in the tools folder that will run the
ZomboidDecompiler for you. Those decompiled files can't be redistributed for legal reasons, and you don't need the
decomp to build or run the server, but it is useful for reference and debugging. The decompiler is a java program, so
you'll need to have java installed to run it.

## License

GPL-3.0 baybeeee