use std::{
    fs::File,
    io::{self, BufReader, Error},
    path::PathBuf,
    time::Duration,
};

use clap::Parser;
use dsp::get_header;
use sdl3::audio::{AudioFormat, AudioSpec};

#[derive(Parser, Debug)]
struct Args {
    input_path: PathBuf,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let path = args.input_path;
    let mut input_file = BufReader::new(File::open(path)?);
    let mut header: [u8; 44] = [0; 44];
    let header = get_header(&mut input_file, &mut header)?;
    println!("{header:#?}");
    crazy_sdl3_demo();

    Ok(())
}

fn crazy_sdl3_demo() {
    use sdl3::audio::{AudioCallback, AudioFormat, AudioSpec, AudioStream};
    use std::time::Duration;

    struct SquareWave {
        phase_inc: f32,
        phase: f32,
        volume: f32,
    }

    impl AudioCallback<f32> for SquareWave {
        fn callback(&mut self, stream: &mut AudioStream, requested: i32) {
            let mut out = Vec::<f32>::with_capacity(requested as usize);
            // Generate a square wave
            for _ in 0..requested {
                out.push(if self.phase <= 0.5 {
                    self.volume
                } else {
                    -self.volume
                });
                self.phase = (self.phase + self.phase_inc) % 1.0;
            }
            stream.put_data_f32(&out);
        }
    }

    let sdl_context = sdl3::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let source_freq = 44100;
    let source_spec = AudioSpec {
        freq: Some(source_freq),
        channels: Some(1),                    // mono
        format: Some(AudioFormat::f32_sys()), // floating 32 bit samples
    };

    // Initialize the audio callback
    let device = audio_subsystem
        .open_playback_stream(
            &source_spec,
            SquareWave {
                phase_inc: 440.0 / source_freq as f32,
                phase: 0.0,
                volume: 0.25,
            },
        )
        .unwrap();

    // Start playback
    device.resume().expect("Failed to start playback");

    // Play for 2 seconds
    std::thread::sleep(Duration::from_millis(2000));
}
