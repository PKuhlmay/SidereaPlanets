//! Generator of the stored reference results in `re-local/reference-own/` (not in Git). Ignored by default; run it with
//! `scripts/gen-reference.sh` and only after an intended, reviewed change of the algorithms: `docs/stacking.md`,
//! section "Regressionsprüfung".
//!
//! It runs the current pipeline on the Jupiter test recording (reference case R1) and writes:
//! - `jupiter-stack.tif`: the stack with the default settings (16-bit TIFF, as *Save image* writes it),
//! - `jupiter-wavelet-default.tif`, `jupiter-wavelet-l1-10.tif`: *Do All* on that stack with the default settings and
//!   with Layer 1 = 10.0,
//! - `jupiter-l1-10.rwv`: the scheme of the second case,
//! - `jupiter-align.json`: candidates, alignpoints and the quality order of the frames.

use planets_core::wavelet::{ColourMode, WaveletSettings, do_all, scheme};
use planets_io::{
    export::{ExportFormat, save_image},
    recording::Recording,
};

mod common;

#[test]
#[ignore = "writes re-local/reference-own/; run through scripts/gen-reference.sh"]
fn generate_the_reference_results() {
    let video =
        common::jupiter_video().expect("JUPITER.AVI not found in re-local/testdata/astropix");
    let dir = common::reference_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let save = |name: &str, image: &planets_core::Image| {
        save_image(image, &dir.join(name), ExportFormat::Tiff16).unwrap();
    };

    let mut rec = Recording::open(&video).unwrap();
    let stack = common::jupiter_stack(&mut rec);
    save("jupiter-stack.tif", &stack);

    // The wavelet cases start from the stored (16-bit rounded) stack, exactly as the tests do.
    let (stored, _) = common::read_tiff(&dir.join("jupiter-stack.tif"));
    let neutral = do_all(&stored, &WaveletSettings::default(), ColourMode::Colour);
    save("jupiter-wavelet-default.tif", &neutral);
    let layer1 = common::layer_one_at_ten();
    save(
        "jupiter-wavelet-l1-10.tif",
        &do_all(&stored, &layer1, ColourMode::Colour),
    );
    std::fs::write(dir.join("jupiter-l1-10.rwv"), scheme::to_rwv(&layer1)).unwrap();

    let mut rec = Recording::open(&video).unwrap();
    let summary = common::jupiter_align(&mut rec);
    std::fs::write(
        dir.join("jupiter-align.json"),
        serde_json::to_string_pretty(&summary).unwrap() + "\n",
    )
    .unwrap();
    eprintln!("wrote the reference results to {}", dir.display());
}
