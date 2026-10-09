#!/usr/bin/env python3
"""Reproducible real-artifact compatibility smoke report.

This checks public contract-spec compatibility, not runtime safety. The two
cross-contract comparisons deliberately use different products, not verified
upgrade pairs. Full provenance: tests/fixtures/real/README.md.
"""

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "tests" / "fixtures" / "real"
COMMAND = ["cargo", "run", "--quiet", "-p", "stellaryn-cli", "--", "compare"]

ARTIFACTS = (
    "testnet_increment.wasm",
    "sdk_constructor.wasm",
    "mainnet_arb_bot.wasm",
    "mainnet_aqua_amm.wasm",
    "compiled_add_i128.wasm",
    "compiled_add_u128.wasm",
)

PAIRS = (
    ("independent testnet versus SDK constructor", "testnet_increment.wasm", "sdk_constructor.wasm"),
    ("different mainnet-dataset contracts", "mainnet_arb_bot.wasm", "mainnet_aqua_amm.wasm"),
    ("compiled add type variant i128-to-u128", "compiled_add_i128.wasm", "compiled_add_u128.wasm"),
)


def analyze(before: str, after: str) -> dict:
    result = subprocess.run(
        COMMAND
        + [
            str(FIXTURES / before),
            str(FIXTURES / after),
            "--format",
            "json",
            "--fail-on",
            "never",
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
        timeout=90,
    )
    if result.returncode:
        raise RuntimeError(
            f"{before} -> {after}: analysis failed with {result.returncode}: "
            f"{result.stderr[:500]}"
        )
    if result.stderr:
        raise RuntimeError(f"unexpected stderr for {before} -> {after}: {result.stderr[:500]}")
    report = json.loads(result.stdout)
    return report["analysis"]


def main() -> int:
    for name in ARTIFACTS:
        result = analyze(name, name)
        assert result["verdict"] == "COMPATIBLE", name
        assert result["totals"]["breaking"] == 0, name
        assert not result["findings"], name
        print(f"SELF  {name}: COMPATIBLE (identical compiled binary)")

    for label, old, new in PAIRS:
        result = analyze(old, new)
        assert result["verdict"] == "INCOMPATIBLE", label
        totals = result["totals"]
        assert totals["breaking"] > 0, label
        if old == "compiled_add_i128.wasm":
            assert any(
                entry["rule"]["id"] == "FUNCTION_PARAMETER_TYPE_CHANGED"
                for entry in result["findings"]
            ), "compiled type change was missed"
        total = sum(totals.values())
        assert total == len(result["findings"]), label
        print(
            f"CROSS {label}: {result['verdict']} "
            f"({totals['breaking']} breaking, {totals['review_required']} review, "
            f"{totals['non_breaking']} non-breaking)"
        )
        for finding in result["findings"][:8]:
            rule = finding["rule"]
            print(
                f"      {finding['classification']} "
                f"{rule['domain']}/{rule['id']} {finding['subject']}"
            )

    print(
        "SCOPE: Cross-comparisons involve different contracts; they are "
        "NOT validated versions of one contract or an upgrade-safety proof."
    )
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, KeyError, AssertionError, RuntimeError) as exc:
        print(f"real-world probe failed: {exc}", file=sys.stderr)
        sys.exit(1)
