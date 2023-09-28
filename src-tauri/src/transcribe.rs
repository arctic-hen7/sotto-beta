use crate::errors::Error;
use hound::WavReader;
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};

/// Transcribes the audio in the given file to a string of text.
pub fn transcribe(file: &Path) -> Result<String, Error> {
    // Load a context and model.
    let ctx = WhisperContext::new("models/ggml-base.en.bin").expect("failed to load model");
    // Create a state
    let mut state = ctx.create_state().expect("failed to create key");

    // Create a params object for running the model.
    // The number of past samples to consider defaults to 0.
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 0 });

    // Edit params as needed.
    // Set the number of threads to use to 1.
    params.set_n_threads(4); // TODO
    params.set_translate(false);
    params.set_language(Some("en")); // TODO
                                     // Disable anything that prints to stdout.
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    // Open the audio file (we've already guaranteed that this is mono f32 audio in 16kHz)
    let mut reader = WavReader::open(file).expect("failed to open file");
    let audio = reader
        .samples::<f32>()
        .map(|s| s.expect("TODO"))
        .collect::<Vec<_>>();

    // Run the model.
    state.full(params, &audio[..]).expect("failed to run model");

    // Iterate through the segments of the transcript.
    let num_segments = state
        .full_n_segments()
        .expect("failed to get number of segments");
    let mut segments = Vec::new();
    for i in 0..num_segments {
        // Get the transcribed text and timestamps for the current segment.
        let segment = state
            .full_get_segment_text(i)
            .expect("failed to get segment");
        segments.push(segment);
    }

    let full_text = segments.join("");

    Ok(full_text)
}
