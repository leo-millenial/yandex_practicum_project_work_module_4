pub mod error;
pub mod plugin_loader;

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use common::{ALPHA_OPAQUE, BYTES_PER_PIXEL, RGB_CHUNK_SIZE};
use error::ImageError;
use png::{ColorType, Decoder, Encoder};

#[derive(Debug, Clone)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub rgba_data: Vec<u8>,
}

impl Image {
    pub fn new(width: u32, height: u32, rgba_data: Vec<u8>) -> Result<Self, ImageError> {
        let expected_size = (width as usize) * (height as usize) * BYTES_PER_PIXEL;
        if rgba_data.len() != expected_size {
            return Err(ImageError::SizeMismatch {
                expected: expected_size,
                actual: rgba_data.len(),
            });
        }
        Ok(Self {
            width,
            height,
            rgba_data,
        })
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ImageError> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(ImageError::LoadError(format!(
                "File does not exist: {}",
                path.display()
            )));
        }

        let file = File::open(path).map_err(|e| {
            ImageError::LoadError(format!("Failed to open file '{}': {}", path.display(), e))
        })?;

        let decoder = Decoder::new(BufReader::new(file));
        let mut reader = decoder
            .read_info()
            .map_err(|e| ImageError::LoadError(format!("Failed to read PNG info: {}", e)))?;

        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader
            .next_frame(&mut buf)
            .map_err(|e| ImageError::LoadError(format!("Failed to decode PNG: {}", e)))?;

        let bytes = &buf[..info.buffer_size()];

        match info.color_type {
            ColorType::Rgba => {
                let rgba_data = bytes.to_vec();
                Ok(Self {
                    width: info.width,
                    height: info.height,
                    rgba_data,
                })
            }
            ColorType::Rgb => {
                let rgb_data = bytes.to_vec();
                let mut rgba_data = Vec::with_capacity(
                    (info.width * info.height * BYTES_PER_PIXEL as u32) as usize,
                );

                for chunk in rgb_data.chunks(RGB_CHUNK_SIZE) {
                    let [r, g, b] = [chunk[0], chunk[1], chunk[2]];
                    rgba_data.extend_from_slice(&[r, g, b, ALPHA_OPAQUE]);
                }

                Ok(Self {
                    width: info.width,
                    height: info.height,
                    rgba_data,
                })
            }
            _ => Err(ImageError::InvalidFormat(format!(
                "Unsupported color type: {:?}. Only RGB and RGBA are supported.",
                info.color_type
            ))),
        }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), ImageError> {
        let path = path.as_ref();

        let file = File::create(path).map_err(|e| {
            ImageError::SaveError(format!("Failed to create file '{}': {}", path.display(), e))
        })?;

        let mut encoder = Encoder::new(BufWriter::new(file), self.width, self.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder
            .write_header()
            .map_err(|e| ImageError::SaveError(format!("Failed to write PNG header: {}", e)))?;

        writer
            .write_image_data(&self.rgba_data)
            .map_err(|e| ImageError::SaveError(format!("Failed to write PNG data: {}", e)))?;

        Ok(())
    }

    pub fn rgba_slice(&self) -> &[u8] {
        &self.rgba_data
    }

    pub fn rgba_slice_mut(&mut self) -> &mut [u8] {
        &mut self.rgba_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_new_valid() {
        let data = vec![0u8; 4 * 4 * BYTES_PER_PIXEL];
        let img = Image::new(4, 4, data).unwrap();
        assert_eq!(img.width, 4);
        assert_eq!(img.height, 4);
    }

    #[test]
    fn test_image_new_invalid_size() {
        let data = vec![0u8; 10];
        let result = Image::new(4, 4, data);
        assert!(result.is_err());
    }
}
