#!/usr/bin/env python3
"""Deterministic Stellaryn release archive builder and smoke verifier.

Stdlib-only, run on each target OS. No GitHub release is published.
Version comes from the workspace Cargo.toml; target comes from rustc -vV.
"""
from __future__ import annotations

import argparse
import gzip
import hashlib
import io
import json
import os
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FILES = {
    "LICENSE": ROOT / "LICENSE",
    "README.md": ROOT / "README.md",
    "docs/getting-started.md": ROOT / "docs/getting-started.md",
    "docs/limitations.md": ROOT / "docs/limitations.md",
    "docs/cli-reports.md": ROOT / "docs/cli-reports.md",
}
FIXTURE_SIGNED = ROOT / "tests/fixtures/real/compiled_add_i128.wasm"
FIXTURE_UNSIGNED = ROOT / "tests/fixtures/real/compiled_add_u128.wasm"
MAX_EXTRACTED_BYTES = 32 * 1024 * 1024


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def workspace_version() -> str:
    with (ROOT / "Cargo.toml").open("rb") as handle:
        return tomllib.load(handle)["workspace"]["package"]["version"]


def rust_target() -> str:
    completed = subprocess.run(
        ["rustc", "-vV"], capture_output=True, text=True, check=True, timeout=20
    )
    for line in completed.stdout.splitlines():
        if line.startswith("host: "):
            return line.removeprefix("host: ").strip()
    raise ValueError("rustc did not report its host target")


def fixed_files(binary: Path, prefix: str, target: str) -> list[tuple[str, bytes, int]]:
    name = "stellaryn.exe" if "windows" in target else "stellaryn"
    if not binary.is_file():
        raise FileNotFoundError(f"missing release executable: {binary}")
    members = [(f"{prefix}/{name}", binary.read_bytes(), 0o755)]
    for relative, path in FILES.items():
        members.append((f"{prefix}/{relative}", path.read_bytes(), 0o644))
    return members


def make_zip(members: list[tuple[str, bytes, int]], output: Path) -> None:
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as arc:
        for name, data, mode in sorted(members):
            entry = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            entry.create_system = 3
            entry.external_attr = ((0o100000 | mode) << 16)
            entry.compress_type = zipfile.ZIP_DEFLATED
            arc.writestr(entry, data, compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)


def make_tar(members: list[tuple[str, bytes, int]], output: Path) -> None:
    with output.open("wb") as target:
        with gzip.GzipFile(fileobj=target, mode="wb", filename="", mtime=0, compresslevel=9) as gz:
            with tarfile.open(fileobj=gz, mode="w", format=tarfile.USTAR_FORMAT) as arc:
                for name, data, mode in sorted(members):
                    entry = tarfile.TarInfo(name=name)
                    entry.size = len(data)
                    entry.mode = mode
                    entry.mtime = 0
                    entry.uid = entry.gid = 0
                    entry.uname = entry.gname = ""
                    arc.addfile(entry, io.BytesIO(data))


def build(binary: Path, destination: Path, target: str) -> Path:
    version = workspace_version()
    prefix = f"stellaryn-{version}-{target}"
    destination.mkdir(parents=True, exist_ok=True)
    extension = ".zip" if "windows" in target else ".tar.gz"
    archive = destination / f"{prefix}{extension}"
    members = fixed_files(binary, prefix, target)
    if extension == ".zip":
        make_zip(members, archive)
    else:
        make_tar(members, archive)
    archive_digest = sha256(archive.read_bytes())
    (destination / f"{archive.name}.sha256").write_text(
        f"{archive_digest}  {archive.name}\n", encoding="utf-8", newline="\n"
    )
    manifest = {
        "schema_version": 1,
        "package": "stellaryn",
        "version": version,
        "target": target,
        "archive": archive.name,
        "sha256": archive_digest,
        "binary_sha256": sha256(binary.read_bytes()),
        "files": [name.removeprefix(prefix + "/") for name, _, _ in sorted(members)],
        "status": "pre-release CI artifact; not a signed or notarized release",
    }
    (destination / f"{archive.name}.manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n",
        encoding="utf-8", newline="\n",
    )
    print(f"Built {archive} (sha256 {archive_digest})")
    return archive


def extract_expected(archive: Path, folder: Path, prefix: str, members: set[str]) -> None:
    """Extract only exact expected member names; reject unexpected archives."""
    if archive.suffix == ".zip":
        with zipfile.ZipFile(archive) as arc:
            if len(arc.namelist()) != len(set(arc.namelist())):
                raise ValueError("duplicate members in archive")
            if set(arc.namelist()) != members:
                raise ValueError("archive contents do not match manifest")
            for member in arc.infolist():
                if member.is_dir() or member.file_size > MAX_EXTRACTED_BYTES:
                    raise ValueError("unexpected directory or oversized archive member")
                target = folder / member.filename
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(arc.read(member))
                if member.filename.endswith("/stellaryn"):
                    target.chmod(0o755)
    else:
        with tarfile.open(archive, "r:gz") as arc:
            names = arc.getnames()
            if len(names) != len(set(names)) or set(names) != members:
                raise ValueError("archive contents do not match manifest")
            for member in arc.getmembers():
                if not member.isfile() or member.size > MAX_EXTRACTED_BYTES:
                    raise ValueError("unexpected member type or oversized archive member")
                input_file = arc.extractfile(member)
                if input_file is None:
                    raise ValueError("missing archive member data")
                data = input_file.read(MAX_EXTRACTED_BYTES + 1)
                if len(data) != member.size:
                    raise ValueError("archive member truncated")
                target = folder / member.name
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(data)
                target.chmod(member.mode)


def run_cli(command: list[str], expected_exit: int) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        command, capture_output=True, text=True, timeout=60, check=False
    )
    if result.returncode != expected_exit:
        raise ValueError(
            f"CLI exited {result.returncode}, expected {expected_exit}: "
            f"stderr={result.stderr[:600]} stdout={result.stdout[:300]}"
        )
    return result


def verify(archive: Path) -> None:
    manifest_file = archive.with_name(archive.name + ".manifest.json")
    checksum_file = archive.with_name(archive.name + ".sha256")
    manifest = json.loads(manifest_file.read_text(encoding="utf-8"))
    version = workspace_version()
    target = rust_target()
    prefix = f"stellaryn-{version}-{target}"
    if (
        manifest["schema_version"] != 1
        or manifest["version"] != version
        or manifest["target"] != target
        or manifest["archive"] != archive.name
        or not archive.name.startswith(prefix)
    ):
        raise ValueError("package version/target/filename mismatch")
    digest = sha256(archive.read_bytes())
    if digest != manifest["sha256"]:
        raise ValueError("archive checksum mismatches manifest")
    if checksum_file.read_text(encoding="utf-8").strip() != f"{digest}  {archive.name}":
        raise ValueError("archive checksum file mismatches manifest")
    members = {f"{prefix}/{name}" for name in manifest["files"]}
    expected = {"LICENSE", "README.md", "docs/getting-started.md", "docs/limitations.md", "docs/cli-reports.md"}
    binary_name = "stellaryn.exe" if "windows" in target else "stellaryn"
    expected.add(binary_name)
    if set(manifest["files"]) != expected:
        raise ValueError("unexpected packaged files")
    with tempfile.TemporaryDirectory(prefix="stellaryn-release-") as tmp:
        root = Path(tmp)
        extract_expected(archive, root, prefix, members)
        binary = root / prefix / binary_name
        if sha256(binary.read_bytes()) != manifest["binary_sha256"]:
            raise ValueError("binary bytes mismatch manifest")
        version_output = run_cli([str(binary), "--version"], 0)
        if version not in version_output.stdout:
            raise ValueError("packaged binary reports wrong version")
        good = run_cli(
            [str(binary), "compare", str(FIXTURE_SIGNED), str(FIXTURE_SIGNED),
             "--format", "json"], 0,
        )
        good_report = json.loads(good.stdout)
        if good_report["analysis"]["verdict"] != "COMPATIBLE":
            raise ValueError("self-comparison did not yield COMPATIBLE")
        bad = run_cli(
            [str(binary), "compare", str(FIXTURE_SIGNED), str(FIXTURE_UNSIGNED),
             "--format", "json"], 2,
        )
        bad_report = json.loads(bad.stdout)
        if (
            bad_report["analysis"]["verdict"] != "INCOMPATIBLE"
            or bad_report["analysis"]["totals"]["breaking"] != 3
        ):
            raise ValueError("packaged binary missed known breaking changes")
    print(f"Verified {archive.name}: sha256, manifest, executable, and real WASM smoke cases")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    make = sub.add_parser("build")
    make.add_argument("--binary", type=Path, required=True)
    make.add_argument("--out", type=Path, default=Path("dist"))
    make.add_argument("--target", default=None)
    check = sub.add_parser("verify")
    check.add_argument("--archive", type=Path)
    check.add_argument("--out", type=Path, default=Path("dist"))
    args = parser.parse_args()
    try:
        if args.command == "build":
            build(args.binary, args.out, args.target or rust_target())
        else:
            if args.archive is not None:
                verify(args.archive)
            else:
                matches = sorted(args.out.glob("stellaryn-*.tar.gz")) + sorted(args.out.glob("stellaryn-*.zip"))
                if len(matches) != 1:
                    raise ValueError(f"expected exactly one release archive in {args.out}, got {len(matches)}")
                verify(matches[0])
    except (OSError, ValueError, KeyError, subprocess.SubprocessError, json.JSONDecodeError) as error:
        print(f"release package error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
