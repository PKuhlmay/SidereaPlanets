//! Uncompressed AVI files (RIFF/AVI, OpenDML `AVIX` segments included).
//!
//! Only `BI_RGB` video is decoded (8 bit with palette, 24 bit, 32 bit); that is what
//! capture programs write for raw planetary recordings. Compressed codecs are rejected.

use planets_core::Image;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

#[derive(Debug, thiserror::Error)]
pub enum AviError {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("not an AVI file")]
    NotAvi,
    #[error("AVI file has no video stream")]
    NoVideoStream,
    #[error("unsupported compression '{0}' (only uncompressed AVI is supported)")]
    UnsupportedCompression(String),
    #[error("unsupported bit depth {0}")]
    UnsupportedBitDepth(u16),
    #[error("frame {index} is truncated")]
    TruncatedFrame { index: u32 },
    #[error("frame {index} out of range ({count} frames)")]
    FrameOutOfRange { index: u32, count: u32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct AviInfo {
    pub width: u32,
    pub height: u32,
    /// Bits per pixel as stored in the file (8, 24 or 32).
    pub bits_per_pixel: u16,
    pub frame_count: u32,
    pub fps: f64,
    /// `true` when the frames are returned as a single channel.
    pub mono: bool,
}

#[derive(Clone, Copy)]
struct FrameLoc {
    offset: u64,
    len: u32,
}

struct Bitmap {
    width: u32,
    height: u32,
    top_down: bool,
    bpp: u16,
    compression: [u8; 4],
    /// RGB palette entries (8 bit video only).
    palette: Vec<[u8; 3]>,
}

#[derive(Default)]
struct Parsed {
    stream_count: u32,
    video_stream: Option<u32>,
    us_per_frame: u32,
    scale: u32,
    rate: u32,
    bitmap: Option<Bitmap>,
    frames: Vec<FrameLoc>,
}

fn read_at(file: &mut File, pos: u64, len: usize) -> std::io::Result<Vec<u8>> {
    file.seek(SeekFrom::Start(pos))?;
    let mut buf = vec![0u8; len];
    file.read_exact(&mut buf)?;
    Ok(buf)
}

fn u32_at(b: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}

fn i32_at(b: &[u8], off: usize) -> i32 {
    u32_at(b, off) as i32
}

/// Video frames are stored as `00db` (uncompressed) or `00dc` (compressed) chunks; the digits are the stream index.
fn is_video_chunk(id: &[u8; 4], stream: u32) -> bool {
    id[0].is_ascii_digit()
        && id[1].is_ascii_digit()
        && u32::from(id[0] - b'0') * 10 + u32::from(id[1] - b'0') == stream
        && (&id[2..] == b"db" || &id[2..] == b"dc")
}

fn parse_bitmap(file: &mut File, body: u64, size: u32) -> Result<Bitmap, AviError> {
    let b = read_at(file, body, 40)?;
    let header_size = u32_at(&b, 0);
    let height = i32_at(&b, 8);
    let bpp = u16::from_le_bytes([b[14], b[15]]);
    let mut palette = Vec::new();
    if bpp <= 8 {
        let used = u32_at(&b, 32);
        let wanted = if used == 0 { 1u32 << bpp } else { used };
        let available = size.saturating_sub(header_size) / 4;
        let entries = wanted.min(available) as usize;
        let raw = read_at(file, body + u64::from(header_size), entries * 4)?;
        palette = raw
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| [c[2], c[1], c[0]])
            .collect();
    }
    Ok(Bitmap {
        width: i32_at(&b, 4).max(0) as u32,
        height: height.unsigned_abs(),
        top_down: height < 0,
        bpp,
        compression: [b[16], b[17], b[18], b[19]],
        palette,
    })
}

fn parse_chunks(
    file: &mut File,
    start: u64,
    end: u64,
    in_movi: bool,
    p: &mut Parsed,
) -> Result<(), AviError> {
    let mut pos = start;
    while pos + 8 <= end {
        let head = read_at(file, pos, 8)?;
        let id = [head[0], head[1], head[2], head[3]];
        let size = u32_at(&head, 4);
        let body = pos + 8;
        let body_end = (body + u64::from(size)).min(end);
        match &id {
            b"LIST" if size >= 4 => {
                let form = read_at(file, body, 4)?;
                match &form[..] {
                    b"hdrl" | b"strl" => parse_chunks(file, body + 4, body_end, false, p)?,
                    b"movi" => parse_chunks(file, body + 4, body_end, true, p)?,
                    b"rec " if in_movi => parse_chunks(file, body + 4, body_end, true, p)?,
                    _ => {}
                }
            }
            b"avih" if size >= 4 => {
                p.us_per_frame = u32_at(&read_at(file, body, 4)?, 0);
            }
            b"strh" if size >= 28 => {
                let b = read_at(file, body, 28)?;
                let index = p.stream_count;
                p.stream_count += 1;
                if &b[..4] == b"vids" && p.video_stream.is_none() {
                    p.video_stream = Some(index);
                    p.scale = u32_at(&b, 20);
                    p.rate = u32_at(&b, 24);
                }
            }
            b"strf"
                if size >= 40
                    && p.bitmap.is_none()
                    && p.video_stream == Some(p.stream_count.wrapping_sub(1)) =>
            {
                p.bitmap = Some(parse_bitmap(file, body, size)?);
            }
            _ if in_movi
                && p.video_stream
                    .is_some_and(|stream| is_video_chunk(&id, stream)) =>
            {
                p.frames.push(FrameLoc {
                    offset: body,
                    len: size,
                });
            }
            _ => {}
        }
        pos = body + u64::from(size) + u64::from(size & 1);
    }
    Ok(())
}

pub struct AviReader {
    file: File,
    info: AviInfo,
    top_down: bool,
    palette: Vec<[u8; 3]>,
    frames: Vec<FrameLoc>,
}

impl AviReader {
    pub fn open(path: &Path) -> Result<Self, AviError> {
        let mut file = File::open(path)?;
        let len = file.metadata()?.len();
        let mut parsed = Parsed::default();
        let mut pos = 0u64;
        let mut is_avi = false;
        while pos + 12 <= len {
            let head = read_at(&mut file, pos, 12)?;
            let form = &head[8..12];
            if &head[..4] != b"RIFF" || (form != b"AVI " && form != b"AVIX") {
                break;
            }
            is_avi = true;
            let size = u64::from(u32_at(&head, 4));
            let end = (pos + 8 + size).min(len);
            parse_chunks(&mut file, pos + 12, end, false, &mut parsed)?;
            pos += 8 + size + (size & 1);
        }
        if !is_avi {
            return Err(AviError::NotAvi);
        }

        let mut bitmap = parsed.bitmap.ok_or(AviError::NoVideoStream)?;
        if bitmap.compression != [0; 4] {
            let name = String::from_utf8_lossy(&bitmap.compression)
                .trim()
                .to_owned();
            return Err(AviError::UnsupportedCompression(name));
        }
        if !matches!(bitmap.bpp, 8 | 24 | 32) {
            return Err(AviError::UnsupportedBitDepth(bitmap.bpp));
        }
        if bitmap.bpp == 8 && bitmap.palette.is_empty() {
            bitmap.palette = (0..=255u8).map(|i| [i, i, i]).collect();
        }
        let mono = bitmap.bpp == 8 && bitmap.palette.iter().all(|c| c[0] == c[1] && c[1] == c[2]);
        let fps = if parsed.scale > 0 && parsed.rate > 0 {
            f64::from(parsed.rate) / f64::from(parsed.scale)
        } else if parsed.us_per_frame > 0 {
            1e6 / f64::from(parsed.us_per_frame)
        } else {
            0.0
        };

        Ok(Self {
            file,
            info: AviInfo {
                width: bitmap.width,
                height: bitmap.height,
                bits_per_pixel: bitmap.bpp,
                frame_count: parsed.frames.len() as u32,
                fps,
                mono,
            },
            top_down: bitmap.top_down,
            palette: bitmap.palette,
            frames: parsed.frames,
        })
    }

    pub fn info(&self) -> &AviInfo {
        &self.info
    }

    /// Reads one frame as normalised RGB samples, or a single channel for grayscale palettes.
    pub fn read_frame(&mut self, index: u32) -> Result<Image, AviError> {
        let Some(&loc) = self.frames.get(index as usize) else {
            return Err(AviError::FrameOutOfRange {
                index,
                count: self.info.frame_count,
            });
        };
        let (w, h) = (self.info.width as usize, self.info.height as usize);
        let bpp = usize::from(self.info.bits_per_pixel);
        // Rows are padded to 4-byte boundaries.
        let stride = (w * bpp).div_ceil(32) * 4;
        if (loc.len as usize) < stride * h {
            return Err(AviError::TruncatedFrame { index });
        }
        let raw = read_at(&mut self.file, loc.offset, stride * h)?;

        let channels: u8 = if self.info.mono { 1 } else { 3 };
        let mut data = Vec::with_capacity(w * h * usize::from(channels));
        for y in 0..h {
            // Bottom-up DIBs store the last image row first.
            let src = if self.top_down { y } else { h - 1 - y };
            let row = &raw[src * stride..src * stride + w * bpp / 8];
            match bpp {
                8 => {
                    for &i in row {
                        let c = self.palette.get(usize::from(i)).copied().unwrap_or([0; 3]);
                        if self.info.mono {
                            data.push(f32::from(c[0]) / 255.0);
                        } else {
                            data.extend(c.map(|v| f32::from(v) / 255.0));
                        }
                    }
                }
                24 => {
                    for px in row.as_chunks::<3>().0 {
                        data.extend([px[2], px[1], px[0]].map(|v| f32::from(v) / 255.0));
                    }
                }
                _ => {
                    for px in row.as_chunks::<4>().0 {
                        data.extend([px[2], px[1], px[0]].map(|v| f32::from(v) / 255.0));
                    }
                }
            }
        }
        Ok(Image {
            width: self.info.width,
            height: self.info.height,
            channels,
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(id: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut v = id.to_vec();
        v.extend_from_slice(&(body.len() as u32).to_le_bytes());
        v.extend_from_slice(body);
        if body.len() % 2 == 1 {
            v.push(0);
        }
        v
    }

    fn list(form: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut inner = form.to_vec();
        inner.extend_from_slice(body);
        chunk(b"LIST", &inner)
    }

    struct Spec<'a> {
        width: i32,
        height: i32,
        bpp: u16,
        compression: &'a [u8; 4],
        /// BGR palette entries, written as BGRX.
        palette: &'a [[u8; 3]],
    }

    fn header_list(s: &Spec, frame_count: u32) -> Vec<u8> {
        let mut avih = vec![0u8; 56];
        avih[0..4].copy_from_slice(&40_000u32.to_le_bytes());
        avih[16..20].copy_from_slice(&frame_count.to_le_bytes());
        avih[24..28].copy_from_slice(&1u32.to_le_bytes());
        avih[32..36].copy_from_slice(&s.width.to_le_bytes());
        avih[36..40].copy_from_slice(&s.height.to_le_bytes());

        let mut strh = vec![0u8; 56];
        strh[0..4].copy_from_slice(b"vids");
        strh[4..8].copy_from_slice(b"DIB ");
        strh[20..24].copy_from_slice(&1000u32.to_le_bytes());
        strh[24..28].copy_from_slice(&25_000u32.to_le_bytes());

        let mut strf = vec![0u8; 40];
        strf[0..4].copy_from_slice(&40u32.to_le_bytes());
        strf[4..8].copy_from_slice(&s.width.to_le_bytes());
        strf[8..12].copy_from_slice(&s.height.to_le_bytes());
        strf[12..14].copy_from_slice(&1u16.to_le_bytes());
        strf[14..16].copy_from_slice(&s.bpp.to_le_bytes());
        strf[16..20].copy_from_slice(s.compression);
        strf[32..36].copy_from_slice(&(s.palette.len() as u32).to_le_bytes());
        for c in s.palette {
            strf.extend_from_slice(&[c[0], c[1], c[2], 0]);
        }

        let strl = list(
            b"strl",
            &[chunk(b"strh", &strh), chunk(b"strf", &strf)].concat(),
        );
        list(b"hdrl", &[chunk(b"avih", &avih), strl].concat())
    }

    /// Frame chunks, each preceded by an audio chunk that must be ignored.
    fn movi(frames: &[Vec<u8>]) -> Vec<u8> {
        let body: Vec<u8> = frames
            .iter()
            .flat_map(|f| [chunk(b"01wb", &[1, 2, 3]), chunk(b"00db", f)].concat())
            .collect();
        list(b"movi", &body)
    }

    fn riff(form: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut inner = form.to_vec();
        inner.extend_from_slice(body);
        chunk(b"RIFF", &inner)
    }

    fn avi(s: &Spec, frames: &[Vec<u8>]) -> Vec<u8> {
        riff(
            b"AVI ",
            &[header_list(s, frames.len() as u32), movi(frames)].concat(),
        )
    }

    fn temp_file(name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "planets-avi-test-{}-{name}.avi",
            std::process::id()
        ));
        std::fs::write(&path, bytes).unwrap();
        path
    }

    const RGB: &[u8; 4] = &[0, 0, 0, 0];

    fn spec<'a>(width: i32, height: i32, bpp: u16, palette: &'a [[u8; 3]]) -> Spec<'a> {
        Spec {
            width,
            height,
            bpp,
            compression: RGB,
            palette,
        }
    }

    #[test]
    fn parses_stream_info() {
        let bytes = avi(&spec(2, 3, 24, &[]), &[vec![0; 24], vec![0; 24]]);
        let r = AviReader::open(&temp_file("info", &bytes)).unwrap();
        let i = r.info();
        assert_eq!(
            (i.width, i.height, i.bits_per_pixel, i.frame_count),
            (2, 3, 24, 2)
        );
        assert!((i.fps - 25.0).abs() < 1e-9);
        assert!(!i.mono);
    }

    #[test]
    fn rejects_non_avi() {
        let path = temp_file("junk", b"this is definitely not an avi file at all");
        assert!(matches!(AviReader::open(&path), Err(AviError::NotAvi)));
    }

    #[test]
    fn rejects_compressed_video() {
        let s = Spec {
            compression: b"MJPG",
            ..spec(2, 2, 24, &[])
        };
        let path = temp_file("mjpg", &avi(&s, &[vec![0; 24]]));
        assert!(
            matches!(AviReader::open(&path), Err(AviError::UnsupportedCompression(c)) if c == "MJPG")
        );
    }

    #[test]
    fn bgr24_is_flipped_and_swapped_with_row_padding() {
        // 1x2 image, stride is 4 bytes (3 pixel bytes + 1 pad). The bottom row comes first in the file.
        let frame = vec![10, 20, 30, 0, 40, 50, 60, 0];
        let path = temp_file("bgr24", &avi(&spec(1, 2, 24, &[]), &[frame]));
        let img = AviReader::open(&path).unwrap().read_frame(0).unwrap();
        assert_eq!((img.width, img.height, img.channels), (1, 2, 3));
        let px = |v: [u8; 3]| v.map(|c| f32::from(c) / 255.0).to_vec();
        // Top row is the file's last row: BGR (40,50,60) -> RGB (60,50,40).
        assert_eq!(img.data[..3], px([60, 50, 40])[..]);
        assert_eq!(img.data[3..], px([30, 20, 10])[..]);
    }

    #[test]
    fn top_down_dib_is_not_flipped() {
        // Negative height means rows are stored top to bottom: red on top, blue below.
        let frame = vec![0, 0, 255, 0, 255, 0, 0, 0];
        let path = temp_file("topdown", &avi(&spec(1, -2, 24, &[]), &[frame]));
        let img = AviReader::open(&path).unwrap().read_frame(0).unwrap();
        assert_eq!(img.height, 2);
        assert_eq!(img.data, vec![1.0, 0.0, 0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn gray_palette_gives_mono_image() {
        let gray: Vec<[u8; 3]> = (0..=255u8).map(|i| [i, i, i]).collect();
        // 2x1 image, stride 4. Values 0 and 255.
        let path = temp_file("gray8", &avi(&spec(2, 1, 8, &gray), &[vec![0, 255, 0, 0]]));
        let mut r = AviReader::open(&path).unwrap();
        assert!(r.info().mono);
        let img = r.read_frame(0).unwrap();
        assert_eq!((img.channels, img.data.len()), (1, 2));
        assert_eq!(img.data, vec![0.0, 1.0]);
    }

    #[test]
    fn colour_palette_is_applied() {
        let palette = [[0, 0, 255], [255, 0, 0]]; // BGR: red, blue
        let path = temp_file("pal8", &avi(&spec(2, 1, 8, &palette), &[vec![0, 1, 0, 0]]));
        let img = AviReader::open(&path).unwrap().read_frame(0).unwrap();
        assert_eq!(img.channels, 3);
        assert_eq!(img.data, vec![1.0, 0.0, 0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn bgra32_drops_alpha() {
        let path = temp_file("bgra", &avi(&spec(1, 1, 32, &[]), &[vec![255, 128, 0, 7]]));
        let img = AviReader::open(&path).unwrap().read_frame(0).unwrap();
        assert_eq!(img.channels, 3);
        assert_eq!(img.data, vec![0.0, 128.0 / 255.0, 1.0]);
    }

    #[test]
    fn frames_in_avix_segments_are_found() {
        let s = spec(1, 1, 8, &[[0, 0, 0], [255, 255, 255]]);
        let first = riff(
            b"AVI ",
            &[header_list(&s, 2), movi(&[vec![0, 0, 0, 0]])].concat(),
        );
        let second = riff(b"AVIX", &movi(&[vec![1, 0, 0, 0]]));
        let path = temp_file("avix", &[first, second].concat());
        let mut r = AviReader::open(&path).unwrap();
        assert_eq!(r.info().frame_count, 2);
        assert_eq!(r.read_frame(1).unwrap().data, vec![1.0]);
    }

    #[test]
    fn frame_index_is_checked() {
        let path = temp_file("range", &avi(&spec(1, 1, 24, &[]), &[vec![0; 4]]));
        let mut r = AviReader::open(&path).unwrap();
        assert!(matches!(
            r.read_frame(1),
            Err(AviError::FrameOutOfRange { index: 1, count: 1 })
        ));
    }
}
