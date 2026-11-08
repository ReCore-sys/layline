#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Extracts packet definitions from the decompiled game source into packets.json.

Packet ids are the ordinals of the `PacketTypes.PacketType` enum. Transport settings come
from either the enum constructor arguments or the `@PacketSetting` annotation on the
handler class.

Usage: uv run tools/extract_packets.py [--decomp decomp] [--output src/network/packets/packets.json]
"""

import argparse
import json
import re
import sys
from pathlib import Path

RELIABILITY = {
    0: "Unreliable",
    1: "UnreliableSequenced",
    2: "Reliable",
    3: "ReliableOrdered",
    4: "ReliableSequenced",
}

ENUM_START = re.compile(r"public enum PacketType\s*\{")
CONSTANT = re.compile(r"public static final \w+ (\w+) = (-?\d+);")
IMPORT = re.compile(r"^import zombie\.([\w.]+)\.(\w+);$", re.MULTILINE)
CLASS_ENTRY = re.compile(r"(\w+)\((\w+)\.class\)")
ARGS_ENTRY = re.compile(r"(\w+)\((.*)\)", re.DOTALL)
ANNOTATION = "@PacketSetting("


def fail(message: str) -> None:
    sys.exit(f"error: {message}")


def parse_int(token: str, constants: dict[str, int], context: str) -> int:
    token = token.strip().removeprefix("(byte)").strip()
    if re.fullmatch(r"-?\d+", token):
        return int(token)
    name = token.rsplit(".", 1)[-1]
    if name in constants:
        return constants[name]
    fail(f"cannot resolve `{token}` for {context}")


def enum_body(source: str) -> str:
    start = ENUM_START.search(source)
    if start is None:
        fail("PacketType enum not found")
    # The constant list ends at the first top level `;` after the opening brace.
    depth = 0
    for index in range(start.end(), len(source)):
        char = source[index]
        if char == "(":
            depth += 1
        elif char == ")":
            depth -= 1
        elif char == ";" and depth == 0:
            return source[start.end() : index]
    fail("unterminated PacketType enum")


def split_entries(body: str) -> list[str]:
    entries, depth, current = [], 0, []
    for char in body:
        if char == "(":
            depth += 1
        elif char == ")":
            depth -= 1
        if char == "," and depth == 0:
            entries.append("".join(current).strip())
            current = []
        else:
            current.append(char)
    tail = "".join(current).strip()
    if tail:
        entries.append(tail)
    return entries


def read_annotation(path: Path, constants: dict[str, int]) -> dict | None:
    source = path.read_text()
    start = source.find(ANNOTATION)
    if start == -1:
        return None
    start += len(ANNOTATION)
    end = source.index(")\n", start)
    settings = dict(re.findall(r"(\w+)\s*=\s*([^,]+?)\s*(?:,|$)", source[start:end].strip()))
    return {
        "priority": parse_int(settings["priority"], constants, path.name),
        "reliability": parse_int(settings["reliability"], constants, path.name),
        "channel": parse_int(settings["ordering"], constants, path.name),
        "capability": settings["requiredCapability"].rsplit(".", 1)[-1],
        "handling": parse_int(settings["handlingType"], constants, path.name),
    }


def read_constructor(name: str, raw_args: str, constants: dict[str, int]) -> dict:
    values = [value.strip() for value in raw_args.split(",")]
    # (priority, reliability, [ordering,] capability, server, client, loading)
    if len(values) not in (6, 7):
        fail(f"unexpected constructor arguments for `{name}`: {raw_args}")
    has_ordering = len(values) == 7
    offset = 3 if has_ordering else 2
    handling = sum(bit for bit, handler in zip((1, 2, 4), values[offset + 1 :]) if handler != "null")
    return {
        "priority": parse_int(values[0], constants, name),
        "reliability": parse_int(values[1], constants, name),
        "channel": parse_int(values[2], constants, name) if has_ordering else 0,
        "capability": values[offset].rsplit(".", 1)[-1],
        "handling": handling,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--decomp", type=Path, default=Path("decomp"))
    parser.add_argument("--output", type=Path, default=Path("src/network/packets/packets.json"))
    args = parser.parse_args()

    source = (args.decomp / "network" / "PacketTypes.java").read_text()
    constants = {name: int(value) for name, value in CONSTANT.findall(source)}
    imports = {name: package for package, name in IMPORT.findall(source)}

    packets = {}
    for ordinal, entry in enumerate(split_entries(enum_body(source))):
        if class_match := CLASS_ENTRY.fullmatch(entry):
            name, handler = class_match.groups()
            package = imports.get(handler)
            if package is None:
                fail(f"no import found for handler `{handler}` of `{name}`")
            handler_path = args.decomp.joinpath(*package.split("."), f"{handler}.java")
            settings = read_annotation(handler_path, constants)
            if settings is None:
                fail(f"{handler_path} has no @PacketSetting annotation")
        elif args_match := ARGS_ENTRY.fullmatch(entry):
            settings = read_constructor(*args_match.groups(), constants)
            name = args_match.group(1)
        else:
            fail(f"unrecognised enum entry: {entry!r}")

        if settings["reliability"] not in RELIABILITY:
            fail(f"`{name}` has unknown reliability {settings['reliability']}")
        if name in packets:
            fail(f"duplicate packet name `{name}`")

        packets[name] = {
            "id": ordinal,
            "reliability": RELIABILITY[settings["reliability"]],
            "channel": settings["channel"],
            "priority": settings["priority"],
            "capability": settings["capability"],
            "handling": settings["handling"],
        }

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(packets, indent=2) + "\n")
    print(f"wrote {len(packets)} packets to {args.output}")


if __name__ == "__main__":
    main()
