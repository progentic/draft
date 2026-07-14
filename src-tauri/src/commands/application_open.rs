use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    application::open_requests::{
        ApplicationOpenQueue, ApplicationOpenQueueError, ApplicationOpenRejection,
        PendingApplicationOpen,
    },
    documents::{
        persistence::{
            OpenDocumentError, OpenDocumentOutcome, open_document as open_selected_document,
        },
        registry::DocumentRegistry,
    },
};

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct OpenApplicationDocumentRequest {
    disposition: ApplicationOpenDisposition,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum ApplicationOpenDisposition {
    Dismiss,
    Open,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(
    tag = "status",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub(crate) enum OpenApplicationDocumentResponse {
    Dismissed,
    None,
    Opened { result: OpenDocumentOutcome },
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum OpenApplicationDocumentError {
    MultipleFilesUnsupported,
    Open { cause: OpenDocumentError },
    QueueUnavailable,
    UnsupportedFileLocation,
}

#[tauri::command]
pub(crate) async fn open_application_document(
    queue: State<'_, ApplicationOpenQueue>,
    registry: State<'_, DocumentRegistry>,
    request: OpenApplicationDocumentRequest,
) -> Result<OpenApplicationDocumentResponse, OpenApplicationDocumentError> {
    open_next_application_document(&queue, &registry, request.disposition)
}

fn open_next_application_document(
    queue: &ApplicationOpenQueue,
    registry: &DocumentRegistry,
    disposition: ApplicationOpenDisposition,
) -> Result<OpenApplicationDocumentResponse, OpenApplicationDocumentError> {
    let pending = queue.take().map_err(map_queue_error)?;
    if disposition == ApplicationOpenDisposition::Dismiss {
        return Ok(match pending {
            Some(_) => OpenApplicationDocumentResponse::Dismissed,
            None => OpenApplicationDocumentResponse::None,
        });
    }
    match pending {
        None => Ok(OpenApplicationDocumentResponse::None),
        Some(PendingApplicationOpen::Rejected(reason)) => Err(map_rejection(reason)),
        Some(PendingApplicationOpen::Document(path)) => {
            open_selected_document(registry, Some(path))
                .map(|result| OpenApplicationDocumentResponse::Opened { result })
                .map_err(|cause| OpenApplicationDocumentError::Open { cause })
        }
    }
}

fn map_queue_error(_: ApplicationOpenQueueError) -> OpenApplicationDocumentError {
    OpenApplicationDocumentError::QueueUnavailable
}

fn map_rejection(reason: ApplicationOpenRejection) -> OpenApplicationDocumentError {
    match reason {
        ApplicationOpenRejection::MultipleFilesUnsupported => {
            OpenApplicationDocumentError::MultipleFilesUnsupported
        }
        ApplicationOpenRejection::UnsupportedFileLocation => {
            OpenApplicationDocumentError::UnsupportedFileLocation
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn command_signature_is_typed() {
        let _ = open_application_document;
    }

    #[test]
    fn request_deserialization_is_stable() {
        assert!(
            serde_json::from_value::<OpenApplicationDocumentRequest>(json!({
                "disposition": "open"
            }))
            .is_ok()
        );
        assert!(
            serde_json::from_value::<OpenApplicationDocumentRequest>(json!({
                "disposition": "open", "path": "/tmp"
            }))
            .is_err()
        );
    }

    #[test]
    fn response_serialization_is_stable() {
        let response = open_next_application_document(
            &ApplicationOpenQueue::default(),
            &DocumentRegistry::new(),
            ApplicationOpenDisposition::Open,
        );

        assert_eq!(response, Ok(OpenApplicationDocumentResponse::None));
        assert_eq!(
            serde_json::to_value(response.unwrap()).unwrap(),
            json!({ "status": "none" })
        );
    }

    #[test]
    fn dismissal_consumes_one_request_without_opening_it() {
        let queue = ApplicationOpenQueue::default();
        queue
            .enqueue(vec![url::Url::from_file_path("/tmp/notes.draft").unwrap()])
            .unwrap();

        assert_eq!(
            open_next_application_document(
                &queue,
                &DocumentRegistry::new(),
                ApplicationOpenDisposition::Dismiss,
            ),
            Ok(OpenApplicationDocumentResponse::Dismissed)
        );
        assert_eq!(queue.take(), Ok(None));
    }

    #[test]
    fn error_serialization_is_stable() {
        let errors = [
            OpenApplicationDocumentError::MultipleFilesUnsupported,
            OpenApplicationDocumentError::QueueUnavailable,
            OpenApplicationDocumentError::UnsupportedFileLocation,
        ];

        assert_eq!(
            serde_json::to_value(errors).unwrap(),
            json!([
                { "code": "multiple_files_unsupported" },
                { "code": "queue_unavailable" },
                { "code": "unsupported_file_location" }
            ])
        );
    }
}
