use std::f32;

enum AudioFormat {
    Float = 3,
}

pub struct RIFFHeader {
    file_type: &'static [u8; 4],
    file_size: [u8; 4],
    file_format: &'static [u8; 4],

    format_block_id: &'static [u8; 4],
    block_size: [u8; 4],
    audio_format: [u8; 2],
    nbr_channels: [u8; 2],
    sample_rate_bytes: [u8; 4],
    byte_per_sec: [u8; 4],
    byte_per_block: [u8; 2],
    bits_per_sample: [u8; 2],

    data_block_id: &'static [u8; 4],
    data_size: [u8; 4],
}
impl RIFFHeader {
    /// Prints out bytes from a constructed
    /// RIFF header
    fn to_header(&self) -> Vec<u8> {
        [
            &self.file_type[..],
            &self.file_size[..],
            &self.file_format[..],
            &self.format_block_id[..],
            &self.block_size[..],
            &self.audio_format[..],
            &self.nbr_channels[..],
            &self.sample_rate_bytes[..],
            &self.byte_per_sec[..],
            &self.byte_per_block[..],
            &self.bits_per_sample[..],
            &self.data_block_id[..],
            &self.data_size[..],
        ]
        .concat()
    }
    /// Generates a wav header from a given size
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn wav_header(sample_rate: u32, sample_count: u32) -> Vec<u8> {
        let file_type = b"RIFF";
        let file_format = b"WAVE";
        let format_block_id = b"fmt ";
        let block_size = 16u32.to_le_bytes(); // 16 bytes wav chunk
        let audio_format = (AudioFormat::Float as u16).to_le_bytes();
        let nbr_channels = 1u16.to_le_bytes(); // mono_audio
        let sample_rate_bytes = sample_rate.to_le_bytes();
        let bit_depth = size_of::<f32>() as u16;
        let byte_per_sec = (sample_rate * u32::from(bit_depth)).to_le_bytes();
        let byte_per_block = bit_depth.to_le_bytes();
        let bits_per_sample = (bit_depth * 8).to_le_bytes();
        let data_block_id = b"data";
        let pcm_size = u32::from(bit_depth) * sample_count;

        let data_size = pcm_size.to_le_bytes();
        let file_size = ((pcm_size + 44) - 8).to_le_bytes();
        let header = Self {
            file_type,
            file_size,
            file_format,
            format_block_id,
            block_size,
            audio_format,
            nbr_channels,
            sample_rate_bytes,
            byte_per_sec,
            byte_per_block,
            bits_per_sample,
            data_block_id,
            data_size,
        };
        header.to_header()
    }
}

/// Lossily coverts u32 to f32 as 23 bits is probably enough fidelity
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn sine_sample(frequency: u16, sample_rate: u32, current_sample: u32) -> f32 {
    let time = f32::from(frequency) / sample_rate as f32 * current_sample as f32;
    (2f32 * f32::consts::PI * time).sin()
}
#[must_use]
pub const fn square_sample(sine_value: f32) -> f32 {
    sine_value.signum()
}
#[must_use]
pub fn triangle_sample(sine_value: f32) -> f32 {
    2f32 / f32::consts::PI * sine_value.asin()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_byte_wave() {
        let items: [u8; 0] = [];
        let mut buf = RIFFHeader::wav_header(44100, 0);
        buf.extend(items.iter());
        let len_bytes: [u8; 4] = buf[4..8].try_into().unwrap();
        assert_eq!(buf.len(), 44);
        assert_eq!(u32::from_le_bytes(len_bytes), 44 - 8);
    }
}
