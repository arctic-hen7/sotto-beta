use crate::errors::Error;
use hound::WavReader;
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};

// TODO Model downloading
const MODEL_PATH: &str = "models/ggml-base.en.bin";

/// Transcribes the audio in the given file to a string of text.
pub fn transcribe(file: &Path) -> Result<String, Error> {
    // Load the model and create a state
    let ctx = WhisperContext::new(MODEL_PATH)
        .map_err(|err| Error::LoadWhisperCtxFailed { source: err })?;
    let mut state = ctx
        .create_state()
        .map_err(|err| Error::CreateWhisperStateFailed { source: err })?;

    // Sampling parameters for the model
    // TODO Configure differently?
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 0 });

    // BUG M1 reports cores weirdly...
    params.set_n_threads(num_cpus::get_physical() as i32);
    params.set_translate(false);
    params.set_language(Some("en"));
    // Disable any printing to stdout (this is what we get for a wrapper over C++!)
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    // Open the audio file (we've already guaranteed that this is mono f32 audio in 16kHz)
    let mut reader =
        WavReader::open(file).map_err(|err| Error::CreateWavReaderFailed { source: err })?;
    let mut audio = Vec::new();
    for sample in reader.samples::<f32>() {
        let sample = sample.map_err(|err| Error::ParseSampleFailed { source: err })?;
        audio.push(sample);
    }

    // Run the inference (this is blocking, and should be called in a blocking task)
    state
        .full(params, &audio[..])
        .map_err(|err| Error::WhisperRunFailed { source: err })?;

    // Iterate through the segments of the transcript to extract the actual text
    let num_segments = state
        .full_n_segments()
        .map_err(|err| Error::GetNumSegmentsFailed { source: err })?;
    let mut segments = Vec::new();
    for i in 0..num_segments {
        let segment = state
            .full_get_segment_text(i)
            .map_err(|err| Error::GetSegmentTextFailed { source: err })?;
        segments.push(segment);
    }
    let full_text = segments.join("");

    Ok(full_text)
}
