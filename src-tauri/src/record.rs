//! The core recording logic of Sotto.
//!
//! This code is largely taken from https://github.com/RustAudio/cpal/blob/master/examples/record_wav.rs.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};
use hound::{WavWriter, WavSpec};
use tokio::sync::oneshot::Receiver;

/// Begin a new recording. This will not actually terminate until the given receiver instructs it to.
///
/// This is a blocking function!
pub fn start_recording(path: &str, rx: Receiver<()>) -> Result<(), String> {
    let host = cpal::default_host();
    let device = host.default_input_device().ok_or(String::from("failed to find input device (do you have a microphone plugged in?)"))?;

    let config = device
        .default_input_config()
        .map_err(|_| String::from("failed to get default input configuration"))?;

    let spec = WavSpec {
        channels: config.channels() as _,
        sample_rate: config.sample_rate().0 as _,
        bits_per_sample: (config.sample_format().sample_size() * 8) as _,
        sample_format: if config.sample_format().is_float() {
            hound::SampleFormat::Float
        } else {
            hound::SampleFormat::Int
        },
    };
    let writer = WavWriter::create(path, spec).map_err(|_| String::from("failed to create audio writer"))?;
    let writer = Arc::new(Mutex::new(Some(writer)));

    // Run the input stream on a separate thread
    let writer_2 = writer.clone();

    // TODO
    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };

    // Dropping this will terminate the recording
    let stream = match config.sample_format() {
        cpal::SampleFormat::I8 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i8, i8>(data, &writer_2),
            err_fn,
            None,
        ).map_err(|_| String::from("failed to build input stream"))?,
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i16, i16>(data, &writer_2),
            err_fn,
            None,
        ).map_err(|_| String::from("failed to build input stream"))?,
        cpal::SampleFormat::I32 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i32, i32>(data, &writer_2),
            err_fn,
            None,
        ).map_err(|_| String::from("failed to build input stream"))?,
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<f32, f32>(data, &writer_2),
            err_fn,
            None,
        ).map_err(|_| String::from("failed to build input stream"))?,
        sample_format => {
            return Err(format!(
                "unsupported sample format '{sample_format}'"
            ))
        }
    };

    stream.play().map_err(|_| String::from("failed to start audio stream"))?;

    // This function will now *synchronously* wait for the termination signal.
    // This would be an error if the sender were dropped, which could indicate
    // messy termination. We should termiante anyway.
    //
    // Waiting asynchronously is impossible because `stream` is not `Send`.
    let _ = rx.blocking_recv();

    drop(stream);
    writer.lock().unwrap().take().unwrap().finalize().expect("failed to terminate audio writer");

    Ok(())
}

type WavWriterHandle = Arc<Mutex<Option<WavWriter<BufWriter<File>>>>>;

fn write_input_data<T, U>(input: &[T], writer: &WavWriterHandle)
where
    T: Sample,
    U: Sample + hound::Sample + FromSample<T>,
{
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in input.iter() {
                let sample: U = U::from_sample(sample);
                writer.write_sample(sample).ok();
            }
        }
    }
}
