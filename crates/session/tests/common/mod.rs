//! A small synthetic recording and the settings to run every step on it, so that the state
//! transitions of the session can be tested without the real videos.

#![allow(dead_code)] // every test file uses a part of this

use planets_core::{Image, wavelet::ColourMode};
use planets_io::export::{ExportFormat, save_image};
use planets_session::{
    AlignSettings, Alignpoints, Intensity, NoProgress, PointPosition, PointSettings, Session,
    StackSettings, wavelet_default_settings,
};
use std::path::PathBuf;

pub const FRAMES: u32 = 6;
const SIZE: u32 = 96;

/// Smooth blobs of fixed pseudo-random position; `shift` moves the content to the right.
fn blobs(shift: f32) -> Image {
    let mut seed = 99u32;
    let mut rand = move || {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (seed >> 8) as f32 / (1u32 << 24) as f32
    };
    let list: Vec<[f32; 4]> = (0..30)
        .map(|_| {
            [
                rand() * SIZE as f32,
                rand() * SIZE as f32,
                2.0 + rand() * 3.0,
                0.1 + rand() * 0.3,
            ]
        })
        .collect();
    let mut image = Image::new(SIZE, SIZE, 1);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let mut v = 0.25;
            for [cx, cy, sigma, amplitude] in &list {
                let (dx, dy) = (x as f32 - cx - shift, y as f32 - cy);
                v += amplitude * (-(dx * dx + dy * dy) / (2.0 * sigma * sigma)).exp();
            }
            image.data[(y * SIZE + x) as usize] = v.min(1.0);
        }
    }
    image
}

/// A session with an open recording of [`FRAMES`] frames, each shifted by one pixel against the last.
pub fn session(name: &str) -> Session {
    let dir = std::env::temp_dir().join(format!("planets-session-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let paths: Vec<PathBuf> = (0..FRAMES)
        .map(|i| {
            let path = dir.join(format!("frame{i}.png"));
            save_image(&blobs(i as f32), &path, ExportFormat::Png16).unwrap();
            path
        })
        .collect();
    let session = Session::new().unwrap();
    let info = session.open_recording(&paths).unwrap();
    assert_eq!(info.frame_count, FRAMES);
    session
}

pub fn point_settings() -> PointSettings {
    PointSettings {
        min_distance: 10,
        edge_distance: 10,
        max_movement: 3,
        roi: None,
        intensity: Intensity::Default,
        lo: 0.0,
        hi: 255.0,
        box_size: 12,
    }
}

pub fn align_settings() -> AlignSettings {
    AlignSettings {
        box_size: 12,
        max_movement: 3,
        roi: None,
        centre_of_gravity: None,
    }
}

pub fn stack_settings(points: &Alignpoints) -> StackSettings {
    StackSettings {
        reference_frame: 0,
        points: points
            .points
            .iter()
            .map(|p| PointPosition { x: p.x, y: p.y })
            .collect(),
        frames: (0..FRAMES).collect(),
        box_size: 12,
        max_movement: 3,
        nearest_points: 3,
        correct_geometry: false,
        normalize: false,
        edge_skip: 0,
        roi: None,
        derotation: None,
        centre_of_gravity: None,
    }
}

/// A session that went through every step up to the colour panels: it holds a stack, a sharpened
/// image and a colour source. Returns the alignpoints of Set Alignpoints as well.
pub fn processed(name: &str) -> (Session, Alignpoints) {
    let session = session(name);
    let points = session.set_alignpoints(0, &point_settings()).unwrap();
    assert!(!points.points.is_empty(), "the synthetic frame has points");
    session.align(&align_settings(), &NoProgress).unwrap();
    session
        .stack(&stack_settings(&points), &NoProgress)
        .unwrap();
    session
        .wavelet_do_all(&wavelet_default_settings(), ColourMode::Colour, &NoProgress)
        .unwrap();
    let stacked = session.stacked_image().unwrap();
    session
        .set_color_source(stacked.width(), stacked.height(), stacked.rgba())
        .unwrap();
    (session, points)
}
