//! The whole flow of the window on JUPITER.AVI, from Select to Save image, through the session only.
//! The recording lives in `re-local/` (not in Git); without it the test is skipped. Every step is checked
//! for a plausible result, not for a match with stored reference results (that is what the `*_reference.rs`
//! tests of `crates/io` do).

use planets_core::{
    crop::CropRect,
    geometry::{Geometry, Orientation, Resampling, Resize, ResizeTarget},
    wavelet::ColourMode,
};
use planets_io::{export::ExportFormat, images::ImageSequenceReader};
use planets_session::{
    ColorSettings, ImageSource, LimitSettings, Operation, PointSettings, RoiSettings, Session,
    SessionError, StackSettings, wavelet_default_settings,
};
use serde_json::json;
use std::{path::PathBuf, sync::Mutex};

/// The first existing `re-local/` folder: `$PLANETS_RE_LOCAL` or the one in the repository.
fn jupiter() -> Option<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        std::env::var_os("PLANETS_RE_LOCAL").map(PathBuf::from),
        Some(manifest.join("../../re-local")),
        Some(manifest.join("../../../../../re-local")),
    ];
    let re_local = candidates.into_iter().flatten().find(|p| p.is_dir())?;
    let path = re_local.join("testdata/astropix/JUPITER.AVI");
    path.exists().then_some(path)
}

/// The defaults of the window (`app/src/logic/settings.ts`).
fn point_settings() -> PointSettings {
    serde_json::from_value(json!({
        "minDistance": 30, "edgeDistance": 20, "maxMovement": 5, "roi": null,
        "intensity": "default", "lo": 30, "hi": 230, "boxSize": 30
    }))
    .unwrap()
}

fn identity_colors() -> ColorSettings {
    serde_json::from_value(json!({
        "shifts": { "red": { "x": 0, "y": 0 }, "green": { "x": 0, "y": 0 }, "blue": { "x": 0, "y": 0 } },
        "red": 100, "green": 100, "blue": 100, "black": 0, "white": 255,
        "gamma": 1, "contrast": 100, "brightness": 0
    }))
    .unwrap()
}

#[test]
fn jupiter_runs_from_select_to_save_image() {
    let Some(path) = jupiter() else {
        eprintln!("JUPITER.AVI not found in re-local/, skipped");
        return;
    };
    let session = Session::new().unwrap();

    // Select.
    let info = session.open_recording(&[path]).unwrap();
    assert_eq!((info.width, info.height, info.frame_count), (1024, 680, 10));
    let frame = session.read_frame(0).unwrap();
    assert_eq!((frame.width(), frame.height()), (1024, 680));
    assert_eq!(frame.rgba().len(), 1024 * 680 * 4);
    assert!(
        session.read_frame(10).is_err(),
        "a frame past the end is an error"
    );

    // Set Alignpoints.
    let points = session.set_alignpoints(0, &point_settings()).unwrap();
    assert!(
        (70..=100).contains(&points.points.len()),
        "{} points",
        points.points.len()
    );
    assert!(points.total >= points.points.len());

    // Align: progress grows to the frame count and carries the operation.
    let seen = Mutex::new(Vec::new());
    let sink = |op, done, total, _| seen.lock().unwrap().push((op, done, total));
    let aligned = session
        .align(
            &planets_session::AlignSettings {
                box_size: 30,
                max_movement: 5,
                roi: None::<RoiSettings>,
                centre_of_gravity: None,
            },
            &sink,
        )
        .unwrap();
    let expected: Vec<_> = (1..=10).map(|done| (Operation::Align, done, 10)).collect();
    assert_eq!(*seen.lock().unwrap(), expected);
    assert_eq!(aligned.frames.len(), 10);
    assert_eq!(aligned.ranking[0], 0, "frame 1 is the best one");

    // Limit.
    let limited = session.limit(&LimitSettings::LowestQuality(80.0)).unwrap();
    assert_eq!(limited.frames.len(), 10);
    assert_eq!(limited.point_shifts.len(), points.points.len());

    // Stack, with the settings the window sends.
    let stack: StackSettings = serde_json::from_value(json!({
        "referenceFrame": 0,
        "points": points.points.iter().map(|p| json!({ "x": p.x, "y": p.y })).collect::<Vec<_>>(),
        "frames": limited.frames,
        "boxSize": 30, "maxMovement": 5, "nearestPoints": 3, "correctGeometry": true,
        "normalize": false, "edgeSkip": 0, "roi": null
    }))
    .unwrap();
    let stacked = session.stack(&stack, &planets_session::NoProgress).unwrap();
    assert_eq!(
        (stacked.width, stacked.height, stacked.frames),
        (1024, 680, 10)
    );
    let stack_image = session.stacked_image().unwrap();
    assert_eq!((stack_image.width(), stack_image.height()), (1024, 680));

    // Wavelets: Do All, then the colour panels on the result.
    let sharpened = session
        .wavelet_do_all(
            &wavelet_default_settings(),
            ColourMode::Colour,
            &planets_session::NoProgress,
        )
        .unwrap();
    assert_eq!((sharpened.width(), sharpened.height()), (1024, 680));
    session
        .set_color_source(sharpened.width(), sharpened.height(), sharpened.rgba())
        .unwrap();
    let adjusted = session.apply_colors(&identity_colors()).unwrap();
    assert_eq!(adjusted.rgba().len(), sharpened.rgba().len());

    // Save image, both tabs, and read the files again.
    let dir = std::env::temp_dir().join(format!("planets-session-jupiter-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for (source, name) in [
        (ImageSource::Stack, "stack"),
        (ImageSource::Wavelet, "sharp"),
    ] {
        let written = session.save_image(
            source,
            ExportFormat::Tiff16,
            &dir.join(name),
            Some(&identity_colors()),
            None,
            None,
        );
        assert!(written.is_ok(), "Save image {name}: {written:?}");
        let written = written.unwrap();
        assert!(written.ends_with(".tif"), "{written}");
        let back = ImageSequenceReader::open(vec![PathBuf::from(&written)])
            .unwrap()
            .read_frame(0)
            .unwrap();
        assert_eq!((back.width, back.height), (1024, 680));
    }

    // A crop writes only that part of the sharpened image, and the pixels are the ones of the whole image.
    let crop = CropRect {
        x: 100,
        y: 50,
        width: 600,
        height: 450,
    };
    let whole = ImageSequenceReader::open(vec![dir.join("sharp.tif")])
        .unwrap()
        .read_frame(0)
        .unwrap();
    let written = session
        .save_image(
            ImageSource::Wavelet,
            ExportFormat::Tiff16,
            &dir.join("cropped"),
            Some(&identity_colors()),
            Some(crop),
            None,
        )
        .unwrap();
    let cropped = ImageSequenceReader::open(vec![PathBuf::from(&written)])
        .unwrap()
        .read_frame(0)
        .unwrap();
    assert_eq!((cropped.width, cropped.height), (600, 450));
    assert_eq!(Some(cropped), whole.cropped(crop));

    // Turned by 90 degrees the sharpened image is 680 x 1024; the crop is set on that picture and the resize comes last (MAC-017).
    let geometry = Geometry {
        orientation: Orientation {
            angle: 90.0,
            ..Orientation::default()
        },
        resize: Some(Resize {
            target: ResizeTarget::Percent(50.0),
            method: Resampling::Lanczos3,
        }),
    };
    let turned_crop = CropRect {
        x: 40,
        y: 100,
        width: 600,
        height: 800,
    };
    let written = session
        .save_image(
            ImageSource::Wavelet,
            ExportFormat::Tiff16,
            &dir.join("turned"),
            Some(&identity_colors()),
            Some(turned_crop),
            Some(&geometry),
        )
        .unwrap();
    let turned = ImageSequenceReader::open(vec![PathBuf::from(&written)])
        .unwrap()
        .read_frame(0)
        .unwrap();
    assert_eq!((turned.width, turned.height), (300, 400));
    // Without the resize it is the part of the turned whole image.
    let written = session
        .save_image(
            ImageSource::Wavelet,
            ExportFormat::Tiff16,
            &dir.join("turned-full"),
            Some(&identity_colors()),
            Some(turned_crop),
            Some(&Geometry {
                resize: None,
                ..geometry
            }),
        )
        .unwrap();
    let turned_full = ImageSequenceReader::open(vec![PathBuf::from(&written)])
        .unwrap()
        .read_frame(0)
        .unwrap();
    assert_eq!(Some(turned_full), whole.turned(1).cropped(turned_crop));

    // Aligning again throws it all away: Save image has nothing left to write.
    session.set_alignpoints(0, &point_settings()).unwrap();
    assert_eq!(
        session.save_image(
            ImageSource::Stack,
            ExportFormat::Tiff16,
            &dir.join("gone"),
            None,
            None,
            None
        ),
        Err(SessionError::NoStackedImage)
    );
}
