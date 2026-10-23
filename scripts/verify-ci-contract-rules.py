#!/usr/bin/env python3
"""Prove that the CI contract rules actually fail when they should.

The two rules in ``tests/ci_policy/contract.rs`` guard promises the product
makes: that a test does not spawn a platform-specific executable by literal,
and that every endpoint the documentation promises is routed. A rule that
never fails is a rule that does not exist, and the failure mode is silent —
the rule looks plausible in review and the gate stays green.

This harness makes that failure mode loud. Each case writes a violation into
a real file, runs the single rule that owns it, records whether the rule
failed, and restores the file byte for byte.

Usage::

    python scripts/verify-ci-contract-rules.py

It exits non-zero when a case behaves differently than specified. It is a
maintenance tool, not a test: ``cargo test --test ci_policy`` carries the
permanent regression tests for the same decisions.
"""

from __future__ import annotations

import filecmp
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

PLATFORM_RULE = "contract::ci_policy_rejects_platform_specific_executables_spawned_by_literal"
ENDPOINT_RULE = "contract::ci_policy_rejects_documented_endpoints_that_do_not_exist"
INLINE_RULE = "contract::ci_policy_rejects_inline_event_handlers_the_content_policy_blocks"

ADAPTER = Path("src/metrics/prometheus/neo_cli_adapter.rs")
LOGS_PAGE = Path("src/web/pages/logs.rs")
OPENAPI = Path("docs/openapi.yaml")
HANDBOOK = Path("docs/AGENT_API.md")

PROBE_MARKER = "ci_contract_injection_probe"

PROBE = """
/// Temporary CI contract injection probe.
pub fn ci_contract_injection_probe() {{
{body}
}}
"""


def run_rule(test_name: str) -> tuple[int, str, bool]:
    """Run one contract rule and return its exit code, output, and whether it compiled.

    A compile error and a rule failure both exit non-zero. They are not the
    same thing: a rule that fails to compile proves nothing about the rule,
    and a harness that counts it as a catch will report a green run for a
    broken build.
    """
    completed = subprocess.run(
        ["cargo", "test", "--test", "ci_policy", "--", "--exact", test_name],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    output = completed.stdout + completed.stderr
    compiled = "error: could not compile" not in output
    return completed.returncode, output, compiled


def newline_of(relative: Path) -> str:
    """The line ending the file already uses."""
    raw = (ROOT / relative).read_bytes()
    return "\r\n" if b"\r\n" in raw else "\n"


def append(relative: Path, text: str) -> None:
    newline = newline_of(relative)
    with (ROOT / relative).open("a", encoding="utf-8", newline="") as handle:
        handle.write(text.replace("\n", newline))


def insert_after(relative: Path, anchor: str, text: str) -> None:
    newline = newline_of(relative)
    anchor = anchor.replace("\n", newline)
    text = text.replace("\n", newline)
    path = ROOT / relative
    with path.open("r", encoding="utf-8", newline="") as handle:
        content = handle.read()
    if content.count(anchor) != 1:
        raise SystemExit(
            f"anchor {anchor!r} occurs {content.count(anchor)} times in {relative}"
        )
    with path.open("w", encoding="utf-8", newline="") as handle:
        handle.write(content.replace(anchor, anchor + text, 1))


CASES = [
    # name, file, operation, payload, rule, must_fail
    (
        "ordinary string literal without a gate",
        ADAPTER,
        "append",
        PROBE.format(body='    let _ = Command::new("/bin/sleep");'),
        PLATFORM_RULE,
        True,
    ),
    (
        "raw string literal without a gate",
        ADAPTER,
        "append",
        PROBE.format(body='    let _ = Command::new(r"/bin/sleep");'),
        PLATFORM_RULE,
        True,
    ),
    (
        "windows-only literal behind a windows gate",
        ADAPTER,
        "append",
        PROBE.format(body='    #[cfg(windows)]\n    let _ = Command::new("taskkill");'),
        PLATFORM_RULE,
        False,
    ),
    (
        "unix-only literal behind a unix gate",
        ADAPTER,
        "append",
        PROBE.format(body='    #[cfg(unix)]\n    let _ = Command::new("sh");'),
        PLATFORM_RULE,
        False,
    ),
    (
        "unix-only literal behind a windows gate",
        ADAPTER,
        "append",
        PROBE.format(body='    #[cfg(windows)]\n    let _ = Command::new("sh");'),
        PLATFORM_RULE,
        True,
    ),
    (
        "unix-only literal behind a feature gate",
        ADAPTER,
        "append",
        PROBE.format(
            body='    #[cfg(feature = "unix-only")]\n    let _ = Command::new("sh");'
        ),
        PLATFORM_RULE,
        True,
    ),
    (
        "endpoint promised by the OpenAPI document",
        OPENAPI,
        "insert",
        '  /api/ci-contract-injection-probe:\n    get:\n      responses:\n        "200":\n          description: probe\n',
        ENDPOINT_RULE,
        True,
    ),
    (
        "endpoint promised by the agent handbook",
        HANDBOOK,
        "append",
        "\n### GET /api/ci-contract-injection-probe\n",
        ENDPOINT_RULE,
        True,
    ),
    (
        "inline event handler in rendered markup",
        LOGS_PAGE,
        "append",
        PROBE.format(
            body='    let _ = r#"<button onclick="return confirm(1)">Delete</button>"#;'
        ),
        INLINE_RULE,
        True,
    ),
    (
        "inline event handler inside a format! argument",
        LOGS_PAGE,
        "append",
        # The escaped quotes are what makes this case worth having: the
        # markup sits inside a macro argument, so the rule has to reach
        # through the token stream to find it. Written with a raw string so
        # the payload itself stays readable.
        PROBE.format(
            body='    let _ = format!(r#"<button onchange="go()">Go</button>"#);'
        ),
        INLINE_RULE,
        True,
    ),
    (
        "a comment that only mentions an event attribute",
        LOGS_PAGE,
        "append",
        PROBE.format(
            body='    // the onclick attribute used to guard this button'
        ),
        INLINE_RULE,
        False,
    ),
    (
        "a delegated data-* attribute",
        LOGS_PAGE,
        "append",
        PROBE.format(
            body='    let _ = r#"<button data-role-preset="1">Pick</button>"#;'
        ),
        INLINE_RULE,
        False,
    ),
]


def main() -> int:
    backup_root = Path(tempfile.mkdtemp(prefix="neo-nexus-contract-injection-"))
    rows = []
    # Every touched file is backed up before the first case runs, and restored
    # again when the harness exits — however it exits. A harness that leaves an
    # injection probe in the source tree is worse than no harness at all: the
    # next person reads it as real code.
    saved_files: dict[Path, Path] = {}
    try:
        for relative in sorted({case[1] for case in CASES}):
            source = ROOT / relative
            saved = backup_root / relative
            saved.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, saved)
            saved_files[relative] = saved

        for name, relative, operation, payload, rule, must_fail in CASES:
            source = ROOT / relative
            saved = saved_files[relative]
            try:
                if operation == "append":
                    append(relative, payload)
                else:
                    insert_after(relative, "paths:\n", payload)
                code, output, compiled = run_rule(rule)
            finally:
                shutil.copy2(saved, source)
            restored = filecmp.cmp(saved, source, shallow=False)
            failed = code != 0
            ok = (failed == must_fail) and restored and compiled
            rows.append((name, ok, failed, must_fail, output))
            verdict = "caught" if failed else "passed"
            expectation = "must fail" if must_fail else "must pass"
            compiled_note = "" if compiled else " [DID NOT COMPILE]"
            print(
                f"[{'ok ' if ok else 'BAD'}] {name}: rule {verdict} ({expectation}), "
                f"file restored={restored}{compiled_note}"
            )
            if not ok:
                print(output[-4000:])
    finally:
        # Restore everything, whatever happened above — including a
        # KeyboardInterrupt, which is how this harness was once stopped
        # mid-case and left a probe behind in the source tree.
        for relative, saved in saved_files.items():
            try:
                shutil.copy2(saved, ROOT / relative)
            except OSError as error:
                print(f"could not restore {relative}: {error}", file=sys.stderr)
        shutil.rmtree(backup_root, ignore_errors=True)

        # Prove the tree is clean rather than assuming it.
        left_over = []
        for relative in saved_files:
            if PROBE_MARKER in (ROOT / relative).read_text(encoding="utf-8", errors="replace"):
                left_over.append(str(relative))
        if left_over:
            print(
                "the harness left an injection probe behind in: "
                + ", ".join(left_over),
                file=sys.stderr,
            )
            return 1

    print()
    print("summary")
    bad = 0
    for name, ok, failed, must_fail, _ in rows:
        expectation = "must fail" if must_fail else "must pass"
        verdict = "caught" if failed else "passed"
        print(f"  {'ok ' if ok else 'BAD'} {name}: {verdict} ({expectation})")
        if not ok:
            bad += 1
    print(f"\n{len(rows) - bad}/{len(rows)} cases behaved as specified")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
