//! GUI-free image processing for the the reference program clone.

/// Interleaved floating point image with samples normalised to `0.0..=1.0`.
#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    pub data: Vec<f32>,
}

impl Image {
    /// Creates a black image.
    pub fn new(width: u32, height: u32, channels: u8) -> Self {
        let len = width as usize * height as usize * channels as usize;
        Self {
            width,
            height,
            channels,
            data: vec![0.0; len],
        }
    }

    /// Converts to 8-bit RGBA for display. Single channel images become gray; with
    /// three or more channels the first three are used as R, G and B.
    pub fn to_rgba8(&self) -> Vec<u8> {
        let to_u8 = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        let channels = usize::from(self.channels).max(1);
        let mut out = Vec::with_capacity(self.data.len() / channels * 4);
        for px in self.data.chunks_exact(channels) {
            let (r, g, b) = if channels >= 3 {
                (px[0], px[1], px[2])
            } else {
                (px[0], px[0], px[0])
            };
            out.extend([to_u8(r), to_u8(g), to_u8(b), 255]);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_image_is_black_and_sized() {
        let img = Image::new(4, 3, 3);
        assert_eq!(img.data.len(), 36);
        assert!(img.data.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn mono_becomes_gray_with_opaque_alpha() {
        let img = Image {
            width: 2,
            height: 1,
            channels: 1,
            data: vec![0.0, 1.0],
        };
        assert_eq!(img.to_rgba8(), vec![0, 0, 0, 255, 255, 255, 255, 255]);
    }

    #[test]
    fn rgb_is_scaled_and_rounded() {
        let img = Image {
            width: 1,
            height: 1,
            channels: 3,
            data: vec![1.0, 0.5, 0.0],
        };
        assert_eq!(img.to_rgba8(), vec![255, 128, 0, 255]);
    }

    #[test]
    fn out_of_range_samples_are_clamped() {
        let img = Image {
            width: 2,
            height: 1,
            channels: 1,
            data: vec![-0.5, 2.0],
        };
        assert_eq!(img.to_rgba8(), vec![0, 0, 0, 255, 255, 255, 255, 255]);
    }
}
