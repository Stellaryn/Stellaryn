"""Regression tests for the docs checker itself."""

import tempfile
import unittest
from pathlib import Path

import check_docs


class DocsCheckerTests(unittest.TestCase):
    def test_extracts_real_links_and_ignores_fenced_markdown(self):
        source = "[Valid](docs/one.md)\n```md\n[Fake](missing.md)\n```\n[Real](two.md#anchor)"
        self.assertEqual(
            check_docs.linked_targets(source),
            [(1, "docs/one.md"), (5, "two.md#anchor")],
        )

    def test_accepts_local_paths_and_external_urls(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve()
            first = root / "README.md"
            second = root / "docs" / "guide.md"
            second.parent.mkdir()
            first.write_text("[Guide](docs/guide.md) [Remote](https://example.com)\n", encoding="utf-8")
            second.write_text("# Guide\n", encoding="utf-8")
            self.assertEqual(check_docs.check_document(first, root), [])

    def test_fails_closed_on_missing_and_parent_traversal_links(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve()
            doc = root / "README.md"
            doc.write_text("[Missing](missing.md) [Escape](../outside.md)\n", encoding="utf-8")
            issues = check_docs.check_document(doc, root)
            self.assertEqual(len(issues), 2)
            self.assertTrue(any("missing local target" in item for item in issues))
            self.assertTrue(any("escapes repository" in item for item in issues))

    def test_gitbook_navigation_requires_known_pages(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve()
            (root / "docs").mkdir()
            (root / "docs/SUMMARY.md").write_text(
                "# Summary\n* [Guide](not-readme.md)\n* [Guide](not-readme.md)\n",
                encoding="utf-8",
            )
            (root / ".gitbook.yaml").write_text("root: ./docs/\n", encoding="utf-8")
            issues = check_docs.check_navigation(root)
            self.assertTrue(any("duplicate" in item for item in issues))
            self.assertTrue(any("first link" in item for item in issues))
            self.assertTrue(any("missing entry" in item for item in issues))


if __name__ == "__main__":
    unittest.main()
