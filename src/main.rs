use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const SAMPLE_RATE: u32 = 44100;
const AMPLITUDE: f32 = 0.5;

fn main() {
    // Initialize the audio host and device
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device available");
    let config = device.default_output_config().expect("Failed to get default output config");

    // Create a shared buffer for audio data
    let audio_buffer = Arc::new(Mutex::new(Vec::new()));

    // Define the FM synthesizer parameters
    let carrier_freq = 440.0; // A4 note
    let modulator_freq = 220.0;
    let modulation_index = 5.0;

    // Generate audio data
    let duration = 2.0; // 2 seconds
    let num_samples = (SAMPLE_RATE as f32 * duration) as usize;
    let mut phase = 0.0;

    for _ in 0..num_samples {
        let modulator = (phase * modulator_freq * 2.0 * PI / SAMPLE_RATE as f32).sin();
        let carrier = (phase * carrier_freq * 2.0 * PI / SAMPLE_RATE as f32 + modulation_index * modulator).sin();
        let sample = carrier * AMPLITUDE;

        audio_buffer.lock().unwrap().push(sample);
        phase += 1.0;
    }

    // Write audio data to a WAV file
    let spec = WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = WavWriter::create("fm_synth.wav", spec).unwrap();
    for sample in audio_buffer.lock().unwrap().iter() {
        writer.write_sample((sample * i16::MAX as f32) as i16).unwrap();
    }
    writer.finalize().unwrap();

    // Play audio using cpal
    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for (i, sample) in audio_buffer.lock().unwrap().iter().enumerate().take(data.len()) {
                data[i] = *sample;
            }
        },
        move |err| {
            eprintln!("An error occurred on the audio stream: {}", err);
        },
    ).unwrap();

    stream.play().unwrap();
    std::thread::sleep(Duration::from_secs_f32(duration));
}

