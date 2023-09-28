use thiserror::Error;

/// Errors that can occur in Sotto.
#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot begin dictation, we're already dictating")]
    AlreadyDictating,
    #[error("cannot end recording, we aren't dictating")]
    NotDictating,
    #[error("failed to create temporary file to record to")]
    TmpFileCreationFailed {
        #[source]
        source: std::io::Error,
    },
    #[error("dictation task panicked")]
    DictationTaskPanicked { source: tokio::task::JoinError },
}
