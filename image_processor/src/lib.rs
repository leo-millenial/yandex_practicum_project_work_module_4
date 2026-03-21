pub mod error;
pub mod plugin_loader;

use std::path::Path;

use common::BYTES_PER_PIXEL;
use error::ImageError;
use image::{DynamicImage, ImageBuffer, Rgba};

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

        let img: DynamicImage = image::open(path).map_err(|e| {
            ImageError::LoadError(format!(
                "Failed to open image '{}': {}. Supported formats: PNG, JPEG, GIF, BMP, TIFF, WebP",
                path.display(),
                e
            ))
        })?;

        // Convert to RGBA8
        let rgba_img = img.into_rgba8();
        let (width, height) = rgba_img.dimensions();
        let rgba_data = rgba_img.into_raw();

        Ok(Self {
            width,
            height,
            rgba_data,
        })
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), ImageError> {
        let path = path.as_ref();

        let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_raw(self.width, self.height, self.rgba_data.clone()).ok_or_else(
                || {
                    ImageError::SaveError(format!(
                        "Failed to create image buffer for {}x{} image",
                        self.width, self.height
                    ))
                },
            )?;

        img.save(path).map_err(|e| {
            ImageError::SaveError(format!("Failed to save image '{}': {}", path.display(), e))
        })
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
