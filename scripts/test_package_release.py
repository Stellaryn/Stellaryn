"""Release-packaging regressions: determinism, checksums, and archive safety."""

import json
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest.mock import patch

import package_release


class ReleasePackageTests(unittest.TestCase):
    def test_deterministic_linux_archives_and_manifest(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            binary = root / "stellaryn"
            binary.write_bytes(b"test-release-binary")
            target = "x86_64-unknown-linux-gnu"
            first = package_release.build(binary, root / "a", target)
            second = package_release.build(binary, root / "b", target)
            self.assertEqual(first.read_bytes(), second.read_bytes())
            self.assertEqual(first.with_name(first.name + ".sha256").read_bytes(),
                             second.with_name(second.name + ".sha256").read_bytes())
            self.assertEqual(first.with_name(first.name + ".manifest.json").read_bytes(),
                             second.with_name(second.name + ".manifest.json").read_bytes())
            manifest = json.loads(first.with_name(first.name + ".manifest.json").read_text())
            self.assertEqual(manifest["binary_sha256"], package_release.sha256(binary.read_bytes()))
            self.assertEqual(len(manifest["files"]), 6)
            with tarfile.open(first, "r:gz") as archive:
                self.assertEqual(len(archive.getmembers()), 6)
                binary_item = next(item for item in archive.getmembers()
                                   if item.name.endswith("/stellaryn"))
                self.assertEqual(binary_item.mode, 0o755)
                self.assertEqual(binary_item.mtime, 0)

    def test_deterministic_windows_zip_uses_exe_even_on_unix_host(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            binary = root / "stellaryn.exe"
            binary.write_bytes(b"test-windows-binary")
            target = "x86_64-pc-windows-msvc"
            first = package_release.build(binary, root / "a", target)
            second = package_release.build(binary, root / "b", target)
            self.assertEqual(first.read_bytes(), second.read_bytes())
            with zipfile.ZipFile(first) as archive:
                self.assertEqual(len(archive.namelist()), 6)
                self.assertTrue(any(name.endswith("/stellaryn.exe") for name in archive.namelist()))
                for info in archive.infolist():
                    self.assertEqual(info.date_time, (1980, 1, 1, 0, 0, 0))

    def test_corrupt_archive_fails_checksum_before_executable_runs(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            binary = root / "stellaryn"
            binary.write_bytes(b"test-binary")
            archive = package_release.build(binary, root / "dist", "x86_64-unknown-linux-gnu")
            archive.write_bytes(archive.read_bytes() + b"tampered")
            with patch.object(package_release, "rust_target", return_value="x86_64-unknown-linux-gnu"):
                with self.assertRaisesRegex(ValueError, "checksum"):
                    package_release.verify(archive)

    def test_extraction_rejects_unexpected_archive_members(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            binary = root / "stellaryn"
            binary.write_bytes(b"test-binary")
            archive = package_release.build(binary, root / "dist", "x86_64-unknown-linux-gnu")
            with self.assertRaisesRegex(ValueError, "contents"):
                package_release.extract_expected(
                    archive, root / "extract", "stellaryn-test",
                    {"stellaryn-test/stellaryn"},
                )


if __name__ == "__main__":
    unittest.main()
