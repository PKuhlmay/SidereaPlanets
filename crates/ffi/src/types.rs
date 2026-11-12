//! The records and enums that cross the boundary: one mirror per request and result type of
//! `planets-session`, converted field by field. There is no logic in here, only translation; counts
//! (`usize`) become `u64` and fixed arrays become `Vec`s because UniFFI has no such types.

use crate::error::PlanetsError;
use planets_core::{refine as core_refine, wavelet as core_wavelet};
use planets_io::{export::ExportFormat as IoExportFormat, project::Project as IoProject};
use planets_session as session;
use std::collections::HashMap;

// --- Images ------------------------------------------------------------------------------------

/// An image as `width * height * 4` bytes: red, green, blue, alpha, row by row from the top left.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct RgbaImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl From<session::RgbaImage> for RgbaImage {
    fn from(image: session::RgbaImage) -> Self {
        Self {
            width: image.width(),
            height: image.height(),
            rgba: image.into_rgba(),
        }
    }
}

// --- Recording ---------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct RecordingInfo {
    pub format: String,
    pub codec: Option<String>,
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub frame_count: u32,
    pub color: String,
    pub fps: Option<f64>,
    pub observer: Option<String>,
    pub instrument: Option<String>,
    pub telescope: Option<String>,
    /// The file has a time for every frame (SER trailer).
    #[uniffi(default = false)]
    pub has_timestamps: bool,
    /// When the recording started, seconds since 1970-01-01 UTC.
    #[uniffi(default = None)]
    pub start_time: Option<f64>,
    /// `start_time` is a guess from the date of the file.
    #[uniffi(default = false)]
    pub start_time_from_file_date: bool,
}

impl From<session::RecordingInfo> for RecordingInfo {
    fn from(i: session::RecordingInfo) -> Self {
        Self {
            format: i.format.to_owned(),
            codec: i.codec,
            width: i.width,
            height: i.height,
            bit_depth: i.bit_depth,
            frame_count: i.frame_count,
            color: i.color,
            fps: i.fps,
            observer: i.observer,
            instrument: i.instrument,
            telescope: i.telescope,
            has_timestamps: i.has_timestamps,
            start_time: i.start_time,
            start_time_from_file_date: i.start_time_from_file_date,
        }
    }
}

// --- Progress ----------------------------------------------------------------------------------

/// The operations that report progress and can be cancelled or paused. `Sequence` is *Save Aligned*
/// and *Save Integrated*; it shares Cancel and Pause with `Stack`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum SessionOperation {
    Align,
    Stack,
    Sequence,
    Wavelet,
}

impl From<SessionOperation> for session::Operation {
    fn from(o: SessionOperation) -> Self {
        match o {
            SessionOperation::Align => Self::Align,
            SessionOperation::Stack => Self::Stack,
            SessionOperation::Sequence => Self::Sequence,
            SessionOperation::Wavelet => Self::Wavelet,
        }
    }
}

impl From<session::Operation> for SessionOperation {
    fn from(o: session::Operation) -> Self {
        match o {
            session::Operation::Align => Self::Align,
            session::Operation::Stack => Self::Stack,
            session::Operation::Sequence => Self::Sequence,
            session::Operation::Wavelet => Self::Wavelet,
        }
    }
}

/// Receives the progress of a running operation: `done` of `total` steps (frames, or layers for the
/// wavelets) and the milliseconds since it started. Called from the calculating thread, so a Swift
/// implementation must be thread-safe and must not block.
#[uniffi::export(with_foreign)]
pub trait ProgressListener: Send + Sync {
    fn progress(&self, operation: SessionOperation, done: u32, total: u32, elapsed_ms: u64);
}

// --- Save --------------------------------------------------------------------------------------

/// The part of the image that Save Image writes, in pixels from the top left corner (MAC-016).
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct CropRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl From<CropRect> for session::CropRect {
    fn from(r: CropRect) -> Self {
        Self {
            x: r.x,
            y: r.y,
            width: r.width,
            height: r.height,
        }
    }
}

// --- Geometry (MAC-017) ------------------------------------------------------------------------

/// How pixels are looked up between the pixels of the source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum Resampling {
    Nearest,
    Bilinear,
    Bicubic,
    Lanczos3,
}

impl From<Resampling> for session::Resampling {
    fn from(m: Resampling) -> Self {
        match m {
            Resampling::Nearest => Self::Nearest,
            Resampling::Bilinear => Self::Bilinear,
            Resampling::Bicubic => Self::Bicubic,
            Resampling::Lanczos3 => Self::Lanczos3,
        }
    }
}

/// *Flip and Rotate*: mirror first, then turn clockwise by `angle` degrees.
#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct OrientationSettings {
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
    pub angle: f32,
    pub crop_to_fit: bool,
    pub method: Resampling,
}

impl From<OrientationSettings> for session::Orientation {
    fn from(o: OrientationSettings) -> Self {
        Self {
            flip_horizontal: o.flip_horizontal,
            flip_vertical: o.flip_vertical,
            angle: o.angle,
            crop_to_fit: o.crop_to_fit,
            method: o.method.into(),
        }
    }
}

/// The size *Resize Image* asks for: a percentage, or pixels where a side that is `None` follows the other.
#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum)]
pub enum ResizeTarget {
    Percent {
        percent: f32,
    },
    Pixels {
        width: Option<u32>,
        height: Option<u32>,
    },
}

impl From<ResizeTarget> for session::ResizeTarget {
    fn from(t: ResizeTarget) -> Self {
        match t {
            ResizeTarget::Percent { percent } => Self::Percent(percent),
            ResizeTarget::Pixels { width, height } => Self::Pixels { width, height },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct ResizeSettings {
    pub target: ResizeTarget,
    pub method: Resampling,
}

impl From<ResizeSettings> for session::Resize {
    fn from(r: ResizeSettings) -> Self {
        Self {
            target: r.target.into(),
            method: r.method.into(),
        }
    }
}

/// What Save Image does to the picture besides the colours: the orientation before the crop, the resize after it.
#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct GeometrySettings {
    pub orientation: OrientationSettings,
    pub resize: Option<ResizeSettings>,
}

impl From<GeometrySettings> for session::Geometry {
    fn from(g: GeometrySettings) -> Self {
        Self {
            orientation: g.orientation.into(),
            resize: g.resize.map(Into::into),
        }
    }
}

/// A size in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct ImageSize {
    pub width: u32,
    pub height: u32,
}

// --- Align -------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct RoiSettings {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl From<RoiSettings> for session::RoiSettings {
    fn from(r: RoiSettings) -> Self {
        Self {
            x: r.x,
            y: r.y,
            width: r.width,
            height: r.height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum Intensity {
    Default,
    Area3x3,
    Lowest,
}

impl From<Intensity> for session::Intensity {
    fn from(i: Intensity) -> Self {
        match i {
            Intensity::Default => Self::Default,
            Intensity::Area3x3 => Self::Area3x3,
            Intensity::Lowest => Self::Lowest,
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct PointSettings {
    pub min_distance: u32,
    pub edge_distance: u32,
    pub max_movement: u32,
    pub roi: Option<RoiSettings>,
    pub intensity: Intensity,
    pub lo: f32,
    pub hi: f32,
    pub box_size: u32,
}

impl From<PointSettings> for session::PointSettings {
    fn from(s: PointSettings) -> Self {
        Self {
            min_distance: s.min_distance,
            edge_distance: s.edge_distance,
            max_movement: s.max_movement,
            roi: s.roi.map(Into::into),
            intensity: s.intensity.into(),
            lo: s.lo,
            hi: s.hi,
            box_size: s.box_size,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct Alignpoints {
    /// The points in use after the threshold.
    pub points: Vec<Point>,
    /// Number of points before the threshold, the upper end of the slider.
    pub total: u64,
    pub threshold: u64,
}

impl From<session::Alignpoints> for Alignpoints {
    fn from(a: session::Alignpoints) -> Self {
        Self {
            points: a
                .points
                .into_iter()
                .map(|p| Point { x: p.x, y: p.y })
                .collect(),
            total: a.total as u64,
            threshold: a.threshold as u64,
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct AlignSettings {
    pub box_size: u32,
    pub max_movement: u32,
    pub roi: Option<RoiSettings>,
}

impl From<AlignSettings> for session::AlignSettings {
    fn from(s: AlignSettings) -> Self {
        Self {
            box_size: s.box_size,
            max_movement: s.max_movement,
            roi: s.roi.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct FrameRating {
    pub index: u32,
    pub quality: f32,
    pub shift_x: f32,
    pub shift_y: f32,
    pub difference: f32,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct Aligned {
    /// One rating per frame, in frame order.
    pub frames: Vec<FrameRating>,
    /// The frame indices from best to worst.
    pub ranking: Vec<u32>,
}

impl From<session::Aligned> for Aligned {
    fn from(a: session::Aligned) -> Self {
        Self {
            frames: a
                .frames
                .into_iter()
                .map(|f| FrameRating {
                    index: f.index,
                    quality: f.quality,
                    shift_x: f.shift_x,
                    shift_y: f.shift_y,
                    difference: f.difference,
                })
                .collect(),
            ranking: a.ranking,
        }
    }
}

/// *Limit*: which frames stay.
#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum)]
pub enum LimitSettings {
    LowestQuality { value: f32 },
    BestPercent { value: f32 },
    BestCount { value: u64 },
    FramesPerPoint { value: u64 },
}

impl From<LimitSettings> for session::LimitSettings {
    fn from(l: LimitSettings) -> Self {
        match l {
            LimitSettings::LowestQuality { value } => Self::LowestQuality(value),
            LimitSettings::BestPercent { value } => Self::BestPercent(value),
            LimitSettings::BestCount { value } => Self::BestCount(value as usize),
            LimitSettings::FramesPerPoint { value } => Self::FramesPerPoint(value as usize),
        }
    }
}

/// How far a point sits from where the movement of the whole frame puts it, in pixels.
#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct PointShift {
    pub dx: f32,
    pub dy: f32,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct Limited {
    /// The frame indices that stay, best first.
    pub frames: Vec<u32>,
    /// One entry per point in use; `None` if the point was never found.
    pub point_shifts: Vec<Option<PointShift>>,
}

impl From<session::Limited> for Limited {
    fn from(l: session::Limited) -> Self {
        Self {
            frames: l.frames,
            point_shifts: l
                .point_shifts
                .into_iter()
                .map(|s| s.map(|[dx, dy]| PointShift { dx, dy }))
                .collect(),
        }
    }
}

// --- Stack and sequences -----------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct PointPosition {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct StackSettings {
    pub reference_frame: u32,
    pub points: Vec<PointPosition>,
    pub frames: Vec<u32>,
    pub box_size: u32,
    pub max_movement: u32,
    pub nearest_points: u64,
    pub correct_geometry: bool,
    pub normalize: bool,
    pub edge_skip: u32,
    pub roi: Option<RoiSettings>,
    /// *De-rotate images*; `None` is off.
    #[uniffi(default = None)]
    pub derotation: Option<DerotationSettings>,
}

impl From<StackSettings> for session::StackSettings {
    fn from(s: StackSettings) -> Self {
        Self {
            reference_frame: s.reference_frame,
            points: s
                .points
                .into_iter()
                .map(|p| session::PointPosition { x: p.x, y: p.y })
                .collect(),
            frames: s.frames,
            box_size: s.box_size,
            max_movement: s.max_movement,
            nearest_points: s.nearest_points as usize,
            correct_geometry: s.correct_geometry,
            normalize: s.normalize,
            edge_skip: s.edge_skip,
            roi: s.roi.map(Into::into),
            derotation: s.derotation.map(Into::into),
        }
    }
}

// --- De-rotation -------------------------------------------------------------------------------

/// The planets whose rotation can be taken out of a stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum DerotatePlanet {
    Jupiter,
    Saturn,
    Mars,
}

impl From<DerotatePlanet> for session::Planet {
    fn from(p: DerotatePlanet) -> Self {
        match p {
            DerotatePlanet::Jupiter => Self::Jupiter,
            DerotatePlanet::Saturn => Self::Saturn,
            DerotatePlanet::Mars => Self::Mars,
        }
    }
}

impl From<session::Planet> for DerotatePlanet {
    fn from(p: session::Planet) -> Self {
        match p {
            session::Planet::Jupiter => Self::Jupiter,
            session::Planet::Saturn => Self::Saturn,
            session::Planet::Mars => Self::Mars,
        }
    }
}

/// *De-rotate images* of the stacking options.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct DerotationSettings {
    pub planet: DerotatePlanet,
    /// Degrees counter-clockwise on the screen; 0 means celestial north is up.
    pub camera_angle_deg: f64,
    pub mirrored: bool,
    /// When the first frame was taken, seconds since 1970-01-01 UTC; only used without time stamps in the file.
    pub start_time_utc: Option<f64>,
    pub use_file_timestamps: bool,
}

impl From<DerotationSettings> for session::DerotationSettings {
    fn from(d: DerotationSettings) -> Self {
        Self {
            planet: d.planet.into(),
            camera_angle_deg: d.camera_angle_deg,
            mirrored: d.mirrored,
            start_time_utc: d.start_time_utc,
            use_file_timestamps: d.use_file_timestamps,
        }
    }
}

/// The disc of the planet as ellipses to draw over the image (pixels of the frame, y down).
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct DiscShape {
    pub x: f64,
    pub y: f64,
    /// Along the equator.
    pub semi_major: f64,
    /// Along the axis of the planet.
    pub semi_minor: f64,
    /// Direction of the major axis, degrees clockwise on the screen from pointing right.
    pub angle: f64,
    /// Outer edge of the ring as `[semi_major, semi_minor]` (Saturn), else empty.
    pub ring: Vec<f64>,
    /// Unit vector from the centre towards the north pole of the planet (x right, y down).
    pub north_x: f64,
    pub north_y: f64,
}

impl From<session::DiscShape> for DiscShape {
    fn from(d: session::DiscShape) -> Self {
        Self {
            x: d.x,
            y: d.y,
            semi_major: d.semi_major,
            semi_minor: d.semi_minor,
            angle: d.angle,
            ring: d.ring.map(|(a, b)| vec![a, b]).unwrap_or_default(),
            north_x: d.north.0,
            north_y: d.north.1,
        }
    }
}

/// What the Stack panel says about de-rotation.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct DerotationInfo {
    pub span_seconds: f64,
    pub rotation_degrees: f64,
    pub sub_earth_latitude: f64,
    pub pole_angle: f64,
    pub disc: Option<DiscShape>,
    /// Why de-rotation cannot work yet, in the words the window shows.
    pub problem: Option<String>,
}

impl From<session::DerotationInfo> for DerotationInfo {
    fn from(i: session::DerotationInfo) -> Self {
        Self {
            span_seconds: i.span_seconds,
            rotation_degrees: i.rotation_degrees,
            sub_earth_latitude: i.sub_earth_latitude,
            pole_angle: i.pole_angle,
            disc: i.disc.map(Into::into),
            problem: i.problem,
        }
    }
}

/// What the UI shows after *Stack*: "Stack size (n=10) - (1024x680)".
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct Stacked {
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    /// How many frames went into the stack.
    pub frames: u32,
    pub elapsed_ms: u64,
}

impl From<session::Stacked> for Stacked {
    fn from(s: session::Stacked) -> Self {
        Self {
            width: s.width,
            height: s.height,
            channels: s.channels,
            frames: s.frames,
            elapsed_ms: s.elapsed_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct SequenceSettings {
    pub stack: StackSettings,
    pub framerate: u32,
    pub sort_on_quality: bool,
    pub average_frames: u64,
}

impl From<SequenceSettings> for session::SequenceSettings {
    fn from(s: SequenceSettings) -> Self {
        Self {
            stack: s.stack.into(),
            framerate: s.framerate,
            sort_on_quality: s.sort_on_quality,
            average_frames: s.average_frames as usize,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct SavedVideo {
    pub path: String,
    pub frames: u32,
}

impl From<session::SavedVideo> for SavedVideo {
    fn from(v: session::SavedVideo) -> Self {
        Self {
            path: v.path,
            frames: v.frames,
        }
    }
}

// --- Wavelets ----------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum WaveletScheme {
    Dyadic,
    Linear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum WaveletFilter {
    Default,
    Gaussian,
}

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct LayerSettings {
    pub enabled: bool,
    pub gain: f32,
    pub denoise: f32,
    pub sharpen: f32,
}

/// Everything the left panel *Wavelets* holds; `layers` has exactly six entries.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct WaveletSettings {
    pub scheme: WaveletScheme,
    pub filter: WaveletFilter,
    pub initial: u32,
    pub step: u32,
    pub layers: Vec<LayerSettings>,
}

impl From<core_wavelet::WaveletSettings> for WaveletSettings {
    fn from(s: core_wavelet::WaveletSettings) -> Self {
        Self {
            scheme: match s.structure.scheme {
                core_wavelet::settings::Scheme::Dyadic => WaveletScheme::Dyadic,
                core_wavelet::settings::Scheme::Linear => WaveletScheme::Linear,
            },
            filter: match s.structure.filter {
                core_wavelet::settings::Filter::Default => WaveletFilter::Default,
                core_wavelet::settings::Filter::Gaussian => WaveletFilter::Gaussian,
            },
            initial: s.structure.initial,
            step: s.structure.step,
            layers: s
                .layers
                .iter()
                .map(|l| LayerSettings {
                    enabled: l.enabled,
                    gain: l.gain,
                    denoise: l.denoise,
                    sharpen: l.sharpen,
                })
                .collect(),
        }
    }
}

impl TryFrom<WaveletSettings> for core_wavelet::WaveletSettings {
    type Error = PlanetsError;

    fn try_from(s: WaveletSettings) -> Result<Self, PlanetsError> {
        let count = s.layers.len();
        let layers: Vec<core_wavelet::settings::LayerSettings> = s
            .layers
            .into_iter()
            .map(|l| core_wavelet::settings::LayerSettings {
                enabled: l.enabled,
                gain: l.gain,
                denoise: l.denoise,
                sharpen: l.sharpen,
            })
            .collect();
        let layers: [core_wavelet::settings::LayerSettings; core_wavelet::settings::LAYERS] =
            layers.try_into().map_err(|_| PlanetsError::Failed {
                message: format!(
                    "wavelet settings need {} layers, got {count}",
                    core_wavelet::settings::LAYERS
                ),
            })?;
        Ok(Self {
            structure: core_wavelet::settings::Structure {
                scheme: match s.scheme {
                    WaveletScheme::Dyadic => core_wavelet::settings::Scheme::Dyadic,
                    WaveletScheme::Linear => core_wavelet::settings::Scheme::Linear,
                },
                filter: match s.filter {
                    WaveletFilter::Default => core_wavelet::settings::Filter::Default,
                    WaveletFilter::Gaussian => core_wavelet::settings::Filter::Gaussian,
                },
                initial: s.initial,
                step: s.step,
            },
            layers,
        })
    }
}

/// How the colour of the image is treated while sharpening.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum ColourMode {
    Colour,
    Lrgb,
    Gray,
}

impl From<ColourMode> for core_wavelet::ColourMode {
    fn from(m: ColourMode) -> Self {
        match m {
            ColourMode::Colour => Self::Colour,
            ColourMode::Lrgb => Self::Lrgb,
            ColourMode::Gray => Self::Gray,
        }
    }
}

/// A rectangle of pixels in image coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl From<Region> for core_wavelet::Region {
    fn from(r: Region) -> Self {
        Self {
            x: r.x,
            y: r.y,
            width: r.width,
            height: r.height,
        }
    }
}

impl From<core_wavelet::Region> for Region {
    fn from(r: core_wavelet::Region) -> Self {
        Self {
            x: r.x,
            y: r.y,
            width: r.width,
            height: r.height,
        }
    }
}

/// Size of the image in the Wavelet tab and of the source of the colour panels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct SourceInfo {
    pub width: u32,
    pub height: u32,
    /// 1 for a mono image: the colour panels do nothing then.
    pub channels: u8,
}

impl From<session::SourceInfo> for SourceInfo {
    fn from(s: session::SourceInfo) -> Self {
        Self {
            width: s.width,
            height: s.height,
            channels: s.channels,
        }
    }
}

/// One rendered area of the preview: `region.width * region.height * 4` RGBA bytes.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct WaveletPreview {
    pub region: Region,
    pub rgba: Vec<u8>,
}

impl From<session::WaveletPreview> for WaveletPreview {
    fn from(p: session::WaveletPreview) -> Self {
        Self {
            region: p.region().into(),
            rgba: p.into_rgba(),
        }
    }
}

// --- Colour and tone ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct ChannelShift {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct ChannelShifts {
    pub red: ChannelShift,
    pub green: ChannelShift,
    pub blue: ChannelShift,
}

impl From<session::ChannelShifts> for ChannelShifts {
    fn from(s: session::ChannelShifts) -> Self {
        let shift = |c: session::ChannelShift| ChannelShift { x: c.x, y: c.y };
        Self {
            red: shift(s.red),
            green: shift(s.green),
            blue: shift(s.blue),
        }
    }
}

impl From<ChannelShifts> for session::ChannelShifts {
    fn from(s: ChannelShifts) -> Self {
        let shift = |c: ChannelShift| session::ChannelShift { x: c.x, y: c.y };
        Self {
            red: shift(s.red),
            green: shift(s.green),
            blue: shift(s.blue),
        }
    }
}

/// *Color Mixing*: nine weights in percent, `matrix[3 * output + input]` in the order red, green, blue
/// (100, 0, 0, 0, 100, 0, 0, 0, 100 is the identity), and whether the brightness of each pixel is kept.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct ColorMix {
    pub matrix: Vec<f32>,
    pub preserve_brightness: bool,
}

impl From<ColorMix> for session::ColorMix {
    fn from(m: ColorMix) -> Self {
        // A wrong length cannot come from the panel; anything missing stays at the identity.
        let mut out = session::ColorMix::default();
        for (slot, value) in out.matrix.iter_mut().zip(m.matrix) {
            *slot = value;
        }
        out.preserve_brightness = m.preserve_brightness;
        out
    }
}

/// The state of all colour and tone panels, in the units the panels show (percent, pixels, 0..255).
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct ColorSettings {
    pub shifts: ChannelShifts,
    pub mix: ColorMix,
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub black: f32,
    pub white: f32,
    pub gamma: f32,
    pub contrast: f32,
    pub brightness: f32,
    /// *Denoise/Deringing* and *Masking*, applied before everything above.
    pub refine: RefineSettings,
}

/// The *Masking* card: percent and pixels as the card shows them, the threshold on a 0..255 scale.
#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct MaskSettings {
    pub enabled: bool,
    pub threshold: f32,
    pub feather: f32,
    pub invert: bool,
}

/// The *Denoise/Deringing* and *Masking* cards.
#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct RefineSettings {
    /// Percent, 0 = off.
    pub denoise: f32,
    /// Percent, 0 = off.
    pub deringing: f32,
    /// Pixels.
    pub deringing_radius: u32,
    pub mask: MaskSettings,
}

impl From<RefineSettings> for core_refine::RefineSettings {
    fn from(s: RefineSettings) -> Self {
        Self {
            denoise: s.denoise,
            deringing: s.deringing,
            deringing_radius: s.deringing_radius,
            mask: core_refine::MaskSettings {
                enabled: s.mask.enabled,
                threshold: s.mask.threshold,
                feather: s.mask.feather,
                invert: s.mask.invert,
            },
        }
    }
}

impl From<core_refine::RefineSettings> for RefineSettings {
    fn from(s: core_refine::RefineSettings) -> Self {
        Self {
            denoise: s.denoise,
            deringing: s.deringing,
            deringing_radius: s.deringing_radius,
            mask: MaskSettings {
                enabled: s.mask.enabled,
                threshold: s.mask.threshold,
                feather: s.mask.feather,
                invert: s.mask.invert,
            },
        }
    }
}

/// The values *Reset* on the cards restores.
#[uniffi::export]
pub fn refine_default_settings() -> RefineSettings {
    core_refine::RefineSettings::default().into()
}

impl From<ColorSettings> for session::ColorSettings {
    fn from(s: ColorSettings) -> Self {
        Self {
            refine: s.refine.into(),
            shifts: s.shifts.into(),
            mix: s.mix.into(),
            red: s.red,
            green: s.green,
            blue: s.blue,
            black: s.black,
            white: s.white,
            gamma: s.gamma,
            contrast: s.contrast,
            brightness: s.brightness,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct HistogramData {
    /// Red, green and blue (one entry for a mono image), each with `bins` counts.
    pub channels: Vec<Vec<u32>>,
    pub luma: Vec<u32>,
    pub bins: u64,
}

impl From<session::HistogramData> for HistogramData {
    fn from(h: session::HistogramData) -> Self {
        Self {
            channels: h.channels,
            luma: h.luma,
            bins: h.bins as u64,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct HistogramPoints {
    pub black: f32,
    pub white: f32,
}

impl From<session::HistogramPoints> for HistogramPoints {
    fn from(p: session::HistogramPoints) -> Self {
        Self {
            black: p.black,
            white: p.white,
        }
    }
}

// --- Saving ------------------------------------------------------------------------------------

/// Which image *Save image* writes: the one of the tab it is called from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum ImageSource {
    Stack,
    Wavelet,
}

impl From<ImageSource> for session::ImageSource {
    fn from(s: ImageSource) -> Self {
        match s {
            ImageSource::Stack => Self::Stack,
            ImageSource::Wavelet => Self::Wavelet,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum ExportFormat {
    Bmp,
    Jpeg,
    Tiff16,
    Png16,
    Fits16,
    Fits32,
    Fits32Float,
}

impl From<ExportFormat> for IoExportFormat {
    fn from(f: ExportFormat) -> Self {
        match f {
            ExportFormat::Bmp => Self::Bmp,
            ExportFormat::Jpeg => Self::Jpeg,
            ExportFormat::Tiff16 => Self::Tiff16,
            ExportFormat::Png16 => Self::Png16,
            ExportFormat::Fits16 => Self::Fits16,
            ExportFormat::Fits32 => Self::Fits32,
            ExportFormat::Fits32Float => Self::Fits32Float,
        }
    }
}

impl From<IoExportFormat> for ExportFormat {
    fn from(f: IoExportFormat) -> Self {
        match f {
            IoExportFormat::Bmp => Self::Bmp,
            IoExportFormat::Jpeg => Self::Jpeg,
            IoExportFormat::Tiff16 => Self::Tiff16,
            IoExportFormat::Png16 => Self::Png16,
            IoExportFormat::Fits16 => Self::Fits16,
            IoExportFormat::Fits32 => Self::Fits32,
            IoExportFormat::Fits32Float => Self::Fits32Float,
        }
    }
}

/// One entry of the file type list of *Save image*.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct FormatInfo {
    pub id: ExportFormat,
    pub label: String,
    /// File extension without the dot.
    pub extension: String,
}

impl From<session::FormatInfo> for FormatInfo {
    fn from(f: session::FormatInfo) -> Self {
        Self {
            id: f.id.into(),
            label: f.label.to_owned(),
            extension: f.extension.to_owned(),
        }
    }
}

// --- Project file ------------------------------------------------------------------------------

/// A project file. The `sections` hold the settings of the processing steps as JSON text each; the
/// layout of a section belongs to its step, the interface only passes the text through.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct Project {
    pub format_version: u32,
    pub recording: Vec<String>,
    pub sections: HashMap<String, String>,
}

impl From<IoProject> for Project {
    fn from(p: IoProject) -> Self {
        Self {
            format_version: p.format_version,
            recording: p.recording,
            sections: p
                .sections
                .into_iter()
                .map(|(name, value)| (name, value.to_string()))
                .collect(),
        }
    }
}

impl TryFrom<Project> for IoProject {
    type Error = PlanetsError;

    fn try_from(p: Project) -> Result<Self, PlanetsError> {
        let mut sections = std::collections::BTreeMap::new();
        for (name, text) in p.sections {
            let value = serde_json::from_str(&text).map_err(|e| PlanetsError::Failed {
                message: format!("project section {name}: {e}"),
            })?;
            sections.insert(name, value);
        }
        Ok(Self {
            format_version: p.format_version,
            recording: p.recording,
            sections,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct LoadedProject {
    pub path: String,
    pub project: Project,
    /// Recording files that are gone, so the interface can say which ones to look for.
    pub missing_files: Vec<String>,
}

impl From<session::LoadedProject> for LoadedProject {
    fn from(l: session::LoadedProject) -> Self {
        Self {
            path: l.path,
            project: l.project.into(),
            missing_files: l.missing_files,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wavelet_settings_survive_the_round_trip() {
        let mut settings = planets_session::wavelet_default_settings();
        settings.layers[2].gain = 3.5;
        let ffi = WaveletSettings::from(settings.clone());
        assert_eq!(ffi.layers.len(), core_wavelet::settings::LAYERS);
        assert_eq!(core_wavelet::WaveletSettings::try_from(ffi), Ok(settings));
    }

    #[test]
    fn a_crop_rectangle_keeps_every_field() {
        let rect = CropRect {
            x: 1,
            y: 2,
            width: 30,
            height: 40,
        };
        let core = session::CropRect::from(rect);
        assert_eq!((core.x, core.y, core.width, core.height), (1, 2, 30, 40));
    }

    #[test]
    fn the_geometry_keeps_every_field() {
        let geometry = GeometrySettings {
            orientation: OrientationSettings {
                flip_horizontal: true,
                flip_vertical: false,
                angle: -12.5,
                crop_to_fit: true,
                method: Resampling::Bicubic,
            },
            resize: Some(ResizeSettings {
                target: ResizeTarget::Pixels {
                    width: Some(800),
                    height: None,
                },
                method: Resampling::Nearest,
            }),
        };
        let core = session::Geometry::from(geometry);
        assert!(core.orientation.flip_horizontal && !core.orientation.flip_vertical);
        assert_eq!(core.orientation.angle, -12.5);
        assert!(core.orientation.crop_to_fit);
        assert_eq!(core.orientation.method, session::Resampling::Bicubic);
        assert_eq!(
            core.resize,
            Some(session::Resize {
                target: session::ResizeTarget::Pixels {
                    width: Some(800),
                    height: None
                },
                method: session::Resampling::Nearest,
            })
        );
        let percent = session::ResizeTarget::from(ResizeTarget::Percent { percent: 150.0 });
        assert_eq!(percent, session::ResizeTarget::Percent(150.0));
        assert_eq!(
            session::Geometry::from(GeometrySettings {
                resize: None,
                ..geometry
            })
            .resize,
            None
        );
    }

    #[test]
    fn wavelet_settings_with_the_wrong_layer_count_are_refused() {
        let mut ffi = WaveletSettings::from(planets_session::wavelet_default_settings());
        ffi.layers.pop();
        let error = core_wavelet::WaveletSettings::try_from(ffi).unwrap_err();
        assert!(error.to_string().contains("6 layers, got 5"), "{error}");
    }

    #[test]
    fn project_sections_travel_as_json_text() {
        let mut project = IoProject::new(vec!["a.ser".to_owned()]);
        project
            .set_section("align", &serde_json::json!({ "minDistance": 30 }))
            .unwrap();
        let ffi = Project::from(project.clone());
        assert_eq!(ffi.sections["align"], r#"{"minDistance":30}"#);
        assert_eq!(IoProject::try_from(ffi), Ok(project));

        let broken = Project {
            format_version: 1,
            recording: vec![],
            sections: HashMap::from([("align".to_owned(), "{".to_owned())]),
        };
        let error = IoProject::try_from(broken).unwrap_err();
        assert!(
            error.to_string().starts_with("project section align"),
            "{error}"
        );
    }

    #[test]
    fn limit_settings_keep_their_mode_and_value() {
        use session::LimitSettings as Core;
        assert_eq!(
            Core::from(LimitSettings::BestCount { value: 200 }),
            Core::BestCount(200)
        );
        assert_eq!(
            Core::from(LimitSettings::LowestQuality { value: 0.5 }),
            Core::LowestQuality(0.5)
        );
    }

    #[test]
    fn de_rotation_settings_cross_unchanged_and_are_off_by_default() {
        let settings = DerotationSettings {
            planet: DerotatePlanet::Saturn,
            camera_angle_deg: -12.5,
            mirrored: true,
            start_time_utc: Some(1_700_000_000.25),
            use_file_timestamps: false,
        };
        let session: session::DerotationSettings = settings.into();
        assert_eq!(session.planet, session::Planet::Saturn);
        assert_eq!(session.camera_angle_deg, -12.5);
        assert!(session.mirrored && !session.use_file_timestamps);
        assert_eq!(session.start_time_utc, Some(1_700_000_000.25));
        assert_eq!(
            DerotatePlanet::from(session::Planet::Mars),
            DerotatePlanet::Mars
        );
    }

    #[test]
    fn the_ring_of_a_disc_becomes_a_list() {
        let shape = session::DiscShape {
            x: 1.0,
            y: 2.0,
            semi_major: 3.0,
            semi_minor: 2.5,
            angle: -7.0,
            ring: Some((6.0, 1.5)),
            north: (0.6, -0.8),
        };
        assert_eq!(DiscShape::from(shape).ring, vec![6.0, 1.5]);
        assert_eq!(
            (
                DiscShape::from(shape).north_x,
                DiscShape::from(shape).north_y
            ),
            (0.6, -0.8)
        );
        let plain = session::DiscShape {
            ring: None,
            ..shape
        };
        assert!(DiscShape::from(plain).ring.is_empty());
    }
}
