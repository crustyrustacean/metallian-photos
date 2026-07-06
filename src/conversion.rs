// src/conversion.rs

use heic::{DecodeOutput, DecoderConfig, PixelLayout};
use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};
use std::io::Cursor;

pub fn heic_decode(image_bytes: &[u8]) -> DecodeOutput {
    DecoderConfig::new()
        .decode(&image_bytes, PixelLayout::Rgba8)
        .unwrap()
}

pub fn pixels_to_imagebuffer(
    width: u32,
    height: u32,
    buffer: Vec<u8>,
) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    ImageBuffer::from_raw(width, height, buffer)
}

pub fn convert_to_jpg(image: RgbaImage) -> Vec<u8> {
    let mut jpeg_bytes = Vec::new();
    let rgb = DynamicImage::from(image).into_rgb8();
    rgb.write_to(&mut Cursor::new(&mut jpeg_bytes), image::ImageFormat::Jpeg)
        .unwrap();

    jpeg_bytes
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn heic_decode_produces_non_zero_output() {
        // Arrange
        let heic_image_bytes = include_bytes!("../tests/fixtures/IMG_2215.HEIC");

        // Act
        let output = heic_decode(heic_image_bytes);

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
        let output = heic_decode(heic_image_bytes);

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
        let output = convert_to_jpg(img);

        // Assert
        assert!(!output.is_empty());
        assert_eq!(output[0], 0xFF);
        assert_eq!(output[1], 0xD8);
    }
}
