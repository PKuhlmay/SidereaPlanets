//! Reads the practice videos from `re-local/testdata/astropix/` (not in Git). Skipped when absent.

use planets_io::recording::Recording;
use std::path::PathBuf;

mod common;

fn testdata(name: &str) -> Option<PathBuf> {
    let path = common::re_local().join("testdata/astropix").join(name);
    path.exists().then_some(path)
}

#[test]
fn jupiter_avi_has_expected_layout() {
    let Some(path) = testdata("JUPITER.AVI") else {
        return;
    };
    let mut rec = Recording::open(&path).unwrap();
    let info = rec.info();
    assert_eq!(info.format, "AVI");
    assert_eq!(info.codec.as_deref(), Some("DIB"));
    assert_eq!((info.width, info.height, info.frame_count), (1024, 680, 10));
    assert_eq!(info.bit_depth, 8);

    let frame = rec.read_frame(0).unwrap();
    assert_eq!(frame.data.len(), 1024 * 680 * usize::from(frame.channels));
    assert!(frame.data.iter().all(|v| (0.0..=1.0).contains(v)));
    let (min, max) = frame
        .data
        .iter()
        .fold((1.0f32, 0.0f32), |(lo, hi), &v| (lo.min(v), hi.max(v)));
    assert!(max - min > 0.2, "frame looks flat: {min}..{max}");
}

#[test]
fn all_practice_videos_decode_every_frame() {
    for name in ["JUPITER.AVI", "CLAVIUS.AVI", "SUNSPOTS.AVI"] {
        let Some(path) = testdata(name) else { continue };
        let mut rec = Recording::open(&path).unwrap();
        for i in 0..rec.info().frame_count {
            rec.read_frame(i)
                .unwrap_or_else(|e| panic!("{name} frame {i}: {e}"));
        }
    }
}
