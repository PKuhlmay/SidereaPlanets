//! FITS writer (primary HDU only), the counterpart of [`crate::fits`].
//!
//! Mono images get `NAXIS = 2`, colour images `NAXIS = 3` with three planes (R, G, B). The first
//! row of a FITS image is the bottom one, so rows are written bottom row first.
//!
//! Integer variants use the unsigned convention (`BZERO` = 2^(n-1)) and map `0.0..=1.0` to the
//! full range of the unsigned type. The float variant stores the samples unchanged, clamped to
//! `0.0..=1.0`, which [`crate::fits::read_image`] reads back as is.

use planets_core::Image;
use std::io::{self, Write};

const BLOCK_LEN: usize = 2880;
const CARD_LEN: usize = 80;

/// Sample type of the written file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitsDepth {
    /// `BITPIX = 16`, `BZERO = 32768`.
    Int16,
    /// `BITPIX = 32`, `BZERO = 2147483648`.
    Int32,
    /// `BITPIX = -32`.
    Float32,
}

impl FitsDepth {
    fn bitpix(self) -> i32 {
        match self {
            Self::Int16 => 16,
            Self::Int32 => 32,
            Self::Float32 => -32,
        }
    }
}

/// Writes `image` as a FITS file. The image must be consistent (`width * height * channels`
/// samples); the caller checks that.
pub fn write_fits<W: Write>(out: &mut W, image: &Image, depth: FitsDepth) -> io::Result<()> {
    let (width, height) = (image.width as usize, image.height as usize);
    let src_channels = usize::from(image.channels).max(1);
    let planes = if src_channels >= 3 { 3 } else { 1 };

    let mut cards = vec![
        card("SIMPLE", "T"),
        card("BITPIX", &depth.bitpix().to_string()),
        card("NAXIS", &(if planes == 3 { 3 } else { 2 }).to_string()),
        card("NAXIS1", &width.to_string()),
        card("NAXIS2", &height.to_string()),
    ];
    if planes == 3 {
        cards.push(card("NAXIS3", "3"));
    }
    match depth {
        FitsDepth::Int16 => {
            cards.push(card("BZERO", "32768"));
            cards.push(card("BSCALE", "1"));
        }
        FitsDepth::Int32 => {
            cards.push(card("BZERO", "2147483648"));
            cards.push(card("BSCALE", "1"));
        }
        FitsDepth::Float32 => {}
    }
    cards.push(format!("{:<CARD_LEN$}", "CREATOR  = 'Siderea Planets'"));
    cards.push(format!("{:<CARD_LEN$}", "END"));

    let mut written = 0;
    for c in &cards {
        out.write_all(c.as_bytes())?;
        written += CARD_LEN;
    }
    written += pad(out, written, b' ')?;
    debug_assert_eq!(written % BLOCK_LEN, 0);

    let mut data_len = 0;
    for plane in 0..planes {
        for y in (0..height).rev() {
            for x in 0..width {
                let v = clean(image.data[(y * width + x) * src_channels + plane]);
                match depth {
                    FitsDepth::Int16 => {
                        let q = (v * 65535.0).round() as i32;
                        out.write_all(&((q - 32768) as i16).to_be_bytes())?;
                        data_len += 2;
                    }
                    FitsDepth::Int32 => {
                        let q = (f64::from(v) * f64::from(u32::MAX)).round() as i64;
                        out.write_all(&((q - 2_147_483_648) as i32).to_be_bytes())?;
                        data_len += 4;
                    }
                    FitsDepth::Float32 => {
                        out.write_all(&v.to_be_bytes())?;
                        data_len += 4;
                    }
                }
            }
        }
    }
    pad(out, data_len, 0)?;
    Ok(())
}

/// Clamps to `0.0..=1.0`; NaN becomes 0.
fn clean(v: f32) -> f32 {
    if v.is_nan() { 0.0 } else { v.clamp(0.0, 1.0) }
}

/// A numeric card: keyword, `= `, value right aligned in columns 11 to 30, padded to 80 bytes.
fn card(key: &str, value: &str) -> String {
    format!("{:<CARD_LEN$}", format!("{key:<8}= {value:>20}"))
}

/// Pads to the next multiple of 2880 bytes with `fill` (blanks after the header, zeros after the data).
fn pad<W: Write>(out: &mut W, len: usize, fill: u8) -> io::Result<usize> {
    let rest = (BLOCK_LEN - len % BLOCK_LEN) % BLOCK_LEN;
    out.write_all(&vec![fill; rest])?;
    Ok(rest)
}
