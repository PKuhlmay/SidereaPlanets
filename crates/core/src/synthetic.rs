//! A synthetic planet for tests: an oblate spheroid with bands, spots and a storm, rotating about a tilted axis, and optionally the
//! rings of Saturn. Rendered by tracing rays in 3D camera coordinates, independent of the projection formulas of
//! [`crate::derotate`], so a test that de-rotates it checks those formulas.
//!
//! Only built for tests (and for the tests of other crates through the `synthetic-planet` feature).

use crate::{Image, derotate::View, planet::Planet};

const D: f64 = std::f64::consts::PI / 180.0;

/// A synthetic planet seen from the camera: the scene is traced with rays in 3D camera coordinates (x right, y up, z towards
/// the viewer), independent of the projection formulas of the module.
pub struct Scene {
    pub planet: Planet,
    /// Latitude of the viewer, degrees.
    pub tilt: f64,
    /// Direction of the axis on the screen, counter-clockwise from up, degrees.
    pub axis_angle: f64,
    pub centre: (f64, f64),
    pub radius: f64,
    /// Draw the ring system (Saturn).
    pub rings: bool,
}

impl Scene {
    /// The north pole of the planet in camera coordinates.
    fn axis(&self) -> [f64; 3] {
        let (s, c) = (self.axis_angle * D).sin_cos();
        let (st, ct) = (self.tilt * D).sin_cos();
        [-s * ct, c * ct, st]
    }

    /// Depth `z` of the ball's surface under the screen position `(u, v)` (units of the equatorial radius), if there is any.
    pub fn ball_hit(&self, u: f64, v: f64) -> Option<f64> {
        let n = self.axis();
        let q = 1.0 - self.planet.flattening();
        // Ray from the viewer: points (u, v, z); the spheroid: |p|^2 - (p.n)^2 (1 - 1/q^2) = 1.
        let pn0 = u * n[0] + v * n[1];
        let k = 1.0 - 1.0 / (q * q);
        let a2 = 1.0 - n[2] * n[2] * k;
        let b1 = -2.0 * pn0 * n[2] * k;
        let c0 = u * u + v * v - pn0 * pn0 * k - 1.0;
        let disc = b1 * b1 - 4.0 * a2 * c0;
        (disc >= 0.0).then(|| (-b1 + disc.sqrt()) / (2.0 * a2))
    }

    /// The ring in front of the ball (or of the empty sky) at pixel `(x, y)`: the radius in the ring plane, if there is one.
    pub fn ring_at(&self, x: u32, y: u32) -> Option<f64> {
        let n = self.axis();
        if !self.rings || n[2].abs() < 1e-3 {
            return None;
        }
        let (u, v) = (
            (f64::from(x) - self.centre.0) / self.radius,
            (self.centre.1 - f64::from(y)) / self.radius,
        );
        // The ring plane is p.n = 0: z = -(u n0 + v n1) / n2.
        let z = -(u * n[0] + v * n[1]) / n[2];
        let rho = (u * u + v * v + z * z).sqrt();
        let ball = self.ball_hit(u, v).unwrap_or(f64::NEG_INFINITY);
        (z > ball && (1.24..=2.27).contains(&rho)).then_some(rho)
    }

    /// The image `seconds` after the start of the rotation.
    pub fn render(&self, width: u32, height: u32, seconds: f64) -> Image {
        let n = self.axis();
        // A fixed longitude origin and its right-handed partner, both perpendicular to the axis.
        let seed = if n[2].abs() < 0.9 {
            [0.0, 0.0, 1.0]
        } else {
            [1.0, 0.0, 0.0]
        };
        let e1 = normalise(cross(n, seed));
        let e2 = cross(n, e1);
        let omega = 2.0 * std::f64::consts::PI / self.planet.rotation_period();
        let mut image = Image::new(width, height, 1);
        for y in 0..height {
            for x in 0..width {
                let (u, v) = (
                    (f64::from(x) - self.centre.0) / self.radius,
                    (self.centre.1 - f64::from(y)) / self.radius,
                );
                let mut value = 0.02;
                if let Some(z) = self.ball_hit(u, v) {
                    let p = [u, v, z];
                    let lon = dot(p, e2).atan2(dot(p, e1)) - omega * seconds;
                    let lat = (dot(p, n) / dot(p, p).sqrt()).asin();
                    value = texture(lon, lat) * (0.45 + 0.55 * z.max(0.0).powf(0.4));
                }
                // The ring is rotationally symmetric, so it does not change when the planet turns.
                if let Some(rho) = self.ring_at(x, y) {
                    value = 0.55 + 0.25 * (rho * 14.0).sin();
                }
                image.data[(y * width + x) as usize] = value as f32;
            }
        }
        image
    }
}

/// Smooth bands and spots of a few pixels' width on a disc of 100 px radius.
fn texture(lon: f64, lat: f64) -> f64 {
    let bands = 0.55 + 0.16 * (lat * 11.0).sin() + 0.1 * (lat * 23.0 + 1.0).cos();
    let spots = 0.14 * (lon * 5.0 + lat * 3.0).sin() * (lat * 4.0).cos()
        + 0.10 * (lon * 11.0 - 0.5).cos() * (lat * 7.0).sin();
    let storm = 0.30 * (-(((lon - 0.7).powi(2)) * 60.0 + ((lat + 0.35).powi(2)) * 220.0)).exp();
    (bands + spots + storm).clamp(0.25, 1.0)
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalise(a: [f64; 3]) -> [f64; 3] {
    let n = dot(a, a).sqrt();
    a.map(|v| v / n)
}

/// A synthetic Jupiter of 100 px radius in a 320 x 240 picture `seconds` after the start, with the view it was made for.
pub fn synthetic_jupiter(seconds: f64) -> (Image, View) {
    (jupiter_scene().render(320, 240, seconds), jupiter_view())
}

pub fn jupiter_scene() -> Scene {
    Scene {
        planet: Planet::Jupiter,
        tilt: -2.5,
        axis_angle: 24.8,
        centre: (160.3, 120.6),
        radius: 100.0,
        rings: false,
    }
}

pub fn jupiter_view() -> View {
    View {
        planet: Planet::Jupiter,
        sub_earth_latitude: -2.5,
        pole_angle: 24.8,
        camera_angle: 0.0,
        mirrored: false,
    }
}
