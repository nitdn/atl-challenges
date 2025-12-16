use std::{
    fs::{File, write},
    io::{Read, Write},
    time::Duration,
};

use dsp::{RIFFHeader, sine_sample, sine_wave, triangle_sample};

fn main() -> std::io::Result<()> {
    let mut input_file = File::open("./result/input.wav")?;
    let mut buf: [u8; 44] = [0; 44];
    input_file.read_exact(&mut buf)?;
    let _ = dbg!(RIFFHeader::from_bytes(&buf));
    let duration = 5; // Dont use time because 64 bit
    let sample_rate = 44100;
    let frequency = 200.0;
    let sample_count = duration * sample_rate;
    let header: Vec<u8> = RIFFHeader::wav_header(sample_rate, sample_count)
        .to_header()
        .collect();
    let mut file = File::create("./result/sine.wav")?;
    let mut file2 = File::create("./result/square.wav")?;
    let mut file3 = File::create("./result/triangle.wav")?;
    file.write_all(&header)?;
    file2.write_all(&header)?;
    file3.write_all(&header)?;
    let duration2 = Duration::from_millis(500);
    let sine_waves = sine_wave(frequency, sample_rate, duration2);
    #[allow(clippy::cast_possible_truncation)]
    let header2 = RIFFHeader::wav_header(sample_rate, (sine_waves.len() / 4) as u32);

    write(
        "./result/sine2.wav",
        [header2.to_header().collect(), sine_waves].concat(),
    )?;
    for _ in 0..duration {
        for current_sample in 0..(sample_rate) {
            let sine_signal = sine_sample(frequency, sample_rate, current_sample);
            let square_signal = sine_signal.signum();
            let triangle_signal = triangle_sample(sine_signal);

            file.write_all(&sine_signal.to_le_bytes())?;
            file2.write_all(&square_signal.to_le_bytes())?;
            file3.write_all(&triangle_signal.to_le_bytes())?;
        }
    }
    Ok(())
}
