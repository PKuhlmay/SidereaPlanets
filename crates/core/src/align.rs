//! Alignment points, registration of frames against a reference and quality rating.
//!
//! The behaviour of the reference program is only known from the outside (`docs/offene-fragen.md`); the
//! algorithms here are our own and are checked against the original's results (`docs/testdaten.md`).

use crate::Image;
use rayon::prelude::*;

/// How the pixel value of a candidate point is tested against *Lo* and *Hi*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IntensityMode {
    /// The centre pixel.
    #[default]
    Default,
    /// The mean of the 3x3 pixels around the centre.
    Area3x3,
    /// The darkest of the 3x3 pixels around the centre.
    LowestPixel,
}

/// The *Region of Interest* (*R.o.Interest*): the rectangle of the frame that alignment is based on,
/// in pixels from the top left corner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Roi {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Roi {
    /// The part of the region that lies inside a frame of `width` x `height`; `None` if nothing is left.
    pub fn clamp(self, width: u32, height: u32) -> Option<Roi> {
        let x1 = self.x.saturating_add(self.width).min(width);
        let y1 = self.y.saturating_add(self.height).min(height);
        (self.x < x1 && self.y < y1).then(|| Roi {
            x: self.x,
            y: self.y,
            width: x1 - self.x,
            height: y1 - self.y,
        })
    }
}

/// Settings of *Set Alignpoints* with the defaults of the reference program (`docs/ui.md`).
#[derive(Debug, Clone, PartialEq)]
pub struct PointParams {
    /// *Minimum distance between* two points, in pixels. The boxes of two points must not reach into
    /// each other's search range, so [`max_movement`](Self::max_movement) comes on top of it, see
    /// [`find_alignpoints`].
    pub min_distance: u32,
    /// *Max Alignpoint movement*, in pixels.
    pub max_movement: u32,
    /// Only look for points inside this region; *Min distance from edge* then counts from its edges.
    pub roi: Option<Roi>,
    /// *Min distance from edge*, in pixels.
    pub edge_distance: u32,
    pub intensity: IntensityMode,
    /// *Lo* and *Hi* on a 0..=255 scale.
    pub lo: f32,
    pub hi: f32,
    /// *Alignmentbox size*; the structure of a point is measured inside this box.
    pub box_size: u32,
}

impl Default for PointParams {
    fn default() -> Self {
        Self {
            min_distance: 30,
            max_movement: 5,
            roi: None,
            edge_distance: 20,
            intensity: IntensityMode::Default,
            lo: 30.0,
            hi: 230.0,
            box_size: 30,
        }
    }
}

/// A place in the reference frame that is tracked through all frames.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlignPoint {
    pub x: u32,
    pub y: u32,
    /// How well the box around the point can be tracked in both directions; larger is easier.
    pub strength: f32,
}

/// Candidate positions are scanned on this pixel grid, which is plenty for points 30 px apart.
const SCAN_STEP: usize = 2;

/// Finds all points that satisfy the spacing rules, strongest first. The *Threshold* slider of the
/// original then drops the weakest ones, see [`apply_threshold`].
///
/// Two points are at least `min_distance + max_movement` apart in both directions at once (square
/// boxes, no overlap of the search ranges). That is a conjecture: the original's INI holds an
/// internal block size of 35 = 30 + 5, and with this spacing the clone finds 87 points on the
/// reference case where the original finds 91 (`docs/testdaten.md`).
pub fn find_alignpoints(frame: &Image, params: &PointParams) -> Vec<AlignPoint> {
    let luma = frame.to_luma();
    let (w, h) = (luma.width as usize, luma.height as usize);
    let area = match params.roi {
        Some(roi) => match roi.clamp(luma.width, luma.height) {
            Some(roi) => roi,
            None => return Vec::new(),
        },
        None => Roi {
            x: 0,
            y: 0,
            width: luma.width,
            height: luma.height,
        },
    };
    let edge = params.edge_distance as usize;
    if area.width as usize <= 2 * edge || area.height as usize <= 2 * edge || w < 3 || h < 3 {
        return Vec::new();
    }
    let (x0, y0) = (area.x as usize + edge, area.y as usize + edge);
    let (x1, y1) = (
        (area.x + area.width) as usize - edge,
        (area.y + area.height) as usize - edge,
    );
    let structure = StructureSums::new(&luma.data, w, h);
    let half = (params.box_size / 2) as usize;
    let (lo, hi) = (params.lo / 255.0, params.hi / 255.0);

    let mut candidates: Vec<AlignPoint> = Vec::new();
    for y in (y0..y1).step_by(SCAN_STEP) {
        for x in (x0..x1).step_by(SCAN_STEP) {
            let value = match params.intensity {
                IntensityMode::Default => luma.data[y * w + x],
                IntensityMode::Area3x3 => {
                    let (sum, n) = neighbours(&luma.data, w, h, x, y)
                        .fold((0.0, 0.0), |(s, n), v| (s + v, n + 1.0));
                    sum / n
                }
                IntensityMode::LowestPixel => {
                    neighbours(&luma.data, w, h, x, y).fold(f32::MAX, f32::min)
                }
            };
            if value < lo || value > hi {
                continue;
            }
            candidates.push(AlignPoint {
                x: x as u32,
                y: y as u32,
                strength: structure.strength(x, y, half) as f32,
            });
        }
    }
    candidates.sort_by(|a, b| {
        b.strength
            .total_cmp(&a.strength)
            .then(a.y.cmp(&b.y))
            .then(a.x.cmp(&b.x))
    });

    let spacing = params.min_distance + params.max_movement;
    let mut chosen: Vec<AlignPoint> = Vec::new();
    for c in candidates {
        let far_enough = chosen
            .iter()
            .all(|p| p.x.abs_diff(c.x).max(p.y.abs_diff(c.y)) >= spacing);
        if far_enough {
            chosen.push(c);
        }
    }
    chosen
}

/// The pixels of the 3x3 neighbourhood that lie inside the image.
fn neighbours(data: &[f32], w: usize, h: usize, x: usize, y: usize) -> impl Iterator<Item = f32> {
    (y.saturating_sub(1)..=(y + 1).min(h - 1)).flat_map(move |yy| {
        (x.saturating_sub(1)..=(x + 1).min(w - 1)).map(move |xx| data[yy * w + xx])
    })
}

/// Share of the points the *Threshold* slider drops after *Set Alignpoints*. The original reports
/// 91 points and puts the slider at 9, which is about a tenth. One observation, unconfirmed.
const DEFAULT_DROP_SHARE: f32 = 0.10;

/// The *Threshold* slider position after *Set Alignpoints*, see [`DEFAULT_DROP_SHARE`].
pub fn default_threshold(point_count: usize) -> usize {
    (point_count as f32 * DEFAULT_DROP_SHARE).round() as usize
}

/// Drops the `threshold` weakest points (`points` must be sorted strongest first).
pub fn apply_threshold(points: &[AlignPoint], threshold: usize) -> &[AlignPoint] {
    &points[..points.len().saturating_sub(threshold)]
}

/// Summed area tables of the gradient products, to measure structure in a box in constant time.
struct StructureSums {
    stride: usize,
    height: usize,
    /// Tables for `gx * gx`, `gy * gy` and `gx * gy`.
    tables: [Vec<f64>; 3],
}

impl StructureSums {
    fn new(luma: &[f32], w: usize, h: usize) -> Self {
        let at = |x: usize, y: usize| f64::from(luma[y * w + x]);
        let stride = w + 1;
        let mut tables = [(); 3].map(|()| vec![0.0; stride * (h + 1)]);
        for y in 0..h {
            let mut rows = [0.0; 3];
            for x in 0..w {
                let gx = if x > 0 && x + 1 < w {
                    (at(x + 1, y) - at(x - 1, y)) / 2.0
                } else {
                    0.0
                };
                let gy = if y > 0 && y + 1 < h {
                    (at(x, y + 1) - at(x, y - 1)) / 2.0
                } else {
                    0.0
                };
                for (row, product) in rows.iter_mut().zip([gx * gx, gy * gy, gx * gy]) {
                    *row += product;
                }
                for (table, row) in tables.iter_mut().zip(rows) {
                    table[(y + 1) * stride + x + 1] = table[y * stride + x + 1] + row;
                }
            }
        }
        Self {
            stride,
            height: h,
            tables,
        }
    }

    /// How well the box of `2 * half` pixels around `(x, y)` can be tracked in both directions: the
    /// smaller eigenvalue of the summed structure tensor. A straight edge, which lets a match slide
    /// along it, scores close to zero.
    fn strength(&self, x: usize, y: usize, half: usize) -> f64 {
        let x0 = x.saturating_sub(half);
        let y0 = y.saturating_sub(half);
        let x1 = (x + half).min(self.stride - 1);
        let y1 = (y + half).min(self.height);
        let sum = |t: &Vec<f64>| {
            let at = |xx: usize, yy: usize| t[yy * self.stride + xx];
            at(x1, y1) - at(x0, y1) - at(x1, y0) + at(x0, y0)
        };
        let [sxx, syy, sxy] = [&self.tables[0], &self.tables[1], &self.tables[2]].map(sum);
        (sxx + syy) / 2.0 - (((sxx - syy) / 2.0).powi(2) + sxy * sxy).sqrt()
    }
}

/// Settings of *Align* with the defaults of the reference program (`docs/ui.md`).
#[derive(Debug, Clone, PartialEq)]
pub struct AlignParams {
    /// *Alignmentbox size*: edge of the square that is matched around each point.
    pub box_size: u32,
    /// *Max Alignpoint movement*: how far a point may move against the reference, in pixels.
    pub max_movement: u32,
}

impl Default for AlignParams {
    fn default() -> Self {
        Self {
            box_size: 30,
            max_movement: 5,
        }
    }
}

/// The zero-mean patch around a point in the reference frame.
struct Template {
    x0: usize,
    y0: usize,
    data: Vec<f32>,
    norm: f32,
}

/// The reference frame reduced to what is needed to register other frames: one template per point.
pub struct Reference {
    params: AlignParams,
    width: u32,
    height: u32,
    points: Vec<AlignPoint>,
    /// `None` where the box does not fit into the frame or the patch is flat.
    templates: Vec<Option<Template>>,
    /// The whole frame at coarse scales, for the global shift.
    coarse: Vec<Level>,
}

/// A frame drifts between exposures. The drift is found at coarse scales first, so that the points
/// only have to search a few pixels. This is the largest drift that is followed, in pixels.
const MAX_DRIFT: i32 = 64;
/// Reduction factors of the coarse search, coarsest first.
const COARSE_FACTORS: [usize; 2] = [8, 2];

/// A luminance image reduced by `factor`.
struct Level {
    factor: usize,
    width: usize,
    height: usize,
    data: Vec<f32>,
    /// The cells `(x0, y0, x1, y1)` that the shift search compares: the region of interest, or the
    /// whole level.
    region: (i32, i32, i32, i32),
}

impl Level {
    fn new(luma: &Image, factor: usize, roi: Option<Roi>) -> Self {
        let (w, h) = (luma.width as usize, luma.height as usize);
        let (cw, ch) = (w / factor, h / factor);
        let mut data = vec![0.0; cw * ch];
        for y in 0..ch {
            for x in 0..cw {
                let mut sum = 0.0;
                for yy in y * factor..(y + 1) * factor {
                    sum += luma.data[yy * w + x * factor..yy * w + (x + 1) * factor]
                        .iter()
                        .sum::<f32>();
                }
                data[y * cw + x] = sum / (factor * factor) as f32;
            }
        }
        let region = match roi {
            Some(r) => (
                (r.x as usize / factor) as i32,
                (r.y as usize / factor) as i32,
                ((r.x + r.width) as usize).div_ceil(factor).min(cw) as i32,
                ((r.y + r.height) as usize).div_ceil(factor).min(ch) as i32,
            ),
            None => (0, 0, cw as i32, ch as i32),
        };
        Self {
            factor,
            width: cw,
            height: ch,
            data,
            region,
        }
    }
}

/// The integer shift `(dx, dy)` within `radius` of `centre` for which `img` shifted by it matches
/// `reference` best, by normalised correlation over the overlap inside the reference's region.
fn best_shift(reference: &Level, img: &Level, centre: (i32, i32), radius: i32) -> (i32, i32) {
    let w = reference.width as i32;
    let (rx0, ry0, rx1, ry1) = reference.region;
    let mut best = (f32::NEG_INFINITY, centre);
    for dy in centre.1 - radius..=centre.1 + radius {
        for dx in centre.0 - radius..=centre.0 + radius {
            let (x0, x1) = (rx0.max(-dx), rx1.min(w - dx));
            let (y0, y1) = (ry0.max(-dy), ry1.min(reference.height as i32 - dy));
            if x1 <= x0 || y1 <= y0 || (x1 - x0) * (y1 - y0) < (rx1 - rx0) * (ry1 - ry0) / 4 {
                continue;
            }
            // Sums of many similar values cancel badly in f32 when the variance is taken from them.
            let (mut sa, mut sb, mut saa, mut sbb, mut sab) =
                (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
            for y in y0..y1 {
                let ra = &reference.data[(y * w + x0) as usize..(y * w + x1) as usize];
                let rb =
                    &img.data[((y + dy) * w + x0 + dx) as usize..((y + dy) * w + x1 + dx) as usize];
                for (&a, &b) in ra.iter().zip(rb) {
                    let (a, b) = (f64::from(a), f64::from(b));
                    sa += a;
                    sb += b;
                    saa += a * a;
                    sbb += b * b;
                    sab += a * b;
                }
            }
            let n = f64::from((x1 - x0) * (y1 - y0));
            let (va, vb) = (saa - sa * sa / n, sbb - sb * sb / n);
            if va > 1e-9 && vb > 1e-9 {
                let ncc = ((sab - sa * sb / n) / (va * vb).sqrt()) as f32;
                if ncc > best.0 {
                    best = (ncc, (dx, dy));
                }
            }
        }
    }
    best.1
}

impl Reference {
    pub fn new(frame: &Image, points: &[AlignPoint], params: &AlignParams) -> Self {
        Self::with_roi(frame, points, params, None)
    }

    /// Like [`Reference::new`], but the drift between frames is measured inside `roi` only, so that
    /// parts of the frame outside it (a moon, the frame border, hot pixels) do not matter. The
    /// points should lie inside the region as well, see [`PointParams::roi`].
    pub fn with_roi(
        frame: &Image,
        points: &[AlignPoint],
        params: &AlignParams,
        roi: Option<Roi>,
    ) -> Self {
        let luma = frame.to_luma();
        let roi = roi.and_then(|r| r.clamp(frame.width, frame.height));
        let (w, h) = (luma.width as usize, luma.height as usize);
        let size = params.box_size as usize;
        let templates = points
            .iter()
            .map(|p| {
                let x0 = (p.x as usize).checked_sub(size / 2)?;
                let y0 = (p.y as usize).checked_sub(size / 2)?;
                if x0 + size > w || y0 + size > h {
                    return None;
                }
                let mut data = patch(&luma.data, w, x0, y0, size);
                let mean = data.iter().map(|&v| f64::from(v)).sum::<f64>() / data.len() as f64;
                data.iter_mut()
                    .for_each(|v| *v = (f64::from(*v) - mean) as f32);
                let norm = data
                    .iter()
                    .map(|&v| f64::from(v).powi(2))
                    .sum::<f64>()
                    .sqrt() as f32;
                // A patch without structure only holds noise, nothing to track.
                (norm > 1e-3).then_some(Template { x0, y0, data, norm })
            })
            .collect();
        Self {
            params: params.clone(),
            width: frame.width,
            height: frame.height,
            points: points.to_vec(),
            templates,
            coarse: COARSE_FACTORS
                .iter()
                .filter(|&&f| {
                    let (rw, rh) = roi.map_or((w, h), |r| (r.width as usize, r.height as usize));
                    rw / f >= 16 && rh / f >= 16
                })
                .map(|&f| Level::new(&luma, f, roi))
                .collect(),
        }
    }

    /// How far `luma` has drifted against the reference, in whole pixels.
    fn global_shift(&self, luma: &Image) -> (i32, i32) {
        let mut shift = (0, 0);
        let mut radius = MAX_DRIFT / COARSE_FACTORS[0] as i32;
        for level in &self.coarse {
            let img = Level::new(luma, level.factor, None);
            let f = level.factor as i32;
            let found = best_shift(level, &img, (shift.0 / f, shift.1 / f), radius);
            shift = (found.0 * f, found.1 * f);
            radius = 3;
        }
        shift
    }

    pub fn points(&self) -> &[AlignPoint] {
        &self.points
    }
}

fn patch(luma: &[f32], w: usize, x0: usize, y0: usize, size: usize) -> Vec<f32> {
    (y0..y0 + size)
        .flat_map(|y| luma[y * w + x0..y * w + x0 + size].iter().copied())
        .collect()
}

/// Where one point of the reference was found in another frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointShift {
    pub dx: f32,
    pub dy: f32,
    /// Normalised correlation of the best match, `-1.0..=1.0`.
    pub score: f32,
    /// The best match lies on the border of the search range, the true movement may be larger.
    pub at_limit: bool,
    /// Sharpness of the box around the point in this frame, larger is better; see
    /// [`FrameAlignment::quality`]. Used by the limit mode *Frames/Apoint*.
    pub quality: f32,
}

/// Result of registering one frame.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameAlignment {
    /// Median movement of all points, the shift of the frame as a whole.
    pub shift: (f32, f32),
    /// One entry per point of the reference; `None` where it could not be tracked.
    pub points: Vec<Option<PointShift>>,
    /// Sharpness of the frame, larger is better: the mean of the quality of its tracked points. Only
    /// the order and the ratios matter.
    pub quality: f32,
    /// Mean distance of the points from the frame shift, in pixels: how much the picture is distorted.
    pub difference: f32,
}

impl Reference {
    /// Finds every point of the reference in `frame` and rates the frame.
    pub fn align_frame(&self, frame: &Image) -> FrameAlignment {
        let luma = frame.to_luma();
        let same_size = frame.width == self.width && frame.height == self.height;
        let drift = if same_size {
            self.global_shift(&luma)
        } else {
            (0, 0)
        };
        let mut matches: Vec<Option<PointShift>> = if same_size {
            self.templates
                .iter()
                .map(|t| t.as_ref().and_then(|t| self.match_point(t, &luma, drift)))
                .collect()
        } else {
            vec![None; self.points.len()]
        };

        let shift = (
            median(matches.iter().flatten().map(|m| m.dx)),
            median(matches.iter().flatten().map(|m| m.dy)),
        );
        let difference = {
            let (sum, n) = matches.iter().flatten().fold((0.0, 0), |(s, n), m| {
                (s + (m.dx - shift.0).hypot(m.dy - shift.1), n + 1)
            });
            if n > 0 { sum / n as f32 } else { 0.0 }
        };
        FrameAlignment {
            shift,
            quality: self.rate_points(&luma, &mut matches),
            points: matches,
            difference,
        }
    }

    /// Searches `template` within the allowed movement and refines the best integer position to
    /// sub-pixel accuracy with a parabola through the correlation values around the peak.
    fn match_point(&self, t: &Template, frame: &Image, drift: (i32, i32)) -> Option<PointShift> {
        let (luma, w, h) = (&frame.data, frame.width as isize, frame.height as isize);
        let size = self.params.box_size as usize;
        let reach = self.params.max_movement as isize;
        // The whole search window has to lie inside the frame.
        let (cx, cy) = (
            t.x0 as isize + drift.0 as isize,
            t.y0 as isize + drift.1 as isize,
        );
        if cx < reach
            || cy < reach
            || cx + size as isize + reach > w
            || cy + size as isize + reach > h
        {
            return None;
        }
        let w = w as usize;
        let side = (2 * reach + 1) as usize;
        let mut surface = vec![f32::NEG_INFINITY; side * side];
        let n = (size * size) as f64;
        for dy in -reach..=reach {
            for dx in -reach..=reach {
                let x0 = (cx + dx) as usize;
                let y0 = (cy + dy) as usize;
                let (mut sum, mut sum_sq, mut dot) = (0.0f64, 0.0f64, 0.0f64);
                for row in 0..size {
                    let line = &luma[(y0 + row) * w + x0..(y0 + row) * w + x0 + size];
                    let tpl = &t.data[row * size..(row + 1) * size];
                    for (&f, &g) in line.iter().zip(tpl) {
                        let f = f64::from(f);
                        sum += f;
                        sum_sq += f * f;
                        dot += f * f64::from(g);
                    }
                }
                let var = sum_sq - sum * sum / n;
                if var > 1e-6 {
                    surface[(dy + reach) as usize * side + (dx + reach) as usize] =
                        (dot / (f64::from(t.norm) * var.sqrt())) as f32;
                }
            }
        }
        let (best, &score) = surface
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))?;
        if !score.is_finite() {
            return None;
        }
        let (bx, by) = (best % side, best / side);
        let at_limit = bx == 0 || by == 0 || bx == side - 1 || by == side - 1;
        let refine = |lo: f32, mid: f32, hi: f32| {
            let denom = lo - 2.0 * mid + hi;
            if denom < -1e-9 {
                ((lo - hi) / (2.0 * denom)).clamp(-0.5, 0.5)
            } else {
                0.0
            }
        };
        let (mut fx, mut fy) = (0.0, 0.0);
        if !at_limit {
            let s = |x: usize, y: usize| surface[y * side + x];
            if [s(bx - 1, by), s(bx + 1, by), s(bx, by - 1), s(bx, by + 1)]
                .iter()
                .all(|v| v.is_finite())
            {
                fx = refine(s(bx - 1, by), score, s(bx + 1, by));
                fy = refine(s(bx, by - 1), score, s(bx, by + 1));
            }
        }
        Some(PointShift {
            dx: (bx as isize - reach + drift.0 as isize) as f32 + fx,
            dy: (by as isize - reach + drift.1 as isize) as f32 + fy,
            score,
            at_limit,
            quality: 0.0,
        })
    }

    /// Rates every tracked point and returns the quality of the frame, the mean of those.
    ///
    /// The quality of a point is the mean gradient magnitude (central differences) inside its box at
    /// the position where it was found, relative to the mean brightness of the frame so that
    /// exposure does not matter. Compared with the order of the original on the reference case this
    /// is closer than the squared gradient used before (Spearman 0.79 against 0.54), but still not
    /// the same measure (`docs/offene-fragen.md`, question 17).
    fn rate_points(&self, luma: &Image, matches: &mut [Option<PointShift>]) -> f32 {
        let (w, h) = (luma.width as isize, luma.height as isize);
        let half = (self.params.box_size / 2) as isize;
        let mean = luma.data.iter().sum::<f32>() / luma.data.len().max(1) as f32;
        let (mut total, mut count) = (0.0f64, 0u32);
        for (p, m) in self.points.iter().zip(matches.iter_mut()) {
            let Some(m) = m else { continue };
            let cx = p.x as isize + m.dx.round() as isize;
            let cy = p.y as isize + m.dy.round() as isize;
            let (mut sum, mut n) = (0.0f64, 0u64);
            for y in (cy - half).max(1)..(cy + half).min(h - 1) {
                for x in (cx - half).max(1)..(cx + half).min(w - 1) {
                    let at = |xx: isize, yy: isize| luma.data[(yy * w + xx) as usize];
                    let gx = (at(x + 1, y) - at(x - 1, y)) / 2.0;
                    let gy = (at(x, y + 1) - at(x, y - 1)) / 2.0;
                    sum += f64::from(gx.hypot(gy));
                    n += 1;
                }
            }
            m.quality = if n == 0 || mean <= 0.0 {
                0.0
            } else {
                (sum / n as f64) as f32 / mean
            };
            total += f64::from(m.quality);
            count += 1;
        }
        if count == 0 {
            0.0
        } else {
            (total / f64::from(count)) as f32
        }
    }
}

fn median(values: impl Iterator<Item = f32>) -> f32 {
    let mut v: Vec<f32> = values.collect();
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(f32::total_cmp);
    let mid = v.len() / 2;
    if v.len() % 2 == 1 {
        v[mid]
    } else {
        (v[mid - 1] + v[mid]) / 2.0
    }
}

/// What the *Limit* step keeps, after the frames are ordered by quality.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LimitMode {
    /// *Lowest Quality (%)*: frames whose quality is at least this share of the best frame's.
    LowestQuality(f32),
    /// *Best Frames (%)*: this share of the frames, the best ones.
    BestPercent(f32),
    /// *Best Frames*: this many frames, the best ones.
    BestCount(usize),
    /// *Frames/Apoint*: every point keeps its own best frames, this many. The frames that stay are
    /// all frames that at least one point keeps; see [`best_frames_per_point`] for who keeps which.
    /// Conjecture from the name and from the count field being enabled for this mode only together
    /// with *Best Frames*, not observed in the original.
    FramesPerPoint(usize),
}

/// Frame indices from best to worst quality. Equal quality keeps the frame order.
pub fn rank_by_quality(results: &[FrameAlignment]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..results.len()).collect();
    order.sort_by(|&a, &b| results[b].quality.total_cmp(&results[a].quality));
    order
}

/// For every point of the reference the `count` frames in which that point is sharpest (at least
/// one), best first. A point that was tracked in no frame gets an empty list.
pub fn best_frames_per_point(results: &[FrameAlignment], count: usize) -> Vec<Vec<usize>> {
    let point_count = results.iter().map(|r| r.points.len()).max().unwrap_or(0);
    let quality_at = |frame: usize, point: usize| {
        results[frame]
            .points
            .get(point)
            .copied()
            .flatten()
            .map(|m| m.quality)
    };
    (0..point_count)
        .map(|point| {
            let mut frames: Vec<usize> = (0..results.len())
                .filter(|&f| quality_at(f, point).is_some())
                .collect();
            frames.sort_by(|&a, &b| {
                let (qa, qb) = (quality_at(a, point), quality_at(b, point));
                qb.partial_cmp(&qa).unwrap_or(std::cmp::Ordering::Equal)
            });
            frames.truncate(count.max(1));
            frames
        })
        .collect()
}

/// The frames that stay after *Limit*, best first. At least one frame stays if there is any.
pub fn limit_frames(results: &[FrameAlignment], mode: LimitMode) -> Vec<usize> {
    let mut order = rank_by_quality(results);
    let keep = match mode {
        LimitMode::LowestQuality(percent) => {
            let best = order.first().map_or(0.0, |&i| results[i].quality);
            let floor = best * percent / 100.0;
            order
                .iter()
                .take_while(|&&i| results[i].quality >= floor)
                .count()
        }
        LimitMode::BestPercent(percent) => (results.len() as f32 * percent / 100.0).ceil() as usize,
        LimitMode::BestCount(count) => count,
        LimitMode::FramesPerPoint(count) => {
            let mut used = vec![false; results.len()];
            for frame in best_frames_per_point(results, count).into_iter().flatten() {
                used[frame] = true;
            }
            order.retain(|&i| used[i]);
            order.len()
        }
    };
    if order.is_empty() {
        // No point was tracked anywhere, so nothing was kept: fall back to the best frame.
        order = rank_by_quality(results);
    }
    order.truncate(keep.max(1));
    order
}

/// Why [`align_sequence`] stopped early.
#[derive(Debug, PartialEq, Eq)]
pub enum AlignError<E> {
    /// Reading a frame failed.
    Read(E),
    /// The progress callback asked to stop.
    Cancelled,
}

/// Whether [`align_sequence`] goes on after a batch of frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Control {
    Continue,
    Cancel,
}

/// Registers all frames against `reference`. Frames are read one after the other (`read` may borrow a
/// file) but aligned in parallel, a batch at a time, so that only a few frames are in memory.
/// `progress` receives the number of finished frames after each batch.
pub fn align_sequence<E>(
    reference: &Reference,
    frame_count: u32,
    mut read: impl FnMut(u32) -> Result<Image, E>,
    mut progress: impl FnMut(u32) -> Control,
) -> Result<Vec<FrameAlignment>, AlignError<E>> {
    let batch = (rayon::current_num_threads() * 2).max(1) as u32;
    let mut results = Vec::with_capacity(frame_count as usize);
    let mut next = 0;
    while next < frame_count {
        let end = (next + batch).min(frame_count);
        let frames = (next..end)
            .map(&mut read)
            .collect::<Result<Vec<_>, _>>()
            .map_err(AlignError::Read)?;
        results.extend(
            frames
                .par_iter()
                .map(|f| reference.align_frame(f))
                .collect::<Vec<_>>(),
        );
        next = end;
        if progress(next) == Control::Cancel {
            return Err(AlignError::Cancelled);
        }
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smooth blobs of deterministic pseudo-random position, as a stand-in for a planetary surface.
    /// `shift` moves the content, so that a frame shifted by `(sx, sy)` is the exact sub-pixel
    /// resampling of the unshifted one. `blur` widens every blob.
    fn blobs(width: u32, height: u32, shift: (f32, f32), blur: f32) -> Image {
        let mut seed = 12345u32;
        let mut rand = move || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / (1u32 << 24) as f32
        };
        let list: Vec<(f32, f32, f32, f32)> = (0..60)
            .map(|_| {
                (
                    rand() * width as f32,
                    rand() * height as f32,
                    3.0 + rand() * 4.0,
                    0.2 + rand() * 0.3,
                )
            })
            .collect();
        let mut img = Image::new(width, height, 1);
        for y in 0..height {
            for x in 0..width {
                let (px, py) = (x as f32 - shift.0, y as f32 - shift.1);
                let v = 0.2
                    + list
                        .iter()
                        .map(|&(cx, cy, s, a)| {
                            let s = s + blur;
                            let d2 = (px - cx).powi(2) + (py - cy).powi(2);
                            a * 3.0 / s * (-d2 / (2.0 * s * s)).exp()
                        })
                        .sum::<f32>();
                img.data[(y * width + x) as usize] = v.min(0.9);
            }
        }
        img
    }

    fn reference_of(frame: &Image) -> Reference {
        let points = find_alignpoints(frame, &PointParams::default());
        Reference::new(frame, &points, &AlignParams::default())
    }

    #[test]
    fn points_keep_distance_edge_and_intensity_range() {
        let frame = blobs(300, 240, (0.0, 0.0), 0.0);
        let params = PointParams::default();
        let points = find_alignpoints(&frame, &params);
        assert!(points.len() > 10);
        for (i, a) in points.iter().enumerate() {
            assert!(a.x >= params.edge_distance && a.x < 300 - params.edge_distance);
            assert!(a.y >= params.edge_distance && a.y < 240 - params.edge_distance);
            let value = frame.data[(a.y * 300 + a.x) as usize] * 255.0;
            assert!((params.lo..=params.hi).contains(&value), "value {value}");
            for b in &points[i + 1..] {
                let apart = a.x.abs_diff(b.x).max(a.y.abs_diff(b.y));
                assert!(
                    apart >= params.min_distance + params.max_movement,
                    "points {a:?} and {b:?} are {apart} apart"
                );
            }
        }
        assert!(points.windows(2).all(|w| w[0].strength >= w[1].strength));
    }

    #[test]
    fn intensity_range_excludes_too_bright_pixels() {
        let mut frame = blobs(300, 240, (0.0, 0.0), 0.0);
        // Everything left of x = 150 becomes too bright for Hi = 230.
        for y in 0..240usize {
            for x in 0..150usize {
                frame.data[y * 300 + x] = 1.0;
            }
        }
        let points = find_alignpoints(&frame, &PointParams::default());
        assert!(!points.is_empty());
        assert!(points.iter().all(|p| p.x >= 150), "{points:?}");
    }

    /// A bright quadrant on a dark field: a corner at (100, 100), or only the vertical edge
    /// at x = 100 when `edge_only` is set.
    fn quadrant(edge_only: bool) -> Image {
        let mut frame = Image::new(200, 200, 1);
        for y in 0..200usize {
            for x in 0..200usize {
                let bright = x >= 100 && (edge_only || y >= 100);
                frame.data[y * 200 + x] = if bright { 0.7 } else { 0.3 };
            }
        }
        frame
    }

    #[test]
    fn a_straight_edge_is_weaker_than_a_corner() {
        // A straight edge lets a match slide along it; only the corner pins the point down.
        let params = PointParams {
            min_distance: 60,
            ..PointParams::default()
        };
        let corner = find_alignpoints(&quadrant(false), &params)[0];
        let edge = find_alignpoints(&quadrant(true), &params)[0];
        assert!(
            corner.x.abs_diff(100) <= 15 && corner.y.abs_diff(100) <= 15,
            "{corner:?}"
        );
        assert!(
            edge.strength < corner.strength * 0.01,
            "{edge:?} vs {corner:?}"
        );
    }

    #[test]
    fn threshold_drops_the_weakest_points() {
        let points: Vec<AlignPoint> = (0..10)
            .map(|i| AlignPoint {
                x: i,
                y: 0,
                strength: 10.0 - i as f32,
            })
            .collect();
        let kept = apply_threshold(&points, 3);
        assert_eq!(kept.len(), 7);
        assert_eq!(kept.last().map(|p| p.x), Some(6));
        assert!(apply_threshold(&points, 99).is_empty());
        assert_eq!(default_threshold(91), 9);
        assert_eq!(default_threshold(100), 10);
        assert_eq!(default_threshold(0), 0);
    }

    #[test]
    fn sub_pixel_shift_beyond_the_point_search_range_is_recovered() {
        let reference = reference_of(&blobs(300, 240, (0.0, 0.0), 0.0));
        let truth = (21.4, -12.3);
        let result = reference.align_frame(&blobs(300, 240, truth, 0.0));
        assert!((result.shift.0 - truth.0).abs() < 0.2, "{:?}", result.shift);
        assert!((result.shift.1 - truth.1).abs() < 0.2, "{:?}", result.shift);
        let tracked = result.points.iter().flatten().count();
        assert!(tracked * 2 > result.points.len(), "tracked {tracked}");
        // Parabolic peak fitting is biased towards whole pixels, so single points scatter a little.
        assert!(result.difference < 0.5, "difference {}", result.difference);
    }

    #[test]
    fn unshifted_frame_has_no_shift_and_perfect_correlation() {
        let frame = blobs(300, 240, (0.0, 0.0), 0.0);
        let result = reference_of(&frame).align_frame(&frame);
        assert!(
            result.shift.0.abs() < 0.01 && result.shift.1.abs() < 0.01,
            "{:?}",
            result.shift
        );
        for m in result.points.iter().flatten() {
            assert!(m.score > 0.999 && !m.at_limit, "{m:?}");
        }
    }

    #[test]
    fn frame_of_other_size_tracks_nothing() {
        let reference = reference_of(&blobs(300, 240, (0.0, 0.0), 0.0));
        let result = reference.align_frame(&blobs(200, 200, (0.0, 0.0), 0.0));
        assert!(result.points.iter().all(Option::is_none));
    }

    #[test]
    fn blurred_frame_has_lower_quality() {
        let sharp = blobs(300, 240, (0.0, 0.0), 0.0);
        let reference = reference_of(&sharp);
        let q_sharp = reference.align_frame(&sharp).quality;
        let q_soft = reference
            .align_frame(&blobs(300, 240, (0.0, 0.0), 2.0))
            .quality;
        assert!(q_sharp > q_soft * 1.05, "{q_sharp} vs {q_soft}");
    }

    #[test]
    fn quality_does_not_depend_on_exposure() {
        let sharp = blobs(300, 240, (0.0, 0.0), 0.0);
        let reference = reference_of(&sharp);
        let mut dim = sharp.clone();
        dim.data.iter_mut().for_each(|v| *v *= 0.5);
        let a = reference.align_frame(&sharp).quality;
        let b = reference.align_frame(&dim).quality;
        assert!((a - b).abs() < a * 0.01, "{a} vs {b}");
    }

    fn rated(qualities: &[f32]) -> Vec<FrameAlignment> {
        qualities
            .iter()
            .map(|&quality| FrameAlignment {
                shift: (0.0, 0.0),
                points: Vec::new(),
                quality,
                difference: 0.0,
            })
            .collect()
    }

    #[test]
    fn frames_are_ranked_best_first_and_ties_keep_frame_order() {
        let results = rated(&[1.0, 4.0, 2.0, 4.0]);
        assert_eq!(rank_by_quality(&results), vec![1, 3, 2, 0]);
    }

    #[test]
    fn limit_by_lowest_quality_keeps_frames_close_to_the_best() {
        let results = rated(&[10.0, 8.0, 7.9, 5.0]);
        assert_eq!(
            limit_frames(&results, LimitMode::LowestQuality(80.0)),
            vec![0, 1]
        );
        assert_eq!(
            limit_frames(&results, LimitMode::LowestQuality(0.0)).len(),
            4
        );
        assert_eq!(
            limit_frames(&results, LimitMode::LowestQuality(100.0)),
            vec![0]
        );
    }

    #[test]
    fn limit_by_percent_and_count_keeps_the_best_frames() {
        let results = rated(&[3.0, 9.0, 1.0, 7.0]);
        assert_eq!(
            limit_frames(&results, LimitMode::BestPercent(50.0)),
            vec![1, 3]
        );
        assert_eq!(
            limit_frames(&results, LimitMode::BestPercent(30.0)),
            vec![1, 3]
        );
        assert_eq!(
            limit_frames(&results, LimitMode::BestCount(3)),
            vec![1, 3, 0]
        );
        assert_eq!(limit_frames(&results, LimitMode::BestCount(99)).len(), 4);
        // Never limits to nothing.
        assert_eq!(limit_frames(&results, LimitMode::BestCount(0)), vec![1]);
        assert!(limit_frames(&[], LimitMode::BestCount(3)).is_empty());
    }

    #[test]
    fn sequence_is_aligned_in_frame_order_with_progress() {
        let reference = reference_of(&blobs(300, 240, (0.0, 0.0), 0.0));
        let mut progress = Vec::new();
        let results = align_sequence::<()>(
            &reference,
            9,
            |i| Ok(blobs(300, 240, (i as f32, 0.0), 0.0)),
            |done| {
                progress.push(done);
                Control::Continue
            },
        )
        .unwrap();
        assert_eq!(results.len(), 9);
        for (i, r) in results.iter().enumerate() {
            assert!(
                (r.shift.0 - i as f32).abs() < 0.2,
                "frame {i}: {:?}",
                r.shift
            );
        }
        assert_eq!(progress.last(), Some(&9));
        assert!(progress.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn sequence_stops_when_cancelled_and_on_read_errors() {
        let reference = reference_of(&blobs(300, 240, (0.0, 0.0), 0.0));
        let frame = blobs(300, 240, (0.0, 0.0), 0.0);
        let cancelled =
            align_sequence::<()>(&reference, 50, |_| Ok(frame.clone()), |_| Control::Cancel);
        assert_eq!(cancelled.err(), Some(AlignError::Cancelled));
        let failed = align_sequence(
            &reference,
            5,
            |i| {
                if i == 2 {
                    Err("broken")
                } else {
                    Ok(frame.clone())
                }
            },
            |_| Control::Continue,
        );
        assert_eq!(failed.err(), Some(AlignError::Read("broken")));
    }

    #[test]
    fn points_stay_inside_the_region_of_interest() {
        let frame = blobs(360, 240, (0.0, 0.0), 0.0);
        let roi = Roi {
            x: 40,
            y: 30,
            width: 150,
            height: 120,
        };
        let params = PointParams {
            roi: Some(roi),
            ..PointParams::default()
        };
        let points = find_alignpoints(&frame, &params);
        assert!(points.len() >= 4, "{} points", points.len());
        for p in &points {
            // The edge distance counts from the edges of the region.
            assert!(p.x >= roi.x + params.edge_distance, "{p:?}");
            assert!(p.x < roi.x + roi.width - params.edge_distance, "{p:?}");
            assert!(p.y >= roi.y + params.edge_distance, "{p:?}");
            assert!(p.y < roi.y + roi.height - params.edge_distance, "{p:?}");
        }
    }

    #[test]
    fn region_of_interest_is_clamped_to_the_frame() {
        let roi = Roi {
            x: 200,
            y: 100,
            width: 1000,
            height: 1000,
        };
        assert_eq!(
            roi.clamp(300, 240),
            Some(Roi {
                x: 200,
                y: 100,
                width: 100,
                height: 140
            })
        );
        assert_eq!(Roi { x: 300, ..roi }.clamp(300, 240), None);

        let frame = blobs(300, 240, (0.0, 0.0), 0.0);
        let outside = PointParams {
            roi: Some(Roi { x: 300, ..roi }),
            ..PointParams::default()
        };
        assert!(find_alignpoints(&frame, &outside).is_empty());
        // Smaller than twice the edge distance: no room for a point.
        let tiny = PointParams {
            roi: Some(Roi {
                x: 100,
                y: 100,
                width: 39,
                height: 100,
            }),
            ..PointParams::default()
        };
        assert!(find_alignpoints(&frame, &tiny).is_empty());
    }

    #[test]
    fn points_keep_the_movement_range_between_their_boxes() {
        let frame = blobs(300, 240, (0.0, 0.0), 0.0);
        let params = PointParams {
            max_movement: 15,
            ..PointParams::default()
        };
        let points = find_alignpoints(&frame, &params);
        assert!(points.len() > 5);
        for (i, a) in points.iter().enumerate() {
            for b in &points[i + 1..] {
                assert!(
                    a.x.abs_diff(b.x).max(a.y.abs_diff(b.y)) >= 45,
                    "{a:?} {b:?}"
                );
            }
        }
    }

    /// The picture left of `split` moved by `shift`, the rest stayed.
    fn partly_moved(width: u32, height: u32, shift: (f32, f32), split: usize) -> Image {
        let moved = blobs(width, height, shift, 0.0);
        let mut img = blobs(width, height, (0.0, 0.0), 0.0);
        for y in 0..height as usize {
            let row = y * width as usize;
            img.data[row..row + split].copy_from_slice(&moved.data[row..row + split]);
        }
        img
    }

    #[test]
    fn drift_is_measured_inside_the_region_of_interest() {
        let still = blobs(360, 240, (0.0, 0.0), 0.0);
        let roi = Roi {
            x: 0,
            y: 0,
            width: 150,
            height: 240,
        };
        let points = find_alignpoints(
            &still,
            &PointParams {
                roi: Some(roi),
                ..PointParams::default()
            },
        );
        assert!(points.len() >= 5, "{} points", points.len());
        let reference = Reference::with_roi(&still, &points, &AlignParams::default(), Some(roi));
        // Left of x = 170 the picture moved by (10, 4); the larger part to the right stayed. Over
        // the whole frame that part wins and the points would be looked for 10 px away, beyond the
        // 5 px they may move.
        let result = reference.align_frame(&partly_moved(360, 240, (10.0, 4.0), 170));
        assert!((result.shift.0 - 10.0).abs() < 0.3, "{:?}", result.shift);
        assert!((result.shift.1 - 4.0).abs() < 0.3, "{:?}", result.shift);
        let tracked = result.points.iter().flatten().count();
        assert!(tracked * 2 > result.points.len(), "tracked {tracked}");
    }

    #[test]
    fn frame_quality_is_the_mean_of_its_point_qualities() {
        let frame = blobs(300, 240, (0.0, 0.0), 0.0);
        let result = reference_of(&frame).align_frame(&frame);
        let qualities: Vec<f32> = result.points.iter().flatten().map(|m| m.quality).collect();
        assert!(!qualities.is_empty() && qualities.iter().all(|&q| q > 0.0));
        let mean = qualities.iter().sum::<f32>() / qualities.len() as f32;
        assert!(
            (result.quality - mean).abs() < mean * 1e-4,
            "{} vs {mean}",
            result.quality
        );
    }

    /// A frame with a given quality whose points have the given qualities (`None`: not tracked).
    fn with_points(quality: f32, points: &[Option<f32>]) -> FrameAlignment {
        FrameAlignment {
            shift: (0.0, 0.0),
            points: points
                .iter()
                .map(|q| {
                    q.map(|quality| PointShift {
                        dx: 0.0,
                        dy: 0.0,
                        score: 1.0,
                        at_limit: false,
                        quality,
                    })
                })
                .collect(),
            quality,
            difference: 0.0,
        }
    }

    fn per_point_fixture() -> Vec<FrameAlignment> {
        vec![
            with_points(5.0, &[Some(1.0), Some(9.0)]),
            with_points(3.5, &[Some(5.0), Some(2.0)]),
            with_points(7.0, &[Some(7.0), None]),
            with_points(5.5, &[Some(3.0), Some(8.0)]),
        ]
    }

    #[test]
    fn every_point_keeps_the_frames_where_it_is_sharpest() {
        let results = per_point_fixture();
        // Point 0: frames 2 (7), 1 (5), 3 (3), 0 (1). Point 1: frames 0 (9), 3 (8), 1 (2); frame 2
        // did not track it.
        assert_eq!(
            best_frames_per_point(&results, 2),
            vec![vec![2, 1], vec![0, 3]]
        );
        assert_eq!(
            best_frames_per_point(&results, 9),
            vec![vec![2, 1, 3, 0], vec![0, 3, 1]]
        );
        // At least one frame per point.
        assert_eq!(best_frames_per_point(&results, 0), vec![vec![2], vec![0]]);
    }

    #[test]
    fn limit_by_frames_per_point_keeps_the_union_ordered_by_frame_quality() {
        let results = per_point_fixture();
        assert_eq!(
            limit_frames(&results, LimitMode::FramesPerPoint(2)),
            vec![2, 3, 0, 1]
        );
        // One frame per point: frame 2 for point 0 and frame 0 for point 1.
        assert_eq!(
            limit_frames(&results, LimitMode::FramesPerPoint(1)),
            vec![2, 0]
        );
    }

    #[test]
    fn limit_by_frames_per_point_without_tracked_points_keeps_the_best_frame() {
        let results = vec![with_points(1.0, &[None]), with_points(2.0, &[None])];
        assert_eq!(
            best_frames_per_point(&results, 3),
            vec![Vec::<usize>::new()]
        );
        assert_eq!(
            limit_frames(&results, LimitMode::FramesPerPoint(3)),
            vec![1]
        );
        assert!(limit_frames(&[], LimitMode::FramesPerPoint(3)).is_empty());
    }
}
