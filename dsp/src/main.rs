use std::{fs::File, io::Write};

use dsp::{RIFFHeader, sine_sample, triangle_sample};

fn main() -> std::io::Result<()> {
    let duration = 5; // Dont use time because 64 bit
    let sample_rate = 44100;
    let frequency = 200;
    let sample_count = sample_rate * 5;
    let header = &RIFFHeader::wav_header(sample_rate, sample_count);
    let mut file = File::create("./result/sine.wav")?;
    let mut file2 = File::create("./result/square.wav")?;
    let mut file3 = File::create("./result/triangle.wav")?;
    file.write_all(header)?;
    file2.write_all(header)?;
    file3.write_all(header)?;
    for _ in 0..duration {
        for current_sample in 0..sample_rate {
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
