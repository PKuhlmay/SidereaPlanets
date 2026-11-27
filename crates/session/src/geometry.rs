//! The preview of *Flip and Rotate*: the window shows the oriented image, so it asks for it as RGBA.

use crate::{RgbaImage, Session, SessionError, color::image_from_rgba};
use planets_core::geometry::Orientation;

impl Session {
    /// The picture `rgba` (8-bit, `width` x `height`) flipped and rotated as `orientation` says; the canvas of a free angle grows to
    /// hold it, so the result can be bigger. The window shows it instead of the plain image.
    pub fn orient_preview(
        &self,
        width: u32,
        height: u32,
        rgba: &[u8],
        orientation: &Orientation,
    ) -> Result<RgbaImage, SessionError> {
        let image = image_from_rgba(width, height, rgba)?;
        if orientation.is_neutral() {
            return Ok(RgbaImage::new(width, height, rgba.to_vec()));
        }
        let oriented = self.compute.install(|| image.oriented(orientation));
        Ok(RgbaImage::from_image(&oriented))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgba(width: u32, height: u32) -> Vec<u8> {
        (0..width * height)
            .flat_map(|i| [i as u8, 100 + i as u8, 200, 255])
            .collect()
    }

    fn turn(angle: f32) -> Orientation {
        Orientation {
            angle,
            ..Orientation::default()
        }
    }

    #[test]
    fn a_quarter_turn_moves_the_rgba_pixels() {
        let session = Session::new().unwrap();
        let input = rgba(3, 2);
        let out = session.orient_preview(3, 2, &input, &turn(90.0)).unwrap();
        assert_eq!((out.width(), out.height()), (2, 3));
        // 0 1 2 / 3 4 5 turned clockwise: 3 0 / 4 1 / 5 2.
        let first: Vec<u8> = out.rgba().chunks(4).map(|p| p[0]).collect();
        assert_eq!(first, vec![3, 0, 4, 1, 5, 2]);
        assert!(out.rgba().chunks(4).all(|p| p[3] == 255));
    }

    #[test]
    fn no_orientation_gives_the_same_bytes() {
        let session = Session::new().unwrap();
        let input = rgba(5, 4);
        let out = session
            .orient_preview(5, 4, &input, &Orientation::default())
            .unwrap();
        assert_eq!((out.width(), out.height()), (5, 4));
        assert_eq!(out.rgba(), &input[..]);
    }

    #[test]
    fn a_free_angle_grows_the_canvas_with_black_corners() {
        let session = Session::new().unwrap();
        let input: Vec<u8> = std::iter::repeat_n([255u8, 255, 255, 255], 30 * 20)
            .flatten()
            .collect();
        let out = session.orient_preview(30, 20, &input, &turn(30.0)).unwrap();
        assert_eq!((out.width(), out.height()), turn(30.0).output_size(30, 20));
        assert_eq!(&out.rgba()[..4], &[0, 0, 0, 255]);
    }

    #[test]
    fn a_wrong_byte_count_is_refused() {
        let session = Session::new().unwrap();
        let error = session
            .orient_preview(3, 2, &[0; 10], &turn(90.0))
            .unwrap_err();
        assert!(
            matches!(error, SessionError::BadImageSize { .. }),
            "{error}"
        );
    }
}
