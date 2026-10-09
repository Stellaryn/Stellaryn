#!/usr/bin/env python3
"""Fail-closed integrity gate for consolidated official v0.1.0 release assets."""

from __future__ import annotations

import hashlib
import sys
from pathlib import Path

ARCHIVES = (
    "stellaryn-v0.1.0-x86_64-unknown-linux-gnu.tar.gz",
    "stellaryn-v0.1.0-x86_64-pc-windows-msvc.zip",
    "stellaryn-v0.1.0-aarch64-apple-darwin.tar.gz",
)
MAX_ARCHIVE_BYTES = 200 * 1024 * 1024


def verify_release_assets(dist: Path) -> None:
    """Require exactly three target archives plus their exact SHA-256 sidecars."""
    if not dist.is_dir() or dist.is_symlink():
        raise ValueError("missing or unsafe release asset directory")
    expected = set(ARCHIVES) | {name + ".sha256" for name in ARCHIVES}
    entries = list(dist.iterdir())
    if {entry.name for entry in entries} != expected:
        raise ValueError(
            f"release asset set mismatch: expected {sorted(expected)}, "
            f"found {sorted(entry.name for entry in entries)}"
        )
    for entry in entries:
        if entry.is_symlink() or not entry.is_file():
            raise ValueError(f"unsafe or non-regular release asset: {entry.name}")
    for name in ARCHIVES:
        archive = dist / name
        if archive.stat().st_size == 0 or archive.stat().st_size > MAX_ARCHIVE_BYTES:
            raise ValueError(f"invalid release asset size: {name}")
        actual = hashlib.sha256(archive.read_bytes()).hexdigest()
        expected_line = f"{actual}  {name}\n"
        try:
            line = (dist / (name + ".sha256")).read_text(encoding="ascii")
        except UnicodeError as error:
            raise ValueError(f"invalid ASCII checksum: {name}") from error
        if line != expected_line:
            raise ValueError(f"SHA-256 mismatch or invalid sidecar: {name}")
        print(f"VERIFIED RELEASE ASSET {name}: sha256 {actual}")


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: verify_release_assets.py DIST_DIR", file=sys.stderr)
        return 1
    try:
        verify_release_assets(Path(sys.argv[1]))
    except (ValueError, OSError) as error:
        print(f"release asset check failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
