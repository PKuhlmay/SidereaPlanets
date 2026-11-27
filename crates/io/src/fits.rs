//! FITS images (primary HDU only), parsed by hand.
//!
//! Supported: `BITPIX` 8, 16, 32, -32 and -64, `BZERO`/`BSCALE`, `NAXIS = 2` (mono) and
//! `NAXIS = 3` with three planes (RGB). Extensions, compression and other layouts are rejected.
//!
//! Normalisation to `0.0..=1.0` (design choice):
//! - Integer data: the physical value (`BZERO + BSCALE * raw`) is divided by the maximum of the
//!   unsigned type of that width (255, 65535, 4294967295). The usual unsigned 16-bit convention
//!   (`BITPIX = 16`, `BZERO = 32768`) therefore maps to the full range. Negative values are
//!   clamped to 0.
//! - Float data: if the maximum is at most 1 the values are used as is; if it is at most 65535
//!   they are divided by 65535 (camera ADU stored as float); otherwise they are divided by the
//!   frame maximum. NaN and infinite samples become 0.
//!
//! FITS stores the first row at the bottom; rows are flipped so the image is top row first.

use planets_core::Image;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

const BLOCK_LEN: usize = 2880;
const CARD_LEN: usize = 80;
/// Upper bound for the header, guards against files that never contain `END`.
const MAX_HEADER_BLOCKS: usize = 1000;

#[derive(Debug, thiserror::Error)]
pub enum FitsError {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("not a FITS file")]
    NotFits,
    #[error("FITS header is incomplete or has no END card")]
    BadHeader,
    #[error("missing or invalid FITS keyword {0}")]
    BadKeyword(&'static str),
    #[error("unsupported BITPIX {0}")]
    UnsupportedBitpix(i64),
    #[error("unsupported FITS layout: {0} axes ({1:?})")]
    UnsupportedAxes(usize, Vec<usize>),
    #[error("FITS data is truncated")]
    Truncated,
}

/// Shape and sample type of a FITS image, read from the header only.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FitsInfo {
    pub width: u32,
    pub height: u32,
    /// 1 (mono) or 3 (RGB planes).
    pub channels: u8,
    /// Bits per sample as stored (32 and 64 for float data).
    pub bit_depth: u8,
}

struct Header {
    info: FitsInfo,
    bitpix: i64,
    bzero: f64,
    bscale: f64,
    /// Length of the header including padding, i.e. offset of the data.
    data_offset: u64,
}

impl Header {
    /// Reads and checks the primary header. The file is left positioned after the header.
    fn read(file: &mut File) -> Result<Self, FitsError> {
        let mut cards: Vec<(String, String)> = Vec::new();
        let mut block = [0u8; BLOCK_LEN];
        let mut blocks = 0;
        'blocks: loop {
            if blocks == MAX_HEADER_BLOCKS {
                return Err(FitsError::BadHeader);
            }
            if file.read_exact(&mut block).is_err() {
                // A short or missing first block means this is not a FITS file at all.
                return Err(if blocks == 0 {
                    FitsError::NotFits
                } else {
                    FitsError::BadHeader
                });
            }
            blocks += 1;
            if blocks == 1 && !block.starts_with(b"SIMPLE") {
                return Err(FitsError::NotFits);
            }
            for card in block.as_chunks::<CARD_LEN>().0 {
                let key = String::from_utf8_lossy(&card[..8]).trim_end().to_owned();
                if key == "END" {
                    break 'blocks;
                }
                if card[8..10] == *b"= " {
                    let raw = String::from_utf8_lossy(&card[10..]);
                    let value = raw.split('/').next().unwrap_or_default().trim().to_owned();
                    cards.push((key, value));
                }
            }
        }

        let find = |key: &str| {
            cards
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.as_str())
        };
        let number =
            |key: &str| find(key).and_then(|v| v.replace(['D', 'd'], "E").parse::<f64>().ok());
        let int = |key: &'static str| {
            find(key)
                .and_then(|v| v.parse::<i64>().ok())
                .ok_or(FitsError::BadKeyword(key))
        };

        if find("SIMPLE") != Some("T") {
            return Err(FitsError::NotFits);
        }
        let bitpix = int("BITPIX")?;
        if !matches!(bitpix, 8 | 16 | 32 | -32 | -64) {
            return Err(FitsError::UnsupportedBitpix(bitpix));
        }
        let naxis = usize::try_from(int("NAXIS")?).map_err(|_| FitsError::BadKeyword("NAXIS"))?;
        let mut dims = Vec::new();
        for key in ["NAXIS1", "NAXIS2", "NAXIS3"].into_iter().take(naxis) {
            dims.push(usize::try_from(int(key)?).map_err(|_| FitsError::BadKeyword(key))?);
        }
        let (width, height, channels) = match (naxis, dims.as_slice()) {
            (2, &[w, h]) | (3, &[w, h, 1]) => (w, h, 1),
            (3, &[w, h, 3]) => (w, h, 3),
            _ => return Err(FitsError::UnsupportedAxes(naxis, dims)),
        };
        let dimension = |n: usize| match u32::try_from(n) {
            Ok(n) if n > 0 => Ok(n),
            _ => Err(FitsError::BadKeyword("NAXIS1")),
        };

        Ok(Self {
            info: FitsInfo {
                width: dimension(width)?,
                height: dimension(height)?,
                channels,
                bit_depth: u8::try_from(bitpix.unsigned_abs()).unwrap_or(0),
            },
            bitpix,
            bzero: number("BZERO").unwrap_or(0.0),
            bscale: number("BSCALE").unwrap_or(1.0),
            data_offset: (blocks * BLOCK_LEN) as u64,
        })
    }
}

pub fn read_info(path: &Path) -> Result<FitsInfo, FitsError> {
    Ok(Header::read(&mut File::open(path)?)?.info)
}

/// Reads the primary image as normalised samples, top row first.
pub fn read_image(path: &Path) -> Result<Image, FitsError> {
    let mut file = File::open(path)?;
    let header = Header::read(&mut file)?;
    let info = header.info;
    let (width, height) = (info.width as usize, info.height as usize);
    let channels = usize::from(info.channels);
    let samples = width * height * channels;
    let data_len = samples * (header.bitpix.unsigned_abs() / 8) as usize;

    file.seek(SeekFrom::Start(header.data_offset))?;
    let mut raw = Vec::with_capacity(data_len);
    file.take(data_len as u64).read_to_end(&mut raw)?;
    if raw.len() < data_len {
        return Err(FitsError::Truncated);
    }

    // Physical values in storage order: plane by plane, bottom row first.
    let physical = |v: f64| header.bzero + header.bscale * v;
    let values: Vec<f64> = match header.bitpix {
        8 => raw.iter().map(|&b| physical(f64::from(b))).collect(),
        16 => raw
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| physical(f64::from(i16::from_be_bytes(*c))))
            .collect(),
        32 => raw
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| physical(f64::from(i32::from_be_bytes(*c))))
            .collect(),
        -32 => raw
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| physical(f64::from(f32::from_be_bytes(*c))))
            .collect(),
        _ => raw
            .as_chunks::<8>()
            .0
            .iter()
            .map(|c| physical(f64::from_be_bytes(*c)))
            .collect(),
    };
    let divisor = match header.bitpix {
        8 => 255.0,
        16 => 65535.0,
        32 => f64::from(u32::MAX),
        _ => float_divisor(&values),
    };

    // Interleave the planes and flip the rows.
    let mut data = vec![0.0f32; samples];
    for c in 0..channels {
        for y in 0..height {
            let src = &values[(c * height + (height - 1 - y)) * width..][..width];
            for (x, &v) in src.iter().enumerate() {
                let v = if v.is_finite() { v / divisor } else { 0.0 };
                data[(y * width + x) * channels + c] = v.clamp(0.0, 1.0) as f32;
            }
        }
    }
    Ok(Image {
        width: info.width,
        height: info.height,
        channels: info.channels,
        data,
    })
}

/// Picks the divisor for float data, see the module documentation.
fn float_divisor(values: &[f64]) -> f64 {
    let max = values
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .fold(0.0, f64::max);
    if max <= 1.0 {
        1.0
    } else if max <= 65535.0 {
        65535.0
    } else {
        max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a FITS file: header cards padded to 2880 bytes, then the data padded likewise.
    fn build(bitpix: i64, dims: &[usize], extra: &[(&str, &str)], data: &[u8]) -> Vec<u8> {
        let mut cards = vec![
            ("SIMPLE".to_owned(), "T".to_owned()),
            ("BITPIX".to_owned(), bitpix.to_string()),
            ("NAXIS".to_owned(), dims.len().to_string()),
        ];
        for (i, d) in dims.iter().enumerate() {
            cards.push((format!("NAXIS{}", i + 1), d.to_string()));
        }
        cards.extend(
            extra
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned())),
        );
        let mut out = Vec::new();
        for (k, v) in cards {
            let card = format!("{k:<8}= {v:>20} / comment");
            out.extend(format!("{card:<80}").bytes());
        }
        out.extend(format!("{:<80}", "END").bytes());
        out.resize(out.len().div_ceil(BLOCK_LEN) * BLOCK_LEN, b' ');
        out.extend_from_slice(data);
        out.resize(out.len().div_ceil(BLOCK_LEN) * BLOCK_LEN, 0);
        out
    }

    fn write(name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "planets-fits-test-{}-{name}.fit",
            std::process::id()
        ));
        std::fs::write(&path, bytes).unwrap();
        path
    }

    fn be16(values: &[i16]) -> Vec<u8> {
        values.iter().flat_map(|v| v.to_be_bytes()).collect()
    }

    fn norm(v: f64, max: f64) -> f32 {
        (v / max) as f32
    }

    #[test]
    fn unsigned_16_bit_with_bzero_is_normalised_and_flipped() {
        // FITS row 0 is the bottom row: [0, 32768, 65535]; row 1 (top): [65535, 0, 16384].
        let stored = |v: u16| (i32::from(v) - 32768) as i16;
        let raw = be16(&[0, 32768, 65535, 65535, 0, 16384].map(stored));
        let path = write(
            "u16",
            &build(16, &[3, 2], &[("BZERO", "32768"), ("BSCALE", "1")], &raw),
        );
        let img = read_image(&path).unwrap();
        assert_eq!((img.width, img.height, img.channels), (3, 2, 1));
        let expect: Vec<f32> = [65535.0, 0.0, 16384.0, 0.0, 32768.0, 65535.0]
            .iter()
            .map(|&v| norm(v, 65535.0))
            .collect();
        assert_eq!(img.data, expect);
    }

    #[test]
    fn bitpix_8_uses_full_byte_range() {
        let path = write("u8", &build(8, &[2, 1], &[], &[0, 255]));
        let img = read_image(&path).unwrap();
        assert_eq!(img.data, vec![0.0, 1.0]);
    }

    #[test]
    fn bscale_and_bzero_are_applied() {
        // physical = 1000 + 2 * raw
        let raw = be16(&[0, 100]);
        let path = write(
            "scale",
            &build(16, &[2, 1], &[("BZERO", "1000"), ("BSCALE", "2")], &raw),
        );
        let img = read_image(&path).unwrap();
        assert_eq!(img.data, vec![norm(1000.0, 65535.0), norm(1200.0, 65535.0)]);
    }

    #[test]
    fn unsigned_32_bit_with_bzero() {
        let stored = |v: u32| (i64::from(v) - 2_147_483_648) as i32;
        let raw: Vec<u8> = [0u32, u32::MAX]
            .map(stored)
            .iter()
            .flat_map(|v| v.to_be_bytes())
            .collect();
        let path = write("u32", &build(32, &[2, 1], &[("BZERO", "2147483648")], &raw));
        let img = read_image(&path).unwrap();
        assert_eq!(img.data, vec![0.0, 1.0]);
    }

    #[test]
    fn float_in_unit_range_is_kept() {
        let raw: Vec<u8> = [0.25f32, 1.0]
            .iter()
            .flat_map(|v| v.to_be_bytes())
            .collect();
        let path = write("f32-unit", &build(-32, &[2, 1], &[], &raw));
        let img = read_image(&path).unwrap();
        assert_eq!(img.data, vec![0.25, 1.0]);
    }

    #[test]
    fn float_adu_range_is_scaled_by_65535() {
        let raw: Vec<u8> = [0.0f32, 4095.0]
            .iter()
            .flat_map(|v| v.to_be_bytes())
            .collect();
        let path = write("f32-adu", &build(-32, &[2, 1], &[], &raw));
        let img = read_image(&path).unwrap();
        assert_eq!(img.data, vec![0.0, norm(4095.0, 65535.0)]);
    }

    #[test]
    fn float_above_16_bit_is_scaled_by_frame_maximum() {
        let raw: Vec<u8> = [100_000.0f64, 400_000.0, f64::NAN]
            .iter()
            .flat_map(|v| v.to_be_bytes())
            .collect();
        let path = write("f64-big", &build(-64, &[3, 1], &[], &raw));
        let img = read_image(&path).unwrap();
        assert_eq!(img.data, vec![0.25, 1.0, 0.0]);
    }

    #[test]
    fn three_planes_become_rgb() {
        // 2x1 image, planes R, G, B stored one after another.
        let raw = [10u8, 20, 30, 40, 50, 60];
        let path = write("rgb", &build(8, &[2, 1, 3], &[], &raw));
        let img = read_image(&path).unwrap();
        assert_eq!((img.width, img.height, img.channels), (2, 1, 3));
        let expect: Vec<f32> = [10.0, 30.0, 50.0, 20.0, 40.0, 60.0]
            .iter()
            .map(|&v| norm(v, 255.0))
            .collect();
        assert_eq!(img.data, expect);
    }

    #[test]
    fn rgb_planes_are_flipped_per_plane() {
        // 1x2 image: each plane has two rows, bottom row first.
        let raw = [1u8, 2, 3, 4, 5, 6];
        let path = write("rgb-flip", &build(8, &[1, 2, 3], &[], &raw));
        let img = read_image(&path).unwrap();
        let expect: Vec<f32> = [2.0, 4.0, 6.0, 1.0, 3.0, 5.0]
            .iter()
            .map(|&v| norm(v, 255.0))
            .collect();
        assert_eq!(img.data, expect);
    }

    #[test]
    fn info_comes_from_the_header_only() {
        // Data is missing entirely; the header alone must be enough.
        let mut bytes = build(16, &[5, 4], &[], &[]);
        bytes.truncate(BLOCK_LEN);
        let path = write("info", &bytes);
        let info = read_info(&path).unwrap();
        assert_eq!(
            info,
            FitsInfo {
                width: 5,
                height: 4,
                channels: 1,
                bit_depth: 16
            }
        );
    }

    #[test]
    fn rgb_info_reports_three_channels() {
        let mut bytes = build(-32, &[2, 2, 3], &[], &[]);
        bytes.truncate(BLOCK_LEN);
        let info = read_info(&write("info-rgb", &bytes)).unwrap();
        assert_eq!((info.channels, info.bit_depth), (3, 32));
    }

    #[test]
    fn non_fits_is_rejected() {
        let path = write("junk", &vec![b'x'; BLOCK_LEN]);
        assert!(matches!(read_image(&path), Err(FitsError::NotFits)));
    }

    #[test]
    fn unsupported_bitpix_is_rejected() {
        let path = write("bitpix", &build(64, &[1, 1], &[], &[0; 8]));
        assert!(matches!(
            read_image(&path),
            Err(FitsError::UnsupportedBitpix(64))
        ));
    }

    #[test]
    fn unsupported_axes_are_rejected() {
        let one_d = write("axes1", &build(8, &[4], &[], &[0; 4]));
        assert!(matches!(
            read_image(&one_d),
            Err(FitsError::UnsupportedAxes(1, _))
        ));
        let cube = write("axes4", &build(8, &[1, 1, 4], &[], &[0; 4]));
        assert!(matches!(
            read_image(&cube),
            Err(FitsError::UnsupportedAxes(3, _))
        ));
    }

    #[test]
    fn truncated_data_is_rejected() {
        let mut bytes = build(16, &[100, 100], &[], &[0; 100]);
        bytes.truncate(BLOCK_LEN + 100);
        let path = write("trunc", &bytes);
        assert!(matches!(read_image(&path), Err(FitsError::Truncated)));
    }

    #[test]
    fn header_without_end_is_rejected() {
        let mut bytes = build(8, &[1, 1], &[], &[0]);
        let end = bytes.windows(3).position(|w| w == b"END").unwrap();
        bytes[end..end + 3].copy_from_slice(b"XXX");
        bytes.truncate(BLOCK_LEN);
        let path = write("noend", &bytes);
        assert!(matches!(read_info(&path), Err(FitsError::BadHeader)));
    }

    #[test]
    fn missing_naxis_is_reported() {
        let mut bytes = build(8, &[1, 1], &[], &[0]);
        let pos = bytes.windows(5).position(|w| w == b"NAXIS").unwrap();
        bytes[pos..pos + 6].copy_from_slice(b"COMMNT");
        let path = write("nonaxis", &bytes);
        assert!(matches!(
            read_info(&path),
            Err(FitsError::BadKeyword("NAXIS"))
        ));
    }
}
