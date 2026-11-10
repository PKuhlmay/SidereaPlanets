//! Decodes the FFmpeg FATE Lagarith samples (`fate-suite/lagarith/`) and compares every frame with
//! ffmpeg's own output, stored as `x.rgb` next to each sample `x.avi`:
//!
//! ```text
//! ffmpeg -i x.avi -fps_mode passthrough -f rawvideo -pix_fmt rgb24 x.rgb
//! # YUV samples: ffmpeg's default conversion is biased by almost 2/255 and interpolates chroma,
//! # so ask for the exact conversion with repeated chroma
//! ffmpeg -i x.avi -fps_mode passthrough -sws_flags accurate_rnd+full_chroma_int+neighbor \
//!        -f rawvideo -pix_fmt rgb24 x.rgb
//! ```
//!
//! The samples are not in Git. They are looked up in `$PLANETS_TESTDATA/codecs/lagarith/`, falling
//! back to `re-local/testdata/` of the repository. Tests pass silently when they are absent.

use planets_io::{
    avi::{AviError, AviReader},
    codec::CodecError,
};
use std::{fs, path::PathBuf};

fn sample_dir() -> Option<PathBuf> {
    let root = std::env::var_os("PLANETS_TESTDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../re-local/testdata")
        });
    let dir = root.join("codecs/lagarith");
    dir.is_dir().then_some(dir)
}

/// Decodes `name`.avi and compares it with `name`.rgb. RGB output must be exact, YUV output
/// may deviate by `tolerance` (rounding of the colour conversion).
fn check(name: &str, tolerance: u8) {
    let Some(dir) = sample_dir() else { return };
    let avi = dir.join(format!("{name}.avi"));
    let rgb = dir.join(format!("{name}.rgb"));
    if !avi.exists() || !rgb.exists() {
        return;
    }
    let expected = fs::read(&rgb).unwrap();
    let mut reader = AviReader::open(&avi).unwrap();
    let info = reader.info().clone();
    let frame_len = info.width as usize * info.height as usize * 3;
    assert_eq!(
        expected.len() % frame_len,
        0,
        "{name}: partial reference frame"
    );
    let reference_frames = expected.len() / frame_len;

    let (mut decoded, mut null_frames, mut cut_off) = (0, 0, 0);
    let mut worst = 0u8;
    for index in 0..info.frame_count {
        let image = match reader.read_frame(index) {
            Ok(image) => image,
            // A zero-length chunk repeats the previous frame; ffmpeg's passthrough output skips it.
            Err(AviError::Codec(CodecError::Unsupported { reason, .. }))
                if reason.starts_with("empty frame") =>
            {
                null_frames += 1;
                continue;
            }
            // Some samples are cut off in their last frame, which ffmpeg still shows damaged.
            Err(AviError::Io(e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof
                    && index == info.frame_count - 1 =>
            {
                cut_off += 1;
                continue;
            }
            Err(e) => panic!("{name} frame {index}: {e}"),
        };
        assert_eq!(image.channels, 3);
        assert_eq!(image.data.len(), frame_len);
        let reference = &expected[decoded * frame_len..][..frame_len];
        for (i, (&v, &r)) in image.data.iter().zip(reference).enumerate() {
            let got = (v * 255.0).round() as u8;
            let diff = got.abs_diff(r);
            worst = worst.max(diff);
            assert!(
                diff <= tolerance,
                "{name} frame {index}: sample {i} (x {}, y {}, channel {}) is {got}, ffmpeg says {r}",
                i / 3 % info.width as usize,
                i / 3 / info.width as usize,
                i % 3,
            );
        }
        decoded += 1;
    }
    assert_eq!(
        decoded + cut_off,
        reference_frames,
        "{name}: decoded {decoded} frames, {null_frames} null, {cut_off} cut off"
    );
    println!(
        "{name}: {decoded} frames compared ({null_frames} null, {cut_off} cut off), max deviation {worst}/255"
    );
}

#[test]
fn rgb24_matches_ffmpeg() {
    check("lag-rgb24", 0);
}

#[test]
fn rgb32_matches_ffmpeg() {
    check("lag-rgb32", 0);
}

#[test]
fn yuy2_matches_ffmpeg() {
    check("lag-yuy2", 2);
}

#[test]
fn yv12_matches_ffmpeg() {
    check("lag-yv12", 2);
}

#[test]
fn black_frames_and_off_by_ones_match_ffmpeg() {
    check("lagarith-1.3.27-black-frames-and-off-by-ones", 0);
}

#[test]
fn red_matches_ffmpeg() {
    check("lagarith-red", 0);
}
