//! What happens to the result of the layers before it is shown or saved: the output tone curve
//! and the treatment of colour (the *Colour* and *LRGB* checkboxes). *Do All* and the live preview both
//! finish through here, so they give the same pixels (`docs/wavelets.md`).

use crate::{Image, color::with_luminance};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// How the colour of the image is treated while sharpening.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColourMode {
    /// Every channel is sharpened on its own (*Colour* ticked).
    #[default]
    Colour,
    /// Only the luminance is sharpened, hue and saturation stay those of the stack (*Colour* and *LRGB* ticked).
    Lrgb,
    /// The image is reduced to its luminance first (*Colour* not ticked).
    Gray,
}

/// Linear part of the output tone curve.
const CURVE_LINEAR: f32 = 0.988_231;
/// Cubic part of the output tone curve.
const CURVE_CUBIC: f32 = 0.015_595;

/// The tone curve applied to the result, even with every slider at 1.0: `0.98823 * x + 0.015595 * x^3`
/// on samples in `0.0..=1.0`, the same for all channels. It darkens the midtones by about one percent.
///
/// Fitted to the 1455 distinct sample values of reference case R1 (Jupiter test recording, `docs/wavelets.md`): within 3 of 65535
/// for all of them. Only the range up to 0.61 occurs there; above it the curve is an extension, and so is
/// the clamp at 0 and 1.
pub fn output_curve(v: f32) -> f32 {
    let x = v.clamp(0.0, 1.0);
    (x * (CURVE_LINEAR + CURVE_CUBIC * x * x)).min(1.0)
}

/// Applies [`output_curve`] to every sample.
pub fn apply_output_curve(image: &mut Image) {
    for v in &mut image.data {
        *v = output_curve(*v);
    }
}

/// The image the layers are built from for a mode: the stack itself, or its luminance (borrowed or owned).
pub fn working_image(source: &Image, mode: ColourMode) -> Cow<'_, Image> {
    if uses_luma(source, mode) {
        Cow::Owned(source.to_luma())
    } else {
        Cow::Borrowed(source)
    }
}

/// Finishes a sharpened image: tone curve, and for [`ColourMode::Lrgb`] the colour of `colour` put back.
/// `colour` is the part of the stack that `worked` was made from.
pub fn finish(colour: &Image, mut worked: Image, mode: ColourMode) -> Image {
    apply_output_curve(&mut worked);
    if mode == ColourMode::Lrgb
        && let Some(recoloured) = with_luminance(colour, &worked)
    {
        return recoloured;
    }
    worked
}

/// Whether the layers are built from the luminance instead of the channels: for [`ColourMode::Lrgb`] and
/// [`ColourMode::Gray`] on an image that has colour at all.
pub fn uses_luma(source: &Image, mode: ColourMode) -> bool {
    mode != ColourMode::Colour && source.channels >= 3
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Input and output sample with neutral settings on reference case R1 (`jupiter-stack.tif` against
    /// `jupiter-wavelet-default.tif`, 16 bit), picked over the range the stack covers (`docs/wavelets.md`).
    const MEASURED: [(u16, u16); 11] = [
        (0, 0),
        (102, 101),
        (510, 505),
        (992, 980),
        (2012, 1988),
        (4987, 4929),
        (10002, 9888),
        (20003, 19797),
        (30005, 29749),
        (40007, 39766),
        (40063, 39823),
    ];

    #[test]
    fn the_curve_follows_the_reference_samples() {
        for (x, y) in MEASURED {
            let got = output_curve(f32::from(x) / 65535.0) * 65535.0;
            // The fit stays within 3 of 65535 over all 1455 values that occur in reference case R1.
            assert!(
                (got - f32::from(y)).abs() <= 3.0,
                "{x} gives {got}, the reference sample is {y}"
            );
        }
    }

    #[test]
    fn the_curve_keeps_black_and_white_and_never_falls() {
        assert_eq!(output_curve(0.0), 0.0);
        assert_eq!(output_curve(1.0), 1.0);
        assert_eq!(output_curve(-0.3), 0.0);
        assert_eq!(output_curve(1.7), 1.0);
        let mut last = 0.0;
        for i in 0..=1000 {
            let y = output_curve(i as f32 / 1000.0);
            assert!(y >= last, "falls at {i}");
            last = y;
        }
    }

    fn colour_image() -> Image {
        let mut img = Image::new(4, 1, 3);
        img.data = vec![
            0.8, 0.4, 0.2, //
            0.1, 0.2, 0.3, //
            0.0, 0.0, 0.0, //
            0.5, 0.5, 0.5,
        ];
        img
    }

    #[test]
    fn gray_and_lrgb_build_their_layers_from_the_luminance() {
        let img = colour_image();
        assert!(!uses_luma(&img, ColourMode::Colour));
        assert!(uses_luma(&img, ColourMode::Lrgb));
        assert!(uses_luma(&img, ColourMode::Gray));
        assert_eq!(working_image(&img, ColourMode::Colour).channels, 3);
        assert_eq!(working_image(&img, ColourMode::Gray).channels, 1);
        // A gray image has no luminance to split off.
        let mono = img.to_luma();
        assert!(!uses_luma(&mono, ColourMode::Lrgb));
        assert_eq!(working_image(&mono, ColourMode::Gray).channels, 1);
    }

    #[test]
    fn lrgb_puts_the_colour_back_onto_the_sharpened_luminance() {
        let img = colour_image();
        let luma = img.to_luma();
        // Pretend the sharpening made every pixel 20 percent brighter.
        let mut worked = luma.clone();
        for v in &mut worked.data {
            *v *= 1.2;
        }
        let out = finish(&img, worked.clone(), ColourMode::Lrgb);
        assert_eq!(out.channels, 3);
        // Hue stays: the channel ratios of the stack survive in the first pixel.
        let (r, g, b) = (out.data[0], out.data[1], out.data[2]);
        assert!((r / g - 0.8 / 0.4).abs() < 1e-3 && (g / b - 0.4 / 0.2).abs() < 1e-3);
        // The luminance is the one that was given, after the tone curve.
        let new_luma = out.to_luma();
        let mut toned = worked;
        apply_output_curve(&mut toned);
        assert!((new_luma.data[1] - toned.data[1]).abs() < 1e-3);
        // Without a colour image to go back to, the luminance stays as it is.
        let alone = finish(&luma, luma.clone(), ColourMode::Lrgb);
        assert_eq!(alone.channels, 1);
    }

    #[test]
    fn colour_and_gray_only_apply_the_curve() {
        let img = colour_image();
        for mode in [ColourMode::Colour, ColourMode::Gray] {
            let out = finish(&img, img.clone(), mode);
            for (o, i) in out.data.iter().zip(&img.data) {
                assert_eq!(*o, output_curve(*i));
            }
        }
    }
}
