// src/conversion.rs

use anyhow::{Context, anyhow};
use heic::{DecodeOutput, DecoderConfig, PixelLayout};
use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};
use std::io::Cursor;

pub fn heic_decode(image_bytes: &[u8]) -> Result<DecodeOutput, anyhow::Error> {
    let decode_output = DecoderConfig::new()
        .decode(image_bytes, PixelLayout::Rgba8)
        .context("Failed to decode the HEIC image")?;

    Ok(decode_output)
}

pub fn pixels_to_imagebuffer(
    width: u32,
    height: u32,
    buffer: Vec<u8>,
) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    ImageBuffer::from_raw(width, height, buffer)
}

pub fn convert_to_jpg(image: RgbaImage) -> Result<Vec<u8>, anyhow::Error> {
    let mut jpeg_bytes = Vec::new();
    let rgb = DynamicImage::from(image).into_rgb8();
    rgb.write_to(&mut Cursor::new(&mut jpeg_bytes), image::ImageFormat::Jpeg).context("Unable to convert to jpg")?;

    Ok(jpeg_bytes)
}

/// Convert HEIC bytes to JPEG bytes.
///
/// This is the single entry point the upload handler calls. It chains the
/// decode → pixel buffer → encode pipeline, propagating errors instead of
/// panicking. A corrupted or non-HEIC input yields `Err`, not a crash.
pub fn convert_heic_to_jpeg(heic_bytes: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let decoded = heic_decode(heic_bytes)?;

    let image_buffer = pixels_to_imagebuffer(decoded.width, decoded.height, decoded.data)
        .ok_or_else(|| anyhow!("Pixel buffer did not match image dimensions"))?;

    convert_to_jpg(image_buffer)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn heic_decode_produces_non_zero_output() {
        // Arrange
        let heic_image_bytes = include_bytes!("../tests/fixtures/IMG_2215.HEIC");

        // Act
        let output = heic_decode(heic_image_bytes).unwrap();

        // Assert
        assert!(output.width > 0);
        assert!(output.height > 0);
        assert!(output.data.len() > 0);
    }

    #[test]
    fn heic_decode_produces_correct_data_length() {
        // Arrange
        let heic_image_bytes = include_bytes!("../tests/fixtures/IMG_2215.HEIC");

        // Act
        let output = heic_decode(heic_image_bytes).unwrap();

        // Assert
        let calculated_length = (output.width * output.height * 4) as usize;
        assert_eq!(output.data.len(), calculated_length);
    }

    #[test]
    fn convert_to_image_buffer_returns_an_rgba_image() {
        // Arrange
        let width = 2;
        let height = 2;
        let fake_buffer = vec![0u8; (width * height * 4) as usize];

        // Act
        let output = pixels_to_imagebuffer(width, height, fake_buffer).unwrap();

        // Assert
        assert_eq!(output.width(), width);
        assert_eq!(output.height(), height);
    }

    #[test]
    fn convert_to_jpg_returns_a_jpg_image() {
        // Arrange
        let img: RgbaImage = ImageBuffer::from_raw(2, 2, vec![0u8; 16]).unwrap();

        // Act
        let output = convert_to_jpg(img).unwrap();

        // Assert
        assert!(!output.is_empty());
        assert_eq!(output[0], 0xFF);
        assert_eq!(output[1], 0xD8);
    }

    #[test]
    fn convert_heic_to_jpeg_produces_valid_jpeg_from_heic_fixture() {
        // Arrange — the real HEIC fixture used across the test suite
        let heic_bytes = include_bytes!("../tests/fixtures/IMG_2215.HEIC");

        // Act — run the full pipeline
        let jpeg_bytes = convert_heic_to_jpeg(heic_bytes).unwrap();

        // Assert — the output is a valid JPEG (starts with the magic bytes)
        assert!(!jpeg_bytes.is_empty());
        assert_eq!(jpeg_bytes[0], 0xFF);
        assert_eq!(jpeg_bytes[1], 0xD8);
    }

    #[test]
    fn convert_heic_to_jpeg_returns_err_for_non_heic_input() {
        // Arrange — arbitrary bytes that aren't a valid HEIC image
        let garbage_bytes = b"definitely not an image";

        // Act
        let result = convert_heic_to_jpeg(garbage_bytes);

        // Assert — the pipeline returns an error instead of panicking
        assert!(result.is_err());
    }
}