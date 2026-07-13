//! Versioned document-domain types owned by the Rust core.

pub(crate) mod atomic_write;
pub(crate) mod dialog;
pub mod envelope;
pub(crate) mod migration;
pub(crate) mod paragraph_format;
pub(crate) mod persistence;
pub mod registry;
#[cfg(test)]
pub(crate) mod test_support;
pub mod text_format;
pub(crate) mod text_import;
