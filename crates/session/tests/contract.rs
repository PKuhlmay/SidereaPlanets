//! The JSON shapes and error texts that the window depends on (`app/src`): the Tauri commands pass
//! the types of the session through as they are, so a change here is a change of the frontend
//! contract.

use planets_session::{
    AlignSettings, Aligned, Alignpoints, FrameRating, Intensity, LimitSettings, Limited, Point,
    PointSettings, SavedVideo, SequenceSettings, Session, SessionError, SourceInfo, StackSettings,
    Stacked,
};
use serde_json::json;

#[test]
fn point_settings_come_as_the_window_sends_them() {
    let settings: PointSettings = serde_json::from_value(json!({
        "minDistance": 30, "edgeDistance": 20, "maxMovement": 5,
        "roi": { "x": 1, "y": 2, "width": 3, "height": 4 },
        "intensity": "area3x3", "lo": 30, "hi": 230, "boxSize": 30
    }))
    .unwrap();
    assert_eq!(settings.intensity, Intensity::Area3x3);
    assert_eq!(settings.roi.map(|r| r.width), Some(3));
    assert_eq!(settings.box_size, 30);
}

#[test]
fn the_three_intensity_names_and_the_fallback() {
    let parse = |name: &str| serde_json::from_value::<Intensity>(json!(name)).unwrap();
    assert_eq!(parse("default"), Intensity::Default);
    assert_eq!(parse("area3x3"), Intensity::Area3x3);
    assert_eq!(parse("lowest"), Intensity::Lowest);
    // Anything else was `default` in the window's own command, too.
    assert_eq!(parse("something else"), Intensity::Default);
}

#[test]
fn limit_settings_are_tagged_by_mode() {
    let parse = |value| serde_json::from_value::<LimitSettings>(value).unwrap();
    assert_eq!(
        parse(json!({ "mode": "lowestQuality", "value": 0.5 })),
        LimitSettings::LowestQuality(0.5)
    );
    assert_eq!(
        parse(json!({ "mode": "bestPercent", "value": 80.0 })),
        LimitSettings::BestPercent(80.0)
    );
    assert_eq!(
        parse(json!({ "mode": "bestCount", "value": 200 })),
        LimitSettings::BestCount(200)
    );
    assert_eq!(
        parse(json!({ "mode": "framesPerPoint", "value": 4 })),
        LimitSettings::FramesPerPoint(4)
    );
}

#[test]
fn the_other_requests_parse_in_camel_case() {
    let align: AlignSettings =
        serde_json::from_value(json!({ "boxSize": 30, "maxMovement": 5, "roi": null })).unwrap();
    assert_eq!(
        (align.box_size, align.max_movement, align.roi),
        (30, 5, None)
    );

    let stack: StackSettings = serde_json::from_value(json!({
        "referenceFrame": 7, "points": [{ "x": 1, "y": 2 }], "frames": [0, 1],
        "boxSize": 30, "maxMovement": 5, "nearestPoints": 3, "correctGeometry": true,
        "normalize": false, "edgeSkip": 2, "roi": null
    }))
    .unwrap();
    assert_eq!(
        (stack.reference_frame, stack.nearest_points, stack.edge_skip),
        (7, 3, 2)
    );

    let sequence: SequenceSettings = serde_json::from_value(json!({
        "stack": {
            "referenceFrame": 0, "points": [], "frames": [], "boxSize": 30, "maxMovement": 5,
            "nearestPoints": 3, "correctGeometry": true, "normalize": false, "edgeSkip": 0,
            "roi": null
        },
        "framerate": 25, "sortOnQuality": true, "averageFrames": 4
    }))
    .unwrap();
    assert!(sequence.sort_on_quality);
    assert_eq!((sequence.framerate, sequence.average_frames), (25, 4));
}

#[test]
fn the_results_go_out_in_camel_case() {
    let points = Alignpoints {
        points: vec![Point { x: 1, y: 2 }],
        total: 5,
        threshold: 1,
    };
    assert_eq!(
        serde_json::to_value(points).unwrap(),
        json!({ "points": [{ "x": 1, "y": 2 }], "total": 5, "threshold": 1 })
    );

    let aligned = Aligned {
        frames: vec![FrameRating {
            index: 0,
            quality: 0.5,
            shift_x: 1.0,
            shift_y: -1.0,
            difference: 0.25,
        }],
        ranking: vec![0],
    };
    assert_eq!(
        serde_json::to_value(aligned).unwrap(),
        json!({
            "frames": [{ "index": 0, "quality": 0.5, "shiftX": 1.0, "shiftY": -1.0, "difference": 0.25 }],
            "ranking": [0]
        })
    );

    let limited = Limited {
        frames: vec![3, 1],
        point_shifts: vec![Some([0.5, -0.5]), None],
    };
    assert_eq!(
        serde_json::to_value(limited).unwrap(),
        json!({ "frames": [3, 1], "pointShifts": [[0.5, -0.5], null] })
    );

    let stacked = Stacked {
        width: 4,
        height: 3,
        channels: 3,
        frames: 10,
        elapsed_ms: 12,
    };
    assert_eq!(
        serde_json::to_value(stacked).unwrap(),
        json!({ "width": 4, "height": 3, "channels": 3, "frames": 10, "elapsedMs": 12 })
    );

    let source = SourceInfo {
        width: 4,
        height: 3,
        channels: 1,
    };
    assert_eq!(
        serde_json::to_value(source).unwrap(),
        json!({ "width": 4, "height": 3, "channels": 1 })
    );

    let saved = SavedVideo {
        path: "a.avi".into(),
        frames: 2,
    };
    assert_eq!(
        serde_json::to_value(saved).unwrap(),
        json!({ "path": "a.avi", "frames": 2 })
    );
}

#[test]
fn the_error_texts_are_the_ones_the_window_shows() {
    let texts = [
        (SessionError::NoRecording, "no recording is open"),
        (SessionError::NoAlignpoints, "set alignpoints first"),
        (SessionError::NotAligned, "align first"),
        (SessionError::NotStacked, "stack first"),
        (SessionError::NoImageToSharpen, "no image to sharpen"),
        (SessionError::NoImageToAdjust, "no image to adjust"),
        (
            SessionError::NoStackedImage,
            "there is no stacked image yet",
        ),
        (
            SessionError::NoProcessedImage,
            "there is no processed image yet",
        ),
        (SessionError::NoFramesSelected, "no frames selected"),
        (
            SessionError::SortNeedsAlign,
            "Sort On Quality needs the result of Align",
        ),
        (SessionError::Cancelled, "cancelled"),
        (
            SessionError::BadImageSize {
                width: 2,
                height: 2,
                bytes: 15,
            },
            "15 bytes do not fit an image of 2x2",
        ),
        (SessionError::Failed("x.avi: gone".into()), "x.avi: gone"),
    ];
    for (error, text) in texts {
        assert_eq!(error.to_string(), text);
    }
}

#[test]
fn a_step_without_its_input_says_what_is_missing() {
    let session = Session::new().unwrap();
    assert_eq!(
        session.read_frame(0).unwrap_err(),
        SessionError::NoRecording
    );
    assert_eq!(
        session.limit(&LimitSettings::BestCount(1)).unwrap_err(),
        SessionError::NotAligned
    );
    assert_eq!(
        session.stacked_image().unwrap_err(),
        SessionError::NotStacked
    );
}

#[test]
fn the_session_can_be_shared_between_threads() {
    fn shared<T: Send + Sync>() {}
    shared::<Session>();
}
