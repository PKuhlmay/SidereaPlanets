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

/// Settings of *Set Alignpoints* with the defaults of the reference program (`docs/ui.md`).
#[derive(Debug, Clone, PartialEq)]
pub struct PointParams {
    /// *Minimum distance between* two points, in pixels.
    pub min_distance: u32,
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
pub fn find_alignpoints(frame: &Image, params: &PointParams) -> Vec<AlignPoint> {
    let luma = frame.to_luma();
    let (w, h) = (luma.width as usize, luma.height as usize);
    let edge = params.edge_distance as usize;
    if w <= 2 * edge || h <= 2 * edge || w < 3 || h < 3 {
        return Vec::new();
    }
    let structure = StructureSums::new(&luma.data, w, h);
    let half = (params.box_size / 2) as usize;
    let (lo, hi) = (params.lo / 255.0, params.hi / 255.0);

    let mut candidates: Vec<AlignPoint> = Vec::new();
    for y in (edge..h - edge).step_by(SCAN_STEP) {
        for x in (edge..w - edge).step_by(SCAN_STEP) {
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

    let min_sq = i64::from(params.min_distance).pow(2);
    let mut chosen: Vec<AlignPoint> = Vec::new();
    for c in candidates {
        let far_enough = chosen.iter().all(|p| {
            let dx = i64::from(p.x) - i64::from(c.x);
            let dy = i64::from(p.y) - i64::from(c.y);
            dx * dx + dy * dy >= min_sq
        });
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
/// 91 points of which it keeps 82 for `JUPITER.AVI`; our spacing rule finds more points on the same
/// frame, and dropping 43 % of them lands at the same count. Fitted to that one case, unconfirmed.
const DEFAULT_DROP_SHARE: f32 = 0.43;

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
}

impl Level {
    fn new(luma: &Image, factor: usize) -> Self {
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
        Self {
            factor,
            width: cw,
            height: ch,
            data,
        }
    }
}

/// The integer shift `(dx, dy)` within `radius` of `centre` for which `img` shifted by it matches
/// `reference` best, by normalised correlation over the overlap.
fn best_shift(reference: &Level, img: &Level, centre: (i32, i32), radius: i32) -> (i32, i32) {
    let (w, h) = (reference.width as i32, reference.height as i32);
    let mut best = (f32::NEG_INFINITY, centre);
    for dy in centre.1 - radius..=centre.1 + radius {
        for dx in centre.0 - radius..=centre.0 + radius {
            let (x0, x1) = (0.max(-dx), w.min(w - dx));
            let (y0, y1) = (0.max(-dy), h.min(h - dy));
            if (x1 - x0) * (y1 - y0) < w * h / 4 {
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
        let luma = frame.to_luma();
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
                .filter(|&&f| w / f >= 16 && h / f >= 16)
                .map(|&f| Level::new(&luma, f))
                .collect(),
        }
    }

    /// How far `luma` has drifted against the reference, in whole pixels.
    fn global_shift(&self, luma: &Image) -> (i32, i32) {
        let mut shift = (0, 0);
        let mut radius = MAX_DRIFT / COARSE_FACTORS[0] as i32;
        for level in &self.coarse {
            let img = Level::new(luma, level.factor);
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
}

/// Result of registering one frame.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameAlignment {
    /// Median movement of all points, the shift of the frame as a whole.
    pub shift: (f32, f32),
    /// One entry per point of the reference; `None` where it could not be tracked.
    pub points: Vec<Option<PointShift>>,
    /// Sharpness of the frame, larger is better. Only the order and the ratios matter.
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
        let matches: Vec<Option<PointShift>> = if same_size {
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
            quality: self.quality(&luma, &matches),
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
        })
    }

    /// Sharpness of the frame: the mean squared gradient inside the boxes around the tracked points,
    /// relative to the squared mean brightness so that exposure does not matter.
    fn quality(&self, luma: &Image, matches: &[Option<PointShift>]) -> f32 {
        let (w, h) = (luma.width as isize, luma.height as isize);
        let half = (self.params.box_size / 2) as isize;
        let mean = luma.data.iter().sum::<f32>() / luma.data.len().max(1) as f32;
        let (mut sum, mut count) = (0.0f64, 0u64);
        for (p, m) in self.points.iter().zip(matches) {
            let Some(m) = m else { continue };
            let cx = p.x as isize + m.dx.round() as isize;
            let cy = p.y as isize + m.dy.round() as isize;
            for y in (cy - half).max(1)..(cy + half).min(h - 1) {
                for x in (cx - half).max(1)..(cx + half).min(w - 1) {
                    let at = |xx: isize, yy: isize| luma.data[(yy * w + xx) as usize];
                    let gx = (at(x + 1, y) - at(x - 1, y)) / 2.0;
                    let gy = (at(x, y + 1) - at(x, y - 1)) / 2.0;
                    sum += f64::from(gx * gx + gy * gy);
                    count += 1;
                }
            }
        }
        if count == 0 || mean <= 0.0 {
            return 0.0;
        }
        (sum / count as f64) as f32 / (mean * mean)
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

/// What the *Limit* step keeps, after the frames are ordered by quality. The mode *Frames/Apoint* of
/// the original is not covered, its behaviour is unobserved (`docs/offene-fragen.md`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LimitMode {
    /// *Lowest Quality (%)*: frames whose quality is at least this share of the best frame's.
    LowestQuality(f32),
    /// *Best Frames (%)*: this share of the frames, the best ones.
    BestPercent(f32),
    /// *Best Frames*: this many frames, the best ones.
    BestCount(usize),
}

/// Frame indices from best to worst quality. Equal quality keeps the frame order.
pub fn rank_by_quality(results: &[FrameAlignment]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..results.len()).collect();
    order.sort_by(|&a, &b| results[b].quality.total_cmp(&results[a].quality));
    order
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
    };
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
                let d =
                    ((a.x as f32 - b.x as f32).powi(2) + (a.y as f32 - b.y as f32).powi(2)).sqrt();
                assert!(
                    d >= params.min_distance as f32,
                    "points {a:?} and {b:?} are {d} apart"
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
        assert_eq!(default_threshold(100), 43);
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
        assert!(q_sharp > q_soft * 1.2, "{q_sharp} vs {q_soft}");
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
}
