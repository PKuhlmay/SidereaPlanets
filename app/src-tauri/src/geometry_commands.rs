//! Flip and Rotate and Resize Image: the values as the window sends them, turned into the types of the core, and the command
//! that turns a picture for the image area. The geometry types of the core have no serde, so the shell has its own (camelCase, the
//! shape of `app/src/logic/geometry.ts`).

use crate::{CommandResult, pack};
use planets_session::{Geometry, Orientation, Resampling, Resize, ResizeTarget, Session};
use serde::Deserialize;
use tauri::{State, ipc::Response};

/// How pixels between pixels are looked up, as the cards name it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Method {
    Nearest,
    Bilinear,
    Bicubic,
    Lanczos3,
}

impl From<Method> for Resampling {
    fn from(m: Method) -> Self {
        match m {
            Method::Nearest => Resampling::Nearest,
            Method::Bilinear => Resampling::Bilinear,
            Method::Bicubic => Resampling::Bicubic,
            Method::Lanczos3 => Resampling::Lanczos3,
        }
    }
}

/// The card Flip and Rotate.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrientationArea {
    flip_horizontal: bool,
    flip_vertical: bool,
    angle: f32,
    crop_to_fit: bool,
    method: Method,
}

impl From<OrientationArea> for Orientation {
    fn from(o: OrientationArea) -> Self {
        Orientation {
            flip_horizontal: o.flip_horizontal,
            flip_vertical: o.flip_vertical,
            angle: o.angle,
            crop_to_fit: o.crop_to_fit,
            method: o.method.into(),
        }
    }
}

/// A side in pixels; `None` follows the other side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Pixels {
    width: Option<u32>,
    height: Option<u32>,
}

/// The size Resize Image asks for: `{ "percent": 50 }` or `{ "pixels": { "width": 800, "height": null } }`.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TargetArea {
    Percent(f32),
    Pixels(Pixels),
}

/// The card Resize Image.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct ResizeArea {
    target: TargetArea,
    method: Method,
}

impl From<ResizeArea> for Resize {
    fn from(r: ResizeArea) -> Self {
        Resize {
            target: match r.target {
                TargetArea::Percent(p) => ResizeTarget::Percent(p),
                TargetArea::Pixels(p) => ResizeTarget::Pixels {
                    width: p.width,
                    height: p.height,
                },
            },
            method: r.method.into(),
        }
    }
}

/// Both cards, as Save Image takes them.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct GeometryArea {
    orientation: OrientationArea,
    resize: Option<ResizeArea>,
}

impl From<GeometryArea> for Geometry {
    fn from(g: GeometryArea) -> Self {
        Geometry {
            orientation: g.orientation.into(),
            resize: g.resize.map(Into::into),
        }
    }
}

/// Turns a picture of the image area as Flip and Rotate says. The body is the raw RGBA bytes, the headers `width`, `height` and
/// `orientation` (JSON of [`OrientationArea`]) say what it is; the answer is binary like `read_frame`.
#[tauri::command]
pub fn orient_preview(
    request: tauri::ipc::Request<'_>,
    session: State<'_, Session>,
) -> CommandResult<Response> {
    let tauri::ipc::InvokeBody::Raw(bytes) = request.body() else {
        return Err("expected the RGBA bytes as raw body".to_owned().into());
    };
    let header = |name: &str| {
        request
            .headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
            .ok_or_else(|| format!("missing header {name}"))
    };
    let number = |name: &str| {
        header(name)?
            .parse::<u32>()
            .map_err(|_| format!("invalid header {name}"))
    };
    let orientation: OrientationArea = serde_json::from_str(&header("orientation")?)
        .map_err(|e| format!("invalid header orientation: {e}"))?;
    Ok(pack(session.orient_preview(
        number("width")?,
        number("height")?,
        bytes,
        &orientation.into(),
    )?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_window_values_become_the_geometry_of_the_core() {
        let json = r#"{
            "orientation": { "flipHorizontal": true, "flipVertical": false, "angle": -12.5, "cropToFit": true, "method": "bilinear" },
            "resize": { "target": { "pixels": { "width": 800, "height": null } }, "method": "nearest" }
        }"#;
        let geometry: Geometry = serde_json::from_str::<GeometryArea>(json).unwrap().into();
        assert_eq!(
            geometry.orientation,
            Orientation {
                flip_horizontal: true,
                flip_vertical: false,
                angle: -12.5,
                crop_to_fit: true,
                method: Resampling::Bilinear,
            }
        );
        assert_eq!(
            geometry.resize,
            Some(Resize {
                target: ResizeTarget::Pixels {
                    width: Some(800),
                    height: None
                },
                method: Resampling::Nearest,
            })
        );
        let percent: GeometryArea = serde_json::from_str(
            r#"{ "orientation": { "flipHorizontal": false, "flipVertical": false, "angle": 0, "cropToFit": false, "method": "bicubic" },
                 "resize": { "target": { "percent": 50 }, "method": "lanczos3" } }"#,
        )
        .unwrap();
        assert_eq!(
            Geometry::from(percent).resize.map(|r| r.target),
            Some(ResizeTarget::Percent(50.0))
        );
    }
}
