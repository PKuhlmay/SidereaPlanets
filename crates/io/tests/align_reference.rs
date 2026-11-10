//! Compares alignment with the results the reference program gave for reference case R1 (`docs/testdaten.md`).
//! The recordings live in `re-local/` and are not in the repository; without them the test is skipped.

use planets_core::align::{
    AlignParams, Control, LimitMode, PointParams, Reference, align_sequence, apply_threshold,
    default_threshold, find_alignpoints, limit_frames, rank_by_quality,
};
use planets_io::recording::Recording;
use std::path::PathBuf;

fn jupiter() -> Option<Recording> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../re-local/testdata/astropix/JUPITER.AVI");
    path.exists().then(|| Recording::open(&path).unwrap())
}

#[test]
fn jupiter_with_default_settings_matches_the_original_where_comparable() {
    let Some(mut rec) = jupiter() else {
        eprintln!("JUPITER.AVI not found in re-local/, skipped");
        return;
    };
    let count = rec.info().frame_count;
    let first = rec.read_frame(0).unwrap();

    // Set Alignpoints: the original reports 82.
    let candidates = find_alignpoints(&first, &PointParams::default());
    let points = apply_threshold(&candidates, default_threshold(candidates.len()));
    assert!(
        (74..=90).contains(&points.len()),
        "{} alignpoints, the original has 82",
        points.len()
    );

    // Align, then Limit with the default 80 %: all 10 frames stay.
    let reference = Reference::new(&first, points, &AlignParams::default());
    let results = align_sequence(
        &reference,
        count,
        |i| rec.read_frame(i),
        |_| Control::Continue,
    )
    .unwrap();
    assert_eq!(results.len(), 10);
    assert_eq!(
        limit_frames(&results, LimitMode::LowestQuality(80.0)).len(),
        10
    );

    // The original orders the frames 1, 4, 3, 2, 10, 9, 8, 5, 6, 7. Ours differs in the middle (see
    // docs/testdaten.md); what both agree on is the best and the two worst frames.
    let order: Vec<usize> = rank_by_quality(&results).iter().map(|i| i + 1).collect();
    eprintln!("quality order {order:?}");
    assert_eq!(order[0], 1);
    assert_eq!(&order[8..], &[6, 7]);

    // The recording drifts: every frame is tracked at (almost) all points, none piles up at the limit.
    for (i, r) in results.iter().enumerate() {
        let tracked = r.points.iter().flatten().count();
        assert!(
            tracked * 10 >= r.points.len() * 9,
            "frame {}: {tracked} points tracked",
            i + 1
        );
    }
}
