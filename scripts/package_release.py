#!/usr/bin/env python3
"""Create and verify reproducible, native-platform Stellaryn release-candidate archives.

Archives are untrusted until verification succeeds. The command works with
a locally built `target/release/stellaryn[.exe]`; it never compiles or
downloads a binary itself.

Python 3.11+ required (tomllib).
"""

from __future__ import annotations

import argparse
import gzip
import hashlib
import io
import json
import os
import stat
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FILES = ("README.md", "LICENSE", "docs/limitations.md")
MAX_UNPACKED_SIZE = 100 * 1024 * 1024


def project_version(root: Path = ROOT) -> str:
    with (root / "Cargo.toml").open("rb") as stream:
        return tomllib.load(stream)["workspace"]["package"]["version"]


def host_target() -> str:
    output = subprocess.check_output(["rustc", "-vV"], text=True)
    for line in output.splitlines():
        if line.startswith("host: "):
            return line.partition(": ")[2]
    raise ValueError("rustc -vV did not report a host target")


def binary_name(target: str) -> str:
    return "stellaryn.exe" if "-windows-" in target else "stellaryn"


def release_basename(version: str, target: str) -> str:
    for value in (version, target):
        if not value or not all(ch.isascii() and (ch.isalnum() or ch in ".-_") for ch in value):
            raise ValueError("version and target must contain only ASCII letters, digits, dots, dashes and underscores")
    return f"stellaryn-v{version}-{target}"


def sha256(content: bytes) -> str:
    return hashlib.sha256(content).hexdigest()


def bundle_files(root: Path, target: str) -> dict[str, tuple[bytes, int]]:
    binary = root / "target" / "release" / binary_name(target)
    if not binary.is_file():
        raise FileNotFoundError(f"compiled native binary missing: {binary}")
    entries: dict[str, tuple[bytes, int]] = {
        binary_name(target): (binary.read_bytes(), 0o755)
    }
    if not entries[binary_name(target)][0]:
        raise ValueError("compiled binary is empty")
    for relative in FILES:
        entries[relative] = ((root / relative).read_bytes(), 0o644)
    return entries


def make_zip(entries: dict[str, tuple[bytes, int]], prefix: str) -> bytes:
    output = io.BytesIO()
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as bundle:
        for name, (content, mode) in sorted(entries.items()):
            item = zipfile.ZipInfo(f"{prefix}/{name}", (1980, 1, 1, 0, 0, 0))
            item.compress_type = zipfile.ZIP_DEFLATED
            item.create_system = 3
            item.external_attr = (stat.S_IFREG | mode) << 16
            bundle.writestr(item, content, compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)
    return output.getvalue()


def make_tar_gz(entries: dict[str, tuple[bytes, int]], prefix: str) -> bytes:
    output = io.BytesIO()
    with gzip.GzipFile(fileobj=output, mode="wb", filename="", mtime=0, compresslevel=9) as zipped:
        with tarfile.open(fileobj=zipped, mode="w", format=tarfile.USTAR_FORMAT) as bundle:
            for name, (content, mode) in sorted(entries.items()):
                info = tarfile.TarInfo(f"{prefix}/{name}")
                info.size = len(content)
                info.mode = mode
                info.uid = 0
                info.gid = 0
                info.uname = ""
                info.gname = ""
                info.mtime = 0
                bundle.addfile(info, io.BytesIO(content))
    return output.getvalue()


def archive_name(version: str, target: str) -> str:
    return release_basename(version, target) + (".zip" if "-windows-" in target else ".tar.gz")


def create_bundle(root: Path, dist: Path, version: str, target: str) -> Path:
    entries = bundle_files(root, target)
    base = release_basename(version, target)
    content = make_zip(entries, base) if "-windows-" in target else make_tar_gz(entries, base)
    dist.mkdir(parents=True, exist_ok=True)
    archive = dist / archive_name(version, target)
    archive.write_bytes(content)
    archive.with_name(archive.name + ".sha256").write_text(
        f"{sha256(content)}  {archive.name}\n", encoding="ascii", newline="\n"
    )
    print(f"PACKAGED {archive.name} ({len(content)} bytes, sha256 {sha256(content)})")
    return archive


def read_verified_entries(archive: Path, version: str, target: str) -> dict[str, bytes]:
    """Read a trusted-by-checksum bundle safely, without filesystem extraction."""
    prefix = release_basename(version, target) + "/"
    files: dict[str, bytes] = {}
    total = 0

    if archive.suffix == ".zip":
        with zipfile.ZipFile(archive) as source:
            for member in source.infolist():
                if member.is_dir():
                    continue
                name = member.filename
                if not name.startswith(prefix) or name in files or name.endswith("/"):
                    raise ValueError(f"unexpected or duplicate bundle path: {name}")
                relative = name[len(prefix):]
                if relative not in set(FILES) | {binary_name(target)}:
                    raise ValueError(f"unrecognized bundle member: {relative}")
                total += member.file_size
                if total > MAX_UNPACKED_SIZE:
                    raise ValueError("release bundle exceeds extracted-size limit")
                files[relative] = source.read(member)
    else:
        with tarfile.open(archive, "r:gz") as source:
            for member in source.getmembers():
                if not member.isfile():
                    raise ValueError(f"non-regular bundle member: {member.name}")
                name = member.name
                if not name.startswith(prefix) or name in files:
                    raise ValueError(f"unexpected or duplicate bundle path: {name}")
                relative = name[len(prefix):]
                if relative not in set(FILES) | {binary_name(target)}:
                    raise ValueError(f"unrecognized bundle member: {relative}")
                total += member.size
                if total > MAX_UNPACKED_SIZE:
                    raise ValueError("release bundle exceeds extracted-size limit")
                opened = source.extractfile(member)
                if opened is None:
                    raise ValueError(f"could not read bundle member: {name}")
                files[relative] = opened.read()

    expected = set(FILES) | {binary_name(target)}
    if files.keys() != expected:
        raise ValueError(f"bundle members mismatch: {sorted(files)} != {sorted(expected)}")
    return files


def verify_bundle(root: Path, dist: Path, version: str, target: str) -> None:
    name = archive_name(version, target)
    archive = dist / name
    content = archive.read_bytes()
    checksum_text = (dist / f"{name}.sha256").read_text(encoding="ascii")
    expected_line = f"{sha256(content)}  {name}\n"
    if checksum_text != expected_line:
        raise ValueError(f"SHA-256 checksum mismatch: {name}")
    files = read_verified_entries(archive, version, target)
    compiled = (root / "target" / "release" / binary_name(target)).read_bytes()
    if sha256(files[binary_name(target)]) != sha256(compiled):
        raise ValueError("packaged binary is not identical to the native release build")

    # Execute the *packaged* executable, not the unarchived target/release binary.
    with tempfile.TemporaryDirectory() as temp:
        target_binary = Path(temp) / binary_name(target)
        target_binary.write_bytes(files[binary_name(target)])
        if os.name != "nt":
            target_binary.chmod(0o755)
        command = [str(target_binary)]
        output = subprocess.run(command + ["--version"], capture_output=True, text=True, timeout=25)
        if output.returncode or version not in output.stdout:
            raise RuntimeError(f"packaged --version failed: {output.stdout} {output.stderr}")
        help_output = subprocess.run(command + ["--help"], capture_output=True, text=True, timeout=25)
        if help_output.returncode or "compare" not in help_output.stdout or "git" not in help_output.stdout:
            raise RuntimeError(f"packaged --help failed: {help_output.stderr}")

        original = root / "tests/fixtures/real/compiled_add_i128.wasm"
        changed = root / "tests/fixtures/real/compiled_add_u128.wasm"
        args = ["compare", str(original), str(changed), "--format", "json"]
        report = subprocess.run(command + args + ["--fail-on", "never"], capture_output=True, text=True, timeout=25)
        if report.returncode != 0 or report.stderr:
            raise RuntimeError(f"packaged JSON smoke failed: {report.stderr}")
        findings = json.loads(report.stdout)["analysis"]
        expected_ids = [
            "FUNCTION_PARAMETER_TYPE_CHANGED",
            "FUNCTION_PARAMETER_TYPE_CHANGED",
            "FUNCTION_OUTPUT_TYPE_CHANGED",
        ]
        ids = [finding["rule"]["id"] for finding in findings["findings"]]
        if findings["verdict"] != "INCOMPATIBLE" or sorted(ids) != sorted(expected_ids):
            raise ValueError("packaged binary missed known real-WASM breaking ABI findings")
        denied = subprocess.run(command + args, capture_output=True, text=True, timeout=25)
        if denied.returncode != 2 or denied.stderr:
            raise ValueError("packaged binary did not enforce default breaking exit 2")
        same = subprocess.run(
            command + ["compare", str(original), str(original), "--format", "json"],
            capture_output=True, text=True, timeout=25
        )
        if same.returncode != 0 or json.loads(same.stdout)["analysis"]["verdict"] != "COMPATIBLE":
            raise ValueError("packaged binary failed valid same-artifact comparison")
    print(f"VERIFIED {name}: checksum, members, executable, help, JSON, breaking + self-diff")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("build", "verify"))
    parser.add_argument("--dist", default="dist", type=Path)
    parser.add_argument("--target", default=None, help="Native Rust triple (derived from rustc by default)")
    args = parser.parse_args()
    version = project_version()
    target = args.target or host_target()
    try:
        if args.command == "build":
            create_bundle(ROOT, args.dist, version, target)
        else:
            verify_bundle(ROOT, args.dist, version, target)
    except (ValueError, OSError, RuntimeError, KeyError, zipfile.BadZipFile, tarfile.TarError) as error:
        print(f"release package check failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
