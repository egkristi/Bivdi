#!/usr/bin/env python3
"""Generate asciinema v2 .cast files from Bivdi's actual CLI output.

Each demo records the *real* output of `bivdi-cli` (never hand-authored), so the
recordings are honest and reproducible. The script types the command at a
human-ish pace and then prints the captured program output verbatim.

Usage:
    python3 demos/generate.py            # regenerates demos/*.cast

Requires the CLI binary to be built:
    cd runtime && cargo build -p bivdi-cli
"""

import json
import os
import subprocess
import sys
import time

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BIN = os.path.join(REPO, "runtime", "target", "debug", "bivdi-cli")
OUT = os.path.join(REPO, "demos")

WIDTH = 80
HEIGHT = 24

# (output filename, shell prompt text, argv for bivdi-cli)
DEMOS = [
    (
        "01-six-primitives.cast",
        "$ bivdi-cli",
        [],
    ),
    (
        "02-agent-scenario.cast",
        "$ bivdi-cli scenario",
        ["scenario"],
    ),
    (
        "03-persistence.cast",
        "$ bivdi-cli persist /tmp/bivdi-store.bivdi",
        ["persist", "/tmp/bivdi-store.bivdi"],
    ),
    (
        "04-wasm.cast",
        "$ bivdi-cli wasm",
        ["wasm"],
    ),
    (
        "05-sandbox.cast",
        "$ bivdi-cli sandbox",
        ["sandbox"],
    ),
]


def run(argv: list[str]) -> str:
    proc = subprocess.run(
        [BIN, *argv],
        cwd=os.path.join(REPO, "runtime"),
        capture_output=True,
        text=True,
    )
    return proc.stdout + proc.stderr


def emit_line(out, t: float, kind: str, data: str):
    out.write(json.dumps([t, kind, data], ensure_ascii=False) + "\n")


def make_cast(filename: str, prompt: str, argv: list[str]) -> None:
    output = run(argv)
    path = os.path.join(OUT, filename)

    t = 0.0
    with open(path, "w", encoding="utf-8") as f:
        f.write(json.dumps(
            {"version": 2, "width": WIDTH, "height": HEIGHT, "timestamp": int(time.time())}
        ) + "\n")

        # Type the prompt, character by character, with a realistic cadence.
        for ch in prompt:
            t += 0.08
            emit_line(f, t, "o", ch)
        t += 0.4
        emit_line(f, t, "o", "\r\n")

        # Print the real program output in a few chunks for a readable pace.
        t += 0.15
        for line in output.splitlines(keepends=True):
            t += 0.04
            emit_line(f, t, "o", line)
        t += 0.3
        emit_line(f, t, "o", "\r\n")

        # Close with a prompt line so asciinema doesn't hang on a blank frame.
        t += 0.1
        emit_line(f, t, "o", "\r\n$ ")

    print(f"wrote {path}")


def main() -> int:
    os.makedirs(OUT, exist_ok=True)
    if not os.path.exists(BIN):
        print(f"error: binary not found at {BIN}; run `cargo build -p bivdi-cli` first",
              file=sys.stderr)
        return 1
    for filename, prompt, argv in DEMOS:
        make_cast(filename, prompt, argv)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
