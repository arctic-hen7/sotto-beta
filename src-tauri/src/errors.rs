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
}
