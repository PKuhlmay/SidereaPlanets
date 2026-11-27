//! Regression test of the stack against the stored result of this program for reference case R1 (Jupiter test
//! recording, `docs/stacking.md`). The recording and the reference image live in `re-local/` and are not in the
//! repository; without them the test is skipped. Regenerate the reference with `scripts/gen-reference.sh`, only after an
//! intended change of the algorithms.

use planets_io::recording::Recording;

mod common;

#[test]
fn jupiter_stack_matches_the_stored_reference_stack() {
    let (Some(video), Some(dir)) = (common::jupiter_video(), common::existing_reference_dir())
    else {
        eprintln!("JUPITER.AVI or re-local/reference-own not found, skipped");
        return;
    };
    let (_, stored) = common::read_tiff(&dir.join("jupiter-stack.tif"));
    let stack = common::jupiter_stack(&mut Recording::open(&video).unwrap());

    let db = common::psnr_u16(&common::to_u16(&stack), &stored);
    eprintln!("stack against the stored reference: {db:.2} dB");
    assert!(
        db >= common::MIN_PSNR_DB,
        "{db:.2} dB against the stored reference, the threshold is {} dB",
        common::MIN_PSNR_DB
    );
}
