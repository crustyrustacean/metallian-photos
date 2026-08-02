// tests/timing.rs — throwaway pipeline timing. Run with:
//   cargo test --test timing -- --nocapture --ignored

use bytes::Bytes;

use metallian_photos::exif::{get_raw_exif, parse_exif};
use std::time::Instant;

#[test]
#[ignore]
fn time_pipeline_steps() {
    let raw = include_bytes!("../tests/fixtures/IMG_2215.HEIC");
    println!(
        "fixture size: {} bytes ({:.1} KB)",
        raw.len(),
        raw.len() as f64 / 1024.0
    );

    // EXIF extraction
    let bytes = Bytes::copy_from_slice(raw);
    let t = Instant::now();
    let raw_exif = get_raw_exif(&bytes).expect("fixture should parse");
    let _exif = parse_exif(&raw_exif);
    println!(
        "EXIF extract:  {:>8.2} ms",
        t.elapsed().as_secs_f64() * 1000.0
    );

    // HEIC decode (the `heic` crate → raw RGBA pixels)
    let t = Instant::now();
    let decoded = metallian_photos::conversion::heic_decode(raw).expect("decode should succeed");
    let decode_ms = t.elapsed().as_secs_f64() * 1000.0;
    println!("HEIC decode:   {:>8.2} ms", decode_ms);
    println!(
        "dimensions:    {}x{} ({} px)",
        decoded.width,
        decoded.height,
        decoded.width as usize * decoded.height as usize
    );

    // JPEG encode (RGBA pixels → JPEG bytes)
    let image_buffer = metallian_photos::conversion::pixels_to_imagebuffer(
        decoded.width,
        decoded.height,
        decoded.data,
    )
    .expect("buffer should build");
    let t = Instant::now();
    let jpeg =
        metallian_photos::conversion::convert_to_jpg(image_buffer).expect("encode should succeed");
    let encode_ms = t.elapsed().as_secs_f64() * 1000.0;
    println!("JPEG encode:   {:>8.2} ms", encode_ms);
    println!("total convert: {:>8.2} ms", decode_ms + encode_ms);
    println!(
        "output JPEG:   {} bytes ({:.1} KB)",
        jpeg.len(),
        jpeg.len() as f64 / 1024.0
    );
}
