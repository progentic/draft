"""Versioned stdin/stdout protocol for allowlisted DRAFT Python helpers."""

from __future__ import annotations

import json
import sys
from dataclasses import dataclass
from typing import Any
from uuid import UUID

PROTOCOL_VERSION = 1
CONTRACT_PROBE_HELPER = "contract_probe"
CONTRACT_PROBE_VERSION = 1
SUPPORTED_LOCALE = "en-US"
MAX_TEXT_BYTES = 32 * 1024
MAX_REQUEST_BYTES = 64 * 1024

_REQUEST_FIELDS = {
    "protocolVersion",
    "requestId",
    "helper",
    "helperVersion",
    "input",
}
_INPUT_FIELDS = {"text", "locale"}


@dataclass(frozen=True)
class ContractProbeRequest:
    """Validated protocol-probe input with no path or authority fields."""

    request_id: str
    text: str
    locale: str


@dataclass(frozen=True)
class ContractProbeResult:
    """Deterministic result returned by the protocol-only helper."""

    utf8_bytes: int


class HelperRequestError(Exception):
    """Bounded request rejection represented by a stable machine code."""

    def __init__(self, code: str) -> None:
        super().__init__(code)
        self.code = code


def process_request(raw_request: bytes) -> tuple[int, bytes]:
    """Validate one request and return one bounded response document."""

    try:
        request = _parse_request(raw_request)
        result = _run_contract_probe(request)
        return 0, _success_response(request, result)
    except HelperRequestError as error:
        return 2, _failure_response(error.code)
    except Exception:
        return 3, _failure_response("internal_failure")


def main() -> int:
    """Read one bounded request from stdin and write one response to stdout."""

    try:
        raw_request = sys.stdin.buffer.read(MAX_REQUEST_BYTES + 1)
        exit_code, response = process_request(raw_request)
    except Exception:
        exit_code, response = 3, _failure_response("internal_failure")
    sys.stdout.buffer.write(response)
    sys.stdout.buffer.flush()
    return exit_code


def _parse_request(raw_request: bytes) -> ContractProbeRequest:
    if len(raw_request) > MAX_REQUEST_BYTES:
        raise HelperRequestError("invalid_request")
    payload = _decode_object(raw_request)
    _require_exact_fields(payload, _REQUEST_FIELDS)
    _require_exact_version(payload, "protocolVersion", PROTOCOL_VERSION, "unsupported_protocol")
    _require_helper(payload)
    request_id = _validated_request_id(payload["requestId"])
    helper_input = payload["input"]
    if not isinstance(helper_input, dict):
        raise HelperRequestError("invalid_request")
    _require_exact_fields(helper_input, _INPUT_FIELDS)
    return ContractProbeRequest(
        request_id=request_id,
        text=_validated_text(helper_input["text"]),
        locale=_validated_locale(helper_input["locale"]),
    )


def _decode_object(raw_request: bytes) -> dict[str, Any]:
    try:
        payload = json.loads(raw_request.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise HelperRequestError("invalid_json") from error
    if not isinstance(payload, dict):
        raise HelperRequestError("invalid_request")
    return payload


def _require_exact_fields(payload: dict[str, Any], expected: set[str]) -> None:
    if set(payload) != expected:
        raise HelperRequestError("invalid_request")


def _require_exact_version(
    payload: dict[str, Any], field: str, expected: int, failure: str
) -> None:
    value = payload[field]
    if type(value) is not int or value != expected:
        raise HelperRequestError(failure)


def _require_helper(payload: dict[str, Any]) -> None:
    if payload["helper"] != CONTRACT_PROBE_HELPER:
        raise HelperRequestError("unsupported_helper")
    _require_exact_version(
        payload, "helperVersion", CONTRACT_PROBE_VERSION, "unsupported_helper"
    )


def _validated_request_id(value: Any) -> str:
    if not isinstance(value, str):
        raise HelperRequestError("invalid_request")
    try:
        parsed = UUID(value)
    except ValueError as error:
        raise HelperRequestError("invalid_request") from error
    if str(parsed) != value:
        raise HelperRequestError("invalid_request")
    return value


def _validated_text(value: Any) -> str:
    if not isinstance(value, str) or not value.strip():
        raise HelperRequestError("invalid_request")
    if len(value.encode("utf-8")) > MAX_TEXT_BYTES:
        raise HelperRequestError("invalid_request")
    return value


def _validated_locale(value: Any) -> str:
    if value != SUPPORTED_LOCALE:
        raise HelperRequestError("invalid_request")
    return value


def _run_contract_probe(request: ContractProbeRequest) -> ContractProbeResult:
    return ContractProbeResult(utf8_bytes=len(request.text.encode("utf-8")))


def _success_response(
    request: ContractProbeRequest, result: ContractProbeResult
) -> bytes:
    return _encode_response(
        {
            "protocolVersion": PROTOCOL_VERSION,
            "requestId": request.request_id,
            "helper": CONTRACT_PROBE_HELPER,
            "helperVersion": CONTRACT_PROBE_VERSION,
            "status": "ok",
            "result": {"utf8Bytes": result.utf8_bytes},
        }
    )


def _failure_response(code: str) -> bytes:
    return _encode_response(
        {
            "protocolVersion": PROTOCOL_VERSION,
            "status": "error",
            "code": code,
        }
    )


def _encode_response(payload: dict[str, Any]) -> bytes:
    return json.dumps(
        payload, ensure_ascii=False, separators=(",", ":")
    ).encode("utf-8")


if __name__ == "__main__":
    raise SystemExit(main())
