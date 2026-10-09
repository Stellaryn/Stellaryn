#!/usr/bin/env python3
"""Offline documentation integrity gate for Stellaryn.

Checks local Markdown links, GitBook navigation and required contributor
entry-points without network access or extra Python dependencies.
Remote URLs are not asserted live.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]
LINK_PATTERN = re.compile(r"(?<!!)\[[^\]\n]*\]\((<[^>]+>|[^\s)]+)(?:\s+[\"'][^\"']+[\"'])?\)")
FENCE_PATTERN = re.compile(r"^\s*(`{3,}|~{3,})")
REQUIRED = (
    "README.md",
    "CONTRIBUTING.md",
    "CODE_OF_CONDUCT.md",
    "SECURITY.md",
    ".gitbook.yaml",
    ".github/pull_request_template.md",
    ".github/ISSUE_TEMPLATE/bug_report.yml",
    ".github/ISSUE_TEMPLATE/compatibility.yml",
    ".github/ISSUE_TEMPLATE/feature_request.yml",
    "docs/README.md",
    "docs/SUMMARY.md",
    "docs/getting-started.md",
    "docs/examples.md",
    "docs/ci-integration.md",
    "docs/troubleshooting.md",
    "docs/limitations.md",
    "docs/rule-authoring.md",
)


def linked_targets(markdown: str) -> list[tuple[int, str]]:
    """Extract Markdown inline links, ignoring fenced examples."""
    targets = []
    fence = None
    for number, line in enumerate(markdown.splitlines(), 1):
        match = FENCE_PATTERN.match(line)
        if match:
            marker = match.group(1)
            if fence is None:
                fence = marker
            elif marker[0] == fence[0] and len(marker) >= len(fence):
                fence = None
            continue
        if fence is not None:
            continue
        for link in LINK_PATTERN.finditer(line):
            raw = link.group(1).strip("<>")
            targets.append((number, raw))
    return targets


def local_target(source: Path, url: str, root: Path) -> Path | None:
    """Resolve a relative Markdown link without accepting links outside root."""
    parts = urlsplit(url)
    if parts.scheme or parts.netloc or url.startswith("#") or url.startswith("//"):
        return None
    if url.startswith("/"):
        return root / "__invalid_root_relative_url__"
    path = unquote(parts.path)
    if not path:
        return None
    return (source.parent / path).resolve()


def check_document(source: Path, root: Path) -> list[str]:
    errors = []
    text = source.read_text(encoding="utf-8")
    for number, url in linked_targets(text):
        target = local_target(source, url, root)
        if target is None:
            continue
        if not target.is_relative_to(root):
            errors.append(f"{source.relative_to(root)}:{number}: local link escapes repository: {url}")
        elif not target.exists():
            errors.append(f"{source.relative_to(root)}:{number}: missing local target: {url}")
    return errors


def check_navigation(root: Path) -> list[str]:
    errors = []
    nav = root / "docs/SUMMARY.md"
    if not nav.exists():
        return ["missing docs/SUMMARY.md"]
    pages = [urlsplit(url).path for _, url in linked_targets(nav.read_text(encoding="utf-8"))]
    if not pages or pages[0] != "README.md":
        errors.append("docs/SUMMARY.md: first link must be README.md")
    if len(pages) != len(set(pages)):
        errors.append("docs/SUMMARY.md: duplicate page references")
    expected = {
        "getting-started.md",
        "examples.md",
        "ci-integration.md",
        "troubleshooting.md",
        "limitations.md",
        "rule-authoring.md",
    }
    for missing in sorted(expected - set(pages)):
        errors.append(f"docs/SUMMARY.md: missing entry for {missing}")
    config = (root / ".gitbook.yaml").read_text(encoding="utf-8") if (root / ".gitbook.yaml").exists() else ""
    for line in ["root: ./docs/", "readme: README.md", "summary: SUMMARY.md"]:
        if line not in config:
            errors.append(f".gitbook.yaml: missing setting {line}")
    return errors


def check_readme_milestones(root: Path) -> list[str]:
    """Keep product-facing README free of internal numbered project phases."""
    readme = root / "README.md"
    if readme.is_file() and re.search(r"\bphase\s+\d+\b", readme.read_text(encoding="utf-8"), re.IGNORECASE):
        return ["README.md: remove numbered development phase references; use docs/build-plan.md"]
    return []


def check_repository(root: Path) -> list[str]:
    root = root.resolve()
    errors = []
    for expected in REQUIRED:
        if not (root / expected).is_file():
            errors.append(f"missing required file: {expected}")
    for doc in sorted(root.rglob("*.md")):
        if ".git" not in doc.parts and "target" not in doc.parts:
            errors.extend(check_document(doc, root))
    errors.extend(check_navigation(root))
    errors.extend(check_readme_milestones(root))
    return errors


def main() -> int:
    issues = check_repository(ROOT)
    if issues:
        for error in issues:
            print(f"docs check: {error}", file=sys.stderr)
        print(f"Documentation checks failed: {len(issues)} issue(s)", file=sys.stderr)
        return 1
    docs = sum(1 for _ in ROOT.rglob("*.md"))
    print(f"Documentation checks passed ({docs} Markdown files; local links and GitBook navigation).")
    print("External links are not checked for availability by this offline gate.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
