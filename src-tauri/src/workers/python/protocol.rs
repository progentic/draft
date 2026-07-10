use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Current helper protocol version accepted across the Rust/Python boundary.
pub const PYTHON_HELPER_PROTOCOL_VERSION: u16 = 1;

/// Current version of the allowlisted protocol-only contract probe.
pub const CONTRACT_PROBE_VERSION: u16 = 1;

/// Maximum UTF-8 input text accepted by the contract probe.
pub const MAX_CONTRACT_PROBE_TEXT_BYTES: usize = 32 * 1024;

/// Maximum serialized request written to helper stdin.
pub const MAX_PYTHON_HELPER_REQUEST_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PythonHelperKind {
    ContractProbe,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub(crate) enum PythonHelperLocale {
    #[serde(rename = "en-US")]
    EnUs,
}

/// Validated input for the protocol-only helper operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractProbeInput {
    text: String,
    locale: PythonHelperLocale,
}

/// Deterministic result returned by the protocol-only helper operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContractProbeResult {
    utf8_bytes: usize,
}

/// Bounded local request validation failures returned before process work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PythonHelperRequestError {
    EmptyText,
    TextTooLong,
}

/// Closed machine-readable failures emitted by the Python protocol.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PythonHelperFailureCode {
    InvalidJson,
    InvalidRequest,
    UnsupportedProtocol,
    UnsupportedHelper,
    InternalFailure,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PythonHelperRequest {
    protocol_version: u16,
    request_id: Uuid,
    helper: PythonHelperKind,
    helper_version: u16,
    input: PythonHelperRequestInput,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct PythonHelperRequestInput {
    text: String,
    locale: PythonHelperLocale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PythonHelperProtocolError {
    RequestTooLarge,
    InvalidSuccess,
    InvalidFailure,
    ResponseMismatch,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct PythonHelperSuccessResponse {
    protocol_version: u16,
    request_id: Uuid,
    helper: PythonHelperKind,
    helper_version: u16,
    #[serde(rename = "status")]
    _status: PythonHelperSuccessStatus,
    result: ContractProbeWireResult,
}

#[derive(Deserialize)]
enum PythonHelperSuccessStatus {
    #[serde(rename = "ok")]
    Ok,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ContractProbeWireResult {
    utf8_bytes: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct PythonHelperFailureResponse {
    protocol_version: u16,
    #[serde(rename = "status")]
    _status: PythonHelperFailureStatus,
    code: PythonHelperFailureCode,
}

#[derive(Deserialize)]
enum PythonHelperFailureStatus {
    #[serde(rename = "error")]
    Error,
}

impl ContractProbeInput {
    pub fn new(text: impl Into<String>) -> Result<Self, PythonHelperRequestError> {
        let text = text.into();
        if text.trim().is_empty() {
            return Err(PythonHelperRequestError::EmptyText);
        }
        if text.len() > MAX_CONTRACT_PROBE_TEXT_BYTES {
            return Err(PythonHelperRequestError::TextTooLong);
        }
        Ok(Self {
            text,
            locale: PythonHelperLocale::EnUs,
        })
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

impl ContractProbeResult {
    pub fn utf8_bytes(self) -> usize {
        self.utf8_bytes
    }
}

impl PythonHelperRequest {
    pub(crate) fn contract_probe(input: ContractProbeInput) -> Self {
        Self {
            protocol_version: PYTHON_HELPER_PROTOCOL_VERSION,
            request_id: Uuid::new_v4(),
            helper: PythonHelperKind::ContractProbe,
            helper_version: CONTRACT_PROBE_VERSION,
            input: PythonHelperRequestInput {
                text: input.text,
                locale: input.locale,
            },
        }
    }

    pub(crate) fn expected_utf8_bytes(&self) -> usize {
        self.input.text.len()
    }
}

impl fmt::Display for PythonHelperRequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::EmptyText => "Python helper input text is empty",
            Self::TextTooLong => "Python helper input text is too long",
        })
    }
}

impl Error for PythonHelperRequestError {}

pub(crate) fn encode_request(
    request: &PythonHelperRequest,
) -> Result<Vec<u8>, PythonHelperProtocolError> {
    let encoded =
        serde_json::to_vec(request).map_err(|_| PythonHelperProtocolError::RequestTooLarge)?;
    if encoded.len() > MAX_PYTHON_HELPER_REQUEST_BYTES {
        return Err(PythonHelperProtocolError::RequestTooLarge);
    }
    Ok(encoded)
}

pub(crate) fn decode_success(
    request: &PythonHelperRequest,
    response: &[u8],
) -> Result<ContractProbeResult, PythonHelperProtocolError> {
    let response = serde_json::from_slice::<PythonHelperSuccessResponse>(response)
        .map_err(|_| PythonHelperProtocolError::InvalidSuccess)?;
    require_matching_response(request, &response)?;
    if response.result.utf8_bytes != request.expected_utf8_bytes() {
        return Err(PythonHelperProtocolError::InvalidSuccess);
    }
    Ok(ContractProbeResult {
        utf8_bytes: response.result.utf8_bytes,
    })
}

pub(crate) fn decode_failure(
    response: &[u8],
) -> Result<PythonHelperFailureCode, PythonHelperProtocolError> {
    let response = serde_json::from_slice::<PythonHelperFailureResponse>(response)
        .map_err(|_| PythonHelperProtocolError::InvalidFailure)?;
    if response.protocol_version != PYTHON_HELPER_PROTOCOL_VERSION {
        return Err(PythonHelperProtocolError::InvalidFailure);
    }
    Ok(response.code)
}

fn require_matching_response(
    request: &PythonHelperRequest,
    response: &PythonHelperSuccessResponse,
) -> Result<(), PythonHelperProtocolError> {
    if response.protocol_version == request.protocol_version
        && response.request_id == request.request_id
        && response.helper == request.helper
        && response.helper_version == request.helper_version
    {
        Ok(())
    } else {
        Err(PythonHelperProtocolError::ResponseMismatch)
    }
}

#[cfg(test)]
#[path = "protocol_tests.rs"]
mod tests;
