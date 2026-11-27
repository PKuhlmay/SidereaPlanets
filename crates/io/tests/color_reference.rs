//! Runs the colour and tone functions on the colour stack of reference case R1 (Jupiter test recording). The files
//! live in `re-local/` and are not in the repository; without them the tests are skipped.
//!
//! There are no stored results for RGB align, balance, gamma or histogram. What the tests do is check the functions on
//! real colour data, where a synthetic scene cannot. The input is the stack stored in `re-local/reference-own/`
//! (`scripts/gen-reference.sh`); the channel means below describe that file and change with it.

use planets_core::{
    Image,
    color::{Adjustments, RgbShifts, Shift, adjust, estimate_rgb_shifts, shift_channels},
    tone::{auto_levels, histogram, stretch_levels},
};
use planets_io::images::ImageSequenceReader;
mod common;

/// The stored 16-bit stack of Jupiter (reference case R1), written by this program.
fn jupiter_stack() -> Option<Image> {
    let path = common::reference_dir().join("jupiter-stack.tif");
    path.exists().then(|| {
        ImageSequenceReader::open(vec![path])
            .unwrap()
            .read_frame(0)
            .unwrap()
    })
}

/// Halves the size by averaging 2x2 pixels, to keep the search of the auto align quick in a debug build.
fn binned(image: &Image) -> Image {
    let (w, h) = (image.width as usize / 2, image.height as usize / 2);
    let stride = usize::from(image.channels);
    let mut out = Image::new(w as u32, h as u32, image.channels);
    for y in 0..h {
        for x in 0..w {
            for c in 0..stride {
                let at = |dx: usize, dy: usize| {
                    image.data[((2 * y + dy) * image.width as usize + 2 * x + dx) * stride + c]
                };
                out.data[(y * w + x) * stride + c] =
                    (at(0, 0) + at(1, 0) + at(0, 1) + at(1, 1)) / 4.0;
            }
        }
    }
    out
}

#[test]
fn reference_stack_has_the_stored_channel_means() {
    let Some(stack) = jupiter_stack() else {
        eprintln!("jupiter-stack.tif not found in re-local/reference-own, skipped");
        return;
    };
    assert_eq!((stack.width, stack.height, stack.channels), (1024, 680, 3));
    // Channel means of the stored 16-bit file: 6724.7 / 5169.5 / 6145.6.
    let pixels = (stack.width * stack.height) as f64;
    for (c, expected) in [6724.7, 5169.5, 6145.6].into_iter().enumerate() {
        let mean = stack
            .data
            .iter()
            .skip(c)
            .step_by(3)
            .map(|&v| f64::from(v))
            .sum::<f64>()
            / pixels;
        assert!(
            (mean * 65535.0 - expected).abs() < 5.0,
            "channel {c}: mean {} of 65535, expected {expected}",
            mean * 65535.0
        );
    }
}

#[test]
fn default_adjustments_leave_the_reference_stack_untouched() {
    let Some(stack) = jupiter_stack() else {
        eprintln!("jupiter-stack.tif not found in re-local/reference-own, skipped");
        return;
    };
    assert_eq!(adjust(&stack, &Adjustments::default()), stack);
}

#[test]
fn auto_levels_fill_the_range_and_the_histogram_counts_every_pixel() {
    let Some(stack) = jupiter_stack() else {
        eprintln!("jupiter-stack.tif not found in re-local/reference-own, skipped");
        return;
    };
    let h = histogram(&stack);
    let pixels = stack.width * stack.height;
    assert_eq!(h.luma.iter().sum::<u32>(), pixels);
    for channel in &h.channels {
        assert_eq!(channel.iter().sum::<u32>(), pixels);
    }

    // The brightest channel peaks at 40371 of 65535, so the stack uses well under the full range.
    let peak = |image: &Image| image.data.iter().copied().fold(0.0f32, f32::max);
    assert!(peak(&stack) < 0.7);
    let levels = auto_levels(&stack, 0.0);
    let mut stretched = stack.clone();
    stretch_levels(&mut stretched, levels);
    // The stretch is measured on the luminance, so the brightest channel is clipped to white.
    assert!(
        peak(&stretched) > 0.999,
        "peak {} after stretching to {levels:?}",
        peak(&stretched)
    );
}

#[test]
fn auto_align_follows_a_known_shift_on_real_colour_data() {
    let Some(stack) = jupiter_stack() else {
        eprintln!("jupiter-stack.tif not found in re-local/reference-own, skipped");
        return;
    };
    let stack = binned(&stack);
    // The stack has some dispersion of its own; whatever it is, moving a channel by a known
    // amount has to move the estimate by the same amount in the other direction.
    let own = estimate_rgb_shifts(&stack, 8);
    eprintln!("dispersion of the reference stack (binned 2x): {own:?}");

    let applied = RgbShifts {
        red: Shift { x: 2.0, y: -1.0 },
        green: Shift::default(),
        blue: Shift { x: -1.5, y: 0.5 },
    };
    let mut moved = stack.clone();
    shift_channels(&mut moved, &applied);
    let estimate = estimate_rgb_shifts(&moved, 8);
    let within = |got: f32, want: f32| (got - want).abs() < 0.4;
    assert!(
        within(estimate.red.x, own.red.x - applied.red.x)
            && within(estimate.red.y, own.red.y - applied.red.y)
            && within(estimate.blue.x, own.blue.x - applied.blue.x)
            && within(estimate.blue.y, own.blue.y - applied.blue.y),
        "estimate {estimate:?}, expected the own shifts {own:?} minus the applied {applied:?}"
    );
}
