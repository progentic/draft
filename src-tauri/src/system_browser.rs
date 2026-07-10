use url::Url;

use crate::research::external_access::{ExternalUrlOpenError, ExternalUrlOpener};

/// Concrete adapter that delegates one validated URL to the default OS browser.
pub(crate) struct SystemBrowser;

impl ExternalUrlOpener for SystemBrowser {
    fn open_external_url(&self, url: &Url) -> Result<(), ExternalUrlOpenError> {
        tauri_plugin_opener::open_url(url.as_str(), None::<&str>).map_err(|_| ExternalUrlOpenError)
    }
}
