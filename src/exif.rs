// EXIF extraction.
//
// The kamadak-exif dependency is intentionally confined to this module — it does
// not leak into `domain`. Callers receive a plain `domain::Exif` value and stay
// unaware of the underlying EXIF library's types.

use crate::domain::Exif;
use bytes::Bytes;
use exif::{Error as ExifError, Exif as RawExif, In, Tag, Value};
use std::io;

/// return an EXIF type from the bytes of the photo
/// 
/// receive the raw photo bytes from the create handler and read them
/// to build the EXIF type

pub fn get_raw_exif(bytes: &Bytes) -> Result<RawExif, ExifError> {
    let mut cursor = io::Cursor::new(bytes);
    let exifreader = exif::Reader::new();
    let raw_exif = exifreader.read_from_container(&mut cursor)?;

    Ok(raw_exif)
}

/// Pull a single ASCII-valued tag out of the primary IFD, if present.
///
/// EXIF ASCII fields are stored as byte strings (often NUL-terminated); this
/// helper normalizes them to an owned `String`. Returns `None` when the tag is
/// absent or holds a non-ASCII value — missing tags are the common case, which
/// is exactly why `domain::Exif` carries `Option<String>` fields.
fn get_ascii(exif: &RawExif, tag: Tag) -> Option<String> {
    exif.get_field(tag, In::PRIMARY)
        .and_then(|field| match &field.value {
            Value::Ascii(v) => Some(String::from_utf8_lossy(&v[0]).to_string()),
            _ => None,
        })
}

/// Convert raw EXIF data into our domain `Exif`.
///
/// Every field is optional by design: real-world images omit tags constantly,
/// and the model should say that honestly rather than papering over it with
/// empty strings. This is a pure transform — no IO, no allocation beyond the
/// returned strings.
pub fn parse_exif(raw: &RawExif) -> Exif {
    Exif {
        date_time_original: get_ascii(raw, Tag::DateTimeOriginal),
        make: get_ascii(raw, Tag::Make),
        model: get_ascii(raw, Tag::Model),
        lens_make: get_ascii(raw, Tag::LensMake),
        lens_model: get_ascii(raw, Tag::LensModel),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_exif_extracts_known_tags_from_fixture() {
        // Arrange — bake the fixture image into the binary at compile time.
        let raw_bytes = include_bytes!("../tests/fixtures/IMG_2215.HEIC");
        let bytes = Bytes::copy_from_slice(raw_bytes);

        // Act — the same pipeline the create handler now runs.
        let raw = get_raw_exif(&bytes).expect("fixture should parse");
        let exif = parse_exif(&raw);

        // Assert — values captured from this fixture via photo-exif-reader.
        assert_eq!(exif.date_time_original.as_deref(), Some("2026:01:24 19:55:19"));
        assert_eq!(exif.make.as_deref(), Some("Apple"));
        assert_eq!(exif.model.as_deref(), Some("iPhone 16 Pro Max"));
        assert_eq!(exif.lens_make.as_deref(), Some("Apple"));
        assert_eq!(
            exif.lens_model.as_deref(),
            Some("iPhone 16 Pro Max back triple camera 15.66mm f/2.8")
        );
    }

    #[test]
    fn get_raw_exif_errors_on_non_image_bytes() {
        // Arrange - create some garbage bytes that aren't a image
        let bytes = Bytes::from_static(b"definitely not an image");

        // Act - try to convert the garbage bytes
        let result = get_raw_exif(&bytes);

        // Assert
        assert!(result.is_err(), "garbage bytes should not parse as EXIF");
    }
}
