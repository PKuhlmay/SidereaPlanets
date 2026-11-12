//! De-rotation in the session (MAC-020): a SER recording of a rotating synthetic Jupiter whose trailer carries the time of
//! every frame, and an AVI without times that needs the start time.

use planets_core::{
    Image,
    synthetic::{jupiter_scene, synthetic_jupiter},
};
use planets_io::avi_write::AviWriter;
use planets_session::{
    DerotationSettings, NoProgress, Planet, PointPosition, Session, SessionError, StackSettings,
    unix_seconds,
};
use std::path::PathBuf;

const UNIX_EPOCH_TICKS: i64 = 621_355_968_000_000_000;

fn temp(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("planets-derotation-{}-{name}", std::process::id()))
}

/// Jupiter as Meeus' example 43.a has it (1992 December 16, `D_E` -2.5, `P` 24.8), five minutes apart.
const SECONDS: [f64; 3] = [0.0, 300.0, 600.0];

fn start() -> f64 {
    unix_seconds(1992, 12, 16, 0, 0, 0.0)
}

fn frames() -> Vec<Image> {
    SECONDS.iter().map(|&t| synthetic_jupiter(t).0).collect()
}

/// A SER file (8 bit, mono) of the frames; with `times` the trailer gets one time per frame (UTC ticks).
fn ser(name: &str, frames: &[Image], times: Option<&[f64]>) -> PathBuf {
    let mut bytes = vec![0u8; 178];
    bytes[..14].copy_from_slice(b"LUCAM-RECORDER");
    bytes[22..26].copy_from_slice(&1i32.to_le_bytes());
    bytes[26..30].copy_from_slice(&320i32.to_le_bytes());
    bytes[30..34].copy_from_slice(&240i32.to_le_bytes());
    bytes[34..38].copy_from_slice(&8i32.to_le_bytes());
    bytes[38..42].copy_from_slice(&(frames.len() as i32).to_le_bytes());
    for frame in frames {
        bytes.extend(frame.data.iter().map(|&v| (v * 255.0).round() as u8));
    }
    for unix in times.into_iter().flatten() {
        let ticks = (unix * 1e7) as i64 + UNIX_EPOCH_TICKS;
        bytes.extend_from_slice(&ticks.to_le_bytes());
    }
    let path = temp(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

fn settings(derotation: Option<DerotationSettings>) -> StackSettings {
    // Alignpoints on a grid over the disc (centre 160, 120; radius 100).
    let points = (60..=180u32)
        .step_by(30)
        .flat_map(|y| {
            (80..=240u32)
                .step_by(30)
                .map(move |x| PointPosition { x, y })
        })
        .collect();
    StackSettings {
        reference_frame: 1,
        points,
        frames: vec![0, 1, 2],
        box_size: 12,
        max_movement: 3,
        nearest_points: 3,
        correct_geometry: false,
        normalize: false,
        edge_skip: 0,
        roi: None,
        derotation,
        centre_of_gravity: None,
    }
}

fn jupiter_settings() -> DerotationSettings {
    DerotationSettings {
        planet: Planet::Jupiter,
        camera_angle_deg: 0.0,
        mirrored: false,
        start_time_utc: None,
        use_file_timestamps: true,
    }
}

/// The stack inside the disc against the middle frame, which is what a de-rotated stack should look like.
fn disc_psnr(session: &Session) -> f64 {
    let stacked = session.stacked_image().unwrap();
    let target = &frames()[1];
    let centre = jupiter_scene().centre;
    let (mut sum, mut n) = (0.0f64, 0usize);
    for y in 0..240usize {
        for x in 0..320usize {
            if (x as f64 - centre.0).hypot(y as f64 - centre.1) < 96.0 {
                let a = f64::from(stacked.rgba()[(y * 320 + x) * 4]) / 255.0;
                let b = f64::from(target.data[y * 320 + x]);
                sum += (a - b).powi(2);
                n += 1;
            }
        }
    }
    10.0 * (1.0 / (sum / n as f64)).log10()
}

#[test]
fn the_time_stamps_of_a_ser_file_turn_the_frames_back() {
    let times: Vec<f64> = SECONDS.iter().map(|t| start() + t).collect();
    let path = ser("stamped.ser", &frames(), Some(&times));
    let session = Session::new().unwrap();
    let info = session.open_recording(&[path]).unwrap();
    assert!(info.has_timestamps);
    assert!((info.start_time.unwrap() - start()).abs() < 1e-3);
    assert!(!info.start_time_from_file_date);

    session.stack(&settings(None), &NoProgress).unwrap();
    let without = disc_psnr(&session);
    session
        .stack(&settings(Some(jupiter_settings())), &NoProgress)
        .unwrap();
    let with = disc_psnr(&session);
    eprintln!("session, 3 frames over 10 min: with {with:.2} dB, without {without:.2} dB");
    assert!(with > without + 3.0, "with {with} dB, without {without} dB");
    assert!(with > 30.0, "with {with} dB");
}

#[test]
fn the_panel_gets_span_rotation_and_disc() {
    let times: Vec<f64> = SECONDS.iter().map(|t| start() + t).collect();
    let session = Session::new().unwrap();
    session
        .open_recording(&[ser("info.ser", &frames(), Some(&times))])
        .unwrap();
    let info = session
        .derotation_info(&jupiter_settings(), 1, &[0, 1, 2])
        .unwrap();
    assert_eq!(info.problem, None);
    assert!((info.span_seconds - 600.0).abs() < 1e-6);
    // 600 s of 35 740.6 s is 6.04 degrees.
    assert!(
        (info.rotation_degrees - 6.044).abs() < 0.01,
        "{}",
        info.rotation_degrees
    );
    // Meeus' example 43.a (1992 December 16, 0h, plus five minutes).
    assert!((info.sub_earth_latitude - -2.48).abs() < 0.15);
    assert!((info.pole_angle - 24.8).abs() < 0.2);
    let disc = info.disc.expect("the disc is measured");
    assert!((disc.semi_major - 100.0).abs() < 1.0 && (disc.x - 160.3).abs() < 0.7);
    assert!(disc.semi_minor < disc.semi_major);
}

#[test]
fn without_time_stamps_the_start_time_and_the_frame_rate_decide() {
    let dir = temp("avi");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("jupiter.avi");
    let mut writer = AviWriter::create(&path, 320, 240, 1).unwrap();
    for frame in frames() {
        writer.write_frame(&frame).unwrap();
    }
    writer.finish().unwrap();

    let session = Session::new().unwrap();
    let opened = session.open_recording(&[path]).unwrap();
    assert!(!opened.has_timestamps);
    // The start is guessed from the file date and says so.
    assert!(opened.start_time.is_some() && opened.start_time_from_file_date);

    // No start time: a problem for the panel, an error for Stack.
    let missing = session
        .derotation_info(&jupiter_settings(), 1, &[0, 1, 2])
        .unwrap();
    assert_eq!(
        missing.problem.as_deref(),
        Some("De-rotation needs the start time of the recording")
    );
    assert!(missing.disc.is_none());
    let error = session
        .stack(&settings(Some(jupiter_settings())), &NoProgress)
        .unwrap_err();
    assert_eq!(
        error,
        SessionError::Failed("De-rotation needs the start time of the recording".into())
    );

    // With it, frame n is n seconds after the start (1 frame per second).
    let given = DerotationSettings {
        start_time_utc: Some(start()),
        ..jupiter_settings()
    };
    let info = session.derotation_info(&given, 1, &[0, 1, 2]).unwrap();
    assert_eq!(info.problem, None);
    assert!(
        (info.span_seconds - 2.0).abs() < 1e-9,
        "{}",
        info.span_seconds
    );
    session.stack(&settings(Some(given)), &NoProgress).unwrap();
}

#[test]
fn a_planet_that_is_not_in_the_frame_is_reported() {
    let dark = vec![Image::new(320, 240, 1); 3];
    let session = Session::new().unwrap();
    let times: Vec<f64> = SECONDS.iter().map(|t| start() + t).collect();
    session
        .open_recording(&[ser("dark.ser", &dark, Some(&times))])
        .unwrap();
    let info = session
        .derotation_info(&jupiter_settings(), 1, &[0, 1, 2])
        .unwrap();
    assert_eq!(
        info.problem.as_deref(),
        Some("De-rotation could not find the planet in the reference frame")
    );
}

#[test]
fn the_request_of_the_window_parses_with_and_without_derotation() {
    let off: StackSettings = serde_json::from_value(serde_json::json!({
        "referenceFrame": 0, "points": [], "frames": [], "boxSize": 12, "maxMovement": 3,
        "nearestPoints": 3, "correctGeometry": true, "normalize": false, "edgeSkip": 0, "roi": null
    }))
    .unwrap();
    assert_eq!(off.derotation, None);
    let on: StackSettings = serde_json::from_value(serde_json::json!({
        "referenceFrame": 0, "points": [], "frames": [], "boxSize": 12, "maxMovement": 3,
        "nearestPoints": 3, "correctGeometry": true, "normalize": false, "edgeSkip": 0, "roi": null,
        "derotation": { "planet": "saturn", "cameraAngleDeg": 12.5, "startTimeUtc": 1700000000.5 }
    }))
    .unwrap();
    let d = on.derotation.unwrap();
    assert_eq!(d.planet, Planet::Saturn);
    assert_eq!(d.camera_angle_deg, 12.5);
    assert_eq!(d.start_time_utc, Some(1_700_000_000.5));
    assert!(d.use_file_timestamps && !d.mirrored);
}
