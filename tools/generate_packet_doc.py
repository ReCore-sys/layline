#!/usr/bin/env python3
"""Generate a packet documentation stub from packets.json.

Usage: python tools/generate_packet_doc.py Login
"""

import argparse
import json
import re
from pathlib import Path

DEFAULT_PACKETS = Path("src/network/packets/packets.json")
DEFAULT_OUTPUT_DIR = Path("docs/networking/packets")
DEFAULT_GAME_VERSION = "42.21.0"
DEFAULT_COMMIT = "4a0e9546ec"
PRIORITIES = {
    0: "Immediate",
    1: "High",
    2: "Medium",
    3: "Low",
}


def slugify(name: str) -> str:
    name = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1-\2", name)
    return re.sub(r"([a-z0-9])([A-Z])", r"\1-\2", name).lower()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("packet", help="packet name from packets.json")
    parser.add_argument("--packets", type=Path, default=DEFAULT_PACKETS)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--game-version", default=DEFAULT_GAME_VERSION)
    parser.add_argument("--commit", default=DEFAULT_COMMIT)
    parser.add_argument(
        "--force", action="store_true", help="replace an existing output file"
    )
    args = parser.parse_args()

    try:
        packets = json.loads(args.packets.read_text())
    except (OSError, json.JSONDecodeError) as error:
        parser.error(f"could not read packet data: {error}")

    packet = packets.get(args.packet)
    if packet is None:
        parser.error(f"packet {args.packet!r} was not found in {args.packets}")

    try:
        packet_id = int(packet["id"])
        priority = int(packet["priority"])
        reliability = packet["reliability"]
        channel = int(packet["channel"])
        priority_name = PRIORITIES[priority]
    except (KeyError, TypeError, ValueError) as error:
        parser.error(f"invalid metadata for packet {args.packet!r}: {error}")

    if not 0 <= packet_id <= 0xFFFF:
        parser.error(f"packet ID for {args.packet!r} is outside the 16-bit range")
    if not isinstance(reliability, str) or not reliability:
        parser.error(f"invalid reliability for packet {args.packet!r}")

    output = args.output or DEFAULT_OUTPUT_DIR / f"{slugify(args.packet)}.md"
    if output.exists() and not args.force:
        parser.error(f"{output} already exists; pass --force to replace it")

    content = f"""# {args.packet}

**Game version:** Project Zomboid {args.game_version} (commit `{args.commit}`).

| Property | Value |
| --- | --- |
| Packet ID | `0x{packet_id:04x}` ({packet_id}) |
| Direction | To be documented |
| State | To be documented |
| RakNet priority | {priority_name} ({priority}) |
| Reliability | `{reliability}` |
| Ordering channel | `{channel}` |

## Packet fields

Use Rust-style names for primitive wire types. For strings, see
[Network data types](../data-types.md) and note the writer-specific encoding.

| Field | Type | Description |
| --- | --- | --- |
|  |  |  |
"""
    try:
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(content)
    except OSError as error:
        parser.error(f"could not write {output}: {error}")

    print(f"wrote packet documentation stub to {output}")


if __name__ == "__main__":
    main()
