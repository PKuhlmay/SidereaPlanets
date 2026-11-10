//! HuffYUV (`HFYU`) and FFVHuff (`FFVH`): lossless video, a predictor per sample plus a Huffman code
//! for the residual.
//!
//! Supported streams (what the HuffYUV 2.x and ffmpeg encoders write):
//! - 4:2:2 (16 bit), RGB (24 bit) and RGB with alpha (32 bit); FFVHuff also 4:2:0 (12 bit)
//! - FFVHuff "version 3" planar streams: gray, YUV (any 8 bit subsampling) and planar RGB
//! - predictors left, gradient ("plane") and median (not for RGB), interlaced prediction
//! - Huffman tables from the codec private data, or per frame when the stream says so
//!
//! Rejected as unsupported: HuffYUV 1.x streams without table data (they rely on built-in tables),
//! more than 8 bits per sample, alpha planes in planar streams, odd widths for 4:2:2/4:2:0.
//!
//! Colour streams are returned as RGB; YUV is converted with BT.601 limited range and chroma is
//! replicated (nearest neighbour). A stream without chroma gives a single channel.
//!
//! The layout follows the public format description; the details were checked against ffmpeg's output.

use super::{CodecError, FrameFormat};
use planets_core::Image;
use std::sync::OnceLock;

#[derive(Clone, Copy)]
struct Codec(&'static str);

impl Codec {
    fn corrupt(self, reason: impl Into<String>) -> CodecError {
        CodecError::Corrupt {
            codec: self.0,
            reason: reason.into(),
        }
    }

    fn unsupported(self, reason: impl Into<String>) -> CodecError {
        CodecError::Unsupported {
            codec: self.0,
            reason: reason.into(),
        }
    }
}

pub(super) fn decode_huffyuv(format: &FrameFormat, data: &[u8]) -> Result<Image, CodecError> {
    decode(Codec("HuffYUV"), format, data)
}

pub(super) fn decode_ffvhuff(format: &FrameFormat, data: &[u8]) -> Result<Image, CodecError> {
    decode(Codec("FFVHuff"), format, data)
}

// ---------------------------------------------------------------------------------------------
// Bit reading

/// The frame bitstream: 32 bit little endian words, most significant bit first.
struct FrameBits<'a> {
    data: &'a [u8],
    next: usize,
    cache: u64,
    count: u32,
    /// Set when a code that is not in the table was read.
    bad_code: bool,
}

impl<'a> FrameBits<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            next: 0,
            cache: 0,
            count: 0,
            bad_code: false,
        }
    }

    /// Makes sure at least 33 bits are cached; the data is padded with zeros.
    #[inline]
    fn refill(&mut self) {
        if self.count <= 32 {
            let rest = self.data.get(self.next..).unwrap_or(&[]);
            let mut word = [0u8; 4];
            let n = rest.len().min(4);
            word[..n].copy_from_slice(&rest[..n]);
            self.cache = (self.cache << 32) | u64::from(u32::from_le_bytes(word));
            self.count += 32;
            self.next += 4;
        }
    }

    /// The next 32 bits without consuming them (call [`Self::refill`] first).
    #[inline]
    fn peek32(&self) -> u32 {
        (self.cache >> (self.count - 32)) as u32
    }

    #[inline]
    fn skip(&mut self, n: u32) {
        self.count -= n;
    }

    /// Reads `n` (1..=32) bits.
    fn read(&mut self, n: u32) -> u32 {
        self.refill();
        let value = self.peek32() >> (32 - n);
        self.skip(n);
        value
    }

    /// Number of bits consumed so far.
    fn position(&self) -> usize {
        self.next * 8 - self.count as usize
    }

    /// True when more bits were consumed than the frame holds.
    fn overrun(&self) -> bool {
        self.position() > self.data.len() * 8
    }
}

/// Reverses the byte order of every 32 bit word (zero padded to a whole number of words).
fn swap_words(data: &[u8]) -> Vec<u8> {
    let mut out = data.to_vec();
    out.resize(data.len().div_ceil(4) * 4, 0);
    out.as_chunks_mut::<4>()
        .0
        .iter_mut()
        .for_each(|word| word.reverse());
    out
}

/// Plain most-significant-bit-first reader over bytes (the table description in the private data).
struct ByteBits<'a> {
    data: &'a [u8],
    pos: usize,
}

impl ByteBits<'_> {
    fn read(&mut self, n: usize) -> Option<usize> {
        let mut value = 0;
        for _ in 0..n {
            let byte = *self.data.get(self.pos / 8)?;
            value = (value << 1) | usize::from((byte >> (7 - self.pos % 8)) & 1);
            self.pos += 1;
        }
        Some(value)
    }
}

// ---------------------------------------------------------------------------------------------
// Huffman tables

const FAST_BITS: u32 = 11;

/// Canonical Huffman code. Codes are assigned from the longest length down, symbols in ascending
/// order within a length; short codes resolve through a lookup table, long ones by range check.
struct Huffman {
    /// `(length << 8) | symbol` for codes of up to `FAST_BITS` bits, 0 otherwise.
    fast: Vec<u16>,
    first_code: [u32; 33],
    count: [u16; 33],
    first_index: [u16; 33],
    /// Symbols in code assignment order.
    symbols: Vec<u8>,
}

impl Huffman {
    fn new(lengths: &[u8; 256]) -> Result<Self, &'static str> {
        let mut count = [0u16; 33];
        for &len in lengths {
            if len > 32 {
                return Err("code length above 32");
            }
            count[usize::from(len)] += 1;
        }
        count[0] = 0;

        let mut first_code = [0u32; 33];
        let mut first_index = [0u16; 33];
        let mut symbols = Vec::new();
        let mut code = 0u64;
        for len in (1..=32usize).rev() {
            first_code[len] = code as u32;
            first_index[len] = symbols.len() as u16;
            for (symbol, _) in lengths
                .iter()
                .enumerate()
                .filter(|(_, l)| usize::from(**l) == len)
            {
                symbols.push(symbol as u8);
                code += 1;
            }
            if code > 1 << len {
                return Err("over-subscribed code lengths");
            }
            if code & 1 != 0 {
                return Err("invalid code lengths");
            }
            code >>= 1;
        }

        let mut fast = vec![0u16; 1 << FAST_BITS];
        for len in 1..=FAST_BITS as usize {
            for k in 0..usize::from(count[len]) {
                let symbol = symbols[usize::from(first_index[len]) + k];
                let prefix = (first_code[len] as usize + k) << (FAST_BITS as usize - len);
                let span = 1 << (FAST_BITS as usize - len);
                fast[prefix..prefix + span].fill(((len as u16) << 8) | u16::from(symbol));
            }
        }
        Ok(Self {
            fast,
            first_code,
            count,
            first_index,
            symbols,
        })
    }

    /// Reads one symbol. An invalid code sets `bits.bad_code` and yields 0.
    #[inline]
    fn get(&self, bits: &mut FrameBits) -> u8 {
        bits.refill();
        let top = bits.peek32();
        let entry = self.fast[(top >> (32 - FAST_BITS)) as usize];
        if entry != 0 {
            bits.skip(u32::from(entry >> 8));
            return entry as u8;
        }
        for len in FAST_BITS as usize + 1..=32 {
            let n = u32::from(self.count[len]);
            if n == 0 {
                continue;
            }
            let code = top >> (32 - len);
            let offset = code.wrapping_sub(self.first_code[len]);
            if code >= self.first_code[len] && offset < n {
                bits.skip(len as u32);
                return self.symbols[usize::from(self.first_index[len]) + offset as usize];
            }
        }
        bits.bad_code = true;
        0
    }
}

/// Reads `n` tables of 256 run-length coded code lengths with `read(bit count)`.
fn read_tables(
    codec: Codec,
    n: usize,
    mut read: impl FnMut(usize) -> Option<usize>,
) -> Result<Vec<Huffman>, CodecError> {
    let truncated = || codec.corrupt("truncated Huffman table");
    let mut tables = Vec::with_capacity(n);
    for _ in 0..n {
        let mut lengths = [0u8; 256];
        let mut i = 0;
        while i < 256 {
            let mut repeat = read(3).ok_or_else(truncated)?;
            let length = read(5).ok_or_else(truncated)?;
            if repeat == 0 {
                repeat = read(8).ok_or_else(truncated)?;
            }
            if i + repeat > 256 {
                return Err(codec.corrupt("Huffman table overflows 256 entries"));
            }
            lengths[i..i + repeat].fill(length as u8);
            i += repeat;
        }
        tables.push(
            Huffman::new(&lengths)
                .map_err(|reason| codec.corrupt(format!("bad Huffman table: {reason}")))?,
        );
    }
    Ok(tables)
}

// ---------------------------------------------------------------------------------------------
// Stream header (codec private data)

#[derive(Clone, Copy, PartialEq, Eq)]
enum Predictor {
    Left,
    /// Gradient: the left-predicted row plus the row above.
    Plane,
    Median,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Layout {
    /// 16 bit: luma, with chroma shared by two pixels in a row.
    Yuv422,
    /// 12 bit (FFVHuff): chroma shared by 2x2 pixels.
    Yuv420,
    /// 24 or 32 bit packed pixels, bottom row first.
    Rgb { alpha: bool },
    /// Version 3: whole planes one after the other.
    Planar {
        chroma: bool,
        /// Planes are G, B, R instead of Y, U, V.
        rgb: bool,
        h_shift: u32,
        v_shift: u32,
    },
}

struct Header {
    predictor: Predictor,
    decorrelate: bool,
    /// Interlaced prediction uses the row two above instead of the row above.
    interlaced: bool,
    /// Tables are stored at the start of every frame.
    context: bool,
    layout: Layout,
}

impl Header {
    fn parse(codec: Codec, format: &FrameFormat) -> Result<Self, CodecError> {
        let extra = format.extra;
        if extra.len() < 4 {
            return Err(codec.unsupported(
                "stream without codec private data (old HuffYUV 1.x with built-in tables)",
            ));
        }
        let predictor = match extra[0] & 63 {
            0 => Predictor::Left,
            1 => Predictor::Plane,
            2 => Predictor::Median,
            other => return Err(codec.unsupported(format!("predictor {other}"))),
        };
        let decorrelate = extra[0] & 64 != 0;
        let interlaced = match (extra[2] >> 4) & 3 {
            1 => true,
            2 => false,
            // Not stated: tall frames are interlaced.
            _ => format.height > 288,
        };
        let context = extra[2] & 0x40 != 0;

        let layout = match extra[3] {
            0 => {
                let bpp = match extra[1] {
                    12 | 16 | 24 | 32 => u16::from(extra[1]),
                    _ => format.bits_per_pixel,
                };
                match bpp {
                    12 => Layout::Yuv420,
                    16 => Layout::Yuv422,
                    24 => Layout::Rgb { alpha: false },
                    32 => Layout::Rgb { alpha: true },
                    other => return Err(codec.unsupported(format!("{other} bits per pixel"))),
                }
            }
            1 => {
                let bits = u32::from(extra[1] >> 4) + 1;
                if bits != 8 {
                    return Err(codec.unsupported(format!("{bits} bits per sample")));
                }
                if extra[2] & 4 != 0 {
                    return Err(codec.unsupported("alpha plane"));
                }
                if interlaced {
                    return Err(codec.unsupported("interlaced planar stream"));
                }
                let chroma = extra[2] & 3 != 0;
                Layout::Planar {
                    chroma,
                    rgb: chroma && extra[2] & 1 == 0,
                    h_shift: u32::from(extra[1] & 3),
                    v_shift: u32::from((extra[1] >> 2) & 3),
                }
            }
            other => return Err(codec.unsupported(format!("stream version {}", other + 2))),
        };

        match layout {
            Layout::Rgb { .. } if predictor == Predictor::Median => {
                return Err(codec.unsupported("median predictor with RGB"));
            }
            Layout::Planar { rgb: true, .. } if decorrelate => {
                return Err(codec.unsupported("channel decorrelation in planar RGB"));
            }
            Layout::Yuv420 if interlaced => {
                return Err(codec.unsupported("interlaced 4:2:0"));
            }
            _ => {}
        }
        Ok(Self {
            predictor,
            decorrelate,
            interlaced,
            context,
            layout,
        })
    }

    fn table_count(&self) -> usize {
        match self.layout {
            Layout::Planar { chroma: false, .. } => 1,
            _ => 3,
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Prediction

/// One plane of samples with the predictor state that carries from row to row.
struct Plane {
    data: Vec<u8>,
    width: usize,
    left: u8,
    /// The top-left neighbour of the next median-predicted sample.
    top_left: u8,
}

impl Plane {
    fn new(width: usize, height: usize) -> Self {
        Self {
            data: vec![0; width * height],
            width,
            left: 0,
            top_left: 0,
        }
    }

    /// Turns the residuals `res[start..]` of row `row` into samples. `lines_back` is the distance to
    /// the row used as "above" (2 for interlaced prediction). With median prediction, `split` is
    /// the number of leading samples of the first predicted row that are only left-predicted
    /// (0 for streams that median-predict the whole row).
    fn predict(
        &mut self,
        predictor: Predictor,
        row: usize,
        interlaced: bool,
        start: usize,
        split: usize,
        res: &[u8],
    ) {
        let w = self.width;
        let back = 1 + usize::from(interlaced);
        let (head, tail) = self.data.split_at_mut(row * w);
        let dst = &mut tail[..w];
        let above = row.checked_sub(back).map(|r| &head[r * w..][..w]);

        match (predictor, above) {
            (Predictor::Median, Some(above)) => {
                let k = if row == back { split.min(w) } else { 0 };
                self.left = add_left(&mut dst[..k], &res[..k], self.left);
                if k > 0 && k < w {
                    self.top_left = above[k - 1];
                }
                if k < w {
                    median_row(
                        &mut dst[k..],
                        &above[k..],
                        &res[k..],
                        &mut self.left,
                        &mut self.top_left,
                    );
                }
            }
            (_, above) => {
                self.left = add_left(&mut dst[start..], &res[start..w], self.left);
                if let (Predictor::Plane, Some(above)) = (predictor, above) {
                    for (d, &a) in dst.iter_mut().zip(above) {
                        *d = d.wrapping_add(a);
                    }
                }
            }
        }
    }
}

/// `dst[i] = dst[i - 1] + res[i]`, starting from `left`. Returns the last sample.
fn add_left(dst: &mut [u8], res: &[u8], mut left: u8) -> u8 {
    for (d, &r) in dst.iter_mut().zip(res) {
        left = left.wrapping_add(r);
        *d = left;
    }
    left
}

fn median3(a: u8, b: u8, c: u8) -> u8 {
    a.min(b).max(a.max(b).min(c))
}

/// Median prediction from left, top and left + top - top-left.
fn median_row(dst: &mut [u8], top: &[u8], res: &[u8], left: &mut u8, top_left: &mut u8) {
    let (mut l, mut tl) = (*left, *top_left);
    for ((d, &t), &r) in dst.iter_mut().zip(top).zip(res) {
        let gradient = l.wrapping_add(t).wrapping_sub(tl);
        l = median3(l, t, gradient).wrapping_add(r);
        tl = t;
        *d = l;
    }
    *left = l;
    *top_left = tl;
}

// ---------------------------------------------------------------------------------------------
// Frame decoding

fn decode(codec: Codec, format: &FrameFormat, data: &[u8]) -> Result<Image, CodecError> {
    // A zero-length chunk is a dropped frame; there is nothing to repeat here, so it is an error.
    if data.is_empty() {
        return Err(codec.corrupt("empty frame (dropped frame)"));
    }
    let (width, height) = (format.width as usize, format.height as usize);
    if width == 0 || height == 0 || width.checked_mul(height).is_none_or(|n| n > 1 << 30) {
        return Err(codec.corrupt(format!("bad frame size {width}x{height}")));
    }
    let header = Header::parse(codec, format)?;

    let owned;
    let (tables, payload) = if header.context {
        // The tables lead the frame as plain bytes, followed by the bitstream, and the whole frame
        // was then stored with its 32 bit words reversed. Undo that to find where the bitstream starts.
        let plain = swap_words(data);
        let mut reader = ByteBits {
            data: &plain,
            pos: 0,
        };
        let tables = read_tables(codec, header.table_count(), |n| reader.read(n))?;
        owned = swap_words(plain.get(reader.pos.div_ceil(8)..).unwrap_or(&[]));
        (tables, &owned[..])
    } else {
        let mut reader = ByteBits {
            data: &format.extra[4..],
            pos: 0,
        };
        let tables = read_tables(codec, header.table_count(), |n| reader.read(n))?;
        (tables, data)
    };
    let mut bits = FrameBits::new(payload);
    let frame = Frame {
        codec,
        header: &header,
        tables: &tables,
        width,
        height,
    };
    match header.layout {
        Layout::Yuv422 | Layout::Yuv420 => frame.decode_interleaved_yuv(&mut bits),
        Layout::Rgb { alpha } => frame.decode_rgb(&mut bits, alpha),
        Layout::Planar {
            chroma,
            rgb,
            h_shift,
            v_shift,
        } => frame.decode_planar(&mut bits, chroma, rgb, h_shift, v_shift),
    }
}

struct Frame<'a> {
    codec: Codec,
    header: &'a Header,
    tables: &'a [Huffman],
    width: usize,
    height: usize,
}

impl Frame<'_> {
    fn check(&self, bits: &FrameBits) -> Result<(), CodecError> {
        if bits.bad_code {
            Err(self.codec.corrupt("invalid Huffman code"))
        } else if bits.overrun() {
            Err(self.codec.corrupt("frame data ends early"))
        } else {
            Ok(())
        }
    }

    /// 4:2:2 and 4:2:0: rows of luma pairs with their chroma samples, the first four samples raw.
    fn decode_interleaved_yuv(&self, bits: &mut FrameBits) -> Result<Image, CodecError> {
        let (w, h) = (self.width, self.height);
        if w % 2 != 0 {
            return Err(self.codec.unsupported("odd width in 4:2:2/4:2:0"));
        }
        let is_420 = self.header.layout == Layout::Yuv420;
        let (cw, ch) = (w / 2, if is_420 { h.div_ceil(2) } else { h });
        let median = self.header.predictor == Predictor::Median;
        let (predictor, interlaced) = (self.header.predictor, self.header.interlaced);
        let (luma, u_table, v_table) = (&self.tables[0], &self.tables[1], &self.tables[2]);

        let mut y_plane = Plane::new(w, h);
        let mut u_plane = Plane::new(cw, ch);
        let mut v_plane = Plane::new(cw, ch);
        let (mut ry, mut ru, mut rv) = (vec![0u8; w], vec![0u8; cw], vec![0u8; cw]);

        // The very first samples are stored as they are: V, Y0, U, Y1.
        let v0 = bits.read(8) as u8;
        let y0 = bits.read(8) as u8;
        let u0 = bits.read(8) as u8;
        let y1 = bits.read(8) as u8;
        y_plane.data[..2].copy_from_slice(&[y0, y1]);
        y_plane.left = y1;
        u_plane.data[0] = u0;
        u_plane.left = u0;
        v_plane.data[0] = v0;
        v_plane.left = v0;

        let mut chroma_row = 0;
        for y in 0..h {
            // 4:2:0 streams carry chroma on every other row. With the median predictor the second
            // row carries it too and the rows after it are shifted accordingly.
            let with_chroma = chroma_row < ch
                && if !is_420 {
                    true
                } else if median {
                    y < 2 || (y >= 4 && y % 2 == 0)
                } else {
                    y % 2 == 0
                };
            let (start_y, start_c) = if y == 0 { (2, 1) } else { (0, 0) };
            if with_chroma {
                for i in start_c..cw {
                    ry[2 * i] = luma.get(bits);
                    ru[i] = u_table.get(bits);
                    ry[2 * i + 1] = luma.get(bits);
                    rv[i] = v_table.get(bits);
                }
            } else {
                for r in &mut ry[start_y..] {
                    *r = luma.get(bits);
                }
            }
            self.check(bits)?;

            y_plane.predict(predictor, y, interlaced, start_y, 4, &ry);
            if with_chroma {
                u_plane.predict(predictor, chroma_row, interlaced, start_c, 2, &ru);
                v_plane.predict(predictor, chroma_row, interlaced, start_c, 2, &rv);
                chroma_row += 1;
            }
        }
        Ok(yuv_to_image(
            w,
            h,
            &y_plane.data,
            (&u_plane.data, &v_plane.data, cw),
            1,
            u32::from(is_420),
        ))
    }

    /// Packed RGB (stored bottom row first, channel order B, G, R, A in memory).
    fn decode_rgb(&self, bits: &mut FrameBits, alpha: bool) -> Result<Image, CodecError> {
        let (w, h) = (self.width, self.height);
        let channels = 3 + usize::from(alpha);
        let (t_b, t_g, t_r) = (&self.tables[0], &self.tables[1], &self.tables[2]);
        let back = 1 + usize::from(self.header.interlaced);
        let plane = self.header.predictor == Predictor::Plane;

        let mut px = vec![0u8; w * h * channels];
        let mut res = vec![0u8; w * channels];
        let mut left = [0u8; 4];

        // The first pixel is stored as it is: A (32 bit only), R, G, B.
        if alpha {
            left[3] = bits.read(8) as u8;
        }
        left[2] = bits.read(8) as u8;
        left[1] = bits.read(8) as u8;
        left[0] = bits.read(8) as u8;
        if !alpha {
            bits.read(8);
        }
        px[..channels].copy_from_slice(&left[..channels]);

        for row in 0..h {
            let start = usize::from(row == 0);
            for p in res[start * channels..].chunks_exact_mut(channels) {
                if self.header.decorrelate {
                    let g = t_g.get(bits);
                    p[1] = g;
                    p[0] = t_b.get(bits).wrapping_add(g);
                    p[2] = t_r.get(bits).wrapping_add(g);
                } else {
                    p[0] = t_b.get(bits);
                    p[1] = t_g.get(bits);
                    p[2] = t_r.get(bits);
                }
                if alpha {
                    p[3] = t_r.get(bits);
                }
            }
            self.check(bits)?;

            let (head, tail) = px.split_at_mut(row * w * channels);
            let dst = &mut tail[..w * channels];
            for (d, r) in dst[start * channels..]
                .chunks_exact_mut(channels)
                .zip(res[start * channels..].chunks_exact(channels))
            {
                for c in 0..channels {
                    left[c] = left[c].wrapping_add(r[c]);
                    d[c] = left[c];
                }
            }
            if plane && row >= back {
                let above = &head[(row - back) * w * channels..][..w * channels];
                for (d, &a) in dst.iter_mut().zip(above) {
                    *d = d.wrapping_add(a);
                }
            }
        }

        let mut data = Vec::with_capacity(w * h * 3);
        for row in (0..h).rev() {
            for p in px[row * w * channels..][..w * channels].chunks_exact(channels) {
                data.extend([p[2], p[1], p[0]].map(sample));
            }
        }
        Ok(Image {
            width: w as u32,
            height: h as u32,
            channels: 3,
            data,
        })
    }

    /// Version 3: each plane is stored completely, starting with a prediction from zero.
    fn decode_planar(
        &self,
        bits: &mut FrameBits,
        chroma: bool,
        rgb: bool,
        h_shift: u32,
        v_shift: u32,
    ) -> Result<Image, CodecError> {
        let (w, h) = (self.width, self.height);
        let (cw, ch) = (w.div_ceil(1 << h_shift), h.div_ceil(1 << v_shift));
        let predictor = self.header.predictor;
        let plane_count = if chroma { 3 } else { 1 };

        let mut planes = Vec::with_capacity(plane_count);
        for index in 0..plane_count {
            let (pw, ph) = if index == 0 { (w, h) } else { (cw, ch) };
            let table = &self.tables[index];
            let mut plane = Plane::new(pw, ph);
            let mut res = vec![0u8; pw];
            for row in 0..ph {
                for r in &mut res {
                    *r = table.get(bits);
                }
                self.check(bits)?;
                plane.predict(predictor, row, false, 0, 0, &res);
                if row == 0 {
                    plane.top_left = plane.data[0];
                }
            }
            planes.push(plane.data);
        }

        if !chroma {
            return Ok(Image {
                width: w as u32,
                height: h as u32,
                channels: 1,
                data: planes[0].iter().map(|&v| sample(v)).collect(),
            });
        }
        if rgb {
            // Planes are G, B, R.
            let mut data = Vec::with_capacity(w * h * 3);
            for ((&r, &g), &b) in planes[2].iter().zip(&planes[0]).zip(&planes[1]) {
                data.extend([r, g, b].map(sample));
            }
            return Ok(Image {
                width: w as u32,
                height: h as u32,
                channels: 3,
                data,
            });
        }
        Ok(yuv_to_image(
            w,
            h,
            &planes[0],
            (&planes[1], &planes[2], cw),
            h_shift,
            v_shift,
        ))
    }
}

// ---------------------------------------------------------------------------------------------
// Output

fn sample(v: u8) -> f32 {
    f32::from(v) / 255.0
}

/// BT.601 limited range YCbCr to RGB, one lookup table per term (values in 0..=255 units).
struct YuvTables {
    y: [f32; 256],
    r_v: [f32; 256],
    g_u: [f32; 256],
    g_v: [f32; 256],
    b_u: [f32; 256],
}

fn yuv_tables() -> &'static YuvTables {
    static TABLES: OnceLock<YuvTables> = OnceLock::new();
    TABLES.get_or_init(|| {
        let mut t = YuvTables {
            y: [0.0; 256],
            r_v: [0.0; 256],
            g_u: [0.0; 256],
            g_v: [0.0; 256],
            b_u: [0.0; 256],
        };
        for i in 0..256 {
            let (y, c) = (i as f32 - 16.0, i as f32 - 128.0);
            t.y[i] = 1.164_383_6 * y;
            t.r_v[i] = 1.596_026_8 * c;
            t.g_u[i] = -0.391_762_3 * c;
            t.g_v[i] = -0.812_967_7 * c;
            t.b_u[i] = 2.017_232_1 * c;
        }
        t
    })
}

/// Converts planar YUV to an RGB image. `chroma` is `(u, v, row stride)`; chroma sample
/// `(x >> h_shift, y >> v_shift)` belongs to pixel `(x, y)`.
fn yuv_to_image(
    w: usize,
    h: usize,
    luma: &[u8],
    chroma: (&[u8], &[u8], usize),
    h_shift: u32,
    v_shift: u32,
) -> Image {
    let t = yuv_tables();
    let (u, v, stride) = chroma;
    let mut data = vec![0.0f32; w * h * 3];
    for (y, out_row) in data.chunks_exact_mut(w * 3).enumerate() {
        let luma_row = &luma[y * w..][..w];
        let chroma_at = (y >> v_shift) * stride;
        let u_row = &u[chroma_at..][..stride];
        let v_row = &v[chroma_at..][..stride];
        for (x, (px, &yy)) in out_row
            .as_chunks_mut::<3>()
            .0
            .iter_mut()
            .zip(luma_row)
            .enumerate()
        {
            let (uu, vv) = (
                usize::from(u_row[x >> h_shift]),
                usize::from(v_row[x >> h_shift]),
            );
            let base = t.y[usize::from(yy)];
            px[0] = (base + t.r_v[vv]).clamp(0.0, 255.0) / 255.0;
            px[1] = (base + t.g_u[uu] + t.g_v[vv]).clamp(0.0, 255.0) / 255.0;
            px[2] = (base + t.b_u[uu]).clamp(0.0, 255.0) / 255.0;
        }
    }
    Image {
        width: w as u32,
        height: h as u32,
        channels: 3,
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push(bits: &mut Vec<bool>, value: usize, count: usize) {
        bits.extend((0..count).rev().map(|i| (value >> i) & 1 == 1));
    }

    fn to_bytes(bits: &[bool]) -> Vec<u8> {
        bits.chunks(8)
            .map(|byte| {
                byte.iter()
                    .enumerate()
                    .fold(0u8, |acc, (i, &b)| acc | (u8::from(b) << (7 - i)))
            })
            .collect()
    }

    /// A frame as stored in the file: the bit string in 32 bit words with reversed byte order.
    fn frame_bytes(bits: &[bool]) -> Vec<u8> {
        swap_words(&to_bytes(bits))
    }

    /// The table description for `(length, run)` pairs; the rest of the 256 entries stay unused.
    fn table_bits(runs: &[(usize, usize)]) -> Vec<bool> {
        let mut bits = Vec::new();
        let mut used = 0;
        let run = |bits: &mut Vec<bool>, length: usize, count: usize| {
            if count < 8 {
                push(bits, count, 3);
                push(bits, length, 5);
            } else {
                push(bits, 0, 3);
                push(bits, length, 5);
                push(bits, count, 8);
            }
        };
        for &(length, count) in runs {
            run(&mut bits, length, count);
            used += count;
        }
        while used < 256 {
            let count = (256 - used).min(255);
            run(&mut bits, 0, count);
            used += count;
        }
        bits
    }

    #[test]
    fn frame_bits_read_little_endian_words_msb_first() {
        let mut bits = FrameBits::new(&[0x01, 0x02, 0x03, 0x04, 0xAA]);
        assert_eq!(bits.read(8), 0x04);
        assert_eq!(bits.read(4), 0x0);
        assert_eq!(bits.read(4), 0x3);
        assert_eq!(bits.read(16), 0x0201);
        // The partial last word is padded with zeros.
        assert_eq!(bits.read(8), 0x00);
        assert!(!bits.overrun());
        assert_eq!(bits.read(8), 0x00);
        assert!(bits.overrun());
    }

    /// Lengths per symbol from `(symbol, length)` pairs.
    fn lengths(pairs: &[(usize, u8)]) -> [u8; 256] {
        let mut l = [0u8; 256];
        for &(symbol, length) in pairs {
            l[symbol] = length;
        }
        l
    }

    #[test]
    fn canonical_codes_start_with_the_longest() {
        // Codes: symbol 7 = "1", symbol 3 = "01", symbol 9 = "000", symbol 200 = "001".
        let table = Huffman::new(&lengths(&[(7, 1), (3, 2), (9, 3), (200, 3)])).unwrap();
        let mut bits = Vec::new();
        for (code, len) in [(0b1, 1), (0b000, 3), (0b001, 3), (0b01, 2), (0b1, 1)] {
            push(&mut bits, code, len);
        }
        let bytes = frame_bytes(&bits);
        let mut reader = FrameBits::new(&bytes);
        let got: Vec<u8> = (0..5).map(|_| table.get(&mut reader)).collect();
        assert_eq!(got, [7, 9, 200, 3, 7]);
        assert!(!reader.bad_code);
    }

    #[test]
    fn codes_longer_than_the_lookup_table_decode() {
        // Unary codes: symbol i (0..=11) is i zeros and a one, symbol 12 is thirteen zeros and
        // symbol 13 is twelve zeros and a one, so the last two codes need the slow path.
        let mut pairs: Vec<(usize, u8)> = (0..13).map(|i| (i, i as u8 + 1)).collect();
        pairs.push((13, 13));
        let table = Huffman::new(&lengths(&pairs)).unwrap();
        let sequence = [13usize, 12, 11, 0, 5, 12, 13];
        let mut bits = Vec::new();
        for &symbol in &sequence {
            match symbol {
                13 => push(&mut bits, 1, 13),
                12 => push(&mut bits, 0, 13),
                i => push(&mut bits, 1, i + 1),
            }
        }
        let bytes = frame_bytes(&bits);
        let mut reader = FrameBits::new(&bytes);
        let got: Vec<usize> = sequence
            .iter()
            .map(|_| usize::from(table.get(&mut reader)))
            .collect();
        assert_eq!(got, sequence);
        assert!(!reader.bad_code);
    }

    #[test]
    fn invalid_length_sets_are_rejected() {
        // Three codes of one bit do not fit.
        assert!(Huffman::new(&lengths(&[(0, 1), (1, 1), (2, 1)])).is_err());
        // A lone one-bit code leaves half of the code space unassigned.
        assert!(Huffman::new(&lengths(&[(5, 1)])).is_err());
    }

    #[test]
    fn code_outside_the_table_is_flagged() {
        // A table without any symbols (an unused plane) cannot decode anything.
        let table = Huffman::new(&[0u8; 256]).unwrap();
        let mut reader = FrameBits::new(&[0xFF; 4]);
        assert_eq!(table.get(&mut reader), 0);
        assert!(reader.bad_code);
    }

    #[test]
    fn table_description_is_run_length_coded() {
        // Two one-bit codes for symbols 0 and 1, written as one run, then 254 unused entries.
        let bits = table_bits(&[(1, 2)]);
        let bytes = to_bytes(&bits);
        let mut reader = ByteBits {
            data: &bytes,
            pos: 0,
        };
        let tables = read_tables(Codec("test"), 1, |n| reader.read(n)).unwrap();
        let mut stream = Vec::new();
        for code in [0, 1, 1, 0] {
            push(&mut stream, code, 1);
        }
        let data = frame_bytes(&stream);
        let mut frame = FrameBits::new(&data);
        let got: Vec<u8> = (0..4).map(|_| tables[0].get(&mut frame)).collect();
        assert_eq!(got, [0, 1, 1, 0]);
        // Reading stops at the end of the table, not at a byte boundary of the next one.
        assert_eq!(reader.pos, bits.len());
    }

    #[test]
    fn truncated_table_is_corrupt() {
        let bytes = to_bytes(&table_bits(&[(1, 2)]));
        let mut reader = ByteBits {
            data: &bytes[..bytes.len() - 2],
            pos: 0,
        };
        assert!(matches!(
            read_tables(Codec("test"), 1, |n| reader.read(n)),
            Err(CodecError::Corrupt { .. })
        ));
    }

    /// A 3x1 gray stream (format version 3): symbols 10 and 20 are the one-bit codes "0" and "1".
    fn gray_stream(predictor: u8, pixels: &[bool]) -> (Vec<u8>, Vec<u8>) {
        let mut extra = vec![predictor, 0x70, 0x20, 1];
        extra.extend(to_bytes(&table_bits(&[(0, 10), (1, 1), (0, 9), (1, 1)])));
        (extra, frame_bytes(pixels))
    }

    #[test]
    fn gray_stream_is_left_predicted_from_zero() {
        // Residuals 10, 20, 10 -> 10, 30, 40.
        let (extra, data) = gray_stream(0, &[false, true, false]);
        let format = FrameFormat {
            width: 3,
            height: 1,
            bits_per_pixel: 24,
            extra: &extra,
        };
        let img = decode_ffvhuff(&format, &data).unwrap();
        assert_eq!((img.width, img.height, img.channels), (3, 1, 1));
        let want = [10.0 / 255.0, 30.0 / 255.0, 40.0 / 255.0];
        assert_eq!(img.data, want);
    }

    #[test]
    fn frame_that_ends_early_is_corrupt() {
        let (extra, data) = gray_stream(0, &[false, true, false]);
        let format = FrameFormat {
            width: 64,
            height: 4,
            bits_per_pixel: 24,
            extra: &extra,
        };
        assert!(matches!(
            decode_ffvhuff(&format, &data),
            Err(CodecError::Corrupt { .. })
        ));
    }

    #[test]
    fn empty_frame_is_corrupt() {
        let (extra, _) = gray_stream(0, &[]);
        let format = FrameFormat {
            width: 3,
            height: 1,
            bits_per_pixel: 24,
            extra: &extra,
        };
        assert!(matches!(
            decode_huffyuv(&format, &[]),
            Err(CodecError::Corrupt { .. })
        ));
    }

    #[test]
    fn old_streams_without_tables_are_unsupported() {
        let format = FrameFormat {
            width: 4,
            height: 4,
            bits_per_pixel: 16,
            extra: &[],
        };
        assert!(matches!(
            decode_huffyuv(&format, &[0; 64]),
            Err(CodecError::Unsupported { .. })
        ));
    }

    #[test]
    fn unsupported_variants_are_named() {
        let (mut extra, data) = gray_stream(0, &[false, true, false]);
        let decode_with = |extra: &[u8], bpp: u16| {
            let format = FrameFormat {
                width: 3,
                height: 1,
                bits_per_pixel: bpp,
                extra,
            };
            decode_ffvhuff(&format, &data)
        };
        // Predictor 5 does not exist.
        extra[0] = 5;
        assert!(matches!(
            decode_with(&extra, 24),
            Err(CodecError::Unsupported { .. })
        ));
        // 10 bits per sample.
        extra[0] = 0;
        extra[1] = 0x90;
        assert!(matches!(
            decode_with(&extra, 24),
            Err(CodecError::Unsupported { .. })
        ));
        // Packed RGB with the median predictor.
        extra[1] = 24;
        extra[0] = 2;
        extra[3] = 0;
        assert!(matches!(
            decode_with(&extra, 24),
            Err(CodecError::Unsupported { .. })
        ));
    }

    #[test]
    fn median_prediction_uses_left_top_and_gradient() {
        let top = [10u8, 20, 30];
        let res = [1u8, 1, 1];
        let mut dst = [0u8; 3];
        let (mut left, mut top_left) = (5u8, 5u8);
        median_row(&mut dst, &top, &res, &mut left, &mut top_left);
        // pixel 0: median(5, 10, 5 + 10 - 5) = 10 -> 11; pixel 1: median(11, 20, 11 + 20 - 10) = 20 -> 21;
        // pixel 2: median(21, 30, 21 + 30 - 20) = 30 -> 31.
        assert_eq!(dst, [11, 21, 31]);
        assert_eq!((left, top_left), (31, 30));
    }
}
