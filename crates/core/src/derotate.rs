//! De-rotation: a planet turns while a video is recorded (Jupiter by 36 degrees an hour), so the details towards the edge of the disc
//! smear in a long stack. Every frame is turned back to one reference time before it is stacked (MAC-020).
//!
//! Own design; `docs/stacking.md` describes the model and its limits.
//!
//! - **Disc.** The position and size of the disc are measured in the reference frame: Otsu threshold, largest connected blob, its
//!   holes filled, then the boundary is fitted with the ellipse the planet must show (axis ratio and orientation come from the
//!   tilt and the position angle of the axis, [`View`]), by iteratively re-weighted least squares so that a terminator (Mars) or the
//!   ring (Saturn: the union of the globe and the outer ring ellipse is fitted) does not pull the fit. Every other frame is found by
//!   the shift of its blob's centre of mass against the reference's, which cancels everything the threshold does to the shape.
//! - **Model.** The globe is an oblate spheroid seen in orthographic projection. A pixel of the disc is traced to the surface, the
//!   surface point is turned about the axis by the angle the planet rotated between the reference time and the frame, projected
//!   back and sampled bilinearly in the frame. Pixels off the disc stay as they are. For a time difference of zero the frame is
//!   returned unchanged, bit for bit.
//! - **Saturn.** The rings do not change when the planet turns, so a pixel that has the ring in front of the globe, at the target
//!   or at the place it is read from, stays as it is.
//! - **Limits.** A feature that turns round the limb has no source: the pixels near the limb that would need it stay as they are. The
//!   shadow of the rings and the shadow of the globe on them are not modelled; the atmosphere is taken to move with the surface
//!   (no wind), and the geometry (axis, tilt) of the reference time is used for the whole video, which is right to within
//!   hundredths of a degree over an hour.

use crate::{Image, planet::Planet};

/// How a planet is seen in the image: the tilt and the position angle of its axis at the reference time, and how the camera sat.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    pub planet: Planet,
    /// `D_E` in degrees (see [`crate::planet::PoleGeometry`]).
    pub sub_earth_latitude: f64,
    /// `P` in degrees, from celestial north through east.
    pub pole_angle: f64,
    /// Rotation of the camera in degrees, counter-clockwise on the screen: 0 shows celestial north up (east to the left).
    pub camera_angle: f64,
    /// The image is a mirror image (east to the right with north up), as a single mirror in the light path makes it.
    pub mirrored: bool,
}

impl View {
    fn q(&self) -> f64 {
        1.0 - self.planet.flattening()
    }

    /// Apparent polar over equatorial axis of the disc: 1 seen pole-on, the polar over the equatorial radius seen from the equator.
    pub fn axis_ratio(&self) -> f64 {
        let d = self.sub_earth_latitude.to_radians();
        let q = self.q();
        (d.sin().powi(2) + (q * d.cos()).powi(2)).sqrt()
    }

    /// Maps the plane of the planet as it is seen with its axis pointing up, `(right, up)` in units of the equatorial radius,
    /// into the image, `(right, up)`: the position angle (counter-clockwise, as east is to the left), then the mirror, then the
    /// camera angle. Orthogonal, so the inverse is the transpose.
    fn matrix(&self) -> [[f64; 2]; 2] {
        let turn = |degrees: f64| {
            let (s, c) = degrees.to_radians().sin_cos();
            [[c, -s], [s, c]]
        };
        let mut m = turn(self.pole_angle);
        if self.mirrored {
            m[0] = [-m[0][0], -m[0][1]];
        }
        let r = turn(self.camera_angle);
        [
            [
                r[0][0] * m[0][0] + r[0][1] * m[1][0],
                r[0][0] * m[0][1] + r[0][1] * m[1][1],
            ],
            [
                r[1][0] * m[0][0] + r[1][1] * m[1][0],
                r[1][0] * m[0][1] + r[1][1] * m[1][1],
            ],
        ]
    }

    /// The disc as ellipses to draw: the globe and, for Saturn, the outer edge of the ring.
    pub fn shape(&self, disc: &Disc) -> DiscShape {
        let m = self.matrix();
        let ratio = self.axis_ratio();
        // The equatorial axis points along the first column of the matrix; y runs down in an image.
        let angle = (-m[1][0]).atan2(m[0][0]).to_degrees();
        let ring = self.planet.ring().map(|(_, outer)| {
            let ring_ratio = self.sub_earth_latitude.to_radians().sin().abs();
            (outer * disc.radius, outer * disc.radius * ring_ratio)
        });
        DiscShape {
            x: disc.x,
            y: disc.y,
            semi_major: disc.radius,
            semi_minor: disc.radius * ratio,
            angle,
            ring,
            north: (m[0][1], -m[1][1]),
        }
    }
}

/// The measured disc of the planet: its centre in pixels (the centre of a pixel has integer coordinates) and its equatorial radius.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Disc {
    pub x: f64,
    pub y: f64,
    pub radius: f64,
}

/// A [`Disc`] as ellipses to draw over the image.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiscShape {
    pub x: f64,
    pub y: f64,
    /// Along the equator.
    pub semi_major: f64,
    /// Along the axis of the planet (the seen polar radius).
    pub semi_minor: f64,
    /// Direction of the major axis in degrees, clockwise on the screen from pointing right (y runs down).
    pub angle: f64,
    /// The outer edge of the ring seen as an ellipse with the same direction: `(semi_major, semi_minor)`.
    pub ring: Option<(f64, f64)>,
    /// Unit vector from the centre to where the north pole of the planet points, in image coordinates (x right, y down).
    pub north: (f64, f64),
}

// --- Measuring the disc --------------------------------------------------------------------------

/// What the threshold made of a frame: the blob of the planet with its holes filled.
struct Silhouette {
    width: usize,
    height: usize,
    inside: Vec<bool>,
    centroid: (f64, f64),
}

/// Grey values of a frame, smoothed by a 3 x 3 box so noise does not tear the edge.
fn smoothed_grey(image: &Image) -> Vec<f32> {
    let (w, h) = (image.width as usize, image.height as usize);
    let channels = usize::from(image.channels).max(1);
    let grey: Vec<f32> = image
        .data
        .chunks_exact(channels)
        .map(|px| px.iter().sum::<f32>() / channels as f32)
        .collect();
    let mut across = vec![0.0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let (a, b) = (x.saturating_sub(1), (x + 1).min(w - 1));
            across[y * w + x] = (grey[y * w + a] + grey[y * w + x] + grey[y * w + b]) / 3.0;
        }
    }
    let mut out = vec![0.0f32; w * h];
    for y in 0..h {
        let (a, b) = (y.saturating_sub(1), (y + 1).min(h - 1));
        for x in 0..w {
            out[y * w + x] = (across[a * w + x] + across[y * w + x] + across[b * w + x]) / 3.0;
        }
    }
    out
}

/// Otsu's threshold of the values, `None` if they are (nearly) all the same.
fn otsu(values: &[f32]) -> Option<f32> {
    let (lo, hi) = values
        .iter()
        .fold((f32::MAX, f32::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
    if hi - lo <= 1e-3 || hi.is_nan() || lo.is_nan() {
        return None;
    }
    const BINS: usize = 256;
    let mut histogram = [0u64; BINS];
    for &v in values {
        histogram[(((v - lo) / (hi - lo)) * (BINS as f32 - 1.0)) as usize] += 1;
    }
    let total = values.len() as f64;
    let sum_all: f64 = histogram
        .iter()
        .enumerate()
        .map(|(i, &n)| i as f64 * n as f64)
        .sum();
    let (mut weight_below, mut sum_below) = (0.0f64, 0.0f64);
    let (mut best, mut best_bin) = (-1.0f64, 0usize);
    for (i, &n) in histogram.iter().enumerate() {
        weight_below += n as f64;
        sum_below += i as f64 * n as f64;
        let weight_above = total - weight_below;
        if weight_below == 0.0 || weight_above == 0.0 {
            continue;
        }
        let mean_below = sum_below / weight_below;
        let mean_above = (sum_all - sum_below) / weight_above;
        let between = weight_below * weight_above * (mean_below - mean_above).powi(2);
        if between > best {
            best = between;
            best_bin = i;
        }
    }
    // The threshold lies between the best bin and the next one.
    Some(lo + (best_bin as f32 + 0.5) / (BINS as f32 - 1.0) * (hi - lo))
}

fn silhouette(image: &Image) -> Option<Silhouette> {
    let (w, h) = (image.width as usize, image.height as usize);
    if w < 3 || h < 3 {
        return None;
    }
    let grey = smoothed_grey(image);
    let threshold = otsu(&grey)?;
    let bright: Vec<bool> = grey.iter().map(|&v| v > threshold).collect();

    // The largest 4-connected blob of bright pixels is the planet; moons, stars and hot pixels are smaller.
    let mut label = vec![0u32; w * h];
    let mut sizes = vec![0usize];
    let mut stack = Vec::new();
    for start in 0..w * h {
        if !bright[start] || label[start] != 0 {
            continue;
        }
        let id = sizes.len() as u32;
        let mut size = 0;
        label[start] = id;
        stack.push(start);
        while let Some(i) = stack.pop() {
            size += 1;
            let (x, y) = (i % w, i / w);
            let mut push = |j: usize| {
                if bright[j] && label[j] == 0 {
                    label[j] = id;
                    stack.push(j);
                }
            };
            if x > 0 {
                push(i - 1);
            }
            if x + 1 < w {
                push(i + 1);
            }
            if y > 0 {
                push(i - w);
            }
            if y + 1 < h {
                push(i + w);
            }
        }
        sizes.push(size);
    }
    let (best, &area) = sizes.iter().enumerate().skip(1).max_by_key(|&(_, &n)| n)?;
    if area < 20 {
        return None;
    }
    let blob: Vec<bool> = label.iter().map(|&l| l as usize == best).collect();

    // What the background reaches from the border is outside; holes in the blob (dark bands) belong to the planet.
    let mut outside = vec![false; w * h];
    for i in (0..w)
        .chain((0..h).map(|y| y * w))
        .chain((0..h).map(|y| y * w + w - 1))
        .chain((0..w).map(|x| (h - 1) * w + x))
    {
        if !blob[i] && !outside[i] {
            outside[i] = true;
            stack.push(i);
        }
    }
    while let Some(i) = stack.pop() {
        let (x, y) = (i % w, i / w);
        let mut push = |j: usize| {
            if !blob[j] && !outside[j] {
                outside[j] = true;
                stack.push(j);
            }
        };
        if x > 0 {
            push(i - 1);
        }
        if x + 1 < w {
            push(i + 1);
        }
        if y > 0 {
            push(i - w);
        }
        if y + 1 < h {
            push(i + w);
        }
    }
    let inside: Vec<bool> = outside.iter().map(|&o| !o).collect();
    let (mut count, mut sum_x, mut sum_y) = (0usize, 0.0f64, 0.0f64);
    for (i, _) in inside.iter().enumerate().filter(|&(_, &v)| v) {
        count += 1;
        sum_x += (i % w) as f64;
        sum_y += (i / w) as f64;
    }
    Some(Silhouette {
        width: w,
        height: h,
        inside,
        centroid: (sum_x / count as f64, sum_y / count as f64),
    })
}

impl Silhouette {
    /// Points on the edge: halfway between every pixel of the blob and its outside neighbour. Edges at the border of the image are
    /// left out, the planet is cut off there.
    fn boundary(&self) -> Vec<(f64, f64)> {
        let (w, h) = (self.width, self.height);
        let mut points = Vec::new();
        for y in 0..h {
            for x in 0..w {
                if !self.inside[y * w + x] {
                    continue;
                }
                let mut edge = |nx: isize, ny: isize| {
                    if nx >= 0
                        && ny >= 0
                        && (nx as usize) < w
                        && (ny as usize) < h
                        && !self.inside[ny as usize * w + nx as usize]
                    {
                        points.push(((x as f64 + nx as f64) / 2.0, (y as f64 + ny as f64) / 2.0));
                    }
                };
                edge(x as isize - 1, y as isize);
                edge(x as isize + 1, y as isize);
                edge(x as isize, y as isize - 1);
                edge(x as isize, y as isize + 1);
            }
        }
        points
    }
}

/// The distance of an edge point from the ellipse `(scale, k)` of centre `(cx, cy)` and equatorial radius `a`, in the plane of the
/// image, with the derivatives with respect to `(cx, cy, a)`. `matrix` maps the frame of the planet to the image.
fn edge_residual(
    point: (f64, f64),
    centre: (f64, f64),
    a: f64,
    matrix: &[[f64; 2]; 2],
    (scale, k): (f64, f64),
) -> (f64, [f64; 3]) {
    let (right, up) = (point.0 - centre.0, centre.1 - point.1);
    let u = matrix[0][0] * right + matrix[1][0] * up;
    let v = matrix[0][1] * right + matrix[1][1] * up;
    let m = (u * u + (v / k).powi(2)).sqrt().max(1e-9);
    let d_cx = (u * -matrix[0][0] + v / (k * k) * -matrix[0][1]) / m;
    let d_cy = (u * matrix[1][0] + v / (k * k) * matrix[1][1]) / m;
    (m - scale * a, [d_cx, d_cy, -scale])
}

/// Fits centre `(x, y)` and equatorial radius `a` of the planet to the edge points. `models` are the ellipses the edge consists of, as
/// `(scale, axis ratio)`: the globe `(1, k)`, for Saturn also the outer ring `(2.27, sin D)`. Every point belongs to the model that
/// fits it best. A random-sample consensus finds the ellipse that most points lie on (a terminator, the shadow side of the ring or
/// the edge of a moon are not on it and must not pull), a few steps of Tukey's biweight refine it.
fn fit_ellipses(
    points: &[(f64, f64)],
    matrix: [[f64; 2]; 2],
    models: &[(f64, f64)],
) -> Option<(f64, f64, f64)> {
    if points.len() < 12 {
        return None;
    }
    let best_residual = |point: (f64, f64), centre: (f64, f64), a: f64| {
        models
            .iter()
            .map(|&model| edge_residual(point, centre, a, &matrix, model))
            .min_by(|p, q| p.0.abs().total_cmp(&q.0.abs()))
            .unwrap_or((0.0, [0.0; 3]))
    };

    // A circle through three points in the space where the ellipse of a model is a circle, mapped back.
    let sample = |&(scale, k): &(f64, f64), i: [usize; 3]| -> Option<(f64, f64, f64)> {
        let circle_space = |&(x, y): &(f64, f64)| {
            let (right, up) = (x, -y);
            (
                matrix[0][0] * right + matrix[1][0] * up,
                (matrix[0][1] * right + matrix[1][1] * up) / k,
            )
        };
        let [(ax, ay), (bx, by), (cx, cy)] = i.map(|j| circle_space(&points[j]));
        let d = 2.0 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by));
        if d.abs() < 1e-9 {
            return None;
        }
        let (sa, sb, sc) = (ax * ax + ay * ay, bx * bx + by * by, cx * cx + cy * cy);
        let ux = (sa * (by - cy) + sb * (cy - ay) + sc * (ay - by)) / d;
        let uy = (sa * (cx - bx) + sb * (ax - cx) + sc * (bx - ax)) / d;
        let radius = (ax - ux).hypot(ay - uy);
        // Back to the image: the centre in (right, up), then y down.
        let (cu, cv) = (ux, uy * k);
        let right = matrix[0][0] * cu + matrix[0][1] * cv;
        let up = matrix[1][0] * cu + matrix[1][1] * cv;
        Some((right, -up, radius / scale))
    };

    // Scoring on at most this many points keeps the search cheap for big discs.
    let step = (points.len() / 1500).max(1);
    let scored: Vec<(f64, f64)> = points.iter().copied().step_by(step).collect();
    let mut seed = 0x2545_F491_4F6C_DD1Du64;
    let mut next = move |bound: usize| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed % bound as u64) as usize
    };
    let mut best: Option<(usize, (f64, f64, f64))> = None;
    for round in 0..600 {
        let model = &models[round % models.len()];
        let picks = [next(points.len()), next(points.len()), next(points.len())];
        if picks[0] == picks[1] || picks[1] == picks[2] || picks[0] == picks[2] {
            continue;
        }
        let Some((x, y, a)) = sample(model, picks) else {
            continue;
        };
        if !(a > 3.0 && a < 1e5) {
            continue;
        }
        let inliers = scored
            .iter()
            .filter(|&&p| best_residual(p, (x, y), a).0.abs() < 1.5)
            .count();
        if best.is_none_or(|(n, _)| inliers > n) {
            best = Some((inliers, (x, y, a)));
        }
    }
    let (_, (mut cx, mut cy, mut a)) = best?;

    for _ in 0..40 {
        let (mut jtj, mut jtr) = ([[0.0f64; 3]; 3], [0.0f64; 3]);
        let mut used = 0usize;
        for &point in points {
            let (r, j) = best_residual(point, (cx, cy), a);
            let t = r / 3.0;
            if t.abs() >= 1.0 {
                continue;
            }
            let weight = (1.0 - t * t).powi(2);
            used += 1;
            for p in 0..3 {
                jtr[p] += weight * j[p] * r;
                for q in 0..3 {
                    jtj[p][q] += weight * j[p] * j[q];
                }
            }
        }
        if used < 10 {
            return None;
        }
        let step = solve3(jtj, jtr)?;
        cx -= step[0];
        cy -= step[1];
        a -= step[2];
        if step.iter().all(|s| s.abs() < 1e-4) {
            break;
        }
    }
    (a.is_finite() && a > 3.0).then_some((cx, cy, a))
}

/// Solves a 3 x 3 system by Gaussian elimination with pivoting; `None` if it is singular.
fn solve3(mut m: [[f64; 3]; 3], mut b: [f64; 3]) -> Option<[f64; 3]> {
    for col in 0..3 {
        let pivot = (col..3).max_by(|&i, &j| m[i][col].abs().total_cmp(&m[j][col].abs()))?;
        if m[pivot][col].abs() < 1e-12 {
            return None;
        }
        m.swap(col, pivot);
        b.swap(col, pivot);
        for row in col + 1..3 {
            let f = m[row][col] / m[col][col];
            let pivot_row = m[col];
            for (value, &above) in m[row].iter_mut().zip(&pivot_row).skip(col) {
                *value -= f * above;
            }
            b[row] -= f * b[col];
        }
    }
    let mut x = [0.0; 3];
    for row in (0..3).rev() {
        let tail: f64 = (row + 1..3).map(|k| m[row][k] * x[k]).sum();
        x[row] = (b[row] - tail) / m[row][row];
    }
    Some(x)
}

/// Measures the disc of the planet in `frame`; `None` if no planet is found.
pub fn measure_disc(frame: &Image, view: &View) -> Option<Disc> {
    measure(frame, view).map(|(disc, _)| disc)
}

/// The disc and the centre of mass of its silhouette.
fn measure(frame: &Image, view: &View) -> Option<(Disc, (f64, f64))> {
    let shape = silhouette(frame)?;
    let k = view.axis_ratio();
    let mut models = vec![(1.0, k)];
    if let Some((_, outer)) = view.planet.ring() {
        // A ring seen edge-on is a line; keep the model from collapsing.
        models.push((
            outer,
            view.sub_earth_latitude.to_radians().sin().abs().max(0.03),
        ));
    }
    let (x, y, radius) = fit_ellipses(&shape.boundary(), view.matrix(), &models)?;
    Some((Disc { x, y, radius }, shape.centroid))
}

// --- Turning a frame -----------------------------------------------------------------------------

/// Turns frames back to the reference time.
#[derive(Debug, Clone)]
pub struct Derotator {
    view: View,
    disc: Disc,
    /// Centre of mass of the silhouette of the reference frame; the frames are found by how far theirs moved.
    reference_centroid: (f64, f64),
}

impl Derotator {
    /// Measures the disc in `reference`. `None` if no planet is found in it.
    pub fn new(view: View, reference: &Image) -> Option<Self> {
        let (disc, reference_centroid) = measure(reference, &view)?;
        Some(Self {
            view,
            disc,
            reference_centroid,
        })
    }

    /// The disc measured in the reference frame.
    pub fn disc(&self) -> Disc {
        self.disc
    }

    pub fn view(&self) -> &View {
        &self.view
    }

    /// `frame` as it would have looked `seconds` earlier: the frame was taken `seconds` after the reference time (negative for a
    /// frame before it). For zero the frame comes back unchanged.
    pub fn derotate(&self, frame: &Image, seconds: f64) -> Image {
        let planet = self.view.planet;
        let psi = (360.0 * seconds / planet.rotation_period()).to_radians();
        if psi == 0.0 || frame.width == 0 || frame.height == 0 {
            return frame.clone();
        }
        // The frame is not where the reference was; its blob has moved by as much as the planet.
        let (cx, cy) = match silhouette(frame) {
            Some(s) => (
                self.disc.x + s.centroid.0 - self.reference_centroid.0,
                self.disc.y + s.centroid.1 - self.reference_centroid.1,
            ),
            None => (self.disc.x, self.disc.y),
        };
        let a = self.disc.radius;
        let (width, height) = (frame.width as usize, frame.height as usize);
        let channels = usize::from(frame.channels).max(1);

        let d = self.view.sub_earth_latitude.to_radians();
        let (sd, cd) = d.sin_cos();
        let q = self.view.q();
        let m = self.view.matrix();
        let ring = planet.ring();
        let (sp, cp) = psi.sin_cos();

        // Quadratic for the depth `w` of the surface point under screen position `(u, v)`: the spheroid in the frame of the planet.
        let qq = 1.0 / (q * q);
        let quad_a = cd * cd + sd * sd * qq;
        let quad_b = 2.0 * sd * cd * (qq - 1.0);

        let ring_in_front = |px: f64, py: f64, pz: f64| -> bool {
            let Some((inner, outer)) = ring else {
                return false;
            };
            if sd.abs() < 1e-3 {
                return false;
            }
            let s = -pz / sd;
            if s <= 0.0 {
                return false;
            }
            let rho = (px + s * cd).hypot(py);
            rho >= inner && rho <= outer
        };

        let mut out = frame.clone();
        let reach = a + 2.0;
        let x0 = ((cx - reach).floor().max(0.0)) as usize;
        let x1 = ((cx + reach).ceil().min(width as f64 - 1.0)).max(0.0) as usize;
        let y0 = ((cy - reach).floor().max(0.0)) as usize;
        let y1 = ((cy + reach).ceil().min(height as f64 - 1.0)).max(0.0) as usize;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let (right, up) = ((x as f64 - cx) / a, (cy - y as f64) / a);
                let u = m[0][0] * right + m[1][0] * up;
                let v = m[0][1] * right + m[1][1] * up;
                let c0 = v * v * (sd * sd + cd * cd * qq) + u * u - 1.0;
                let disc = quad_b * quad_b * v * v - 4.0 * quad_a * c0;
                if disc < 0.0 {
                    continue;
                }
                let w = (-quad_b * v + disc.sqrt()) / (2.0 * quad_a);
                // The surface point in the frame of the planet: x towards the sub-observer meridian, y east, z north.
                let (px, py, pz) = (-v * sd + w * cd, u, v * cd + w * sd);
                if ring_in_front(px, py, pz) {
                    continue;
                }
                // By the time the frame was taken the surface had turned (counter-clockwise seen from the north) by psi.
                let (nx, ny) = (px * cp - py * sp, px * sp + py * cp);
                if nx * cd + pz * sd <= 0.0 || ring_in_front(nx, ny, pz) {
                    continue;
                }
                let (nu, nv) = (ny, -nx * sd + pz * cd);
                let right_back = m[0][0] * nu + m[0][1] * nv;
                let up_back = m[1][0] * nu + m[1][1] * nv;
                let (sx, sy) = (cx + right_back * a, cy - up_back * a);
                if !(sx >= 0.0
                    && sy >= 0.0
                    && sx <= (width - 1) as f64
                    && sy <= (height - 1) as f64)
                {
                    continue;
                }
                let (ix, iy) = (sx.floor() as usize, sy.floor() as usize);
                let (jx, jy) = ((ix + 1).min(width - 1), (iy + 1).min(height - 1));
                let (tx, ty) = ((sx - ix as f64) as f32, (sy - iy as f64) as f32);
                for c in 0..channels {
                    let at = |xx: usize, yy: usize| frame.data[(yy * width + xx) * channels + c];
                    let top = at(ix, iy) + (at(jx, iy) - at(ix, iy)) * tx;
                    let bottom = at(ix, jy) + (at(jx, jy) - at(ix, jy)) * tx;
                    out.data[(y * width + x) * channels + c] = top + (bottom - top) * ty;
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stack::psnr;

    use crate::synthetic::{Scene, jupiter_scene, jupiter_view};

    /// Mean squared error inside the disc only (the dark sky would flatter the PSNR of the whole picture).
    fn disc_psnr(a: &Image, b: &Image, centre: (f64, f64), radius: f64) -> f64 {
        let (mut sum, mut count) = (0.0f64, 0usize);
        for y in 0..a.height {
            for x in 0..a.width {
                if (f64::from(x) - centre.0).hypot(f64::from(y) - centre.1) < radius * 0.97 {
                    let i = (y * a.width + x) as usize;
                    sum += f64::from(a.data[i] - b.data[i]).powi(2);
                    count += 1;
                }
            }
        }
        10.0 * (1.0 / (sum / count as f64)).log10()
    }

    #[test]
    fn a_five_minute_rotation_of_jupiter_is_taken_back() {
        let scene = jupiter_scene();
        let before = scene.render(320, 240, 0.0);
        let later = scene.render(320, 240, 300.0);
        let derotator = Derotator::new(jupiter_view(), &before).expect("the planet is found");
        let back = derotator.derotate(&later, 300.0);
        let (with, without) = (
            disc_psnr(&back, &before, scene.centre, scene.radius),
            disc_psnr(&later, &before, scene.centre, scene.radius),
        );
        eprintln!("Jupiter 5 min: with {with:.2} dB, without {without:.2} dB");
        assert!(without < 33.0, "the rotation must be visible: {without}");
        assert!(with > 40.0, "derotated: {with} dB");
        assert!(
            with > without + 10.0,
            "with {with} dB, without {without} dB"
        );
    }

    #[test]
    fn the_wrong_direction_is_clearly_worse() {
        // Guards the sign: turning the other way doubles the error instead of removing it.
        let scene = jupiter_scene();
        let before = scene.render(320, 240, 0.0);
        let later = scene.render(320, 240, 300.0);
        let derotator = Derotator::new(jupiter_view(), &before).unwrap();
        let wrong = derotator.derotate(&later, -300.0);
        let right = derotator.derotate(&later, 300.0);
        assert!(
            disc_psnr(&wrong, &before, scene.centre, scene.radius) + 10.0
                < disc_psnr(&right, &before, scene.centre, scene.radius)
        );
    }

    #[test]
    fn a_frame_before_the_reference_is_turned_forward() {
        let scene = jupiter_scene();
        let reference = scene.render(320, 240, 600.0);
        let earlier = scene.render(320, 240, 420.0);
        let derotator = Derotator::new(jupiter_view(), &reference).unwrap();
        let forward = derotator.derotate(&earlier, -180.0);
        assert!(disc_psnr(&forward, &reference, scene.centre, scene.radius) > 34.0);
    }

    #[test]
    fn zero_time_difference_returns_the_frame_bit_for_bit() {
        let scene = jupiter_scene();
        let frame = scene.render(320, 240, 123.0);
        let derotator = Derotator::new(jupiter_view(), &frame).unwrap();
        assert_eq!(derotator.derotate(&frame, 0.0), frame);
    }

    #[test]
    fn pixels_off_the_disc_stay_as_they_are() {
        let scene = jupiter_scene();
        let before = scene.render(320, 240, 0.0);
        let mut later = scene.render(320, 240, 600.0);
        // A star next to the planet.
        later.data[20 * 320 + 20] = 0.9;
        let derotator = Derotator::new(jupiter_view(), &before).unwrap();
        let back = derotator.derotate(&later, 600.0);
        for y in 0..240usize {
            for x in 0..320usize {
                let r = (x as f64 - scene.centre.0).hypot(y as f64 - scene.centre.1);
                if r > scene.radius + 3.0 {
                    assert_eq!(back.data[y * 320 + x], later.data[y * 320 + x]);
                }
            }
        }
    }

    #[test]
    fn a_frame_that_drifted_across_the_picture_is_turned_about_its_own_centre() {
        let scene = jupiter_scene();
        let reference = scene.render(320, 240, 0.0);
        let moved = Scene {
            centre: (scene.centre.0 + 17.4, scene.centre.1 - 9.2),
            ..jupiter_scene()
        };
        let later = moved.render(320, 240, 420.0);
        let target = moved.render(320, 240, 0.0);
        let derotator = Derotator::new(jupiter_view(), &reference).unwrap();
        let back = derotator.derotate(&later, 420.0);
        let (with, without) = (
            disc_psnr(&back, &target, moved.centre, moved.radius),
            disc_psnr(&later, &target, moved.centre, moved.radius),
        );
        assert!(
            with > 30.0 && with > without + 8.0,
            "with {with}, without {without}"
        );
    }

    #[test]
    fn camera_angle_and_mirror_are_followed() {
        // A camera turned by 40 degrees: the axis shows at P + 40 on the screen.
        let rolled = Scene {
            axis_angle: 24.8 + 40.0,
            ..jupiter_scene()
        };
        let before = rolled.render(320, 240, 0.0);
        let later = rolled.render(320, 240, 300.0);
        let view = View {
            camera_angle: 40.0,
            ..jupiter_view()
        };
        let back = Derotator::new(view, &before)
            .unwrap()
            .derotate(&later, 300.0);
        assert!(disc_psnr(&back, &before, rolled.centre, rolled.radius) > 34.0);

        // The mirror image of the unrolled scene (about the vertical axis through the middle of the picture).
        let scene = Scene {
            centre: (159.5, 120.6),
            ..jupiter_scene()
        };
        let flip = |image: &Image| {
            let mut out = image.clone();
            for y in 0..image.height as usize {
                for x in 0..image.width as usize {
                    out.data[y * 320 + x] = image.data[y * 320 + 319 - x];
                }
            }
            out
        };
        let before = flip(&scene.render(320, 240, 0.0));
        let later = flip(&scene.render(320, 240, 300.0));
        let mirrored = View {
            mirrored: true,
            ..jupiter_view()
        };
        let back = Derotator::new(mirrored, &before)
            .unwrap()
            .derotate(&later, 300.0);
        let (with, without) = (
            disc_psnr(&back, &before, scene.centre, scene.radius),
            disc_psnr(&later, &before, scene.centre, scene.radius),
        );
        assert!(
            with > 34.0 && with > without + 10.0,
            "with {with}, without {without}"
        );
        // Without saying so the picture is turned the wrong way.
        let unaware = Derotator::new(jupiter_view(), &before)
            .unwrap()
            .derotate(&later, 300.0);
        assert!(disc_psnr(&unaware, &before, scene.centre, scene.radius) < with - 8.0);
    }

    #[test]
    fn saturn_is_turned_and_its_rings_are_left_alone() {
        let scene = Scene {
            planet: Planet::Saturn,
            tilt: 19.0,
            axis_angle: 6.7,
            centre: (200.4, 150.2),
            radius: 60.0,
            rings: true,
        };
        let view = View {
            planet: Planet::Saturn,
            sub_earth_latitude: 19.0,
            pole_angle: 6.7,
            camera_angle: 0.0,
            mirrored: false,
        };
        let before = scene.render(400, 300, 0.0);
        let later = scene.render(400, 300, 600.0);
        let derotator = Derotator::new(view, &before).expect("Saturn is found");
        // The ring (2.27 radii) is what bounds the silhouette, the fit still gives the globe's radius.
        let disc = derotator.disc();
        assert!((disc.radius - 60.0).abs() < 1.5, "radius {}", disc.radius);
        assert!(
            (disc.x - 200.4).abs() < 0.8 && (disc.y - 150.2).abs() < 0.8,
            "centre {disc:?}"
        );
        let back = derotator.derotate(&later, 600.0);
        let ball = |a: &Image, b: &Image| disc_psnr(a, b, scene.centre, scene.radius);
        let (with, without) = (ball(&back, &before), ball(&later, &before));
        eprintln!("Saturn 10 min: with {with:.2} dB, without {without:.2} dB");
        assert!(
            with > 31.0 && with > without + 5.0,
            "with {with}, without {without}"
        );
        // Where the ring is in front nothing changes at all.
        let whole = psnr(&back, &before).unwrap();
        assert!(whole > psnr(&later, &before).unwrap() + 3.0);
        let mut checked = 0;
        for y in 0..300u32 {
            for x in 0..400u32 {
                let (u, v) = ((f64::from(x) - 200.4) / 60.0, (150.2 - f64::from(y)) / 60.0);
                let on_ball = scene.ball_hit(u * 1.05, v * 1.05).is_some();
                if on_ball
                    && scene
                        .ring_at(x, y)
                        .is_some_and(|rho| (1.3..2.2).contains(&rho))
                {
                    let i = (y * 400 + x) as usize;
                    assert_eq!(back.data[i], later.data[i], "ring pixel ({x}, {y}) changed");
                    checked += 1;
                }
            }
        }
        assert!(
            checked > 100,
            "only {checked} ring pixels in front of the globe"
        );
    }

    #[test]
    fn mars_with_a_dark_limb_is_measured_by_the_lit_edge() {
        // A gibbous Mars: the terminator cuts a sliver off the disc on the left.
        let scene = Scene {
            planet: Planet::Mars,
            tilt: 12.0,
            axis_angle: 348.0,
            centre: (100.0, 90.0),
            radius: 50.0,
            rings: false,
        };
        let mut image = scene.render(200, 180, 0.0);
        for y in 0..180usize {
            for x in 0..200usize {
                // The lit part: right of a line a little to the left of the middle, curved like a terminator.
                let dx = x as f64 - 100.0;
                let dy = y as f64 - 90.0;
                let terminator = -28.0 + 0.004 * dy * dy;
                if dx < terminator {
                    image.data[y * 200 + x] = 0.02;
                }
            }
        }
        let view = View {
            planet: Planet::Mars,
            sub_earth_latitude: 12.0,
            pole_angle: 348.0,
            camera_angle: 0.0,
            mirrored: false,
        };
        let disc = measure_disc(&image, &view).expect("Mars is found");
        assert!((disc.radius - 50.0).abs() < 1.0, "radius {}", disc.radius);
        assert!(
            (disc.x - 100.0).abs() < 1.5 && (disc.y - 90.0).abs() < 1.5,
            "centre {disc:?}"
        );
    }

    #[test]
    fn the_disc_is_found_among_noise_and_a_moon_and_holes() {
        let scene = jupiter_scene();
        let mut image = scene.render(320, 240, 0.0);
        // A moon, noise and a dark hole inside the disc.
        for y in 30..36usize {
            for x in 250..256usize {
                image.data[y * 320 + x] = 0.8;
            }
        }
        let mut seed = 12345u32;
        for v in image.data.iter_mut() {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            *v += ((seed >> 16) as f32 / 65535.0 - 0.5) * 0.06;
        }
        for y in 110..122usize {
            for x in 150..162usize {
                image.data[y * 320 + x] = 0.0;
            }
        }
        let disc = measure_disc(&image, &jupiter_view()).expect("found");
        assert!((disc.radius - 100.0).abs() < 1.0, "radius {}", disc.radius);
        assert!(
            (disc.x - 160.3).abs() < 0.7 && (disc.y - 120.6).abs() < 0.7,
            "centre {disc:?}"
        );
    }

    #[test]
    fn an_empty_sky_has_no_disc() {
        let image = Image::new(64, 64, 1);
        assert!(measure_disc(&image, &jupiter_view()).is_none());
    }

    #[test]
    fn the_drawn_ellipse_follows_tilt_and_angle() {
        let view = View {
            planet: Planet::Jupiter,
            sub_earth_latitude: 0.0,
            pole_angle: 0.0,
            camera_angle: 0.0,
            mirrored: false,
        };
        let shape = view.shape(&Disc {
            x: 10.0,
            y: 20.0,
            radius: 50.0,
        });
        assert!((shape.semi_minor - 50.0 * (1.0 - 0.06487)).abs() < 1e-9);
        assert!(shape.angle.abs() < 1e-9 && shape.ring.is_none());
        // The north pole of the planet points up in the picture.
        assert!(shape.north.0.abs() < 1e-9 && (shape.north.1 - -1.0).abs() < 1e-9);
        // The axis turned to the left (counter-clockwise) by 30 degrees: the equator turns the same way, which is up to the right.
        let turned = View {
            pole_angle: 30.0,
            ..view
        };
        assert!(
            (turned
                .shape(&Disc {
                    x: 0.0,
                    y: 0.0,
                    radius: 1.0
                })
                .angle
                - -30.0)
                .abs()
                < 1e-9
        );
    }
}
