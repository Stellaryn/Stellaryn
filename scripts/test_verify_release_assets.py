"""Negative and positive tests for official release asset collation."""

import tempfile
import unittest
from pathlib import Path

import verify_release_assets


class ReleaseAssetsTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.dist = Path(self.temp.name)
        for name in verify_release_assets.ARCHIVES:
            content = ("binary-for-" + name).encode("ascii")
            import hashlib
            (self.dist / name).write_bytes(content)
            (self.dist / (name + ".sha256")).write_text(
                f"{hashlib.sha256(content).hexdigest()}  {name}\n", encoding="ascii"
            )

    def test_accepts_exact_three_verified_native_assets(self):
        verify_release_assets.verify_release_assets(self.dist)

    def test_missing_archive_fails_closed(self):
        (self.dist / verify_release_assets.ARCHIVES[0]).unlink()
        with self.assertRaisesRegex(ValueError, "asset set mismatch"):
            verify_release_assets.verify_release_assets(self.dist)

    def test_unknown_file_fails_closed(self):
        (self.dist / "debug-symbols.txt").write_text("extra", encoding="ascii")
        with self.assertRaisesRegex(ValueError, "asset set mismatch"):
            verify_release_assets.verify_release_assets(self.dist)

    def test_tampered_archive_fails_closed(self):
        (self.dist / verify_release_assets.ARCHIVES[1]).write_bytes(b"tampered")
        with self.assertRaisesRegex(ValueError, "SHA-256 mismatch"):
            verify_release_assets.verify_release_assets(self.dist)

    def test_wrong_checksum_filename_fails_closed(self):
        path = self.dist / (verify_release_assets.ARCHIVES[2] + ".sha256")
        path.write_text("f" * 64 + "  unrelated.tar.gz\n", encoding="ascii")
        with self.assertRaisesRegex(ValueError, "SHA-256 mismatch"):
            verify_release_assets.verify_release_assets(self.dist)

    def test_empty_archive_fails_closed(self):
        (self.dist / verify_release_assets.ARCHIVES[0]).write_bytes(b"")
        with self.assertRaisesRegex(ValueError, "invalid release asset size"):
            verify_release_assets.verify_release_assets(self.dist)

    def test_symlink_archive_fails_closed(self):
        target = self.dist / verify_release_assets.ARCHIVES[1]
        target.unlink()
        try:
            target.symlink_to(self.dist / verify_release_assets.ARCHIVES[0])
        except OSError:
            self.skipTest("Symlink creation not supported by test runner")
        with self.assertRaisesRegex(ValueError, "unsafe"):
            verify_release_assets.verify_release_assets(self.dist)


if __name__ == "__main__":
    unittest.main()
