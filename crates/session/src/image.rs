//! Images that leave the session: 8-bit RGBA, as the windows display them.

use planets_core::Image;

/// An image as `width * height * 4` bytes, red, green, blue, alpha, row by row from the top left.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbaImage {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl RgbaImage {
    pub(crate) fn new(width: u32, height: u32, rgba: Vec<u8>) -> Self {
        debug_assert_eq!(rgba.len(), width as usize * height as usize * 4);
        Self {
            width,
            height,
            rgba,
        }
    }

    pub(crate) fn from_image(image: &Image) -> Self {
        Self::new(image.width, image.height, image.to_rgba8())
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }

    pub fn into_rgba(self) -> Vec<u8> {
        self.rgba
    }
}
