//! Lagarith lossless video (FourCC `LAGS`).
//!
//! Every frame starts with a type byte. Compressed frames split the picture into planes
//! (B, G, R[, A] or Y, U, V) and give each plane its own start offset in the frame header:
//!
//! ```text
//! type:u8  offset of plane 1:u32  offset of plane 2:u32  [offset of plane 3:u32]  plane 0 ...
//! ```
//!
//! Plane 0 begins right after the header. A plane starts with an escape byte:
//!
//! - `0..=3`: range coded. The value is the number of zeros after which a run length follows
//!   (0 = no zero-run escape). With an escape, a `u32` follows that older encoders may have left out.
//!   Then come the symbol frequencies (Fibonacci prefix code), then the range coded symbols.
//! - `4`: stored uncompressed. `5..=7`: zero-run coding without a range coder (not supported).
//! - `0xFF`: the whole plane is one value, the next byte.
//!
//! Except for solid planes, the samples are differences: the first row is left-predicted, every
//! other row uses the median predictor, with the last pixel of the previous row as left neighbour.
//! RGB stores B-G, G and R-G (in this plane order) in bottom-up planes; YUV is stored top-down.

use super::{CodecError, FrameFormat};
use planets_core::Image;

const CODEC: &str = "Lagarith";

const FRAME_RAW: u8 = 1;
const FRAME_UNALIGNED_RGB24: u8 = 2;
const FRAME_ARITH_YUY2: u8 = 3;
const FRAME_ARITH_RGB24: u8 = 4;
const FRAME_SOLID_GRAY: u8 = 5;
const FRAME_SOLID_COLOR: u8 = 6;
const FRAME_OLD_ARITH_RGB: u8 = 7;
const FRAME_ARITH_RGBA: u8 = 8;
const FRAME_SOLID_RGBA: u8 = 9;
const FRAME_ARITH_YV12: u8 = 10;
const FRAME_REDUCED_RES: u8 = 11;

/// Bytes the range decoder may read past the end of its data before the plane counts as corrupt.
const MAX_OVERREAD: usize = 16;
/// Range coder precision: `range >> scale` must stay non-zero.
const MAX_SCALE: u32 = 23;

fn corrupt(reason: impl Into<String>) -> CodecError {
    CodecError::Corrupt {
        codec: CODEC,
        reason: reason.into(),
    }
}

fn unsupported(reason: impl Into<String>) -> CodecError {
    CodecError::Unsupported {
        codec: CODEC,
        reason: reason.into(),
    }
}

/// Decodes one frame. A zero-length chunk (repeat the previous frame) is rejected, because a
/// decoder call has no access to earlier frames.
pub(super) fn decode(format: &FrameFormat, data: &[u8]) -> Result<Image, CodecError> {
    let (w, h) = (format.width as usize, format.height as usize);
    if w == 0 || h == 0 || w.checked_mul(h).and_then(|n| n.checked_mul(4)).is_none() {
        return Err(corrupt(format!("invalid frame size {w}x{h}")));
    }
    let Some((&kind, body)) = data.split_first() else {
        return Err(unsupported(
            "empty frame (repeat of the previous frame) cannot be decoded on its own",
        ));
    };
    match kind {
        FRAME_RAW => raw_frame(format, body),
        FRAME_UNALIGNED_RGB24 => dib_frame(body, w * 3, 3, w, h),
        FRAME_ARITH_RGB24 => arith_rgb(data, w, h, false),
        FRAME_ARITH_RGBA => arith_rgb(data, w, h, true),
        FRAME_ARITH_YUY2 => arith_yuv(data, w, h, false),
        FRAME_ARITH_YV12 => arith_yuv(data, w, h, true),
        FRAME_SOLID_GRAY => {
            let &gray = body
                .first()
                .ok_or_else(|| corrupt("solid gray frame too short"))?;
            // YUV streams (12/16 bit) store the luma value; the chroma is neutral.
            let px = if matches!(format.bits_per_pixel, 12 | 16) {
                yuv_to_rgb(gray, 128, 128)
            } else {
                [f32::from(gray) / 255.0; 3]
            };
            Ok(solid_image(w, h, px))
        }
        FRAME_SOLID_COLOR | FRAME_SOLID_RGBA => {
            // Colour bytes are in DIB order (B, G, R, A); alpha is dropped.
            let &[b, g, r, ..] = body else {
                return Err(corrupt("solid colour frame too short"));
            };
            Ok(solid_image(w, h, rgb_to_unit(r, g, b)))
        }
        FRAME_OLD_ARITH_RGB => Err(unsupported("legacy arithmetic RGB frame (type 7)")),
        FRAME_REDUCED_RES => Err(unsupported("reduced resolution frame (type 11)")),
        other => Err(unsupported(format!("frame type {other}"))),
    }
}

fn rgb_to_unit(r: u8, g: u8, b: u8) -> [f32; 3] {
    [r, g, b].map(|v| f32::from(v) / 255.0)
}

fn solid_image(w: usize, h: usize, px: [f32; 3]) -> Image {
    let mut data = Vec::with_capacity(w * h * 3);
    for _ in 0..w * h {
        data.extend_from_slice(&px);
    }
    Image {
        width: w as u32,
        height: h as u32,
        channels: 3,
        data,
    }
}

/// Uncompressed frame; the layout follows the stream's bit depth.
fn raw_frame(format: &FrameFormat, body: &[u8]) -> Result<Image, CodecError> {
    let (w, h) = (format.width as usize, format.height as usize);
    match format.bits_per_pixel {
        bpp @ (24 | 32) => {
            let bytes = usize::from(bpp) / 8;
            let unpadded = w * bytes;
            let padded = unpadded.div_ceil(4) * 4;
            // Rows are padded like a DIB; accept packed rows too, whichever the data length fits.
            let stride = if body.len() >= padded * h {
                padded
            } else {
                unpadded
            };
            dib_frame(body, stride, bytes, w, h)
        }
        16 => {
            let stride = w.div_ceil(2) * 4;
            if body.len() < stride * h {
                return Err(corrupt("uncompressed YUY2 frame too short"));
            }
            let mut data = Vec::with_capacity(w * h * 3);
            for row in body.chunks_exact(stride).take(h) {
                for x in 0..w {
                    let pair = x / 2 * 4;
                    data.extend(yuv_to_rgb(row[x * 2], row[pair + 1], row[pair + 3]));
                }
            }
            Ok(Image {
                width: w as u32,
                height: h as u32,
                channels: 3,
                data,
            })
        }
        other => Err(unsupported(format!("uncompressed frame at {other} bit"))),
    }
}

/// Bottom-up BGR(A) rows to a top-down RGB image.
fn dib_frame(
    body: &[u8],
    stride: usize,
    bytes: usize,
    w: usize,
    h: usize,
) -> Result<Image, CodecError> {
    if body.len() < stride * (h - 1) + w * bytes {
        return Err(corrupt("uncompressed frame too short"));
    }
    let mut data = Vec::with_capacity(w * h * 3);
    for y in 0..h {
        let row = &body[(h - 1 - y) * stride..][..w * bytes];
        for px in row.chunks_exact(bytes) {
            data.extend(rgb_to_unit(px[2], px[1], px[0]));
        }
    }
    Ok(Image {
        width: w as u32,
        height: h as u32,
        channels: 3,
        data,
    })
}

fn u32_at(data: &[u8], at: usize) -> usize {
    u32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]) as usize
}

/// The bytes of the plane starting at `offset`, which must lie after the header.
fn plane_at(data: &[u8], offset: usize, header: usize) -> Result<&[u8], CodecError> {
    if offset < header || offset >= data.len() {
        return Err(corrupt(format!("plane offset {offset} outside the frame")));
    }
    Ok(&data[offset..])
}

fn arith_rgb(data: &[u8], w: usize, h: usize, alpha: bool) -> Result<Image, CodecError> {
    let header = if alpha { 13 } else { 9 };
    if data.len() < header {
        return Err(corrupt("frame header truncated"));
    }
    // The alpha plane (offset in bytes 9..13) is not needed and not decoded.
    let offsets = [header, u32_at(data, 1), u32_at(data, 5)];
    let n = w * h;
    let mut planes = [vec![0u8; n], vec![0u8; n], vec![0u8; n]];
    for (plane, offset) in planes.iter_mut().zip(offsets) {
        decode_plane(plane_at(data, offset, header)?, w, h, Predictor::Rgb, plane)?;
    }
    let [blue, green, red] = &planes;

    let mut out = Vec::with_capacity(n * 3);
    for y in 0..h {
        // Planes are stored bottom-up.
        let row = (h - 1 - y) * w;
        for x in 0..w {
            let g = green[row + x];
            let r = red[row + x].wrapping_add(g);
            let b = blue[row + x].wrapping_add(g);
            out.extend(rgb_to_unit(r, g, b));
        }
    }
    Ok(Image {
        width: w as u32,
        height: h as u32,
        channels: 3,
        data: out,
    })
}

/// YUY2 (4:2:2) or, with `yv12`, 4:2:0 with the chroma planes in V, U order.
fn arith_yuv(data: &[u8], w: usize, h: usize, yv12: bool) -> Result<Image, CodecError> {
    if data.len() < 9 {
        return Err(corrupt("frame header truncated"));
    }
    let (cw, ch) = (w.div_ceil(2), if yv12 { h.div_ceil(2) } else { h });
    let (luma_mode, chroma_mode) = if yv12 {
        (Predictor::Planar, Predictor::Planar)
    } else {
        (Predictor::Yuy2Luma, Predictor::Yuy2Chroma)
    };
    let mut y_plane = vec![0u8; w * h];
    let mut first = vec![0u8; cw * ch];
    let mut second = vec![0u8; cw * ch];
    decode_plane(plane_at(data, 9, 9)?, w, h, luma_mode, &mut y_plane)?;
    decode_plane(
        plane_at(data, u32_at(data, 1), 9)?,
        cw,
        ch,
        chroma_mode,
        &mut first,
    )?;
    decode_plane(
        plane_at(data, u32_at(data, 5), 9)?,
        cw,
        ch,
        chroma_mode,
        &mut second,
    )?;
    let (u_plane, v_plane) = if yv12 {
        (second, first)
    } else {
        (first, second)
    };

    // Chroma is repeated (nearest neighbour) over the luma samples it covers.
    let mut out = Vec::with_capacity(w * h * 3);
    for y in 0..h {
        let chroma_row = if yv12 { y / 2 } else { y } * cw;
        for (x, &luma) in y_plane[y * w..][..w].iter().enumerate() {
            let c = chroma_row + x / 2;
            out.extend(yuv_to_rgb(luma, u_plane[c], v_plane[c]));
        }
    }
    Ok(Image {
        width: w as u32,
        height: h as u32,
        channels: 3,
        data: out,
    })
}

const KY: f32 = 255.0 / 219.0;
const KR_V: f32 = 1.402 * 255.0 / 224.0;
const KG_U: f32 = 0.344_136 * 255.0 / 224.0;
const KG_V: f32 = 0.714_136 * 255.0 / 224.0;
const KB_U: f32 = 1.772 * 255.0 / 224.0;

/// BT.601 limited range (Y 16..235, U/V 16..240) to RGB in `0.0..=1.0`.
fn yuv_to_rgb(y: u8, u: u8, v: u8) -> [f32; 3] {
    let luma = (f32::from(y) - 16.0) * KY;
    let (u, v) = (f32::from(u) - 128.0, f32::from(v) - 128.0);
    [luma + KR_V * v, luma - KG_U * u - KG_V * v, luma + KB_U * u]
        .map(|c| c.clamp(0.0, 255.0) / 255.0)
}

// ---------------------------------------------------------------------------------------------
// Plane decoding

/// Decodes the plane in `src` into `out` (`w * h` samples, row by row) and undoes the prediction.
fn decode_plane(
    src: &[u8],
    w: usize,
    h: usize,
    mode: Predictor,
    out: &mut [u8],
) -> Result<(), CodecError> {
    let n = w * h;
    let Some(&esc) = src.first() else {
        return Err(corrupt("empty plane"));
    };
    match esc {
        0..=3 => {
            if src.len() < 5 {
                return Err(corrupt("plane too short"));
            }
            // Older encoders omit the length field; it is only trusted when it is plausible.
            let offset = if esc != 0 && u32_at(src, 1) < n { 5 } else { 1 };
            let mut bits = BitReader::new(&src[offset..]);
            let table = read_prob_table(&mut bits).map_err(corrupt)?;
            let mut rc = RangeDecoder::new(&src[offset + bits.byte_len()..], &table);
            fill_plane(out, u32::from(esc), || rc.next());
            if rc.overread() > MAX_OVERREAD {
                return Err(corrupt("range coded data ends early"));
            }
        }
        4..=7 => {
            if esc > 4 {
                return Err(unsupported("zero-run coded plane without range coder"));
            }
            let body = &src[1..];
            if body.len() < n {
                return Err(corrupt("uncompressed plane too short"));
            }
            out.copy_from_slice(&body[..n]);
        }
        0xFF => {
            let &value = src.get(1).ok_or_else(|| corrupt("solid plane too short"))?;
            out.fill(value);
            // A solid plane is stored as is, without prediction.
            return Ok(());
        }
        other => return Err(corrupt(format!("invalid plane escape code {other:#x}"))),
    }
    predict_plane(out, w, h, mode);
    Ok(())
}

/// Number of zeros an escape index stands for: the signed byte folded to non-negative
/// (0, -1, 1, -2, ... become 0, 1, 2, 3, ...), doubled where it was positive.
fn zero_run_len(index: u8) -> usize {
    let x = i32::from(index as i8);
    ((x * 2) ^ (x >> 7)) as usize
}

/// Fills `out` from a symbol source. After `esc` zeros in a row (0 = never) the next symbol is
/// an escape index that tells how many more zeros follow; runs may continue into the next row.
fn fill_plane(out: &mut [u8], esc: u32, mut next: impl FnMut() -> u8) {
    let esc = if esc == 0 { u32::MAX } else { esc };
    let mut zeros = 0u32;
    let mut i = 0;
    while i < out.len() {
        let value = next();
        out[i] = value;
        i += 1;
        if value != 0 {
            zeros = 0;
            continue;
        }
        zeros += 1;
        if zeros == esc {
            let run = zero_run_len(next()).min(out.len() - i);
            out[i..i + run].fill(0);
            i += run;
            zeros = 0;
        }
    }
}

/// The prediction rules differ slightly between the frame types, down to what happens with the
/// first pixels of the first two rows.
#[derive(Clone, Copy)]
enum Predictor {
    /// RGB planes: the first pixel of the second row is predicted from the pixel above.
    Rgb,
    /// 4:2:0 planes: the first pixel of the second row is predicted from the last pixel of the
    /// first row.
    Planar,
    /// Luma of a YUY2 frame: the first row keeps its first sample as is and predicts from the
    /// second one on; the second row starts with a four sample left-predicted head.
    Yuy2Luma,
    /// Chroma of a YUY2 frame: like the luma, but with a head of two samples and nothing special
    /// in the first row.
    Yuy2Chroma,
}

impl Predictor {
    /// YUY2 planes reduce the gradient `left + top - top_left` to a byte before taking the
    /// median (as HuffYUV does), the other types keep it as a signed number.
    fn wraps_gradient(self) -> bool {
        matches!(self, Self::Yuy2Luma | Self::Yuy2Chroma)
    }
}

fn median(a: i32, b: i32, c: i32) -> i32 {
    a.min(b).max(a.max(b).min(c))
}

/// Median-predicts `cur[from..]` from the row above. `left` and `top_left` are the neighbours of
/// `cur[from]`.
fn median_row(
    cur: &mut [u8],
    prev: &[u8],
    from: usize,
    mut left: u8,
    mut top_left: u8,
    wrap: bool,
) {
    for (c, &top) in cur[from..].iter_mut().zip(&prev[from..]) {
        let mut guess = i32::from(left) + i32::from(top) - i32::from(top_left);
        if wrap {
            guess &= 0xFF;
        }
        left = median(i32::from(left), i32::from(top), guess) as u8;
        left = left.wrapping_add(*c);
        *c = left;
        top_left = top;
    }
}

/// Undoes the prediction in place.
fn predict_plane(out: &mut [u8], w: usize, h: usize, mode: Predictor) {
    let first_row = &mut out[..w];
    match mode {
        Predictor::Rgb | Predictor::Planar | Predictor::Yuy2Chroma => {
            let mut acc = 0u8;
            for v in first_row {
                acc = acc.wrapping_add(*v);
                *v = acc;
            }
        }
        Predictor::Yuy2Luma => {
            let mut acc = 0u8;
            for v in &mut first_row[1..] {
                acc = acc.wrapping_add(*v);
                *v = acc;
            }
        }
    }
    for row in 1..h {
        let (done, rest) = out.split_at_mut(row * w);
        let prev = &done[(row - 1) * w..];
        let cur = &mut rest[..w];
        // The pixel left of a row start is the last pixel of the row above, and its top-left
        // neighbour is the last pixel of the row above that.
        let mut left = prev[w - 1];
        let mut from = 0;
        let top_left = if row == 1 {
            match mode {
                Predictor::Rgb => {
                    left = prev[0];
                    prev[0]
                }
                Predictor::Planar => prev[0],
                Predictor::Yuy2Luma | Predictor::Yuy2Chroma => {
                    // The head of the second row is only left-predicted.
                    let head = if matches!(mode, Predictor::Yuy2Luma) {
                        4
                    } else {
                        2
                    }
                    .min(w);
                    for v in &mut cur[..head] {
                        left = left.wrapping_add(*v);
                        *v = left;
                    }
                    from = head;
                    prev[head - 1]
                }
            }
        } else {
            done[(row - 1) * w - 1]
        };
        median_row(cur, prev, from, left, top_left, mode.wraps_gradient());
    }
}

// ---------------------------------------------------------------------------------------------
// Symbol frequencies and range decoder

/// MSB-first bit reader.
struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn bit(&mut self) -> Option<u32> {
        let byte = *self.data.get(self.pos / 8)?;
        let bit = byte >> (7 - self.pos % 8) & 1;
        self.pos += 1;
        Some(u32::from(bit))
    }

    fn bits(&mut self, n: u32) -> Option<u32> {
        let mut value = 0;
        for _ in 0..n {
            value = value << 1 | self.bit()?;
        }
        Some(value)
    }

    /// Bytes touched so far (a partly read byte counts).
    fn byte_len(&self) -> usize {
        self.pos.div_ceil(8)
    }
}

/// Reads one number of the frequency table. The length of the value is sent as a Fibonacci code
/// (no two adjacent ones, closed by a second one), followed by the value bits below the top one.
fn read_prob_value(bits: &mut BitReader) -> Option<u32> {
    const FIB: [u32; 7] = [1, 2, 3, 5, 8, 13, 21];
    let (mut bit, mut prev, mut length) = (0, 0, 0);
    for weight in FIB {
        if prev == 1 && bit == 1 {
            break;
        }
        prev = bit;
        bit = bits.bit()?;
        if bit == 1 && prev == 0 {
            length += weight;
        }
    }
    let length = length.checked_sub(1).filter(|&l| l <= 31)?;
    if length == 0 {
        return Some(0);
    }
    Some((bits.bits(length)? | 1 << length) - 1)
}

/// Cumulative symbol frequencies, scaled to a total of `1 << scale`.
struct ProbTable {
    /// `start[s]` is where symbol `s` begins; `start[256]` is the total.
    start: [u32; 257],
    scale: u32,
}

/// 2^(52 + bit length of denom - 1) / denom with rounding; a fixed point reciprocal that
/// reproduces the arithmetic of the encoder.
fn reciprocal(denom: u32) -> u64 {
    let shift = 32 - (denom - 1).leading_zeros();
    let d = u64::from(denom);
    let mut quotient = (1u64 << 52) / d;
    let mut rest = (1u64 << 52) - quotient * d;
    quotient <<= shift;
    rest <<= shift;
    rest += d / 2;
    quotient + rest / d
}

/// `x * mantissa / 2^52` with the encoder's rounding.
fn scaled(x: u32, mantissa: u64) -> u32 {
    let x = u64::from(x);
    let mut low = x * (mantissa & 0xFFFF_FFFF);
    let mut high = x * (mantissa >> 32);
    high += low >> 32;
    low &= 0xFFFF_FFFF;
    low += 1 << (high >> 21).checked_ilog2().unwrap_or(0);
    high += low >> 32;
    (high >> 20) as u32
}

fn read_prob_table(bits: &mut BitReader) -> Result<ProbTable, &'static str> {
    const BAD: &str = "invalid symbol frequency table";
    let mut freq = [0u32; 256];
    let mut total = 0u64;
    let mut i = 0;
    while i < 256 {
        let f = read_prob_value(bits).ok_or(BAD)?;
        freq[i] = f;
        total += u64::from(f);
        if total > u64::from(u32::MAX) {
            return Err(BAD);
        }
        if f == 0 {
            // A zero is followed by the number of zeros after it.
            let run = read_prob_value(bits).ok_or(BAD)? as usize;
            i += run.min(255 - i);
        }
        i += 1;
    }
    let total = total as u32;
    if total == 0 {
        return Err("all symbol frequencies are zero");
    }

    let mut scale = total.ilog2();
    if !total.is_power_of_two() {
        let mantissa = reciprocal(total);
        let mut sum = 0u64;
        for f in &mut freq {
            *f = scaled(*f, mantissa);
            sum += u64::from(*f);
        }
        scale += 1;
        let target = 1u64 << scale;
        if sum > target {
            return Err(BAD);
        }
        // The encoder hands the rounding loss out one by one to the first 128 symbols.
        let mut spare = target - sum;
        if spare > 0 && freq[..128].iter().all(|&f| f == 0) {
            return Err(BAD);
        }
        let mut i = 0;
        while spare > 0 {
            if freq[i] != 0 {
                freq[i] += 1;
                spare -= 1;
            }
            i = (i + 1) & 127;
        }
    }
    if scale > MAX_SCALE {
        return Err("symbol frequencies too precise");
    }

    let mut start = [0u32; 257];
    for s in 0..256 {
        start[s + 1] = start[s] + freq[s];
    }
    Ok(ProbTable { start, scale })
}

/// Range decoder over a bit stream that begins seven bits into the first byte.
struct RangeDecoder<'a> {
    data: &'a [u8],
    /// Index of the byte that supplies the low bit of the next 8-bit refill.
    pos: usize,
    low: u32,
    range: u32,
    scale: u32,
    hash_shift: u32,
    start: [u32; 257],
    /// Symbol containing the beginning of each 1/256th slice of the scaled range.
    first_symbol: [u8; 256],
}

impl<'a> RangeDecoder<'a> {
    fn new(data: &'a [u8], table: &ProbTable) -> Self {
        let hash_shift = table.scale.max(8) - 8;
        let mut first_symbol = [0u8; 256];
        let mut symbol = 0usize;
        for (slice, slot) in first_symbol.iter_mut().enumerate() {
            let at = (slice as u32) << hash_shift;
            while symbol < 255 && table.start[symbol + 1] <= at {
                symbol += 1;
            }
            *slot = symbol as u8;
        }
        Self {
            data,
            pos: 0,
            low: u32::from(data.first().copied().unwrap_or(0) >> 1),
            range: 0x80,
            scale: table.scale,
            hash_shift,
            start: table.start,
            first_symbol,
        }
    }

    fn byte(&self, at: usize) -> u8 {
        self.data.get(at).copied().unwrap_or(0)
    }

    /// Bytes consumed beyond the end of the data (they read as zero).
    fn overread(&self) -> usize {
        (self.pos + 1).saturating_sub(self.data.len())
    }

    fn next(&mut self) -> u8 {
        while self.range <= 0x80_0000 {
            let byte = self.byte(self.pos) << 7 | self.byte(self.pos + 1) >> 1;
            self.low = self.low << 8 | u32::from(byte);
            self.range <<= 8;
            self.pos += 1;
        }
        let unit = self.range >> self.scale;
        let last_start = unit * self.start[255];
        let symbol = if self.low < last_start {
            let target = self.low / unit;
            let mut s =
                usize::from(self.first_symbol[(target >> self.hash_shift).min(255) as usize]);
            while s < 255 && self.start[s + 1] <= target {
                s += 1;
            }
            self.range = unit * (self.start[s + 1] - self.start[s]);
            s
        } else {
            // The last symbol also owns the rounding remainder of the range.
            self.range = (self.range - last_start).max(1);
            255
        };
        self.low = self.low.wrapping_sub(unit * self.start[symbol]);
        symbol as u8
    }
}

#[cfg(test)]
mod tests;
