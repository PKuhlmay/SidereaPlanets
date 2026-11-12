//! Cropping: the rectangle that *Save image* cuts out of the finished image (MAC-016).

use crate::Image;

/// The part of an image to keep, in pixels from the top left corner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CropRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl CropRect {
    /// The part of the rectangle that lies inside an image of `width` x `height`; `None` if nothing is left.
    pub fn clamp(self, width: u32, height: u32) -> Option<CropRect> {
        let x1 = self.x.saturating_add(self.width).min(width);
        let y1 = self.y.saturating_add(self.height).min(height);
        (self.x < x1 && self.y < y1).then(|| CropRect {
            x: self.x,
            y: self.y,
            width: x1 - self.x,
            height: y1 - self.y,
        })
    }

    /// Whether the rectangle covers the whole image, so cropping changes nothing.
    pub fn is_full(self, width: u32, height: u32) -> bool {
        self.x == 0 && self.y == 0 && self.width >= width && self.height >= height
    }
}

impl Image {
    /// The pixels inside `rect` as a new image; the rectangle is clamped to the image first. `None` if no pixel is inside.
    pub fn cropped(&self, rect: CropRect) -> Option<Image> {
        let rect = rect.clamp(self.width, self.height)?;
        let channels = usize::from(self.channels).max(1);
        let row_len = rect.width as usize * channels;
        let mut data = Vec::with_capacity(row_len * rect.height as usize);
        for row in rect.y..rect.y + rect.height {
            let start = (row as usize * self.width as usize + rect.x as usize) * channels;
            data.extend_from_slice(&self.data[start..start + row_len]);
        }
        Some(Image {
            width: rect.width,
            height: rect.height,
            channels: self.channels,
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: u32, y: u32, width: u32, height: u32) -> CropRect {
        CropRect {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn a_rectangle_inside_the_image_stays_as_it_is() {
        assert_eq!(rect(2, 3, 4, 5).clamp(10, 10), Some(rect(2, 3, 4, 5)));
    }

    #[test]
    fn a_rectangle_over_the_edge_is_cut_at_the_edge() {
        assert_eq!(rect(8, 6, 10, 10).clamp(10, 10), Some(rect(8, 6, 2, 4)));
        // A huge size must not overflow.
        assert_eq!(
            rect(1, 1, u32::MAX, u32::MAX).clamp(10, 8),
            Some(rect(1, 1, 9, 7))
        );
    }

    #[test]
    fn a_rectangle_without_pixels_in_the_image_is_none() {
        assert_eq!(rect(10, 0, 5, 5).clamp(10, 10), None);
        assert_eq!(rect(0, 12, 5, 5).clamp(10, 10), None);
        assert_eq!(rect(3, 3, 0, 5).clamp(10, 10), None);
        assert_eq!(rect(3, 3, 5, 0).clamp(10, 10), None);
    }

    #[test]
    fn the_full_image_is_recognised() {
        assert!(rect(0, 0, 10, 8).is_full(10, 8));
        assert!(rect(0, 0, 99, 99).is_full(10, 8));
        assert!(!rect(1, 0, 9, 8).is_full(10, 8));
        assert!(!rect(0, 0, 10, 7).is_full(10, 8));
    }

    fn numbered(width: u32, height: u32, channels: u8) -> Image {
        let mut image = Image::new(width, height, channels);
        for (i, v) in image.data.iter_mut().enumerate() {
            *v = i as f32;
        }
        image
    }

    #[test]
    fn the_cropped_image_has_the_size_of_the_rectangle() {
        let image = numbered(10, 8, 3);
        let out = image.cropped(rect(2, 1, 5, 4)).unwrap();
        assert_eq!((out.width, out.height, out.channels), (5, 4, 3));
        assert_eq!(out.data.len(), 5 * 4 * 3);
    }

    #[test]
    fn the_cropped_pixels_are_the_pixels_of_the_region() {
        for channels in [1u8, 3] {
            let image = numbered(10, 8, channels);
            let c = usize::from(channels);
            let out = image.cropped(rect(2, 1, 5, 4)).unwrap();
            for row in 0..4usize {
                for col in 0..5usize {
                    for ch in 0..c {
                        let from = ((1 + row) * 10 + 2 + col) * c + ch;
                        let to = (row * 5 + col) * c + ch;
                        assert_eq!(
                            out.data[to], image.data[from],
                            "row {row} col {col} ch {ch}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn cropping_to_nothing_gives_none() {
        let image = numbered(4, 4, 1);
        assert_eq!(image.cropped(rect(4, 0, 2, 2)), None);
    }
}
