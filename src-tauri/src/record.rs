use crate::errors::Error;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    StreamConfig,
};
use std::path::Path;
use tokio::sync::oneshot::Receiver;

pub fn start_recording(path: &Path, rx: Receiver<()>) -> Result<(), Error> {
    let host = cpal::default_host();
    let input_device = host.default_input_device().ok_or(Error::NoInputDevice)?;
    let dflt_config = input_device
        .default_input_config()
        .map_err(|err| Error::GetDefaultInputConfigFailed { source: err })?;

    // Initialize the WAV writer
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16_000,
        bits_per_sample: (dflt_config.sample_format().sample_size() * 8) as u16,
        sample_format: hound::SampleFormat::Float,
    };
    let config = StreamConfig {
        channels: 1,
        sample_rate: cpal::SampleRate(16_000),
        buffer_size: cpal::BufferSize::Default,
    };

    let mut writer = hound::WavWriter::create(path, spec)
        .map_err(|err| Error::CreateWavWriterFailed { source: err })?;

    // Initialize the CPAL audio input stream
    let input_stream = input_device
        .build_input_stream(
            &config,
            move |data: &[f32], _| {
                // Callback function to receive audio data
                for sample in data {
                    if let Err(err) = writer.write_sample(*sample) {
                        eprintln!("Error writing audio data to WAV file: {err:?}");
                    }
                }
            },
            |err| {
                // Error callback
                eprintln!("Error in audio stream: {:?}", err);
            },
            None,
        )
        .map_err(|err| Error::BuildInputStreamFailed { source: err })?;

    // Start the audio stream
    input_stream
        .play()
        .map_err(|err| Error::StartStreamFailed { source: err })?;

    // Wait for a signal from the receiver to stop recording.
    // If this fails, that just implies the app has started to terminate, and we should halt
    // anyway.
    let _ = rx.blocking_recv();

    // Stop and close the audio stream
    drop(input_stream);

    Ok(())
}
