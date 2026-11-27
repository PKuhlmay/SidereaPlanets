//! Regression test of the stack against the stored reference result for reference case R1 (Jupiter test recording,
//! `docs/stacking.md`). The recording and the reference image live in `re-local/` and are not in the
//! repository; without them the test is skipped.

use planets_core::{
    Image,
    align::{
        AlignParams, Control, PointParams, Reference, apply_threshold, default_threshold,
        find_alignpoints,
    },
    stack::{StackParams, psnr, stack_sequence},
};
use planets_io::recording::Recording;
use std::path::PathBuf;

mod common;

fn re_local(path: &str) -> PathBuf {
    common::re_local().join(path)
}

/// The stored 16 bit reference TIFF. Its samples are 8 bit values times 256,
/// so dividing by 256 * 255 puts them on the scale of our images, where 1.0 is 255 in 8 bit.
fn load_reference(path: &PathBuf) -> Image {
    let tiff = image::open(path).unwrap().to_rgb16();
    Image {
        width: tiff.width(),
        height: tiff.height(),
        channels: 3,
        data: tiff
            .into_raw()
            .into_iter()
            .map(|v| f32::from(v) / (256.0 * 255.0))
            .collect(),
    }
}

fn channel_means(img: &Image) -> [f64; 3] {
    let mut sums = [0.0; 3];
    for (i, &v) in img.data.iter().enumerate() {
        sums[i % 3] += f64::from(v) * 65280.0;
    }
    sums.map(|s| s / (img.data.len() / 3) as f64)
}

#[test]
fn jupiter_stack_is_close_to_the_stored_reference_stack() {
    let video = re_local("testdata/astropix/JUPITER.AVI");
    let tiff = re_local("reference/jupiter-stack.tif");
    if !video.exists() || !tiff.exists() {
        eprintln!("JUPITER.AVI or jupiter-stack.tif not found in re-local/, skipped");
        return;
    }
    let mut rec = Recording::open(&video).unwrap();
    let count = rec.info().frame_count;
    let first = rec.read_frame(0).unwrap();
    let stored = load_reference(&tiff);

    // Set Alignpoints and Align as in R1; Limit keeps all ten frames.
    let candidates = find_alignpoints(&first, &PointParams::default());
    let points = apply_threshold(&candidates, default_threshold(candidates.len()));
    let reference = Reference::new(&first, points, &AlignParams::default());
    let frames: Vec<u32> = (0..count).collect();

    let mut stack = |params: StackParams| {
        stack_sequence(
            &reference,
            &first,
            &params,
            &frames,
            |i| rec.read_frame(i),
            |_| Control::Continue,
        )
        .unwrap()
    };
    let local = stack(StackParams::default());
    let global = stack(StackParams {
        correct_geometry: false,
        ..StackParams::default()
    });
    assert_eq!(local.frames, 10);

    // Figures for `docs/stacking.md`.
    let db = |img: &Image| psnr(img, &stored).unwrap();
    eprintln!(
        "local correction against the reference: {:.2} dB",
        db(&local.image)
    );
    eprintln!(
        "frame shift only against the reference: {:.2} dB",
        db(&global.image)
    );
    eprintln!("frame 1 against the reference: {:.2} dB", db(&first));
    eprintln!("channel means ours {:?}", channel_means(&local.image));
    eprintln!("channel means reference {:?}", channel_means(&stored));

    // The threshold of SP-006.
    assert!(
        db(&local.image) >= 45.0,
        "{:.2} dB against the reference, the threshold is 45 dB",
        db(&local.image)
    );
}
