use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use clap::Parser;

const SAMPLE_RATE: u32 = 44100;
const AMPLITUDE: f32 = 0.5;

/// A simple FM synthesizer
#[derive(Parser, Debug)]
#[command(name = "fm_synth")]
#[command(about = "A simple FM synthesizer with configurable carrier and modulator frequencies.")]
struct Cli {
    /// Carrier frequency in Hz (e.g., 440.0 for A4)
    #[arg(short, long, default_value_t = 440.0)]
    carrier_freq: f32,

    /// Modulator frequency in Hz (e.g., 220.0)
    #[arg(short, long, default_value_t = 220.0)]
    modulator_freq: f32,

    /// Modulation index (e.g., 5.0)
    #[arg(short, long, default_value_t = 5.0)]
    amount_modulation: f32,

    /// Duration of the sound in seconds (e.g., 2.0)
    #[arg(short, long, default_value_t = 2.0)]
    duration: f32,
}

fn main() {
    // Parse command-line arguments
    let args = Cli::parse();

    // Print the selected parameters
    println!("Carrier frequency: {} Hz", args.carrier_freq);
    println!("Modulator frequency: {} Hz", args.modulator_freq);
    println!("Modulation index: {}", args.amount_modulation);
    println!("Duration: {} seconds", args.duration);

    // Initialize the audio host and device
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device available");
    let config = device.default_output_config().expect("Failed to get default output config");

    // Print the sample rate from the config
    println!("Sample rate: {} Hz", config.sample_rate().0);

    // Create a shared buffer for audio data
    let audio_buffer = Arc::new(Mutex::new(Vec::new()));

    // Generate audio data
    let num_samples = (SAMPLE_RATE as f32 * args.duration) as usize;
    let mut phase = 0.0;

    for _ in 0..num_samples {
        // Modulator signal
        let modulator = (phase * args.modulator_freq * 2.0 * PI / SAMPLE_RATE as f32).sin();

        // Carrier signal with frequency modulation
        let carrier_phase = phase * args.carrier_freq * 2.0 * PI / SAMPLE_RATE as f32
            + args.amount_modulation * modulator;

        let sample = carrier_phase.sin() * AMPLITUDE;

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
    let audio_buffer_clone = Arc::clone(&audio_buffer);
    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let buffer = audio_buffer_clone.lock().unwrap();
            for (i, sample) in buffer.iter().enumerate().take(data.len()) {
                data[i] = *sample;
            }
        },
        move |err| {
            eprintln!("An error occurred on the audio stream: {}", err);
        },
    ).unwrap();

    stream.play().unwrap();
    std::thread::sleep(Duration::from_secs_f32(args.duration));
}
