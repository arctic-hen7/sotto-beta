use crate::errors::Error;
use crate::model::Model;
use std::future::Future;
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;
use tokio::sync::oneshot;
use whisper_rs::WhisperContext;

/// The app's state on the backend.
pub struct AppState {
    /// A sender that can be used to terminate the dictation process if it's
    /// ongoing. Note that this might still be `Some(_)` if recording has been
    /// completed, but if transcription is still ongoing.
    dictation: Arc<Mutex<DictationState>>,
    /// The Whisper context, cached for all uses. A new state will be created for
    /// each use.
    ///
    /// This will be maintained for the lifetime of the app.
    whisper_ctx: &'static WhisperContext,
}
impl AppState {
    /// Creates a new [`AppState`] using the given model as the default for all transcriptions.
    /// This model will be loaded and cached immediately.
    pub async fn new(dflt_model: Model) -> Result<Self, Error> {
        let model_path = dflt_model.get_or_download().await?;
        // Load the desired model
        let ctx = WhisperContext::new(&model_path.to_string_lossy())
            .map_err(|err| Error::LoadWhisperCtxFailed { source: err })?;
        let ctx = Box::leak(Box::new(ctx));

        Ok(Self {
            dictation: Arc::new(Mutex::new(DictationState::None)),
            whisper_ctx: ctx,
        })
    }
    /// Executes a dictation. This is not in itself asynchronous, but will
    /// return a future that will resolve when transcription is complete. Once
    /// this function returns that future (*before* its resolution), the app
    /// state will have been modified to be prepared for a signal to end
    /// a recording.
    pub fn dictate(&self) -> Result<impl Future<Output = Result<String, Error>>, Error> {
        // TODO Recover to `None` if poisoned
        let mut dictation_sender = self.dictation.lock().unwrap();
        if let DictationState::None = &*dictation_sender {
            let path =
                NamedTempFile::new().map_err(|err| Error::TmpFileCreationFailed { source: err })?;
            // Create a channel for terminating the recording and starting
            // transcription
            let (tx, rx) = oneshot::channel::<()>();
            *dictation_sender = DictationState::Recording(tx);
            drop(dictation_sender);

            // And a thread to perform both in sequence
            let dictation_sender = self.dictation.clone();
            let whisper_ctx = self.whisper_ctx;
            let task = tokio::task::spawn_blocking(move || {
                // This will complete when the receiver gets a signal
                crate::record::start_recording(&path.path(), rx)?;
                // NOTE: We aren't responsible for the state change from `Recording` -> `Transcribing`,
                // that has to be done by whoever sends the signal to end the recording, in order
                // to actually access the underlying sender (Rust's ownership system enforces this!).

                let result = crate::transcribe::transcribe(&path.path(), whisper_ctx)?;
                // Update the state so we're ready to finish up
                // TODO Poisoning doesn't matter (and really should be impossible...)
                *dictation_sender.lock().unwrap() = DictationState::None;

                Ok::<String, Error>(result)
            });

            // Morph this into a future that makes the errors neater from joining to
            // a blocking task
            let task_fut = async move {
                let res = task
                    .await
                    .map_err(|err| Error::DictationTaskPanicked { source: err })?;
                res
            };

            Ok(task_fut)
        } else {
            Err(Error::AlreadyDictating)
        }
    }
    /// Sends a signal to the dictation thread to end recording and begin transcription.
    /// It is assumed that the caller of the original dictation will still be holding the
    /// future that will yield the actual transcription result.
    pub async fn end_recording(&self) -> Result<(), Error> {
        // TODO Recover to `None`
        let mut dictation_sender = self.dictation.lock().unwrap();
        if let DictationState::Recording(_) = &*dictation_sender {
            // We need the actual sender itself, which will be consumed by this call
            let state = std::mem::replace(&mut *dictation_sender, DictationState::Transcribing);
            if let DictationState::Recording(sender) = state {
                // If the receiver has been dropped, the error will be received by the holder
                // of the dictation thread future (so we'll ignore it here)
                let _ = sender.send(());
            } else {
                unreachable!();
            }

            Ok(())
        } else {
            Err(Error::NotDictating)
        }
    }
}

enum DictationState {
    Recording(oneshot::Sender<()>),
    Transcribing,
    None,
}
