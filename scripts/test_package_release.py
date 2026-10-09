"""Tests for reproducible release-candidate archives; no Rust build required."""

import tempfile
import unittest
from pathlib import Path

import package_release


class PackagingTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "target/release").mkdir(parents=True)
        (self.root / "docs").mkdir(parents=True)
        (self.root / "README.md").write_text("# Stellaryn\n", encoding="utf-8")
        (self.root / "LICENSE").write_text("MIT test license\n", encoding="utf-8")
        (self.root / "docs/limitations.md").write_text("Spec comparison only\n", encoding="utf-8")
        self.dist = self.root / "dist"

    def test_tar_archive_is_deterministic_and_has_expected_members(self):
        (self.root / "target/release/stellaryn").write_bytes(b"native test program")
        triple = "x86_64-unknown-linux-gnu"
        a = package_release.create_bundle(self.root, self.dist, "0.1.0-alpha.1", triple)
        initial = a.read_bytes()
        package_release.create_bundle(self.root, self.dist, "0.1.0-alpha.1", triple)
        self.assertEqual(initial, a.read_bytes())
        names = package_release.read_verified_entries(a, "0.1.0-alpha.1", triple)
        self.assertEqual(names["stellaryn"], b"native test program")
        self.assertIn("docs/limitations.md", names)

    def test_windows_archive_has_exe_and_stable_checksum(self):
        (self.root / "target/release/stellaryn.exe").write_bytes(b"windows test exe")
        triple = "x86_64-pc-windows-msvc"
        archive = package_release.create_bundle(self.root, self.dist, "0.1.0-alpha.1", triple)
        self.assertEqual(archive.suffix, ".zip")
        self.assertEqual(package_release.read_verified_entries(archive, "0.1.0-alpha.1", triple)["stellaryn.exe"], b"windows test exe")
        self.assertEqual(archive.with_name(archive.name + ".sha256").read_text(encoding="ascii"), f"{package_release.sha256(archive.read_bytes())}  {archive.name}\n")

    def test_rejects_unsafe_version_and_target_strings(self):
        for unsafe in ["", "../escape", "x y", "foo/bar", "🪐"]:
            with self.assertRaises(ValueError):
                package_release.release_basename(unsafe, "x86_64-unknown-linux-gnu")

    def test_rejects_zip_member_paths_not_in_allowlist(self):
        (self.root / "target/release/stellaryn.exe").write_bytes(b"windows test exe")
        triple = "x86_64-pc-windows-msvc"
        archive = package_release.create_bundle(self.root, self.dist, "0.1.0-alpha.1", triple)
        # An independent zip with a traversal member cannot pass the member check.
        import zipfile
        with zipfile.ZipFile(archive, "w") as output:
            output.writestr(f"{package_release.release_basename('0.1.0-alpha.1', triple)}/../../evil.txt", b"not allowed")
        with self.assertRaises(ValueError):
            package_release.read_verified_entries(archive, "0.1.0-alpha.1", triple)


if __name__ == "__main__":
    unittest.main()
