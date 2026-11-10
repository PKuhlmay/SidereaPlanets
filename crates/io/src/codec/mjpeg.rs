//! Motion JPEG (`MJPG`): every frame is a baseline JPEG, decoded with `zune-jpeg`.
//!
//! AVI writers may leave out the Huffman tables and rely on the standard tables of JPEG Annex K
//! (OpenDML); those are inserted before decoding when a frame has no DHT segment.
//! Interlaced MJPEG, where one chunk holds two field images, is rejected.

use super::{CodecError, FrameFormat};
use planets_core::Image;
use std::sync::OnceLock;
use zune_core::{bytestream::ZCursor, colorspace::ColorSpace, options::DecoderOptions};
use zune_jpeg::JpegDecoder;

const NAME: &str = "MJPEG";

fn corrupt(reason: impl Into<String>) -> CodecError {
    CodecError::Corrupt {
        codec: NAME,
        reason: reason.into(),
    }
}

// JPEG Annex K.3: standard Huffman tables as (code length counts, symbols).
const DC_LUMA: ([u8; 16], &[u8]) = (
    [0, 1, 5, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0],
    &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
);
const DC_CHROMA: ([u8; 16], &[u8]) = (
    [0, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0],
    &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
);
const AC_LUMA: ([u8; 16], &[u8]) = (
    [0, 2, 1, 3, 3, 2, 4, 3, 5, 5, 4, 4, 0, 0, 1, 0x7D],
    &[
        0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06, 0x13, 0x51, 0x61,
        0x07, 0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xA1, 0x08, 0x23, 0x42, 0xB1, 0xC1, 0x15, 0x52,
        0xD1, 0xF0, 0x24, 0x33, 0x62, 0x72, 0x82, 0x09, 0x0A, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x25,
        0x26, 0x27, 0x28, 0x29, 0x2A, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x43, 0x44, 0x45,
        0x46, 0x47, 0x48, 0x49, 0x4A, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x63, 0x64,
        0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x83,
        0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99,
        0x9A, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6,
        0xB7, 0xB8, 0xB9, 0xBA, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xD2, 0xD3,
        0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xE1, 0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8,
        0xE9, 0xEA, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA,
    ],
);
const AC_CHROMA: ([u8; 16], &[u8]) = (
    [0, 2, 1, 2, 4, 4, 3, 4, 7, 5, 4, 4, 0, 1, 2, 0x77],
    &[
        0x00, 0x01, 0x02, 0x03, 0x11, 0x04, 0x05, 0x21, 0x31, 0x06, 0x12, 0x41, 0x51, 0x07, 0x61,
        0x71, 0x13, 0x22, 0x32, 0x81, 0x08, 0x14, 0x42, 0x91, 0xA1, 0xB1, 0xC1, 0x09, 0x23, 0x33,
        0x52, 0xF0, 0x15, 0x62, 0x72, 0xD1, 0x0A, 0x16, 0x24, 0x34, 0xE1, 0x25, 0xF1, 0x17, 0x18,
        0x19, 0x1A, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x43, 0x44,
        0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x63,
        0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A,
        0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97,
        0x98, 0x99, 0x9A, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xB2, 0xB3, 0xB4,
        0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0xCA,
        0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7,
        0xE8, 0xE9, 0xEA, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA,
    ],
);

/// A complete DHT segment (marker, length and the four standard tables).
fn standard_dht() -> &'static [u8] {
    static DHT: OnceLock<Vec<u8>> = OnceLock::new();
    DHT.get_or_init(|| {
        let tables = [
            (0x00, DC_LUMA),
            (0x10, AC_LUMA),
            (0x01, DC_CHROMA),
            (0x11, AC_CHROMA),
        ];
        let mut body = Vec::new();
        for (class_and_id, (counts, symbols)) in tables {
            body.push(class_and_id);
            body.extend_from_slice(&counts);
            body.extend_from_slice(symbols);
        }
        let mut segment = vec![0xFF, 0xC4];
        segment.extend_from_slice(&((body.len() + 2) as u16).to_be_bytes());
        segment.extend_from_slice(&body);
        segment
    })
}

/// Walks the marker segments up to the scan. Returns the offset of the SOS marker when the frame
/// has no DHT segment before it, `None` when Huffman tables are present.
fn missing_dht_offset(data: &[u8]) -> Result<Option<usize>, CodecError> {
    if data.get(..2) != Some(&[0xFF, 0xD8]) {
        return Err(corrupt("missing SOI marker"));
    }
    let mut pos = 2;
    loop {
        // Markers may be preceded by any number of 0xFF fill bytes.
        while data.get(pos) == Some(&0xFF) && data.get(pos + 1) == Some(&0xFF) {
            pos += 1;
        }
        let (Some(&0xFF), Some(&marker)) = (data.get(pos), data.get(pos + 1)) else {
            return Err(corrupt("no scan found"));
        };
        match marker {
            0xC4 => return Ok(None),
            0xDA => return Ok(Some(pos)),
            // Standalone markers have no length field.
            0x01 | 0xD0..=0xD9 => pos += 2,
            _ => {
                let len = data
                    .get(pos + 2..pos + 4)
                    .map(|b| usize::from(u16::from_be_bytes([b[0], b[1]])))
                    .ok_or_else(|| corrupt("truncated marker segment"))?;
                pos += 2 + len;
            }
        }
    }
}

pub(super) fn decode(format: &FrameFormat, data: &[u8]) -> Result<Image, CodecError> {
    // A zero-length chunk is a dropped frame; there is nothing to repeat here, so it is an error.
    if data.is_empty() {
        return Err(corrupt("empty frame (dropped frame)"));
    }
    let patched;
    let jpeg = match missing_dht_offset(data)? {
        None => data,
        Some(at) => {
            let dht = standard_dht();
            let mut buf = Vec::with_capacity(data.len() + dht.len());
            buf.extend_from_slice(&data[..at]);
            buf.extend_from_slice(dht);
            buf.extend_from_slice(&data[at..]);
            patched = buf;
            &patched
        }
    };

    let mut decoder = JpegDecoder::new_with_options(ZCursor::new(jpeg), DecoderOptions::default());
    decoder
        .decode_headers()
        .map_err(|e| corrupt(e.to_string()))?;
    let Some(info) = decoder.info() else {
        return Err(corrupt("no image header"));
    };
    let (width, height) = (u32::from(info.width), u32::from(info.height));
    if width != format.width || height != format.height {
        if width == format.width && height * 2 == format.height {
            return Err(CodecError::Unsupported {
                codec: NAME,
                reason: "interlaced frames (two fields per chunk)".into(),
            });
        }
        return Err(corrupt(format!(
            "frame is {width}x{height}, stream says {}x{}",
            format.width, format.height
        )));
    }

    let (colorspace, channels) = match decoder.input_colorspace() {
        Some(ColorSpace::Luma) => (ColorSpace::Luma, 1),
        Some(ColorSpace::YCbCr | ColorSpace::RGB) => (ColorSpace::RGB, 3),
        other => {
            return Err(CodecError::Unsupported {
                codec: NAME,
                reason: format!("colour space {other:?}"),
            });
        }
    };
    decoder.set_options(DecoderOptions::default().jpeg_set_out_colorspace(colorspace));

    let mut samples = vec![0u8; usize::from(info.width) * usize::from(info.height) * channels];
    decoder
        .decode_into(&mut samples)
        .map_err(|e| corrupt(e.to_string()))?;
    Ok(Image {
        width,
        height,
        channels: channels as u8,
        data: samples.iter().map(|&v| f32::from(v) / 255.0).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const COLOUR_AVI: &[u8] = include_bytes!("../../tests/data/mjpg_444.avi");
    const GRAY_AVI: &[u8] = include_bytes!("../../tests/data/mjpg_gray.avi");

    /// First video chunk of a fixture AVI: the first `00dc` header followed by a JPEG SOI.
    fn first_frame(avi: &[u8]) -> &[u8] {
        let mut from = 0;
        while let Some(at) = avi[from..].windows(4).position(|w| w == b"00dc") {
            let head = from + at;
            let len = u32::from_le_bytes(avi[head + 4..head + 8].try_into().unwrap()) as usize;
            if avi.get(head + 8..head + 10) == Some(&[0xFF, 0xD8]) {
                return &avi[head + 8..head + 8 + len];
            }
            from = head + 4;
        }
        panic!("no JPEG frame in fixture");
    }

    /// Copy of the JPEG without its DHT segments, the way some AVI writers store frames.
    fn strip_dht(jpeg: &[u8]) -> Vec<u8> {
        let mut out = vec![0xFF, 0xD8];
        let mut pos = 2;
        loop {
            let marker = jpeg[pos + 1];
            if marker == 0xDA {
                out.extend_from_slice(&jpeg[pos..]);
                return out;
            }
            let len = usize::from(u16::from_be_bytes([jpeg[pos + 2], jpeg[pos + 3]]));
            if marker != 0xC4 {
                out.extend_from_slice(&jpeg[pos..pos + 2 + len]);
            }
            pos += 2 + len;
        }
    }

    fn format(width: u32, height: u32) -> FrameFormat<'static> {
        FrameFormat {
            width,
            height,
            bits_per_pixel: 24,
            extra: &[],
        }
    }

    #[test]
    fn frames_without_huffman_tables_use_the_standard_ones() {
        for (avi, channels) in [(COLOUR_AVI, 3), (GRAY_AVI, 1)] {
            let frame = first_frame(avi);
            let stripped = strip_dht(frame);
            assert!(stripped.len() < frame.len(), "fixture has a DHT segment");
            assert!(missing_dht_offset(frame).unwrap().is_none());
            assert!(missing_dht_offset(&stripped).unwrap().is_some());

            let with = decode(&format(48, 32), frame).unwrap();
            let without = decode(&format(48, 32), &stripped).unwrap();
            assert_eq!(without.channels, channels);
            // The fixtures were written with the standard tables, so both decodes are identical.
            assert_eq!(with, without);
        }
    }

    #[test]
    fn empty_frame_is_corrupt() {
        assert!(matches!(
            decode(&format(48, 32), &[]),
            Err(CodecError::Corrupt { .. })
        ));
    }

    #[test]
    fn two_field_frames_are_unsupported() {
        // The stream claims twice the height of the stored image, as for interlaced MJPEG.
        let frame = first_frame(COLOUR_AVI);
        assert!(matches!(
            decode(&format(48, 64), frame),
            Err(CodecError::Unsupported { .. })
        ));
    }

    #[test]
    fn size_mismatch_is_corrupt() {
        let frame = first_frame(COLOUR_AVI);
        assert!(matches!(
            decode(&format(50, 32), frame),
            Err(CodecError::Corrupt { .. })
        ));
    }

    #[test]
    fn truncated_frame_is_an_error() {
        let frame = first_frame(COLOUR_AVI);
        assert!(decode(&format(48, 32), &frame[..frame.len() / 2]).is_err());
    }
}
