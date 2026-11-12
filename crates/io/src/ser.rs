//! SER video files (public format by Lucam-Recorder, version 3).

use planets_core::Image;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

const HEADER_LEN: usize = 178;

#[derive(Debug, thiserror::Error)]
pub enum SerError {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("not a SER file")]
    BadMagic,
    #[error("unsupported pixel depth {0}")]
    BadDepth(i32),
    #[error("unsupported color id {0}")]
    BadColorId(i32),
    #[error("frame {index} out of range ({count} frames)")]
    FrameOutOfRange { index: u32, count: u32 },
}

/// Colour layout of the stored frames. Bayer variants are returned as raw mono planes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorId {
    Mono,
    Bayer(BayerPattern),
    Rgb,
    Bgr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BayerPattern {
    Rggb,
    Grbg,
    Gbrg,
    Bggr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerHeader {
    pub color: ColorId,
    pub little_endian: bool,
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub frame_count: u32,
    pub observer: String,
    pub instrument: String,
    pub telescope: String,
    /// `DateTimeUTC` of the header: .NET ticks (100 ns since 0001-01-01), 0 if the program did not write it.
    pub date_time_utc: i64,
}

/// .NET ticks (100 ns since 0001-01-01) per second, and the ticks at the Unix epoch.
const TICKS_PER_SECOND: f64 = 1e7;
const UNIX_EPOCH_TICKS: i64 = 621_355_968_000_000_000;

/// Unix seconds of a .NET tick count, `None` if it is not a time between 1990 and 2100 (cameras that write no time leave zeros).
fn unix_from_ticks(ticks: i64) -> Option<f64> {
    let seconds = (ticks - UNIX_EPOCH_TICKS) as f64 / TICKS_PER_SECOND;
    (631_152_000.0..4_102_444_800.0)
        .contains(&seconds)
        .then_some(seconds)
}

fn i32_at(b: &[u8], off: usize) -> i32 {
    i32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}

fn text_at(b: &[u8], off: usize) -> String {
    let raw = &b[off..off + 40];
    let end = raw.iter().position(|&c| c == 0).unwrap_or(raw.len());
    String::from_utf8_lossy(&raw[..end]).trim().to_owned()
}

impl SerHeader {
    pub fn parse(bytes: &[u8]) -> Result<Self, SerError> {
        if bytes.len() < HEADER_LEN || &bytes[..14] != b"LUCAM-RECORDER" {
            return Err(SerError::BadMagic);
        }
        let color = match i32_at(bytes, 18) {
            0 => ColorId::Mono,
            8 => ColorId::Bayer(BayerPattern::Rggb),
            9 => ColorId::Bayer(BayerPattern::Grbg),
            10 => ColorId::Bayer(BayerPattern::Gbrg),
            11 => ColorId::Bayer(BayerPattern::Bggr),
            100 => ColorId::Rgb,
            101 => ColorId::Bgr,
            other => return Err(SerError::BadColorId(other)),
        };
        let depth = i32_at(bytes, 34);
        if !(1..=16).contains(&depth) {
            return Err(SerError::BadDepth(depth));
        }
        Ok(Self {
            color,
            little_endian: i32_at(bytes, 22) != 0,
            width: i32_at(bytes, 26).max(0) as u32,
            height: i32_at(bytes, 30).max(0) as u32,
            bit_depth: depth as u8,
            frame_count: i32_at(bytes, 38).max(0) as u32,
            observer: text_at(bytes, 42),
            instrument: text_at(bytes, 82),
            telescope: text_at(bytes, 122),
            date_time_utc: i64::from_le_bytes(bytes[170..178].try_into().unwrap_or([0; 8])),
        })
    }

    fn planes(&self) -> usize {
        match self.color {
            ColorId::Rgb | ColorId::Bgr => 3,
            _ => 1,
        }
    }

    fn bytes_per_sample(&self) -> usize {
        if self.bit_depth > 8 { 2 } else { 1 }
    }

    fn frame_len(&self) -> usize {
        self.width as usize * self.height as usize * self.planes() * self.bytes_per_sample()
    }
}

/// The trailer after the last frame: one 64-bit tick count (UTC) per frame.
fn read_timestamps(file: &mut File, header: &SerHeader) -> Result<Option<Vec<f64>>, SerError> {
    let count = header.frame_count as usize;
    let start = (HEADER_LEN + count * header.frame_len()) as u64;
    if count == 0 || file.metadata()?.len() < start + 8 * count as u64 {
        return Ok(None);
    }
    let mut raw = vec![0u8; 8 * count];
    file.seek(SeekFrom::Start(start))?;
    file.read_exact(&mut raw)?;
    let times: Option<Vec<f64>> = raw
        .as_chunks::<8>()
        .0
        .iter()
        .map(|c| unix_from_ticks(i64::from_le_bytes(*c)))
        .collect();
    Ok(times)
}

/// A SER recording opened for random frame access.
pub struct SerReader {
    file: File,
    header: SerHeader,
    /// The time of every frame in Unix seconds (UTC) from the trailer, if the file has one.
    timestamps: Option<Vec<f64>>,
}

impl SerReader {
    pub fn open(path: &Path) -> Result<Self, SerError> {
        let mut file = File::open(path)?;
        let mut buf = [0u8; HEADER_LEN];
        let n = file.read(&mut buf)?;
        let header = SerHeader::parse(&buf[..n])?;
        let timestamps = read_timestamps(&mut file, &header)?;
        Ok(Self {
            file,
            header,
            timestamps,
        })
    }

    /// The time of every frame in Unix seconds (UTC), from the trailer of the file. `None` if the file has none or the times
    /// are not plausible (all zero, or not between 1990 and 2100).
    pub fn timestamps(&self) -> Option<&[f64]> {
        self.timestamps.as_deref()
    }

    /// When the recording started in Unix seconds (UTC): the first frame's time, else the `DateTimeUTC` of the header.
    pub fn start_time(&self) -> Option<f64> {
        self.timestamps
            .as_ref()
            .and_then(|t| t.first().copied())
            .or_else(|| unix_from_ticks(self.header.date_time_utc))
    }

    pub fn header(&self) -> &SerHeader {
        &self.header
    }

    /// Reads one frame as normalised samples. BGR is swapped to RGB, Bayer stays a raw mono plane.
    pub fn read_frame(&mut self, index: u32) -> Result<Image, SerError> {
        let h = &self.header;
        if index >= h.frame_count {
            return Err(SerError::FrameOutOfRange {
                index,
                count: h.frame_count,
            });
        }
        let frame_len = h.frame_len();
        let mut raw = vec![0u8; frame_len];
        self.file.seek(SeekFrom::Start(
            (HEADER_LEN + index as usize * frame_len) as u64,
        ))?;
        self.file.read_exact(&mut raw)?;

        // Depths between 9 and 16 bit are stored in 16-bit words; scale by the true maximum.
        let max = ((1u32 << h.bit_depth) - 1) as f32;
        let mut data: Vec<f32> = if h.bytes_per_sample() == 1 {
            raw.iter().map(|&v| f32::from(v) / max).collect()
        } else {
            raw.as_chunks::<2>()
                .0
                .iter()
                .map(|c| {
                    let word = if h.little_endian {
                        u16::from_le_bytes([c[0], c[1]])
                    } else {
                        u16::from_be_bytes([c[0], c[1]])
                    };
                    f32::from(word) / max
                })
                .collect()
        };
        if h.color == ColorId::Bgr {
            data.as_chunks_mut::<3>()
                .0
                .iter_mut()
                .for_each(|px| px.swap(0, 2));
        }
        Ok(Image {
            width: h.width,
            height: h.height,
            channels: h.planes() as u8,
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a SER file in memory: header followed by the given raw frame bytes.
    fn build(
        color_id: i32,
        little_endian: i32,
        w: i32,
        h: i32,
        depth: i32,
        frames: &[Vec<u8>],
    ) -> Vec<u8> {
        let mut b = vec![0u8; HEADER_LEN];
        b[..14].copy_from_slice(b"LUCAM-RECORDER");
        b[18..22].copy_from_slice(&color_id.to_le_bytes());
        b[22..26].copy_from_slice(&little_endian.to_le_bytes());
        b[26..30].copy_from_slice(&w.to_le_bytes());
        b[30..34].copy_from_slice(&h.to_le_bytes());
        b[34..38].copy_from_slice(&depth.to_le_bytes());
        b[38..42].copy_from_slice(&(frames.len() as i32).to_le_bytes());
        b[42..42 + 5].copy_from_slice(b"Alice");
        b[82..82 + 7].copy_from_slice(b"ASI120M");
        b[122..122 + 3].copy_from_slice(b"C11");
        for f in frames {
            b.extend_from_slice(f);
        }
        b
    }

    fn temp_file(name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "planets-ser-test-{}-{name}.ser",
            std::process::id()
        ));
        std::fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn parses_header_fields() {
        let bytes = build(0, 1, 4, 2, 8, &[vec![0; 8], vec![0; 8]]);
        let h = SerHeader::parse(&bytes).unwrap();
        assert_eq!(h.color, ColorId::Mono);
        assert!(h.little_endian);
        assert_eq!(
            (h.width, h.height, h.bit_depth, h.frame_count),
            (4, 2, 8, 2)
        );
        assert_eq!(h.observer, "Alice");
        assert_eq!(h.instrument, "ASI120M");
        assert_eq!(h.telescope, "C11");
    }

    #[test]
    fn rejects_wrong_magic() {
        let mut bytes = build(0, 1, 1, 1, 8, &[vec![0]]);
        bytes[0] = b'X';
        assert!(matches!(SerHeader::parse(&bytes), Err(SerError::BadMagic)));
    }

    #[test]
    fn reads_8bit_mono_frame_normalised() {
        let frame = vec![0, 51, 102, 255];
        let path = temp_file("mono8", &build(0, 1, 2, 2, 8, &[frame]));
        let mut r = SerReader::open(&path).unwrap();
        let img = r.read_frame(0).unwrap();
        assert_eq!((img.width, img.height, img.channels), (2, 2, 1));
        assert!((img.data[1] - 0.2).abs() < 1e-6);
        assert!((img.data[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn reads_16bit_little_endian_frame() {
        let frame = 0xFFFFu16
            .to_le_bytes()
            .into_iter()
            .chain(0x8000u16.to_le_bytes())
            .collect();
        let path = temp_file("mono16", &build(0, 1, 2, 1, 16, &[frame]));
        let img = SerReader::open(&path).unwrap().read_frame(0).unwrap();
        assert!((img.data[0] - 1.0).abs() < 1e-6);
        assert!((img.data[1] - 32768.0 / 65535.0).abs() < 1e-6);
    }

    #[test]
    fn bgr_frame_is_returned_as_rgb() {
        let path = temp_file("bgr", &build(101, 1, 1, 1, 8, &[vec![255, 0, 0]]));
        let img = SerReader::open(&path).unwrap().read_frame(0).unwrap();
        assert_eq!(img.channels, 3);
        assert_eq!(img.data, vec![0.0, 0.0, 1.0]);
    }

    #[test]
    fn frame_index_is_checked() {
        let path = temp_file("range", &build(0, 1, 1, 1, 8, &[vec![0]]));
        let mut r = SerReader::open(&path).unwrap();
        assert!(matches!(
            r.read_frame(1),
            Err(SerError::FrameOutOfRange { index: 1, count: 1 })
        ));
    }

    #[test]
    fn second_frame_is_found() {
        let path = temp_file("second", &build(0, 1, 1, 1, 8, &[vec![0], vec![255]]));
        let img = SerReader::open(&path).unwrap().read_frame(1).unwrap();
        assert_eq!(img.data, vec![1.0]);
    }

    /// The .NET ticks of a Unix time.
    fn ticks(unix: f64) -> i64 {
        (unix * TICKS_PER_SECOND) as i64 + UNIX_EPOCH_TICKS
    }

    /// A SER file of `frames` 1 x 1 8-bit frames with the given header time and trailer ticks.
    fn timed(tag: &str, header_ticks: i64, trailer: &[i64]) -> std::path::PathBuf {
        let frames: Vec<Vec<u8>> = (0..trailer.len().max(2)).map(|i| vec![i as u8]).collect();
        let mut bytes = build(0, 1, 1, 1, 8, &frames);
        bytes[170..178].copy_from_slice(&header_ticks.to_le_bytes());
        for t in trailer {
            bytes.extend_from_slice(&t.to_le_bytes());
        }
        temp_file(&format!("timed-{tag}"), &bytes)
    }

    #[test]
    fn reads_the_frame_timestamps_of_the_trailer() {
        let t0 = planets_core::planet::unix_seconds(2023, 3, 10, 21, 5, 0.0);
        let path = timed("trailer", 0, &[ticks(t0), ticks(t0 + 0.1), ticks(t0 + 0.2)]);
        let r = SerReader::open(&path).unwrap();
        let times = r.timestamps().expect("the trailer is read");
        assert_eq!(times.len(), 3);
        assert!((times[0] - t0).abs() < 1e-6, "{}", times[0]);
        assert!((times[2] - (t0 + 0.2)).abs() < 1e-6);
        assert_eq!(r.start_time(), Some(times[0]));
    }

    #[test]
    fn a_file_without_trailer_has_no_timestamps() {
        let path = temp_file("notrailer", &build(0, 1, 1, 1, 8, &[vec![0], vec![1]]));
        let r = SerReader::open(&path).unwrap();
        assert!(r.timestamps().is_none());
        assert_eq!(r.start_time(), None);
    }

    #[test]
    fn the_header_time_is_the_start_when_there_is_no_trailer() {
        let t0 = planets_core::planet::unix_seconds(2024, 9, 20, 3, 30, 15.0);
        let path = timed("header", ticks(t0), &[]);
        let r = SerReader::open(&path).unwrap();
        assert!(r.timestamps().is_none());
        assert!((r.start_time().unwrap() - t0).abs() < 1e-6);
    }

    #[test]
    fn a_trailer_of_zeros_is_not_a_time() {
        let path = timed("zeros", 0, &[0, 0]);
        let r = SerReader::open(&path).unwrap();
        assert!(r.timestamps().is_none());
        assert_eq!(r.start_time(), None);
    }

    #[test]
    fn frames_are_still_read_with_a_trailer() {
        let t0 = planets_core::planet::unix_seconds(2023, 3, 10, 21, 5, 0.0);
        let path = timed("read", 0, &[ticks(t0), ticks(t0 + 1.0)]);
        let mut r = SerReader::open(&path).unwrap();
        assert_eq!(r.read_frame(1).unwrap().data, vec![1.0 / 255.0]);
    }
}
