"""Hostile process fixtures used only by Rust helper-runner tests."""

from __future__ import annotations

import json
import os
import sys
import time


def main() -> int:
    mode = sys.argv[1]
    request = json.loads(sys.stdin.buffer.read().decode("utf-8"))
    if mode == "hang":
        time.sleep(30)
        return 1
    if mode == "malformed":
        sys.stdout.write("not-json")
        return 0
    if mode == "oversized":
        sys.stdout.write("x" * (64 * 1024 + 1))
        return 0
    if mode == "stderr":
        sys.stderr.write("bounded fixture diagnostic")
        write_success(request)
        return 0
    if mode == "rejected":
        write_json(
            {
                "protocolVersion": 1,
                "status": "error",
                "code": "internal_failure",
            }
        )
        return 3
    if mode == "environment" and "PATH" in os.environ:
        return 4
    write_success(request)
    return 0


def write_success(request: dict[str, object]) -> None:
    helper_input = request["input"]
    assert isinstance(helper_input, dict)
    text = helper_input["text"]
    assert isinstance(text, str)
    write_json(
        {
            "protocolVersion": request["protocolVersion"],
            "requestId": request["requestId"],
            "helper": request["helper"],
            "helperVersion": request["helperVersion"],
            "status": "ok",
            "result": {"utf8Bytes": len(text.encode("utf-8"))},
        }
    )


def write_json(payload: dict[str, object]) -> None:
    sys.stdout.write(json.dumps(payload, separators=(",", ":")))


if __name__ == "__main__":
    raise SystemExit(main())
