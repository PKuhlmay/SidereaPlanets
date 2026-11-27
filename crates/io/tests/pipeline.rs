//! The whole chain the window runs on JUPITER.AVI, step by step as the Tauri commands call it
//! (`app/src-tauri/src/`): Set Alignpoints -> Align -> Limit -> Stack -> Wavelet (Process and Do All) ->
//! Save image and read the file again. Every step is checked for a plausible result, not for a match with
//! stored reference results (that is what the `*_reference.rs` tests do). The recording lives in `re-local/` (not in Git);
//! without it the test is skipped.

use planets_core::{
    Image,
    align::{
        AlignError, AlignParams, Control, LimitMode, PointParams, Reference, align_sequence,
        apply_threshold, default_threshold, find_alignpoints, limit_frames, rank_by_quality,
    },
    stack::{StackError, StackParams, points_from_positions, psnr, stack_sequence},
    wavelet::{ColourMode, Previewer, Region, WaveletSettings, do_all, rgba_region},
};
use planets_io::{
    export::{ExportFormat, save_image},
    images::ImageSequenceReader,
    recording::Recording,
};
use std::{path::PathBuf, sync::Arc};

mod common;

fn jupiter() -> Option<PathBuf> {
    let path = common::re_local().join("testdata/astropix/JUPITER.AVI");
    path.exists().then_some(path)
}

fn assert_plausible(image: &Image, what: &str, width: u32, height: u32) {
    assert_eq!(
        (image.width, image.height, image.channels),
        (width, height, 3),
        "{what}: size"
    );
    assert_eq!(
        image.data.len(),
        width as usize * height as usize * 3,
        "{what}: sample count"
    );
    assert!(
        image.data.iter().all(|v| v.is_finite()),
        "{what}: not finite"
    );
}

fn mean(image: &Image) -> f64 {
    image.data.iter().map(|&v| f64::from(v)).sum::<f64>() / image.data.len() as f64
}

/// Mean absolute difference between neighbouring luma values: grows with fine detail.
fn detail(image: &Image) -> f64 {
    let luma = image.to_luma();
    let (w, h) = (luma.width as usize, luma.height as usize);
    let mut sum = 0.0;
    for y in 0..h - 1 {
        for x in 0..w - 1 {
            let v = luma.data[y * w + x];
            sum += f64::from(
                (v - luma.data[y * w + x + 1]).abs() + (v - luma.data[(y + 1) * w + x]).abs(),
            );
        }
    }
    sum / ((w - 1) * (h - 1)) as f64
}

/// The settings as the window sends them (`toBackend` in `app/src/logic/wavelet.ts`): camelCase, flat, six layers.
fn wavelet_settings(layer1_gain: f64) -> WaveletSettings {
    let layer = |gain: f64| serde_json::json!({ "enabled": true, "gain": gain, "denoise": 0, "sharpen": 0.1 });
    let json = serde_json::json!({
        "scheme": "linear",
        "filter": "gaussian",
        "initial": 1,
        "step": 0,
        "layers": [layer(layer1_gain), layer(1.0), layer(1.0), layer(1.0), layer(1.0), layer(1.0)],
    });
    serde_json::from_value(json).expect("the settings of the window must parse")
}

#[test]
fn jupiter_runs_through_the_whole_chain() {
    let Some(path) = jupiter() else {
        eprintln!("JUPITER.AVI not found in re-local/, skipped");
        return;
    };

    // Select: open_recording and read_frame.
    let mut recording = Recording::open_files(&[path]).unwrap();
    let info = recording.info();
    let (width, height, count) = (info.width, info.height, info.frame_count);
    assert_eq!((width, height, count), (1024, 680, 10));
    let first = recording.read_frame(0).unwrap();
    assert_plausible(&first, "frame 1", width, height);
    assert_eq!(first.to_rgba8().len(), width as usize * height as usize * 4);
    assert!(
        recording.read_frame(count).is_err(),
        "a frame past the end must be an error"
    );

    // Set Alignpoints with the defaults of the window.
    let candidates = find_alignpoints(&first, &PointParams::default());
    let threshold = default_threshold(candidates.len());
    let points = apply_threshold(&candidates, threshold);
    assert!(
        (70..=100).contains(&points.len()),
        "{} alignpoints",
        points.len()
    );
    assert!(
        points.iter().all(|p| p.x < width && p.y < height),
        "a point lies outside the frame"
    );

    // Align: progress reaches the frame count, every frame gets a finite rating, frame 1 does not move.
    let reference = Reference::new(&first, points, &AlignParams::default());
    let mut progress = Vec::new();
    let results = align_sequence(
        &reference,
        count,
        |i| recording.read_frame(i),
        |done| {
            progress.push(done);
            Control::Continue
        },
    )
    .unwrap();
    assert_eq!(results.len(), count as usize);
    assert_eq!(progress.last(), Some(&count));
    assert!(
        progress.windows(2).all(|w| w[0] < w[1]),
        "progress must grow: {progress:?}"
    );
    for (i, r) in results.iter().enumerate() {
        assert!(
            r.quality.is_finite() && r.quality > 0.0,
            "frame {i}: quality {}",
            r.quality
        );
        assert!(
            r.shift.0.is_finite() && r.shift.1.is_finite(),
            "frame {i}: shift"
        );
        assert!(
            r.shift.0.abs() <= 20.0 && r.shift.1.abs() <= 20.0,
            "frame {i}: shift {:?}",
            r.shift
        );
    }
    assert!(
        results[0].shift.0.abs() < 0.5 && results[0].shift.1.abs() < 0.5,
        "frame 1 against itself"
    );
    let mut ranking = rank_by_quality(&results);
    assert_eq!(ranking[0], 0, "frame 1 is the best one");
    ranking.sort_unstable();
    assert_eq!(
        ranking,
        (0..count as usize).collect::<Vec<_>>(),
        "the ranking must list every frame once"
    );

    // Cancel stops Align after the first batch and reports it as cancelled.
    let cancelled = align_sequence(
        &reference,
        count,
        |i| recording.read_frame(i),
        |_| Control::Cancel,
    );
    assert!(matches!(cancelled, Err(AlignError::Cancelled)));

    // Limit: the default keeps all frames, the count modes keep as many as asked, best first.
    let kept = limit_frames(&results, LimitMode::LowestQuality(80.0));
    assert_eq!(kept.len(), 10);
    let best_five = limit_frames(&results, LimitMode::BestCount(5));
    assert_eq!(best_five.len(), 5);
    assert_eq!(best_five[0], 0);
    assert_eq!(
        limit_frames(&results, LimitMode::BestPercent(50.0)).len(),
        5
    );

    // Stack: the window sends the positions of the points and the indices of the checked frames.
    let positions = points.iter().map(|p| (p.x, p.y));
    let stack_points = points_from_positions(positions);
    let stack_reference = Reference::new(&first, &stack_points, &AlignParams::default());
    let frames: Vec<u32> = kept.iter().map(|&i| i as u32).collect();
    let stacked = stack_sequence(
        &stack_reference,
        &first,
        &StackParams::default(),
        &frames,
        |i| recording.read_frame(i),
        |_| Control::Continue,
    )
    .unwrap();
    assert_eq!(stacked.frames, 10);
    let stack = stacked.image;
    assert_plausible(&stack, "stack", width, height);
    assert!(
        stack.data.iter().all(|&v| (0.0..=1.0).contains(&v)),
        "the stack leaves 0..1"
    );
    assert!(
        (mean(&stack) - mean(&first)).abs() < 0.02,
        "stack {} against frame 1 {}",
        mean(&stack),
        mean(&first)
    );
    assert!(
        psnr(&stack, &first).unwrap() > 30.0,
        "the stack must resemble frame 1"
    );

    // Fewer frames, and Cancel and an empty selection.
    let some = stack_sequence(
        &stack_reference,
        &first,
        &StackParams::default(),
        &frames[..4],
        |i| recording.read_frame(i),
        |_| Control::Continue,
    )
    .unwrap();
    assert_eq!(some.frames, 4);
    let none = stack_sequence(
        &stack_reference,
        &first,
        &StackParams::default(),
        &[],
        |i| recording.read_frame(i),
        |_| Control::Continue,
    );
    assert!(matches!(none, Err(StackError::NoFrames)));
    let stopped = stack_sequence(
        &stack_reference,
        &first,
        &StackParams::default(),
        &frames,
        |i| recording.read_frame(i),
        |_| Control::Cancel,
    );
    assert!(matches!(stopped, Err(StackError::Cancelled)));

    // Wavelet, Do All with the defaults: only the output tone curve acts, the image stays close.
    let neutral = do_all(&stack, &wavelet_settings(1.0), ColourMode::Colour);
    assert!(
        psnr(&neutral, &stack).unwrap() > 40.0,
        "default settings must only apply the tone curve"
    );

    // Wavelet, Do All with Layer 1 = 10.0: more fine detail, same size, finite, close to the stack.
    let sharp_settings = wavelet_settings(10.0);
    let sharp = do_all(&stack, &sharp_settings, ColourMode::Colour);
    assert_plausible(&sharp, "sharpened", width, height);
    let (before, after) = (detail(&stack), detail(&sharp));
    assert!(
        after > before * 1.05,
        "detail {before:.5} -> {after:.5}: the sharpening must add detail"
    );
    assert!(
        psnr(&sharp, &stack).unwrap() > 25.0,
        "the sharpening must not wreck the image"
    );

    // Process (preview of the Processing Area) shows what Do All gives in that area.
    let area = Region::centered(256, width, height);
    let mut previewer = Previewer::new(Arc::new(stack.clone()));
    let preview = previewer.render(&sharp_settings, ColourMode::Colour, area);
    assert_eq!(preview.region, area);
    assert_eq!(preview.rgba.len(), 256 * 256 * 4);
    let whole = rgba_region(&sharp, area);
    let worst = preview
        .rgba
        .iter()
        .zip(&whole)
        .map(|(&a, &b)| a.abs_diff(b))
        .max()
        .unwrap();
    assert!(
        worst <= 1,
        "preview and Do All differ by {worst} in the area"
    );
    assert_ne!(
        preview.rgba,
        previewer.before(area).rgba,
        "the preview must differ from Before"
    );

    // Save image, in every format of the Stack tab, and read the file again.
    let dir = std::env::temp_dir().join(format!("planets-pipeline-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let clamped = Image {
        data: sharp.data.iter().map(|v| v.clamp(0.0, 1.0)).collect(),
        ..sharp.clone()
    };
    for &format in ExportFormat::STACK {
        let written = save_image(&sharp, &dir.join("jupiter"), format).unwrap();
        assert_eq!(
            written.extension().and_then(|e| e.to_str()),
            Some(format.extension())
        );
        let back = ImageSequenceReader::open(vec![written])
            .unwrap()
            .read_frame(0)
            .unwrap();
        assert_plausible(&back, &format!("{format:?} read back"), width, height);
        let db = psnr(&back, &clamped).unwrap();
        let wanted = match format {
            ExportFormat::Jpeg => 35.0,
            ExportFormat::Bmp => 45.0,
            _ => 70.0,
        };
        assert!(
            db >= wanted,
            "{format:?}: {db:.1} dB after the round trip, wanted {wanted}"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}
