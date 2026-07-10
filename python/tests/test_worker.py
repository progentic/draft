"""Contract tests for the allowlisted Python helper protocol."""

from __future__ import annotations

import json
import unittest
from uuid import UUID

from draft_helpers import (
    CONTRACT_PROBE_HELPER,
    CONTRACT_PROBE_VERSION,
    PACKAGE_NAME,
    PROTOCOL_VERSION,
    TEXT_ANALYSIS_HELPER,
    TEXT_ANALYSIS_VERSION,
    ContractProbeRequest,
    ContractProbeResult,
    TextAnalysisFinding,
    TextAnalysisRequest,
    TextAnalysisResult,
    process_request,
)
from draft_helpers.worker import MAX_REQUEST_BYTES, MAX_TEXT_BYTES


class HelperWorkerTest(unittest.TestCase):
    def test_package_exports_typed_protocol(self) -> None:
        self.assertEqual(PACKAGE_NAME, "draft_helpers")
        self.assertEqual(PROTOCOL_VERSION, 1)
        self.assertEqual(CONTRACT_PROBE_HELPER, "contract_probe")
        self.assertEqual(CONTRACT_PROBE_VERSION, 1)
        self.assertEqual(TEXT_ANALYSIS_HELPER, "text_analysis")
        self.assertEqual(TEXT_ANALYSIS_VERSION, 1)
        self.assertEqual(
            ContractProbeRequest(REQUEST_ID, "text", "en-US").request_id,
            REQUEST_ID,
        )
        self.assertEqual(ContractProbeResult(4).utf8_bytes, 4)
        self.assertEqual(TextAnalysisRequest(REQUEST_ID, "text", "en-US").text, "text")
        finding = TextAnalysisFinding("repeated_word", 5, 9)
        self.assertEqual(TextAnalysisResult((finding,)).findings, (finding,))

    def test_valid_request_returns_stable_typed_response(self) -> None:
        exit_code, raw_response = process_request(request_bytes("Résumé"))

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            json.loads(raw_response),
            {
                "protocolVersion": 1,
                "requestId": REQUEST_ID,
                "helper": "contract_probe",
                "helperVersion": 1,
                "status": "ok",
                "result": {"utf8Bytes": 8},
            },
        )

    def test_invalid_json_and_unknown_fields_fail_typed(self) -> None:
        self.assert_failure(b"not-json", "invalid_json")
        payload = request_payload("text")
        payload["unexpected"] = True
        self.assert_failure(encode(payload), "invalid_request")

    def test_protocol_and_helper_allowlist_fail_closed(self) -> None:
        protocol = request_payload("text")
        protocol["protocolVersion"] = 2
        self.assert_failure(encode(protocol), "unsupported_protocol")

        helper = request_payload("text")
        helper["helper"] = "formatting"
        self.assert_failure(encode(helper), "unsupported_helper")

        version = request_payload("text")
        version["helperVersion"] = 2
        self.assert_failure(encode(version), "unsupported_helper")

        boolean_version = request_payload("text")
        boolean_version["helperVersion"] = True
        self.assert_failure(encode(boolean_version), "unsupported_helper")

    def test_request_identity_and_input_bounds_fail_closed(self) -> None:
        invalid_id = request_payload("text")
        invalid_id["requestId"] = "frontend-id"
        self.assert_failure(encode(invalid_id), "invalid_request")

        self.assert_failure(request_bytes(" "), "invalid_request")
        self.assert_failure(
            request_bytes("x" * (MAX_TEXT_BYTES + 1)), "invalid_request"
        )

        locale = request_payload("text")
        locale["input"]["locale"] = "fr-FR"
        self.assert_failure(encode(locale), "invalid_request")

    def test_oversized_serialized_request_fails_closed(self) -> None:
        self.assert_failure(b"x" * (MAX_REQUEST_BYTES + 1), "invalid_request")

    def test_text_analysis_returns_all_review_categories(self) -> None:
        long_sentence = " ".join(f"word{index}" for index in range(31))
        text = (
            "I write the the draft. Because this is IMPORTANT. "
            f"Because we revise it. {long_sentence}."
        )
        response = self.text_analysis(text)

        self.assertEqual(
            {finding["code"] for finding in response["result"]["findings"]},
            {
                "repeated_word",
                "all_caps_emphasis",
                "repeated_sentence_opener",
                "mixed_first_person",
                "long_sentence",
            },
        )

    def test_text_analysis_thresholds_are_explicit(self) -> None:
        thirty_words = " ".join(f"word{index}" for index in range(30)) + "."
        thirty_one_words = " ".join(f"word{index}" for index in range(31)) + "."

        self.assertNotIn(
            "long_sentence", finding_codes(self.text_analysis(thirty_words))
        )
        self.assertIn(
            "long_sentence", finding_codes(self.text_analysis(thirty_one_words))
        )
        self.assertNotIn("all_caps_emphasis", finding_codes(self.text_analysis("NASA")))
        self.assertIn("all_caps_emphasis", finding_codes(self.text_analysis("LOUDER")))

    def test_text_analysis_offsets_use_utf8_bytes(self) -> None:
        response = self.text_analysis("Café café")

        self.assertEqual(
            response["result"]["findings"],
            [{"code": "repeated_word", "startByte": 6, "endByte": 11}],
        )

    def test_text_analysis_is_deterministic_sorted_and_bounded(self) -> None:
        text = " ".join(["word"] * 120) + "."
        first = self.text_analysis(text)
        second = self.text_analysis(text)
        findings = first["result"]["findings"]

        self.assertEqual(first, second)
        self.assertLessEqual(len(findings), 100)
        self.assertEqual(
            sum(finding["code"] == "repeated_word" for finding in findings),
            20,
        )
        self.assertEqual(
            findings,
            sorted(
                findings,
                key=lambda finding: (
                    finding["startByte"],
                    finding["endByte"],
                    finding["code"],
                ),
            ),
        )

    def test_text_analysis_false_positive_guards(self) -> None:
        response = self.text_analysis(
            "NASA is clear. I write alone. Because one. However two."
        )

        self.assertEqual(response["result"]["findings"], [])

    def text_analysis(self, text: str) -> dict[str, object]:
        exit_code, raw_response = process_request(
            encode(request_payload(text, helper="text_analysis"))
        )
        self.assertEqual(exit_code, 0)
        return json.loads(raw_response)

    def assert_failure(self, request: bytes, expected_code: str) -> None:
        exit_code, raw_response = process_request(request)
        self.assertNotEqual(exit_code, 0)
        self.assertEqual(
            json.loads(raw_response),
            {
                "protocolVersion": 1,
                "status": "error",
                "code": expected_code,
            },
        )


REQUEST_ID = str(UUID("00000000-0000-4000-8000-000000000000"))


def request_bytes(text: str) -> bytes:
    return encode(request_payload(text))


def request_payload(
    text: str, helper: str = "contract_probe"
) -> dict[str, object]:
    return {
        "protocolVersion": 1,
        "requestId": REQUEST_ID,
        "helper": helper,
        "helperVersion": 1,
        "input": {"text": text, "locale": "en-US"},
    }


def encode(payload: dict[str, object]) -> bytes:
    return json.dumps(payload, ensure_ascii=False).encode("utf-8")


def finding_codes(response: dict[str, object]) -> set[str]:
    result = response["result"]
    assert isinstance(result, dict)
    findings = result["findings"]
    assert isinstance(findings, list)
    return {finding["code"] for finding in findings}


if __name__ == "__main__":
    unittest.main()
