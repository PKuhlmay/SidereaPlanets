//! How long the Rust side of a slider step takes in the session: `wavelet_preview` for the Processing Area,
//! on a frame of JUPITER.AVI, after Do All. The window adds transfer and drawing; `app/src/previewBench.ts` measures
//! those (`docs/wavelets.md`, Vorschau). Numbers only mean something in a release build:
//! `cargo test -p planets-session --release --test preview_timing -- --ignored --nocapture`

use planets_core::wavelet::{ColourMode, Region};
use planets_session::{Session, wavelet_default_settings};
use std::{path::PathBuf, time::Instant};

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

fn percentile(sorted: &[f64], p: f64) -> f64 {
    sorted[((sorted.len() - 1) as f64 * p).round() as usize]
}

#[test]
#[ignore = "timing, run in release"]
fn a_slider_step_in_the_session() {
    let Some(path) = jupiter() else {
        eprintln!("JUPITER.AVI not found in re-local/, skipped");
        return;
    };
    let session = Session::new().unwrap();
    let info = session.open_recording(&[path]).unwrap();
    session.wavelet_set_source_frame(0).unwrap();
    let mut settings = wavelet_default_settings();
    settings.layers[1].gain = 2.0;
    session
        .wavelet_do_all(&settings, ColourMode::Colour, &planets_session::NoProgress)
        .unwrap();
    let area = Region::centered(512, info.width, info.height);

    let mut steps = Vec::new();
    let mut packs = Vec::new();
    for i in 0..200 {
        settings.layers[0].gain = 1.0 + (i % 40) as f32 * 0.25;
        let started = Instant::now();
        let preview = session
            .wavelet_preview(Some(&settings), area, ColourMode::Colour)
            .unwrap();
        steps.push(started.elapsed().as_secs_f64() * 1000.0);
        // What the command does with the answer: header and pixels into one buffer.
        let started = Instant::now();
        let mut out = Vec::with_capacity(16 + preview.rgba().len());
        out.extend([0u8; 16]);
        out.extend(preview.into_rgba());
        packs.push(started.elapsed().as_secs_f64() * 1000.0);
        std::hint::black_box(out);
    }
    steps.sort_by(f64::total_cmp);
    packs.sort_by(f64::total_cmp);
    eprintln!(
        "{}x{}, area 512: session step median {:.2} ms, p95 {:.2} ms; pack copy median {:.2} ms",
        info.width,
        info.height,
        percentile(&steps, 0.5),
        percentile(&steps, 0.95),
        percentile(&packs, 0.5)
    );
}
