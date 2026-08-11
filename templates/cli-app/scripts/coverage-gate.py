#!/usr/bin/env python3
"""Two-threshold line-coverage gate — see AGENTS.md §4.3.

    85%  line coverage over the whole project
    95%  line coverage on every file declared in `.security-sensitive`

`cargo llvm-cov --fail-under-lines` enforces a single project-wide number, which is why the
second floor needs a few lines of glue: this script asks cargo-llvm-cov for its JSON export and
applies both thresholds to it. Python only runs the gate — nothing in the build depends on it.

Usage:
    ./scripts/coverage-gate.py                 # collect coverage, then check both floors
    ./scripts/coverage-gate.py --json cov.json # re-check an existing llvm-cov JSON export
    ./scripts/coverage-gate.py --min-all 90    # raise a floor (lowering one is not a fix)

Exit status: 0 both floors met, 1 a floor was missed, 2 the gate could not run.
"""

from __future__ import annotations

import argparse
import fnmatch
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import NoReturn

MIN_ALL = 85.0
MIN_SECURITY = 95.0
MANIFEST = ".security-sensitive"


def project_root() -> Path:
    """The directory holding `Cargo.toml` — this script lives in `<root>/scripts/`."""
    return Path(__file__).resolve().parent.parent


def load_patterns(root: Path) -> list[str]:
    """Read the security-sensitive path patterns, skipping blanks and `#` comments."""
    manifest = root / MANIFEST
    if not manifest.is_file():
        return []
    patterns = []
    for line in manifest.read_text(encoding="utf-8").splitlines():
        line = line.split("#", 1)[0].strip()
        if line:
            patterns.append(line)
    return patterns


def collect(root: Path, destination: Path) -> None:
    """Run the test suite under instrumentation and write the JSON export."""
    if shutil.which("cargo-llvm-cov") is None:
        fail(
            "cargo-llvm-cov is not installed.\n"
            "    cargo install cargo-llvm-cov\n"
            "Or pass an existing export with --json."
        )
    subprocess.run(
        [
            "cargo",
            "llvm-cov",
            "--all-features",
            "--workspace",
            "--summary-only",
            "--json",
            "--output-path",
            str(destination),
        ],
        cwd=root,
        check=True,
    )


def read_report(path: Path, root: Path) -> tuple[float, list[tuple[str, int, int, float]]]:
    """Return the project-wide line percentage and a per-file `(path, covered, count, pct)` list.

    Paths are made relative to the project root; anything outside it (a dependency compiled from
    a local checkout, say) is not this project's code and is dropped.
    """
    export = json.loads(path.read_text(encoding="utf-8"))
    data = export["data"][0]
    files = []
    for entry in data["files"]:
        try:
            relative = Path(entry["filename"]).resolve().relative_to(root)
        except ValueError:
            continue
        lines = entry["summary"]["lines"]
        files.append(
            (relative.as_posix(), lines["covered"], lines["count"], float(lines["percent"]))
        )
    return float(data["totals"]["lines"]["percent"]), sorted(files)


def matches(relative: str, patterns: list[str]) -> bool:
    """`fnmatch` semantics, so `*` crosses directory separators: `src/auth*` covers the tree."""
    return any(fnmatch.fnmatch(relative, pattern) for pattern in patterns)


def fail(message: str) -> NoReturn:
    print(f"coverage-gate: {message}", file=sys.stderr)
    raise SystemExit(2)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--json",
        type=Path,
        metavar="PATH",
        help="check an existing llvm-cov JSON export instead of collecting coverage",
    )
    parser.add_argument("--min-all", type=float, default=MIN_ALL, metavar="PCT")
    parser.add_argument("--min-security", type=float, default=MIN_SECURITY, metavar="PCT")
    args = parser.parse_args()

    root = project_root()
    patterns = load_patterns(root)

    with tempfile.TemporaryDirectory() as scratch:
        report = args.json
        if report is None:
            report = Path(scratch) / "coverage.json"
            try:
                collect(root, report)
            except subprocess.CalledProcessError as err:
                fail(f"cargo llvm-cov exited with {err.returncode}; fix the tests first.")
        elif not report.is_file():
            fail(f"{report} does not exist.")
        try:
            total, files = read_report(report, root)
        except (KeyError, IndexError, ValueError) as err:
            fail(f"{report} is not an llvm-cov JSON export ({err}).")

    sensitive = []

    print()
    print(f"{'file':<52} {'lines':>13} {'covered':>9}   gate")
    print("-" * 88)
    for relative, covered, count, pct in files:
        gate = ""
        if matches(relative, patterns):
            sensitive.append((relative, pct))
            gate = f"security ≥ {args.min_security:.0f}%"
            if pct < args.min_security:
                gate += " FAIL"
        print(f"{relative:<52} {f'{covered}/{count}':>13} {pct:>8.2f}%   {gate}")
    print("-" * 88)
    print(f"{'TOTAL':<52} {'':>13} {total:>8.2f}%   all ≥ {args.min_all:.0f}%")
    print()

    failures = []
    if total < args.min_all:
        failures.append(
            f"overall line coverage {total:.2f}% is below the {args.min_all:.0f}% floor"
        )
    for relative, pct in sensitive:
        if pct < args.min_security:
            failures.append(
                f"{relative} is security-sensitive and covers {pct:.2f}% of lines, "
                f"below the {args.min_security:.0f}% floor"
            )

    if not patterns:
        print(
            f"note: no patterns in {MANIFEST}, so the {args.min_security:.0f}% floor matched "
            "nothing.\n"
            "      Declare every file that implements or enforces a security control there,\n"
            "      in the same commit that creates it — see AGENTS.md §4.3."
        )
    elif not sensitive:
        print(
            f"note: the {len(patterns)} pattern(s) in {MANIFEST} matched no covered file yet.\n"
            f"      That is expected until the first security control lands; keep {MANIFEST}\n"
            "      in step with the tree so the floor applies the moment one does."
        )

    if failures:
        print()
        for message in failures:
            print(f"FAIL: {message}")
        print()
        print(
            "Add the missing tests — the error branches first. Lowering a floor is not a fix.\n"
            "Browse the gaps with: cargo llvm-cov --all-features --workspace --html"
        )
        return 1

    print(
        f"OK: {total:.2f}% overall (floor {args.min_all:.0f}%), "
        f"{len(sensitive)} security-sensitive file(s) at or above {args.min_security:.0f}%."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
