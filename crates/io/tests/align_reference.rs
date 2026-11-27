//! Regression test of alignment against stored reference results for reference case R1 (Jupiter test recording).
//! The recordings live in `re-local/` and are not in the repository; without them the test is skipped.

use planets_core::align::{
    AlignParams, Control, LimitMode, PointParams, Reference, Roi, align_sequence, apply_threshold,
    default_threshold, find_alignpoints, limit_frames, rank_by_quality,
};
use planets_io::recording::Recording;
mod common;

fn jupiter() -> Option<Recording> {
    let path = common::re_local().join("testdata/astropix/JUPITER.AVI");
    path.exists().then(|| Recording::open(&path).unwrap())
}

#[test]
fn jupiter_with_default_settings_matches_the_stored_reference_results() {
    let Some(mut rec) = jupiter() else {
        eprintln!("JUPITER.AVI not found in re-local/, skipped");
        return;
    };
    let count = rec.info().frame_count;
    let first = rec.read_frame(0).unwrap();

    // Set Alignpoints: the reference results hold 91 candidates and 82 points after the default threshold.
    let candidates = find_alignpoints(&first, &PointParams::default());
    assert!(
        (82..=100).contains(&candidates.len()),
        "{} candidates, the reference has 91",
        candidates.len()
    );
    let points = apply_threshold(&candidates, default_threshold(candidates.len()));
    assert!(
        (74..=90).contains(&points.len()),
        "{} alignpoints, the reference has 82",
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

    // The reference order of the frames is 1, 4, 3, 2, 10, 9, 8, 5, 6, 7. Ours differs in the middle;
    // what both agree on is the best and the two worst frames.
    let order: Vec<usize> = rank_by_quality(&results).iter().map(|i| i + 1).collect();
    eprintln!("quality order {order:?}");
    assert_eq!(order[0], 1);
    assert_eq!(&order[8..], &[6, 7]);
    // Rank correlation with the reference order, 0.79 when measured; the three best frames are not the same.
    let reference_order = [1usize, 4, 3, 2, 10, 9, 8, 5, 6, 7];
    let rho = spearman(&order, &reference_order);
    assert!(rho > 0.7, "Spearman correlation {rho}");

    // Frames/Apoint keeps a subset of the frames, ordered like the ranking.
    let per_point = limit_frames(&results, LimitMode::FramesPerPoint(1));
    assert!(!per_point.is_empty() && per_point.len() <= 10);
    assert!(per_point.windows(2).all(|w| {
        let ranking = rank_by_quality(&results);
        ranking.iter().position(|&i| i == w[0]) < ranking.iter().position(|&i| i == w[1])
    }));

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

/// Spearman rank correlation of two orders of the frames 1..=n (best first).
fn spearman(order: &[usize], other: &[usize]) -> f64 {
    let n = order.len();
    let rank_of = |o: &[usize], frame: usize| o.iter().position(|&f| f == frame).unwrap() as f64;
    let d2: f64 = (1..=n)
        .map(|f| (rank_of(order, f) - rank_of(other, f)).powi(2))
        .sum();
    1.0 - 6.0 * d2 / (n as f64 * ((n * n) as f64 - 1.0))
}

#[test]
fn alignment_on_a_region_of_interest_tracks_the_planet() {
    let Some(mut rec) = jupiter() else {
        eprintln!("JUPITER.AVI not found in re-local/, skipped");
        return;
    };
    let count = rec.info().frame_count;
    let first = rec.read_frame(0).unwrap();
    let roi = Roi {
        x: 300,
        y: 150,
        width: 500,
        height: 400,
    };
    let candidates = find_alignpoints(
        &first,
        &PointParams {
            roi: Some(roi),
            ..PointParams::default()
        },
    );
    let points = apply_threshold(&candidates, default_threshold(candidates.len()));
    assert!(
        points.len() >= 30,
        "{} alignpoints in the region",
        points.len()
    );
    // The default edge distance of 20 px counts from the region's edges.
    assert!(points.iter().all(|p| {
        (roi.x + 20..roi.x + roi.width - 20).contains(&p.x)
            && (roi.y + 20..roi.y + roi.height - 20).contains(&p.y)
    }));

    let reference = Reference::with_roi(&first, points, &AlignParams::default(), Some(roi));
    let results = align_sequence(
        &reference,
        count,
        |i| rec.read_frame(i),
        |_| Control::Continue,
    )
    .unwrap();
    for (i, r) in results.iter().enumerate() {
        let tracked = r.points.iter().flatten().count();
        assert!(
            tracked * 10 >= r.points.len() * 9,
            "frame {}: {tracked} of {} points tracked",
            i + 1,
            r.points.len()
        );
    }
}
