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
}
