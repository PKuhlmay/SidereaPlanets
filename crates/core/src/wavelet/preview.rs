//! Live preview of the Wavelet tab: only the visible area is processed and converted to RGBA
//! (measures from `docs/gui-toolkit.md`, section *Maßnahmen für die Live-Vorschau*).

use super::{ColourMode, Decomposition, LAYERS, WaveletSettings, finish, uses_luma};
use crate::Image;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A rectangle of pixels in image coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Region {
    pub fn full(width: u32, height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
        }
    }

    /// A square of `size` pixels in the middle of an image, cut to the image (the *Processing Area*). A
    /// half pixel of offset rounds up, like `processingArea` in `app/src/logic/viewport.ts`, so that the
    /// overlay and the rendered area coincide.
    pub fn centered(size: u32, width: u32, height: u32) -> Self {
        let w = size.min(width);
        let h = size.min(height);
        Self {
            x: (width - w).div_ceil(2),
            y: (height - h).div_ceil(2),
            width: w,
            height: h,
        }
    }

    /// The part of the region that lies inside an image of the given size. Never empty for a non-empty image.
    pub fn clamped(self, width: u32, height: u32) -> Self {
        let x = self.x.min(width.saturating_sub(1));
        let y = self.y.min(height.saturating_sub(1));
        Self {
            x,
            y,
            width: self.width.min(width - x).max(1.min(width)),
            height: self.height.min(height - y).max(1.min(height)),
        }
    }

    /// The region grown by `margin` pixels on every side, cut to the image.
    pub fn expanded(self, margin: u32, width: u32, height: u32) -> Self {
        let x0 = self.x.saturating_sub(margin);
        let y0 = self.y.saturating_sub(margin);
        let x1 = (self.x + self.width).saturating_add(margin).min(width);
        let y1 = (self.y + self.height).saturating_add(margin).min(height);
        Self {
            x: x0,
            y: y0,
            width: x1 - x0,
            height: y1 - y0,
        }
    }

    pub fn contains(&self, other: &Region) -> bool {
        self.x <= other.x
            && self.y <= other.y
            && self.x + self.width >= other.x + other.width
            && self.y + self.height >= other.y + other.height
    }
}

/// Converts a region of the image to 8-bit RGBA, row by row in parallel. Single channel images become
/// gray, otherwise the first three channels are R, G and B. The region must lie inside the image
/// (see [`Region::clamped`]).
pub fn rgba_region(image: &Image, region: Region) -> Vec<u8> {
    let channels = usize::from(image.channels).max(1);
    let width = region.width as usize;
    let stride = image.width as usize * channels;
    let to_u8 = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    let mut out = vec![0u8; width * region.height as usize * 4];
    out.par_chunks_mut(width * 4)
        .enumerate()
        .for_each(|(row, dst)| {
            let start = (region.y as usize + row) * stride + region.x as usize * channels;
            let src = &image.data[start..start + width * channels];
            for (d, px) in dst
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip(src.chunks_exact(channels))
            {
                let (r, g, b) = if channels >= 3 {
                    (px[0], px[1], px[2])
                } else {
                    (px[0], px[0], px[0])
                };
                *d = [to_u8(r), to_u8(g), to_u8(b), 255];
            }
        });
    out
}

/// Copies a region out of an image.
fn crop(image: &Image, region: Region) -> Image {
    let channels = usize::from(image.channels).max(1);
    let stride = image.width as usize * channels;
    let row_len = region.width as usize * channels;
    let mut data = Vec::with_capacity(row_len * region.height as usize);
    for y in region.y..region.y + region.height {
        let start = y as usize * stride + region.x as usize * channels;
        data.extend_from_slice(&image.data[start..start + row_len]);
    }
    Image {
        width: region.width,
        height: region.height,
        channels: image.channels,
        data,
    }
}

/// An RGBA preview and the region of the image it shows.
pub struct Preview {
    pub region: Region,
    pub rgba: Vec<u8>,
    /// The image has no colour (fewer than three channels), so red, green and blue of every pixel in `rgba` are the same.
    pub mono: bool,
}

struct Cached {
    outer: Region,
    /// Whether the layers come from the luminance of the source (see [`uses_luma`]).
    luma: bool,
    decomposition: Decomposition,
}

/// Renders previews of one source image. The layers of the area are kept between calls, so moving a
/// slider recombines them instead of decomposing again.
pub struct Previewer {
    source: Arc<Image>,
    /// The luminance of the source, made when [`ColourMode::Lrgb`] or [`ColourMode::Gray`] first needs it.
    luma: Option<Arc<Image>>,
    cache: Option<Cached>,
}

impl Previewer {
    pub fn new(source: Arc<Image>) -> Self {
        Self {
            source,
            luma: None,
            cache: None,
        }
    }

    pub fn source(&self) -> &Arc<Image> {
        &self.source
    }

    /// The unprocessed image in the region (the *Before* view).
    pub fn before(&self, region: Region) -> Preview {
        let region = region.clamped(self.source.width, self.source.height);
        Preview {
            region,
            rgba: rgba_region(&self.source, region),
            mono: self.source.channels < 3,
        }
    }

    /// The image in the region with the settings applied, as *Do All* ([`super::do_all`]) shows it: with the
    /// output tone curve and the colour treatment of `mode`. The region is cut to the image; the
    /// returned [`Preview::region`] says what was rendered.
    ///
    /// The area around the region is processed too, as far as the layers reach, so that the result is
    /// the same as cutting the region out of [`super::do_all`]. Only the noise level that *Denoise*
    /// relies on is estimated from the processed area instead of from the whole image.
    pub fn render(
        &mut self,
        settings: &WaveletSettings,
        mode: ColourMode,
        region: Region,
    ) -> Preview {
        let (width, height) = (self.source.width, self.source.height);
        let region = region.clamped(width, height);
        let settings = settings.clone().clamped();
        let luma = uses_luma(&self.source, mode);
        let working = if luma {
            Arc::clone(
                self.luma
                    .get_or_insert_with(|| Arc::new(self.source.to_luma())),
            )
        } else {
            Arc::clone(&self.source)
        };

        let worked = if settings.is_neutral() {
            crop(&working, region)
        } else {
            let margin = settings.structure.margin(LAYERS) as u32;
            let wanted = region.expanded(margin, width, height);
            let reusable = self.cache.as_ref().is_some_and(|c| {
                c.luma == luma
                    && c.decomposition.structure() == &settings.structure
                    && c.outer.contains(&wanted)
            });
            if !reusable {
                self.cache = Some(Cached {
                    outer: wanted,
                    luma,
                    decomposition: Decomposition::new(&crop(&working, wanted), settings.structure),
                });
            }
            let cached = self.cache.as_ref().expect("cache was filled above");
            let composed = cached.decomposition.compose(&settings);
            let inner = Region {
                x: region.x - cached.outer.x,
                y: region.y - cached.outer.y,
                ..region
            };
            crop(&composed, inner)
        };
        let image = finish(&crop(&self.source, region), worked, mode);
        Preview {
            region,
            rgba: rgba_region(&image, Region::full(region.width, region.height)),
            mono: image.channels < 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wavelet::{apply_output_curve, do_all};

    fn test_image(width: u32, height: u32) -> Arc<Image> {
        let mut img = Image::new(width, height, 3);
        for y in 0..height as usize {
            for x in 0..width as usize {
                for c in 0..3 {
                    img.data[(y * width as usize + x) * 3 + c] =
                        0.5 + 0.3 * ((x as f32 * 0.8 + c as f32).sin() * (y as f32 * 0.6).cos());
                }
            }
        }
        Arc::new(img)
    }

    fn boosted() -> WaveletSettings {
        let mut s = WaveletSettings::default();
        s.layers[0].gain = 5.0;
        s.layers[2].gain = 3.0;
        s
    }

    #[test]
    fn region_is_cut_to_the_image() {
        let r = Region {
            x: 90,
            y: 5,
            width: 50,
            height: 500,
        }
        .clamped(100, 80);
        assert_eq!(
            r,
            Region {
                x: 90,
                y: 5,
                width: 10,
                height: 75
            }
        );
        let outside = Region {
            x: 500,
            y: 500,
            width: 10,
            height: 10,
        }
        .clamped(100, 80);
        assert_eq!(
            outside,
            Region {
                x: 99,
                y: 79,
                width: 1,
                height: 1
            }
        );
    }

    #[test]
    fn centered_area_is_in_the_middle_and_cut_to_small_images() {
        assert_eq!(
            Region::centered(512, 1024, 680),
            Region {
                x: 256,
                y: 84,
                width: 512,
                height: 512
            }
        );
        assert_eq!(
            Region::centered(512, 1023, 681),
            Region {
                x: 256,
                y: 85,
                width: 512,
                height: 512
            }
        );
        assert_eq!(Region::centered(512, 300, 200), Region::full(300, 200));
    }

    #[test]
    fn expanded_region_stops_at_the_image_edge() {
        let r = Region {
            x: 3,
            y: 10,
            width: 20,
            height: 20,
        }
        .expanded(8, 40, 100);
        assert_eq!(
            r,
            Region {
                x: 0,
                y: 2,
                width: 31,
                height: 36
            }
        );
        assert!(r.contains(&Region {
            x: 3,
            y: 10,
            width: 20,
            height: 20
        }));
        assert!(!r.contains(&Region::full(40, 100)));
    }

    #[test]
    fn rgba_region_matches_the_whole_image_conversion() {
        let img = test_image(30, 20);
        let all = img.to_rgba8();
        assert_eq!(rgba_region(&img, Region::full(30, 20)), all);
        let part = rgba_region(
            &img,
            Region {
                x: 4,
                y: 3,
                width: 5,
                height: 2,
            },
        );
        for row in 0..2usize {
            let from = ((3 + row) * 30 + 4) * 4;
            assert_eq!(part[row * 20..(row + 1) * 20], all[from..from + 20]);
        }
    }

    #[test]
    fn neutral_settings_show_the_source_with_the_tone_curve() {
        let img = test_image(64, 48);
        let region = Region {
            x: 10,
            y: 8,
            width: 30,
            height: 20,
        };
        let mut previewer = Previewer::new(img.clone());
        let preview = previewer.render(&WaveletSettings::default(), ColourMode::Colour, region);
        assert_eq!(preview.region, region);
        // The output tone curve darkens the image even when nothing is sharpened ...
        assert_ne!(preview.rgba, previewer.before(region).rgba);
        // ... and the preview shows exactly that.
        let mut toned = (*img).clone();
        apply_output_curve(&mut toned);
        assert_eq!(preview.rgba, rgba_region(&toned, region));
    }

    #[test]
    fn preview_equals_the_region_of_the_full_result() {
        let img = test_image(120, 90);
        let settings = boosted();
        let full = rgba_region(
            &do_all(&img, &settings, ColourMode::Colour),
            Region::full(120, 90),
        );
        // One region inside, one touching the image corner.
        for region in [
            Region {
                x: 40,
                y: 30,
                width: 30,
                height: 25,
            },
            Region {
                x: 0,
                y: 0,
                width: 25,
                height: 20,
            },
        ] {
            let preview = Previewer::new(img.clone()).render(&settings, ColourMode::Colour, region);
            assert_eq!(preview.region, region);
            for row in 0..region.height as usize {
                let from = ((region.y as usize + row) * 120 + region.x as usize) * 4;
                let len = region.width as usize * 4;
                assert_eq!(
                    preview.rgba[row * len..(row + 1) * len],
                    full[from..from + len],
                    "row {row} of {region:?}"
                );
            }
        }
    }

    #[test]
    fn moving_a_slider_reuses_the_layers() {
        let img = test_image(100, 80);
        let region = Region {
            x: 20,
            y: 20,
            width: 40,
            height: 30,
        };
        let mut previewer = Previewer::new(img.clone());
        let mut settings = boosted();
        previewer.render(&settings, ColourMode::Colour, region);
        let outer = previewer.cache.as_ref().unwrap().outer;
        settings.layers[0].gain = 8.0;
        let preview = previewer.render(&settings, ColourMode::Colour, region);
        assert_eq!(previewer.cache.as_ref().unwrap().outer, outer);
        let expected = Previewer::new(img).render(&settings, ColourMode::Colour, region);
        assert_eq!(preview.rgba, expected.rgba);
    }

    #[test]
    fn another_structure_or_a_region_outside_the_cache_rebuilds_it() {
        let img = test_image(400, 300);
        let region = Region {
            x: 20,
            y: 20,
            width: 20,
            height: 20,
        };
        let mut previewer = Previewer::new(img);
        let mut settings = boosted();
        previewer.render(&settings, ColourMode::Colour, region);
        let first = previewer.cache.as_ref().unwrap().outer;

        settings.structure.scheme = crate::wavelet::Scheme::Dyadic;
        previewer.render(&settings, ColourMode::Colour, region);
        let second = previewer.cache.as_ref().unwrap().outer;
        assert!(second.contains(&first), "dyadic reaches further");
        assert_ne!(second, first);

        previewer.render(
            &settings,
            ColourMode::Colour,
            Region {
                x: 60,
                y: 50,
                ..region
            },
        );
        assert_ne!(previewer.cache.as_ref().unwrap().outer, second);
    }

    #[test]
    fn a_preview_without_colour_says_so() {
        let region = Region {
            x: 4,
            y: 4,
            width: 10,
            height: 8,
        };
        let colour = test_image(40, 30);
        let mono = Arc::new(colour.to_luma());
        let settings = boosted();
        let mut previewer = Previewer::new(colour);
        assert!(!previewer.render(&settings, ColourMode::Colour, region).mono);
        assert!(!previewer.render(&settings, ColourMode::Lrgb, region).mono);
        // Gray works on the luminance, so the result has no colour whatever the source had.
        let gray = previewer.render(&settings, ColourMode::Gray, region);
        assert!(gray.mono);
        assert!(
            gray.rgba
                .as_chunks::<4>()
                .0
                .iter()
                .all(|p| p[0] == p[1] && p[1] == p[2])
        );
        assert!(!previewer.before(region).mono);
        let mut previewer = Previewer::new(mono);
        assert!(previewer.render(&settings, ColourMode::Colour, region).mono);
        assert!(previewer.before(region).mono);
    }

    #[test]
    fn preview_equals_do_all_in_every_colour_mode() {
        let img = test_image(120, 90);
        let region = Region {
            x: 30,
            y: 20,
            width: 40,
            height: 30,
        };
        for settings in [boosted(), WaveletSettings::default()] {
            let mut previewer = Previewer::new(img.clone());
            // Switching between the modes on one previewer must not serve layers of the wrong kind.
            for mode in [
                ColourMode::Colour,
                ColourMode::Gray,
                ColourMode::Lrgb,
                ColourMode::Colour,
            ] {
                let full = rgba_region(&do_all(&img, &settings, mode), Region::full(120, 90));
                let preview = previewer.render(&settings, mode, region);
                for row in 0..region.height as usize {
                    let from = ((region.y as usize + row) * 120 + region.x as usize) * 4;
                    let len = region.width as usize * 4;
                    assert_eq!(
                        preview.rgba[row * len..(row + 1) * len],
                        full[from..from + len],
                        "row {row} in {mode:?}"
                    );
                }
            }
        }
    }
}
