//! Regression test of wavelet sharpening against stored results of this program for reference case R1 (Jupiter test
//! recording, `docs/wavelets.md`). The reference files live in `re-local/reference-own/` and are not in the repository;
//! without them the tests are skipped. Input is the stored reference stack, so the stacking does not count. Regenerate
//! the references with `scripts/gen-reference.sh`, only after an intended change of the algorithms.

use planets_core::Image;
use planets_core::wavelet::{ColourMode, WaveletSettings, do_all, scheme};

mod common;

struct Stored {
    stack: Image,
    stack_u16: Vec<u16>,
    neutral: Vec<u16>,
    layer1: Vec<u16>,
    layer1_settings: WaveletSettings,
}

fn stored() -> Option<Stored> {
    let Some(dir) = common::existing_reference_dir() else {
        eprintln!("re-local/reference-own not found, skipped");
        return None;
    };
    let samples = |name: &str| common::read_tiff(&dir.join(name)).1;
    let (stack, stack_u16) = common::read_tiff(&dir.join("jupiter-stack.tif"));
    let rwv = std::fs::read_to_string(dir.join("jupiter-l1-10.rwv")).unwrap();
    Some(Stored {
        stack,
        stack_u16,
        neutral: samples("jupiter-wavelet-default.tif"),
        layer1: samples("jupiter-wavelet-l1-10.tif"),
        layer1_settings: scheme::from_rwv(&rwv).unwrap(),
    })
}

#[test]
fn default_settings_match_the_stored_reference() {
    let Some(r) = stored() else { return };
    let result = common::to_u16(&do_all(
        &r.stack,
        &WaveletSettings::default(),
        ColourMode::Colour,
    ));
    let db = common::psnr_u16(&result, &r.neutral);
    eprintln!("default settings against the stored reference: {db:.2} dB");
    assert!(db >= common::MIN_PSNR_DB, "{db:.2} dB");
}

#[test]
fn layer_one_at_ten_matches_the_stored_reference() {
    let Some(r) = stored() else { return };
    // The stored scheme is the one the generator wrote; it must still read as the same settings.
    assert_eq!(r.layer1_settings, common::layer_one_at_ten());
    let result = common::to_u16(&do_all(&r.stack, &r.layer1_settings, ColourMode::Colour));

    let db = common::psnr_u16(&result, &r.layer1);
    // The reference is only meaningful if the sharpening moves the image further than the threshold allows.
    let unsharpened = common::psnr_u16(&r.stack_u16, &r.layer1);
    eprintln!(
        "Layer 1 = 10.0 against the stored reference: {db:.2} dB (the unsharpened stack: {unsharpened:.2} dB)"
    );
    assert!(db >= common::MIN_PSNR_DB, "{db:.2} dB");
    assert!(
        unsharpened < common::MIN_PSNR_DB,
        "the sharpened reference must differ from the stack by more than the threshold ({unsharpened:.2} dB)"
    );
}
