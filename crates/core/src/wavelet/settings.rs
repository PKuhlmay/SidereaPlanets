//! Settings of the Wavelet tab: what the left panel *Wavelets* holds (`docs/ui.md`).

use serde::{Deserialize, Serialize};

/// Number of wavelet layers of the original.
pub const LAYERS: usize = 6;

/// Reference value of *Sharpen*: with it a layer acts exactly as its slider says (`docs/wavelets.md`).
pub const DEFAULT_SHARPEN: f32 = 0.1;

/// How the width of the layers grows (*Waveletscheme*).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Scheme {
    /// Dyadic (2^n): the width doubles from layer to layer.
    Dyadic,
    /// Linear (default): the width grows by a fixed step.
    Linear,
}

/// Smoothing kernel the layers are built from (*Wavelet filter*).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Filter {
    Default,
    /// Default of the original.
    Gaussian,
}

/// One of the six layers: the row with the checkbox, *Denoise*, *Sharpen* and the slider.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerSettings {
    /// The checkbox. A layer that is off stays as it is.
    pub enabled: bool,
    /// Slider value shown on the *Preview* button (position ÷ 10), `-5.0..=100.0`; 1.0 leaves the layer unchanged.
    pub gain: f32,
    /// Field *Denoise*, `0.0` = off.
    pub denoise: f32,
    /// Field *Sharpen*, `0.1` = reference value.
    pub sharpen: f32,
}

impl Default for LayerSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            gain: 1.0,
            denoise: 0.0,
            sharpen: DEFAULT_SHARPEN,
        }
    }
}

impl LayerSettings {
    /// Smallest and largest value of the slider.
    pub const GAIN_RANGE: (f32, f32) = (-5.0, 100.0);

    /// Whether the layer is left as it is, so that it costs nothing.
    pub fn is_neutral(&self) -> bool {
        !self.enabled || (self.gain == 1.0 && self.denoise <= 0.0)
    }

    /// How much of the layer's deviation from 1.0 is applied: the slider, scaled by *Sharpen*.
    pub fn effective_gain(&self) -> f32 {
        1.0 + (self.gain - 1.0) * (self.sharpen / DEFAULT_SHARPEN)
    }
}

/// The part of the settings that decides what the layers look like. Everything else only
/// recombines them, so the layers of a preview can be kept while a slider moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Structure {
    pub scheme: Scheme,
    pub filter: Filter,
    /// *Initial Layer*, at least 1.
    pub initial: u32,
    /// *Step Increment*.
    pub step: u32,
}

impl Structure {
    /// Distance in pixels between the taps of the smoothing kernel in layer `layer` (0-based).
    pub fn spacing(&self, layer: usize) -> usize {
        let n = layer as u32;
        let initial = self.initial.max(1);
        match self.scheme {
            Scheme::Linear => (initial + n * (1 + self.step)) as usize,
            Scheme::Dyadic => (initial as usize) << n.min(24),
        }
    }

    /// Taps of the one-dimensional smoothing kernel, symmetric, summing to 1.
    pub fn taps(&self) -> &'static [f32] {
        match self.filter {
            Filter::Gaussian => &[0.05, 0.9, 0.05],
            Filter::Default => &[1.0 / 16.0, 4.0 / 16.0, 6.0 / 16.0, 4.0 / 16.0, 1.0 / 16.0],
        }
    }

    /// How far (in pixels) a result pixel depends on pixels around it, through the first `layers` layers.
    pub fn margin(&self, layers: usize) -> usize {
        let reach = self.taps().len() / 2;
        (0..layers).map(|l| self.spacing(l) * reach).sum()
    }
}

/// Everything the left panel *Wavelets* holds. Serialises as the JSON the UI sends and receives;
/// the `.rwv` file of the original is handled in [`super::scheme`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WaveletSettings {
    #[serde(flatten)]
    pub structure: Structure,
    pub layers: [LayerSettings; LAYERS],
}

impl Default for WaveletSettings {
    fn default() -> Self {
        Self {
            structure: Structure {
                scheme: Scheme::Linear,
                filter: Filter::Gaussian,
                initial: 1,
                step: 0,
            },
            layers: [LayerSettings::default(); LAYERS],
        }
    }
}

impl WaveletSettings {
    /// *Reset Wavelets*.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Whether the settings leave the image unchanged.
    pub fn is_neutral(&self) -> bool {
        self.layers.iter().all(LayerSettings::is_neutral)
    }

    /// Number of layers that have to be built: up to and including the last one that changes something.
    pub fn layers_needed(&self) -> usize {
        self.layers
            .iter()
            .rposition(|l| !l.is_neutral())
            .map_or(0, |i| i + 1)
    }

    /// Brings every value into the range the clone supports (the original accepts anything typed,
    /// see `docs/offene-fragen.md`, question 1): gain `-5..=100`, *Denoise* and *Sharpen* `0..=100`,
    /// *Initial Layer* `1..=64`, *Step Increment* `0..=64`, no NaN.
    pub fn clamped(mut self) -> Self {
        let (lo, hi) = LayerSettings::GAIN_RANGE;
        for l in &mut self.layers {
            l.gain = if l.gain.is_nan() {
                1.0
            } else {
                l.gain.clamp(lo, hi)
            };
            l.denoise = if l.denoise.is_nan() {
                0.0
            } else {
                l.denoise.clamp(0.0, 100.0)
            };
            l.sharpen = if l.sharpen.is_nan() {
                DEFAULT_SHARPEN
            } else {
                l.sharpen.clamp(0.0, 100.0)
            };
        }
        self.structure.initial = self.structure.initial.clamp(1, 64);
        self.structure.step = self.structure.step.min(64);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_follow_the_original() {
        let s = WaveletSettings::default();
        assert_eq!(s.structure.scheme, Scheme::Linear);
        assert_eq!(s.structure.filter, Filter::Gaussian);
        assert_eq!((s.structure.initial, s.structure.step), (1, 0));
        assert!(
            s.layers
                .iter()
                .all(|l| l.enabled && l.gain == 1.0 && l.denoise == 0.0)
        );
        assert!(s.layers.iter().all(|l| (l.sharpen - 0.1).abs() < 1e-6));
        assert!(s.is_neutral());
        assert_eq!(s.layers_needed(), 0);
    }

    #[test]
    fn layers_needed_stops_at_the_last_active_layer() {
        let mut s = WaveletSettings::default();
        s.layers[2].gain = 3.0;
        s.layers[4].enabled = false;
        s.layers[4].gain = 9.0;
        assert_eq!(s.layers_needed(), 3);
        s.layers[5].denoise = 0.5;
        assert_eq!(s.layers_needed(), 6);
    }

    #[test]
    fn spacing_grows_linearly_or_doubles() {
        let linear = |initial, step| Structure {
            scheme: Scheme::Linear,
            filter: Filter::Gaussian,
            initial,
            step,
        };
        let spacings = |s: Structure| (0..4).map(|l| s.spacing(l)).collect::<Vec<_>>();
        assert_eq!(spacings(linear(1, 0)), [1, 2, 3, 4]);
        assert_eq!(spacings(linear(2, 1)), [2, 4, 6, 8]);
        let dyadic = Structure {
            scheme: Scheme::Dyadic,
            ..linear(1, 0)
        };
        assert_eq!(spacings(dyadic), [1, 2, 4, 8]);
    }

    #[test]
    fn margin_adds_up_the_reach_of_all_layers() {
        let s = WaveletSettings::default().structure;
        assert_eq!(s.margin(3), 1 + 2 + 3);
        let b3 = Structure {
            filter: Filter::Default,
            ..s
        };
        assert_eq!(b3.margin(3), 2 * (1 + 2 + 3));
    }

    #[test]
    fn kernels_sum_to_one() {
        for filter in [Filter::Default, Filter::Gaussian] {
            let s = Structure {
                filter,
                ..WaveletSettings::default().structure
            };
            assert!((s.taps().iter().sum::<f32>() - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn sharpen_scales_the_slider_deviation() {
        let layer = LayerSettings {
            gain: 3.0,
            sharpen: 0.2,
            ..LayerSettings::default()
        };
        assert!((layer.effective_gain() - 5.0).abs() < 1e-5);
        let reference = LayerSettings {
            gain: 3.0,
            ..LayerSettings::default()
        };
        assert!((reference.effective_gain() - 3.0).abs() < 1e-5);
    }

    #[test]
    fn clamped_limits_the_values() {
        let mut s = WaveletSettings::default();
        s.layers[0].gain = 999.0;
        s.layers[1].gain = -50.0;
        s.layers[2].denoise = -1.0;
        s.layers[3].sharpen = f32::NAN;
        s.structure.initial = 0;
        let s = s.clamped();
        assert_eq!(s.layers[0].gain, 100.0);
        assert_eq!(s.layers[1].gain, -5.0);
        assert_eq!(s.layers[2].denoise, 0.0);
        assert_eq!(s.layers[3].sharpen, DEFAULT_SHARPEN);
        assert_eq!(s.structure.initial, 1);
    }

    #[test]
    fn json_roundtrip_keeps_everything() {
        let mut s = WaveletSettings::default();
        s.structure.scheme = Scheme::Dyadic;
        s.layers[1].gain = 7.5;
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("\"scheme\":\"dyadic\""), "{json}");
        assert_eq!(serde_json::from_str::<WaveletSettings>(&json).unwrap(), s);
    }
}
