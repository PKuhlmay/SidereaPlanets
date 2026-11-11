//! Measures how fast the preview answers while a slider moves (`docs/wavelets.md`, section Vorschau).
//! The numbers only mean something in a release build, so the test is ignored by default:
//! `cargo test -p planets-core --release --test wavelet_preview_timing -- --ignored --nocapture`

use planets_core::{
    Image,
    wavelet::{Previewer, Region, WaveletSettings},
};
use std::{sync::Arc, time::Instant};

/// Largest time for one slider step in the Processing Area of 512 px (20 frames per second).
const BUDGET_MS: f64 = 50.0;

fn synthetic(width: u32, height: u32) -> Image {
    let mut img = Image::new(width, height, 3);
    for (i, v) in img.data.iter_mut().enumerate() {
        let (x, y) = ((i / 3) % width as usize, (i / 3) / width as usize);
        *v = 0.5 + 0.4 * ((x as f32 * 0.37).sin() * (y as f32 * 0.23).cos());
    }
    img
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

#[test]
#[ignore = "timing, run in release"]
fn a_slider_step_in_the_processing_area_fits_the_frame_budget() {
    for (width, height) in [(1024, 680), (2048, 2048)] {
        let mut previewer = Previewer::new(Arc::new(synthetic(width, height)));
        let area = Region::centered(512, width, height);
        let mut settings = WaveletSettings::default();
        settings.layers[0].gain = 3.0;

        // First call: builds the layers of the area.
        let started = Instant::now();
        previewer.render(&settings, area);
        let first = started.elapsed().as_secs_f64() * 1000.0;

        // Every further call only recombines them.
        let steps: Vec<f64> = (0..15)
            .map(|i| {
                settings.layers[0].gain = 3.0 + i as f32 * 0.5;
                let started = Instant::now();
                previewer.render(&settings, area);
                started.elapsed().as_secs_f64() * 1000.0
            })
            .collect();
        let step = median(steps);
        eprintln!(
            "{width}x{height}, area 512: first {first:.1} ms, slider step {step:.1} ms (median of 15)"
        );
        assert!(step <= BUDGET_MS, "{step:.1} ms per slider step");
    }
}
