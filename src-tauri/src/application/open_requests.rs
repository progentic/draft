use std::{collections::VecDeque, path::PathBuf, sync::Mutex};

use tauri::{AppHandle, Manager, RunEvent};
use url::Url;

use crate::events::application_open::{ApplicationOpenEvent, emit_application_open_event};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum PendingApplicationOpen {
    Document(PathBuf),
    Rejected(ApplicationOpenRejection),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ApplicationOpenRejection {
    MultipleFilesUnsupported,
    UnsupportedFileLocation,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ApplicationOpenQueueError {
    Unavailable,
}

#[derive(Default)]
pub(crate) struct ApplicationOpenQueue {
    pending: Mutex<VecDeque<PendingApplicationOpen>>,
}

impl ApplicationOpenQueue {
    pub(crate) fn enqueue(&self, urls: Vec<Url>) -> Result<(), ApplicationOpenQueueError> {
        let request = request_from_urls(urls);
        self.pending
            .lock()
            .map_err(|_| ApplicationOpenQueueError::Unavailable)?
            .push_back(request);
        Ok(())
    }

    pub(crate) fn take(&self) -> Result<Option<PendingApplicationOpen>, ApplicationOpenQueueError> {
        self.pending
            .lock()
            .map_err(|_| ApplicationOpenQueueError::Unavailable)
            .map(|mut pending| pending.pop_front())
    }
}

pub(crate) fn handle_run_event(app: &AppHandle, event: RunEvent) {
    if let Some(urls) = opened_urls(event) {
        queue_open_request(app, urls);
    }
}

#[cfg(target_os = "macos")]
fn opened_urls(event: RunEvent) -> Option<Vec<Url>> {
    match event {
        RunEvent::Opened { urls } => Some(urls),
        _ => None,
    }
}

#[cfg(not(target_os = "macos"))]
fn opened_urls(event: RunEvent) -> Option<Vec<Url>> {
    let _ = event;
    None
}

fn queue_open_request(app: &AppHandle, urls: Vec<Url>) {
    let event = match app.state::<ApplicationOpenQueue>().enqueue(urls) {
        Ok(()) => ApplicationOpenEvent::Available,
        Err(ApplicationOpenQueueError::Unavailable) => ApplicationOpenEvent::QueueUnavailable,
    };
    emit_application_open_event(app, event);
}

fn request_from_urls(urls: Vec<Url>) -> PendingApplicationOpen {
    if urls.len() != 1 {
        return PendingApplicationOpen::Rejected(
            ApplicationOpenRejection::MultipleFilesUnsupported,
        );
    }
    urls.into_iter()
        .next()
        .and_then(|url| url.to_file_path().ok())
        .map(PendingApplicationOpen::Document)
        .unwrap_or(PendingApplicationOpen::Rejected(
            ApplicationOpenRejection::UnsupportedFileLocation,
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_file_url_is_queued_without_exposing_it() {
        let queue = ApplicationOpenQueue::default();
        queue
            .enqueue(vec![Url::from_file_path("/tmp/notes.draft").unwrap()])
            .unwrap();

        assert_eq!(
            queue.take(),
            Ok(Some(PendingApplicationOpen::Document(PathBuf::from(
                "/tmp/notes.draft"
            ))))
        );
        assert_eq!(queue.take(), Ok(None));
    }

    #[test]
    fn multiple_or_non_file_urls_fail_closed() {
        assert_eq!(
            request_from_urls(Vec::new()),
            PendingApplicationOpen::Rejected(ApplicationOpenRejection::MultipleFilesUnsupported)
        );
        assert_eq!(
            request_from_urls(vec![
                Url::from_file_path("/tmp/one.draft").unwrap(),
                Url::from_file_path("/tmp/two.draft").unwrap(),
            ]),
            PendingApplicationOpen::Rejected(ApplicationOpenRejection::MultipleFilesUnsupported)
        );
        assert_eq!(
            request_from_urls(vec![
                Url::parse("https://example.invalid/notes.draft").unwrap()
            ]),
            PendingApplicationOpen::Rejected(ApplicationOpenRejection::UnsupportedFileLocation)
        );
    }
}
