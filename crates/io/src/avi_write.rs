//! Uncompressed AVI writer (24 bit `BI_RGB`) for *Save Aligned* and *Save Integrated*.
//!
//! Frames are streamed one by one, so sequences with thousands of frames never have to fit in
//! memory. The reference program offers Video-for-Windows codecs here; the clone only writes uncompressed
//! video, which every player and the reader in [`crate::avi`] understand. There is no OpenDML
//! support, so a file is limited to 4 GiB (writing fails cleanly beyond that).

use crate::export::part_path;
use planets_core::Image;
use std::{
    fs::{self, File},
    io::{BufWriter, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

/// RIFF sizes are 32 bit; stay well below the limit so headers and index still fit.
const MAX_FILE_LEN: u64 = 4_000_000_000;

#[derive(Debug, thiserror::Error)]
pub enum AviWriteError {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("frame is {found}, but the video is {expected}")]
    SizeMismatch { expected: String, found: String },
    #[error("the video would exceed 4 GiB")]
    TooLarge,
    #[error("a video needs at least one frame")]
    NoFrames,
    #[error("invalid frame size or rate")]
    InvalidParameters,
}

/// Writes an uncompressed AVI. Call [`AviWriter::finish`] to complete the file; a writer that is
/// dropped before leaves no file behind (the data goes to a temporary name until then).
pub struct AviWriter {
    out: BufWriter<File>,
    path: PathBuf,
    part: PartGuard,
    width: u32,
    height: u32,
    /// Bytes per row including the padding to a multiple of 4.
    stride: usize,
    /// (offset relative to `movi`, size) of every frame chunk, for the index.
    index: Vec<(u32, u32)>,
    /// Absolute position of the `movi` list body (the `movi` tag).
    movi_start: u64,
    /// Next write position relative to `movi_start`.
    movi_len: u32,
    scratch: Vec<u8>,
}

// Fixed positions in the header written by `create`; they are patched in `finish`.
const RIFF_SIZE_POS: u64 = 4;
const AVIH_TOTAL_FRAMES_POS: u64 = 48;
const STRH_LENGTH_POS: u64 = 140;
const MOVI_SIZE_POS: u64 = 216;

impl AviWriter {
    /// Starts a video of `width` x `height` pixels at `fps` frames per second.
    pub fn create(path: &Path, width: u32, height: u32, fps: u32) -> Result<Self, AviWriteError> {
        if width == 0 || height == 0 || fps == 0 || width > 65_535 || height > 65_535 {
            return Err(AviWriteError::InvalidParameters);
        }
        let part = part_path(path);
        let mut out = BufWriter::new(File::create(&part)?);
        let stride = (width as usize * 3).next_multiple_of(4);
        let frame_len = (stride * height as usize) as u32;

        // RIFF header; sizes that depend on the frame count are patched in `finish`.
        out.write_all(b"RIFF")?;
        out.write_all(&0u32.to_le_bytes())?;
        out.write_all(b"AVI ")?;

        // hdrl list: 4 (tag) + avih chunk (8 + 56) + strl list (8 + 4 + strh 8 + 56 + strf 8 + 40)
        let hdrl_body = 4 + 8 + 56 + 8 + 4 + (8 + 56) + (8 + 40);
        out.write_all(b"LIST")?;
        out.write_all(&(hdrl_body as u32).to_le_bytes())?;
        out.write_all(b"hdrl")?;

        out.write_all(b"avih")?;
        out.write_all(&56u32.to_le_bytes())?;
        let avih = avih_fields(fps, frame_len, width, height);
        for v in avih {
            out.write_all(&v.to_le_bytes())?;
        }

        out.write_all(b"LIST")?;
        out.write_all(&((4 + (8 + 56) + (8 + 40)) as u32).to_le_bytes())?;
        out.write_all(b"strl")?;

        out.write_all(b"strh")?;
        out.write_all(&56u32.to_le_bytes())?;
        out.write_all(b"vids")?;
        out.write_all(b"DIB ")?;
        for v in [0u32, 0, 0, 1, fps, 0, 0, frame_len, u32::MAX, 0] {
            // flags, priority/language, initial frames, scale, rate, start, length, buffer,
            // quality, sample size
            out.write_all(&v.to_le_bytes())?;
        }
        out.write_all(&[0u8; 8])?; // rcFrame

        out.write_all(b"strf")?;
        out.write_all(&40u32.to_le_bytes())?;
        out.write_all(&40u32.to_le_bytes())?;
        out.write_all(&width.to_le_bytes())?;
        out.write_all(&height.to_le_bytes())?; // positive: bottom-up
        out.write_all(&1u16.to_le_bytes())?;
        out.write_all(&24u16.to_le_bytes())?;
        out.write_all(&[0u8; 4])?; // BI_RGB
        out.write_all(&frame_len.to_le_bytes())?;
        out.write_all(&[0u8; 16])?;

        out.write_all(b"LIST")?;
        out.write_all(&0u32.to_le_bytes())?;
        out.write_all(b"movi")?;
        let movi_start = out.stream_position()? - 4;
        debug_assert_eq!(movi_start - 8, MOVI_SIZE_POS - 4);

        Ok(Self {
            out,
            path: path.to_path_buf(),
            part: PartGuard(part),
            width,
            height,
            stride,
            index: Vec::new(),
            movi_start,
            movi_len: 4,
            scratch: Vec::new(),
        })
    }

    /// Appends one frame. It must have the size the video was created with; mono frames are
    /// written as gray RGB.
    pub fn write_frame(&mut self, image: &Image) -> Result<(), AviWriteError> {
        if (image.width, image.height) != (self.width, self.height) {
            return Err(AviWriteError::SizeMismatch {
                expected: format!("{}x{}", self.width, self.height),
                found: format!("{}x{}", image.width, image.height),
            });
        }
        let frame_len = self.stride * self.height as usize;
        let chunk_len = 8 + frame_len as u64;
        if self.movi_start
            + u64::from(self.movi_len)
            + chunk_len
            + 16 * (self.index.len() as u64 + 1)
            > MAX_FILE_LEN
        {
            return Err(AviWriteError::TooLarge);
        }

        let channels = usize::from(image.channels).max(1);
        let w = self.width as usize;
        let to_u8 = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        self.scratch.clear();
        self.scratch.resize(frame_len, 0);
        for y in 0..self.height as usize {
            // Bottom-up: the last image row is stored first.
            let dst_row =
                &mut self.scratch[(self.height as usize - 1 - y) * self.stride..][..w * 3];
            let src_row = &image.data[y * w * channels..][..w * channels];
            for (dst, px) in dst_row
                .as_chunks_mut::<3>()
                .0
                .iter_mut()
                .zip(src_row.chunks_exact(channels))
            {
                let (r, g, b) = if channels >= 3 {
                    (px[0], px[1], px[2])
                } else {
                    (px[0], px[0], px[0])
                };
                *dst = [to_u8(b), to_u8(g), to_u8(r)];
            }
        }

        self.index.push((self.movi_len, frame_len as u32));
        self.out.write_all(b"00db")?;
        self.out.write_all(&(frame_len as u32).to_le_bytes())?;
        self.out.write_all(&self.scratch)?;
        self.movi_len += 8 + frame_len as u32; // rows are padded to 4 bytes, so chunks stay even
        Ok(())
    }

    /// Writes the index, patches the sizes and moves the file into place.
    pub fn finish(mut self) -> Result<(), AviWriteError> {
        let frames = self.index.len() as u32;
        if frames == 0 {
            return Err(AviWriteError::NoFrames);
        }
        self.out.write_all(b"idx1")?;
        self.out.write_all(&(frames * 16).to_le_bytes())?;
        for (offset, size) in &self.index {
            self.out.write_all(b"00db")?;
            self.out.write_all(&0x10u32.to_le_bytes())?; // AVIIF_KEYFRAME
            self.out.write_all(&offset.to_le_bytes())?;
            self.out.write_all(&size.to_le_bytes())?;
        }
        let file_len = self.out.stream_position()?;

        let patch = |out: &mut BufWriter<File>, pos: u64, value: u32| -> std::io::Result<()> {
            out.seek(SeekFrom::Start(pos))?;
            out.write_all(&value.to_le_bytes())
        };
        patch(&mut self.out, RIFF_SIZE_POS, (file_len - 8) as u32)?;
        patch(&mut self.out, AVIH_TOTAL_FRAMES_POS, frames)?;
        patch(&mut self.out, STRH_LENGTH_POS, frames)?;
        patch(&mut self.out, MOVI_SIZE_POS, self.movi_len)?;
        self.out.flush()?;
        // The file handle must be closed before the rename on Windows.
        let Self {
            out, part, path, ..
        } = self;
        drop(out);
        fs::rename(&part.0, &path)?;
        Ok(())
    }
}

/// Removes the temporary file when dropped; after a successful rename there is nothing left to remove.
struct PartGuard(PathBuf);

impl Drop for PartGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

/// The 14 numeric fields of the main AVI header after the chunk header.
fn avih_fields(fps: u32, frame_len: u32, width: u32, height: u32) -> [u32; 14] {
    [
        1_000_000 / fps,               // microseconds per frame
        frame_len.saturating_mul(fps), // max bytes per second
        0,                             // padding granularity
        0x10,                          // flags: AVIF_HASINDEX
        0,                             // total frames, patched in `finish` (position 48)
        0,                             // initial frames
        1,                             // streams
        frame_len,                     // suggested buffer size
        width,
        height,
        0,
        0,
        0,
        0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avi::AviReader;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "planets-avi-write-{}-{name}.avi",
            std::process::id()
        ))
    }

    fn frame(width: u32, height: u32, channels: u8, seed: f32) -> Image {
        let n = width as usize * height as usize * usize::from(channels);
        Image {
            width,
            height,
            channels,
            data: (0..n)
                .map(|i| ((i as f32 * 0.037 + seed) % 1.0).abs())
                .collect(),
        }
    }

    #[test]
    fn written_video_is_read_back() {
        // Width 5 forces row padding (15 bytes per row become 16).
        let path = temp_path("roundtrip");
        let frames: Vec<Image> = (0..3).map(|i| frame(5, 4, 3, i as f32 * 0.2)).collect();
        let mut writer = AviWriter::create(&path, 5, 4, 25).unwrap();
        for f in &frames {
            writer.write_frame(f).unwrap();
        }
        writer.finish().unwrap();

        let mut reader = AviReader::open(&path).unwrap();
        let info = reader.info().clone();
        assert_eq!((info.width, info.height, info.frame_count), (5, 4, 3));
        assert_eq!(info.codec, "DIB");
        assert_eq!(info.bits_per_pixel, 24);
        assert!((info.fps - 25.0).abs() < 1e-6);
        for (i, f) in frames.iter().enumerate() {
            let back = reader.read_frame(i as u32).unwrap();
            for (a, b) in back.data.iter().zip(&f.data) {
                assert!((a - b).abs() <= 0.5 / 255.0 + 1e-6, "frame {i}: {a} vs {b}");
            }
        }
    }

    #[test]
    fn mono_frames_become_gray() {
        let path = temp_path("mono");
        let mut writer = AviWriter::create(&path, 2, 2, 10).unwrap();
        writer.write_frame(&frame(2, 2, 1, 0.1)).unwrap();
        writer.finish().unwrap();
        let mut reader = AviReader::open(&path).unwrap();
        let back = reader.read_frame(0).unwrap();
        assert_eq!(back.channels, 3);
        for px in back.data.as_chunks::<3>().0 {
            assert_eq!(px[0], px[1]);
            assert_eq!(px[1], px[2]);
        }
    }

    #[test]
    fn wrong_frame_size_is_rejected() {
        let path = temp_path("mismatch");
        let mut writer = AviWriter::create(&path, 4, 4, 25).unwrap();
        assert!(matches!(
            writer.write_frame(&frame(3, 4, 3, 0.0)),
            Err(AviWriteError::SizeMismatch { .. })
        ));
    }

    #[test]
    fn unfinished_video_leaves_no_file() {
        let path = temp_path("dropped");
        {
            let mut writer = AviWriter::create(&path, 4, 4, 25).unwrap();
            writer.write_frame(&frame(4, 4, 3, 0.0)).unwrap();
        }
        assert!(!path.exists());
        assert!(!part_path(&path).exists());
    }

    #[test]
    fn empty_video_and_bad_parameters_are_rejected() {
        let path = temp_path("empty");
        let writer = AviWriter::create(&path, 4, 4, 25).unwrap();
        assert!(matches!(writer.finish(), Err(AviWriteError::NoFrames)));
        assert!(matches!(
            AviWriter::create(&path, 0, 4, 25),
            Err(AviWriteError::InvalidParameters)
        ));
        assert!(matches!(
            AviWriter::create(&path, 4, 4, 0),
            Err(AviWriteError::InvalidParameters)
        ));
    }
}
