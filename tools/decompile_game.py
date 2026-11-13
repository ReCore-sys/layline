#!/usr/bin/env python3
"""Download ZomboidDecompiler and refresh the repository's decomp/ directory.

Run from any directory with `python3 tools/decompile_game.py`; pass --game-dir
if the Zomboid installation is not detected automatically.
"""

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import zipfile
from pathlib import Path
from urllib.error import URLError
from urllib.request import Request, urlopen

RELEASE_URL = "https://api.github.com/repos/demiurgeQuantified/ZomboidDecompiler/releases/latest"
USER_AGENT = "layline-decomp-setup"


def fail(message: str) -> None:
    sys.exit(f"error: {message}")


def java_executable() -> str:
    java = shutil.which("java")
    if java is None:
        fail("Java 17 or newer is required; install a JDK and make `java` available on PATH")

    try:
        result = subprocess.run([java, "-version"], capture_output=True, text=True, check=False)
    except OSError as error:
        fail(f"could not run Java: {error}")
    if result.returncode != 0:
        fail(f"`{java} -version` failed:\n{result.stderr.strip()}")

    version_text = f"{result.stdout}\n{result.stderr}"
    match = re.search(r'\bversion\s+"([^"]+)"', version_text)
    if match is None:
        match = re.search(r"\b(?:openjdk|java)\s+(\d+(?:\.\d+)*)", version_text, re.IGNORECASE)
    if match is None:
        fail("could not determine the Java version from `java -version`")

    version = match.group(1)
    numbers = [int(part) for part in version.split(".")]
    major = numbers[1] if numbers[0] == 1 and len(numbers) > 1 else numbers[0]
    if major < 17:
        fail(f"Java 17 or newer is required; found Java {version}")
    return java


def fetch_release() -> dict:
    request = Request(RELEASE_URL, headers={"Accept": "application/vnd.github+json", "User-Agent": USER_AGENT})
    try:
        with urlopen(request, timeout=30) as response:
            return json.load(response)
    except (URLError, TimeoutError, json.JSONDecodeError) as error:
        fail(f"could not retrieve the latest ZomboidDecompiler release: {error}")


def release_zip(release: dict) -> dict:
    assets = release.get("assets", [])
    exact_match = next((asset for asset in assets if asset.get("name") == "ZomboidDecompiler.zip"), None)
    if exact_match is not None:
        return exact_match

    zip_assets = [asset for asset in assets if asset.get("name", "").lower().endswith(".zip")]
    if len(zip_assets) == 1:
        return zip_assets[0]
    fail("the latest ZomboidDecompiler release does not contain one unambiguous ZIP asset")


def extract_archive(archive: Path, destination: Path) -> None:
    with zipfile.ZipFile(archive) as zip_file:
        root = destination.resolve()
        for member in zip_file.infolist():
            path = (destination / member.filename).resolve()
            if not path.is_relative_to(root):
                fail(f"unsafe path in decompiler archive: {member.filename}")
        zip_file.extractall(destination)


def launcher_path(extraction: Path) -> Path:
    names = ("ZomboidDecompiler.bat",) if os.name == "nt" else ("ZomboidDecompiler", "ZomboidDecompiler.sh")
    for name in names:
        launcher = next((path for path in extraction.rglob(name) if path.is_file()), None)
        if launcher is not None:
            return launcher
    fail("could not find the ZomboidDecompiler launcher in the downloaded release")


def run_decompiler(launcher: Path, game_dir: Path | None) -> Path:
    if os.name == "nt":
        command = ["cmd", "/c", str(launcher)]
    elif os.access(launcher, os.X_OK):
        command = [str(launcher)]
    else:
        command = ["sh", str(launcher)]
    if game_dir is not None:
        command.append(str(game_dir))

    print(f"Running {launcher.name}; decompilation may take a few minutes...")
    try:
        result = subprocess.run(command, cwd=launcher.parent, check=False)
    except OSError as error:
        fail(f"could not start ZomboidDecompiler: {error}")
    if result.returncode != 0:
        fail(f"ZomboidDecompiler exited with status {result.returncode}")

    output = launcher.parent / "output" / "source" / "zombie"
    if not (output / "network" / "PacketTypes.java").is_file():
        fail(f"decompiler output is missing expected file: {output / 'network' / 'PacketTypes.java'}")
    return output


def download_asset(asset: dict, archive: Path) -> None:
    request = Request(asset["browser_download_url"], headers={"User-Agent": USER_AGENT})
    try:
        with urlopen(request, timeout=60) as response, archive.open("wb") as downloaded:
            shutil.copyfileobj(response, downloaded)
    except (URLError, TimeoutError, OSError, KeyError) as error:
        fail(f"could not download the ZomboidDecompiler release: {error}")

    digest = asset.get("digest")
    if digest and digest.startswith("sha256:"):
        with archive.open("rb") as downloaded:
            actual = hashlib.file_digest(downloaded, "sha256").hexdigest()
        if actual != digest.removeprefix("sha256:"):
            fail("the downloaded ZomboidDecompiler ZIP failed its SHA-256 check")
    if not zipfile.is_zipfile(archive):
        fail("the downloaded ZomboidDecompiler asset is not a valid ZIP file")


def install_output(output: Path, destination: Path, temporary: Path) -> int:
    if destination.exists() and not destination.is_dir():
        fail(f"decomp output path exists but is not a directory: {destination}")

    java_files = sum(1 for path in output.rglob("*.java") if path.is_file())
    if java_files == 0:
        fail("ZomboidDecompiler completed without producing any Java source files")

    backup = temporary / "previous-decomp"
    had_previous = destination.exists()
    if had_previous:
        os.replace(destination, backup)
    try:
        os.replace(output, destination)
    except OSError:
        if had_previous:
            os.replace(backup, destination)
        raise
    return java_files


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--game-dir",
        type=Path,
        help="Project Zomboid installation directory (normally detected automatically)",
    )
    args = parser.parse_args()

    java_executable()
    if args.game_dir is not None:
        args.game_dir = args.game_dir.expanduser().resolve()
        if not args.game_dir.is_dir():
            parser.error(f"--game-dir is not a directory: {args.game_dir}")

    repo_root = Path(__file__).resolve().parents[1]
    destination = repo_root / "decomp"
    with tempfile.TemporaryDirectory(prefix=".zomboid-decompiler-", dir=repo_root) as temp_name:
        temporary = Path(temp_name)
        release = fetch_release()
        asset = release_zip(release)
        archive = temporary / asset["name"]
        download_asset(asset, archive)

        extraction = temporary / "release"
        extraction.mkdir()
        extract_archive(archive, extraction)
        output = run_decompiler(launcher_path(extraction), args.game_dir)
        java_files = install_output(output, destination, temporary)

    print(f"Updated {destination.relative_to(repo_root)} with {java_files} Java files.")


if __name__ == "__main__":
    main()
