"""Versioned stdin/stdout protocol for allowlisted DRAFT Python helpers."""

from __future__ import annotations

import json
import re
import sys
from dataclasses import dataclass
from typing import Any
from uuid import UUID

PROTOCOL_VERSION = 1
CONTRACT_PROBE_HELPER = "contract_probe"
CONTRACT_PROBE_VERSION = 1
TEXT_ANALYSIS_HELPER = "text_analysis"
TEXT_ANALYSIS_VERSION = 1
SUPPORTED_LOCALE = "en-US"
MAX_TEXT_BYTES = 32 * 1024
MAX_REQUEST_BYTES = 64 * 1024
MAX_FINDINGS = 100
MAX_FINDINGS_PER_CHECK = 20
LONG_SENTENCE_WORDS = 30
MIN_ALL_CAPS_LETTERS = 5
MIN_REPEATED_OPENER_LETTERS = 4

_WORD_PATTERN = re.compile(r"[^\W\d_]+(?:['’][^\W\d_]+)?", re.UNICODE)
_SENTENCE_PATTERN = re.compile(r"[^.!?]+(?:[.!?]+|$)", re.UNICODE)
_SINGULAR_PRONOUNS = {"i", "me", "my", "mine"}
_PLURAL_PRONOUNS = {"we", "us", "our", "ours"}

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


@dataclass(frozen=True)
class TextAnalysisRequest:
    """Validated text-analysis input with no document mutation authority."""

    request_id: str
    text: str
    locale: str


@dataclass(frozen=True)
class TextAnalysisFinding:
    """One closed finding code and half-open UTF-8 byte range."""

    code: str
    start_byte: int
    end_byte: int


@dataclass(frozen=True)
class TextAnalysisResult:
    """Deterministic bounded findings returned for one input snapshot."""

    findings: tuple[TextAnalysisFinding, ...]


class HelperRequestError(Exception):
    """Bounded request rejection represented by a stable machine code."""

    def __init__(self, code: str) -> None:
        super().__init__(code)
        self.code = code


def process_request(raw_request: bytes) -> tuple[int, bytes]:
    """Validate one request and return one bounded response document."""

    try:
        request = _parse_request(raw_request)
        result = _dispatch_request(request)
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


def _parse_request(raw_request: bytes) -> ContractProbeRequest | TextAnalysisRequest:
    if len(raw_request) > MAX_REQUEST_BYTES:
        raise HelperRequestError("invalid_request")
    payload = _decode_object(raw_request)
    _require_exact_fields(payload, _REQUEST_FIELDS)
    _require_exact_version(
        payload, "protocolVersion", PROTOCOL_VERSION, "unsupported_protocol"
    )
    helper = _validated_helper(payload)
    request_id = _validated_request_id(payload["requestId"])
    helper_input = payload["input"]
    if not isinstance(helper_input, dict):
        raise HelperRequestError("invalid_request")
    _require_exact_fields(helper_input, _INPUT_FIELDS)
    text = _validated_text(helper_input["text"])
    locale = _validated_locale(helper_input["locale"])
    if helper == CONTRACT_PROBE_HELPER:
        return ContractProbeRequest(request_id, text, locale)
    return TextAnalysisRequest(request_id, text, locale)


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


def _validated_helper(payload: dict[str, Any]) -> str:
    helper = payload["helper"]
    version = payload["helperVersion"]
    if type(version) is not int:
        raise HelperRequestError("unsupported_helper")
    if helper == CONTRACT_PROBE_HELPER and version == CONTRACT_PROBE_VERSION:
        return helper
    if helper == TEXT_ANALYSIS_HELPER and version == TEXT_ANALYSIS_VERSION:
        return helper
    raise HelperRequestError("unsupported_helper")


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


def _dispatch_request(
    request: ContractProbeRequest | TextAnalysisRequest,
) -> ContractProbeResult | TextAnalysisResult:
    if isinstance(request, ContractProbeRequest):
        return _run_contract_probe(request)
    return _run_text_analysis(request)


def _run_text_analysis(request: TextAnalysisRequest) -> TextAnalysisResult:
    checks = (
        _repeated_word_findings(request.text),
        _long_sentence_findings(request.text),
        _all_caps_findings(request.text),
        _repeated_opener_findings(request.text),
        _mixed_first_person_findings(request.text),
    )
    findings = [finding for check in checks for finding in check]
    findings.sort(
        key=lambda finding: (finding.start_byte, finding.end_byte, finding.code)
    )
    return TextAnalysisResult(findings=tuple(findings[:MAX_FINDINGS]))


def _repeated_word_findings(text: str) -> list[TextAnalysisFinding]:
    findings: list[TextAnalysisFinding] = []
    words = list(_WORD_PATTERN.finditer(text))
    for previous, current in zip(words, words[1:]):
        separator = text[previous.end() : current.start()]
        same_word = previous.group().casefold() == current.group().casefold()
        if separator and separator.isspace() and same_word:
            findings.append(
                _finding("repeated_word", text, current.start(), current.end())
            )
            if len(findings) == MAX_FINDINGS_PER_CHECK:
                break
    return findings


def _long_sentence_findings(text: str) -> list[TextAnalysisFinding]:
    findings: list[TextAnalysisFinding] = []
    for sentence in _SENTENCE_PATTERN.finditer(text):
        start, end = _trimmed_range(text, sentence.start(), sentence.end())
        word_count = len(list(_WORD_PATTERN.finditer(text[start:end])))
        if start < end and word_count > LONG_SENTENCE_WORDS:
            findings.append(_finding("long_sentence", text, start, end))
            if len(findings) == MAX_FINDINGS_PER_CHECK:
                break
    return findings


def _all_caps_findings(text: str) -> list[TextAnalysisFinding]:
    findings: list[TextAnalysisFinding] = []
    for word in _WORD_PATTERN.finditer(text):
        value = word.group()
        if len(value) >= MIN_ALL_CAPS_LETTERS and value.isupper():
            findings.append(
                _finding("all_caps_emphasis", text, word.start(), word.end())
            )
            if len(findings) == MAX_FINDINGS_PER_CHECK:
                break
    return findings


def _repeated_opener_findings(text: str) -> list[TextAnalysisFinding]:
    findings: list[TextAnalysisFinding] = []
    sentences = list(_SENTENCE_PATTERN.finditer(text))
    for previous, current in zip(sentences, sentences[1:]):
        previous_opener = _first_word(text, previous.start(), previous.end())
        current_opener = _first_word(text, current.start(), current.end())
        if _same_substantial_opener(previous_opener, current_opener):
            assert current_opener is not None
            findings.append(
                _finding(
                    "repeated_sentence_opener",
                    text,
                    current_opener.start(),
                    current_opener.end(),
                )
            )
            if len(findings) == MAX_FINDINGS_PER_CHECK:
                break
    return findings


def _mixed_first_person_findings(text: str) -> list[TextAnalysisFinding]:
    singular = None
    plural = None
    for word in _WORD_PATTERN.finditer(text):
        normalized = word.group().casefold()
        singular = singular or (word if normalized in _SINGULAR_PRONOUNS else None)
        plural = plural or (word if normalized in _PLURAL_PRONOUNS else None)
        if singular is not None and plural is not None:
            later = plural if singular.start() < plural.start() else singular
            return [_finding("mixed_first_person", text, later.start(), later.end())]
    return []


def _finding(code: str, text: str, start: int, end: int) -> TextAnalysisFinding:
    return TextAnalysisFinding(
        code=code,
        start_byte=_utf8_offset(text, start),
        end_byte=_utf8_offset(text, end),
    )


def _utf8_offset(text: str, character_offset: int) -> int:
    return len(text[:character_offset].encode("utf-8"))


def _trimmed_range(text: str, start: int, end: int) -> tuple[int, int]:
    while start < end and text[start].isspace():
        start += 1
    while end > start and text[end - 1].isspace():
        end -= 1
    return start, end


def _first_word(text: str, start: int, end: int) -> re.Match[str] | None:
    return _WORD_PATTERN.search(text, start, end)


def _same_substantial_opener(
    previous: re.Match[str] | None,
    current: re.Match[str] | None,
) -> bool:
    if previous is None or current is None:
        return False
    previous_text = previous.group()
    current_text = current.group()
    return (
        len(previous_text) >= MIN_REPEATED_OPENER_LETTERS
        and previous_text.casefold() == current_text.casefold()
    )


def _success_response(
    request: ContractProbeRequest | TextAnalysisRequest,
    result: ContractProbeResult | TextAnalysisResult,
) -> bytes:
    helper, helper_version, result_payload = _success_payload(request, result)
    return _encode_response(
        {
            "protocolVersion": PROTOCOL_VERSION,
            "requestId": request.request_id,
            "helper": helper,
            "helperVersion": helper_version,
            "status": "ok",
            "result": result_payload,
        }
    )


def _success_payload(
    request: ContractProbeRequest | TextAnalysisRequest,
    result: ContractProbeResult | TextAnalysisResult,
) -> tuple[str, int, dict[str, Any]]:
    if isinstance(request, ContractProbeRequest) and isinstance(
        result, ContractProbeResult
    ):
        payload = {"utf8Bytes": result.utf8_bytes}
        return CONTRACT_PROBE_HELPER, CONTRACT_PROBE_VERSION, payload
    if isinstance(request, TextAnalysisRequest) and isinstance(
        result, TextAnalysisResult
    ):
        findings = [
            {
                "code": finding.code,
                "startByte": finding.start_byte,
                "endByte": finding.end_byte,
            }
            for finding in result.findings
        ]
        return TEXT_ANALYSIS_HELPER, TEXT_ANALYSIS_VERSION, {"findings": findings}
    raise TypeError("helper request and result do not match")


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
