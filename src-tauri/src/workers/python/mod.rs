//! Rust-owned process and protocol boundary for allowlisted Python helpers.

mod protocol;
mod runner;
mod text_analysis;

pub use protocol::{
    ContractProbeInput, ContractProbeResult, PYTHON_HELPER_PROTOCOL_VERSION,
    PythonHelperFailureCode, PythonHelperRequestError,
};
pub use runner::{PythonHelperConfigurationError, PythonHelperRunError, PythonHelperRunner};
pub use text_analysis::{
    TextAnalysisCategory, TextAnalysisFinding, TextAnalysisFindingCode, TextAnalysisInput,
    TextAnalysisResult, TextAnalysisSeverity,
};
