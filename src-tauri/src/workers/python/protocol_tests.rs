use serde_json::{Value, json};

use super::*;

#[test]
fn contract_probe_input_is_bounded_before_request_creation() {
    assert_eq!(
        ContractProbeInput::new(" "),
        Err(PythonHelperRequestError::EmptyText)
    );
    assert_eq!(
        ContractProbeInput::new("x".repeat(MAX_CONTRACT_PROBE_TEXT_BYTES + 1)),
        Err(PythonHelperRequestError::TextTooLong)
    );
}

#[test]
fn request_serialization_is_stable_and_allowlisted() {
    let request = request("Résumé");
    let value = serde_json::to_value(&request).unwrap();

    assert_eq!(value["protocolVersion"], 1);
    assert_eq!(value["helper"], "contract_probe");
    assert_eq!(value["helperVersion"], 1);
    assert_eq!(
        value["input"],
        json!({ "text": "Résumé", "locale": "en-US" })
    );
    assert!(value["requestId"].as_str().unwrap().parse::<Uuid>().is_ok());
    assert!(encode_request(&request).unwrap().len() <= MAX_PYTHON_HELPER_REQUEST_BYTES);
}

#[test]
fn matching_success_response_is_validated() {
    let request = request("Résumé");
    let response = success_response(&request, 8);

    assert_eq!(decode_success(&request, &response).unwrap().utf8_bytes(), 8);
}

#[test]
fn unknown_or_malformed_success_output_fails_closed() {
    let request = request("text");
    let mut response = success_value(&request, 4);
    response["unexpected"] = Value::Bool(true);

    assert_eq!(
        decode_success(&request, &serde_json::to_vec(&response).unwrap()),
        Err(PythonHelperProtocolError::InvalidSuccess)
    );
    assert_eq!(
        decode_success(&request, b"not-json"),
        Err(PythonHelperProtocolError::InvalidSuccess)
    );
}

#[test]
fn response_identity_and_versions_must_match_request() {
    let request = request("text");
    for field in ["protocolVersion", "helperVersion"] {
        let mut response = success_value(&request, 4);
        response[field] = json!(2);
        assert_eq!(
            decode_success(&request, &serde_json::to_vec(&response).unwrap()),
            Err(PythonHelperProtocolError::ResponseMismatch)
        );
    }

    let mut response = success_value(&request, 4);
    response["requestId"] = json!(Uuid::new_v4());
    assert_eq!(
        decode_success(&request, &serde_json::to_vec(&response).unwrap()),
        Err(PythonHelperProtocolError::ResponseMismatch)
    );
}

#[test]
fn impossible_contract_probe_result_fails_validation() {
    let request = request("text");
    assert_eq!(
        decode_success(&request, &success_response(&request, 5)),
        Err(PythonHelperProtocolError::InvalidSuccess)
    );
}

#[test]
fn helper_failure_response_is_typed_and_strict() {
    let response = serde_json::to_vec(&json!({
        "protocolVersion": 1,
        "status": "error",
        "code": "unsupported_helper"
    }))
    .unwrap();
    assert_eq!(
        decode_failure(&response),
        Ok(PythonHelperFailureCode::UnsupportedHelper)
    );

    let response = serde_json::to_vec(&json!({
        "protocolVersion": 1,
        "status": "error",
        "code": "unsupported_helper",
        "detail": "must not cross boundary"
    }))
    .unwrap();
    assert_eq!(
        decode_failure(&response),
        Err(PythonHelperProtocolError::InvalidFailure)
    );
}

#[test]
fn request_errors_do_not_include_input_content() {
    assert_eq!(
        PythonHelperRequestError::TextTooLong.to_string(),
        "Python helper input text is too long"
    );
}

fn request(text: &str) -> PythonHelperRequest {
    PythonHelperRequest::contract_probe(ContractProbeInput::new(text).unwrap())
}

fn success_response(request: &PythonHelperRequest, utf8_bytes: usize) -> Vec<u8> {
    serde_json::to_vec(&success_value(request, utf8_bytes)).unwrap()
}

fn success_value(request: &PythonHelperRequest, utf8_bytes: usize) -> Value {
    json!({
        "protocolVersion": 1,
        "requestId": request.request_id,
        "helper": "contract_probe",
        "helperVersion": 1,
        "status": "ok",
        "result": { "utf8Bytes": utf8_bytes }
    })
}
