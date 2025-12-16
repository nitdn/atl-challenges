use std::{array::TryFromSliceError, f32, time::Duration};

#[derive(Debug, Clone, Copy)]
enum AudioFormat {
    Float = 3,
}

trait ToRIFFBytes<const N: usize> {
    fn to_riff_bytes(&self) -> [u8; N];
}

impl ToRIFFBytes<2> for u16 {
    fn to_riff_bytes(&self) -> [u8; 2] {
        self.to_le_bytes()
    }
}

impl ToRIFFBytes<4> for u32 {
    fn to_riff_bytes(&self) -> [u8; 4] {
        self.to_le_bytes()
    }
}

type FourCC<'a> = &'a [u8; 4];
impl ToRIFFBytes<4> for FourCC<'_> {
    fn to_riff_bytes(&self) -> [u8; 4] {
        **self
    }
}

#[derive(Debug)]
pub struct RIFFHeader<'a> {
    file_type: FourCC<'a>,
    file_size: u32,
    file_format: FourCC<'a>,

    format_block_id: FourCC<'a>,
    block_size: u32,
    audio_format: u16,
    nbr_channels: u16,
    sample_rate_bytes: u32,
    byte_per_sec: u32,
    byte_per_block: u16,
    bits_per_sample: u16,

    data_block_id: FourCC<'a>,
    data_size: u32,
}

macro_rules! chain_riff_headers {
    ($first:expr $(, $others:expr )* $(,)? ) => {
    {
        let iter = $first.to_riff_bytes().into_iter();
        $(
            let other = $others.to_riff_bytes().into_iter();
            let iter = iter.chain(other);
            ) *
        iter
        }
    };
}

impl<'a> RIFFHeader<'a> {
    /// Prints out bytes from a constructed
    /// RIFF header
    pub fn to_header(&self) -> impl Iterator<Item = u8> {
        chain_riff_headers!(
            self.file_type,
            self.file_size,
            self.file_format,
            self.format_block_id,
            self.block_size,
            self.audio_format,
            self.nbr_channels,
            self.sample_rate_bytes,
            self.byte_per_sec,
            self.byte_per_block,
            self.bits_per_sample,
            self.data_block_id,
            self.data_size,
        )
    }
    /// Build a header from bytes
    ///
    /// # Errors
    ///
    /// This function will return an error if the conversion fails
    pub fn from_bytes(header: &'a [u8]) -> Result<Self, TryFromSliceError> {
        let instance = Self {
            file_type: header[0..4].try_into()?,
            file_size: u32::from_le_bytes(header[4..8].try_into()?),
            file_format: header[8..12].try_into()?,
            format_block_id: header[12..16].try_into()?,
            block_size: u32::from_le_bytes(header[16..20].try_into()?),
            audio_format: u16::from_le_bytes(header[20..22].try_into()?),
            nbr_channels: u16::from_le_bytes(header[22..24].try_into()?),
            sample_rate_bytes: u32::from_le_bytes(header[24..28].try_into()?),
            byte_per_sec: u32::from_le_bytes(header[28..32].try_into()?),
            byte_per_block: u16::from_le_bytes(header[32..34].try_into()?),
            bits_per_sample: u16::from_le_bytes(header[34..36].try_into()?),
            data_block_id: header[36..40].try_into()?,
            data_size: u32::from_le_bytes(header[40..44].try_into()?),
        };
        Ok(instance)
    }

    /// Generates a wav header from a given size
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn wav_header(sample_rate: u32, sample_count: u32) -> Self {
        let bit_depth = size_of::<f32>() as u16;
        let pcm_size = u32::from(bit_depth) * sample_count;

        Self {
            file_type: b"RIFF",
            file_size: (pcm_size + 44) - 8,
            file_format: b"WAVE",
            format_block_id: b"fmt ",
            block_size: 16u32, // 16 bytes metadata block
            audio_format: AudioFormat::Float as u16,
            nbr_channels: 1u16, // mono audio
            sample_rate_bytes: sample_rate,
            byte_per_sec: sample_rate * u32::from(bit_depth),
            byte_per_block: bit_depth,
            bits_per_sample: bit_depth * 8,
            data_block_id: b"data",
            data_size: pcm_size,
        }
    }
}

#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn sine_sample(frequency: f32, sample_rate: u32, current_sample: u32) -> f32 {
    let time = frequency / sample_rate as f32 * current_sample as f32;
    (2f32 * f32::consts::PI * time).sin()
}

#[must_use]
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn sine_wave(frequency: f32, sample_rate: u32, duration: Duration) -> Vec<u8> {
    (0..(sample_rate as f32 * duration.as_secs_f32()) as u32)
        .flat_map(|current_sample| {
            sine_sample(frequency, sample_rate, current_sample).to_le_bytes()
        })
        .collect()
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
        let mut buf: Vec<_> = RIFFHeader::wav_header(44100, 0).to_header().collect();
        buf.extend(items.iter());
        let len_bytes: [u8; 4] = buf[4..8].try_into().unwrap();
        assert_eq!(buf.len(), 44);
        assert_eq!(u32::from_le_bytes(len_bytes), 44 - 8);
    }
}
