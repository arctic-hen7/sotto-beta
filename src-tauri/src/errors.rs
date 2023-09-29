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
    #[error("no audio input device found (do you have a microphone connected?)")]
    NoInputDevice,
    #[error("failed to get configuration for default audio input device")]
    GetDefaultInputConfigFailed {
        #[source]
        source: cpal::DefaultStreamConfigError,
    },
    #[error("failed to create a wav file writer to record audio")]
    CreateWavWriterFailed {
        #[source]
        source: hound::Error,
    },
    #[error("failed to build input stream for recording audio")]
    BuildInputStreamFailed {
        #[source]
        source: cpal::BuildStreamError,
    },
    #[error("failed to start audio input stream for recording")]
    StartStreamFailed {
        #[source]
        source: cpal::PlayStreamError,
    },
    #[error("failed to instantiate context for speech recognition")]
    LoadWhisperCtxFailed {
        #[source]
        source: whisper_rs::WhisperError,
    },
    #[error("failed to create state for speech recognition")]
    CreateWhisperStateFailed {
        #[source]
        source: whisper_rs::WhisperError,
    },
    #[error("failed to create a reader for recorded audio")]
    CreateWavReaderFailed {
        #[source]
        source: hound::Error,
    },
    #[error("failed to parse recorded audio sample (try again, but your microphone may not be compatible with Sotto)")]
    ParseSampleFailed {
        #[source]
        source: hound::Error,
    },
    #[error("failed to run speech recognition model")]
    WhisperRunFailed {
        #[source]
        source: whisper_rs::WhisperError,
    },
    #[error("failed to get number of segments in speech recognition output")]
    GetNumSegmentsFailed {
        #[source]
        source: whisper_rs::WhisperError,
    },
    #[error("failed to extract text from speech recognition output")]
    GetSegmentTextFailed {
        #[source]
        source: whisper_rs::WhisperError,
    },
    #[error(
        "couldn't find a home directory (please execute this program on a sane operating system)"
    )]
    NoHomeDir,
    #[error("failed to download model (are you connected to the internet?)")]
    DownloadModelFailed {
        #[source]
        source: reqwest::Error,
    },
    #[error("downloading model failed with http status code {status}")]
    DownloadModelBadStatus { status: u16 },
    #[error("failed to create a file for the downloaded model")]
    CreateModelFileFailed {
        #[source]
        source: std::io::Error,
    },
    #[error("found bad chunk in downloaded model response")]
    BadChunk {
        #[source]
        source: reqwest::Error,
    },
    #[error("failed to write chunk of downloaded model to file")]
    WriteChunkFailed {
        #[source]
        source: std::io::Error,
    },
    #[error("failed to get model index to download model (are you connected to the internet?)")]
    GetModelIndexFailed { source: reqwest::Error },
    #[error("getting model index failed with http status code {status}")]
    GetModelIndexBadStatus { status: u16 },
    #[error(transparent)]
    ModelIndexError(#[from] ModelIndexError),
    #[error("failed to create `.sotto` directory in your home directory for storing models")]
    CreateSottoDirFailed {
        #[source]
        source: std::io::Error,
    },
}

/// Errors that can occur with the model index. These are specifically errors that herald a severe
/// problem with the model index which should be immediately reported to prevent issues for
/// all users of Sotto worldwide.
#[derive(Debug, Error)]
pub enum ModelIndexError {
    #[error("failed to parse global model index (this is a bug in Sotto, and will be fixed as soon as possible)")]
    ParseFailed {
        #[source]
        source: serde_json::Error,
    },
    #[error("the global model index is missing a key (this is a bug in Sotto, and will be fixed as soon as possible)")]
    Incomplete { missing_key: &'static str },
}
