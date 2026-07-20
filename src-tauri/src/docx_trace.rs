use std::fmt;

const TRACE_ENVIRONMENT_VARIABLE: &str = "DRAFT_DOCX_TRACE";

pub(crate) fn emit(stage: &'static str, details: fmt::Arguments<'_>) {
    if std::env::var_os(TRACE_ENVIRONMENT_VARIABLE).is_some() {
        eprintln!("[draft-docx] stage={stage} {details}");
    }
}
