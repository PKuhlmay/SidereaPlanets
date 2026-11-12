//! The three planets whose rotation shows within a video (Jupiter, Saturn, Mars): how fast they turn, how flattened they
//! are, and how their axis is tilted towards the camera at a given time (MAC-020).
//!
//! The tilt comes from a simplified ephemeris after Meeus, "Astronomical Algorithms", 2nd ed.: the mean orbital elements of
//! Earth and the planet (table 31.A, referred to the equinox of the date) give the geocentric direction of the planet, with the
//! light time of chapters 42, 43 and 45; the pole of the planet is the IAU pole (the one Meeus uses in the physical
//! ephemerides). From the direction and the pole follow the two numbers the derotation needs:
//!
//! - `D_E`, the latitude of the Earth in the planet's frame (Meeus' `D_E` for Mars and Jupiter, `B` for Saturn): the angle by
//!   which the north pole is tilted towards the viewer, positive when it is tilted towards us;
//! - `P`, the position angle of the north pole in the sky, measured from celestial north through east.
//!
//! Perturbations between the planets, nutation and aberration are left out. Against JPL Horizons over 1992 to 2030 the result
//! agrees to better than half a degree (tests below); the derotation needs about one degree.

use std::f64::consts::PI;

/// A planet that can be de-rotated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Planet {
    Jupiter,
    Saturn,
    Mars,
}

/// How the axis of a planet is seen from Earth at one moment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PoleGeometry {
    /// `D_E` in degrees: the latitude of the Earth in the planet's frame. Positive when the north pole is tilted towards us.
    pub sub_earth_latitude: f64,
    /// `P` in degrees, `0..360`: the position angle of the north pole, from celestial north through east.
    pub pole_angle: f64,
    /// Distance of the planet from Earth in astronomical units.
    pub distance_au: f64,
}

/// Mean elements of an orbit: each is a polynomial in `T`, Julian centuries since J2000.0 (Meeus table 31.A).
struct Elements {
    /// Mean longitude `L`, degrees.
    l: [f64; 4],
    /// Semi-major axis, au.
    a: [f64; 3],
    /// Eccentricity.
    e: [f64; 4],
    /// Inclination, degrees.
    i: [f64; 4],
    /// Longitude of the ascending node `Ω`, degrees.
    node: [f64; 4],
    /// Longitude of the perihelion `ϖ`, degrees.
    peri: [f64; 4],
}

const EARTH: Elements = Elements {
    l: [100.466457, 36000.7698278, 0.00030322, 0.000000020],
    a: [1.000001018, 0.0, 0.0],
    e: [0.01670863, -0.000042037, -0.0000001267, 0.00000000014],
    i: [0.0; 4],
    node: [0.0; 4],
    peri: [102.937348, 1.7195366, 0.00045688, -0.000000018],
};

const MARS: Elements = Elements {
    l: [355.433000, 19140.2993039, 0.00000261, -0.000000003],
    a: [1.523679342, 0.0, 0.0],
    e: [0.09340065, 0.000090484, -0.00000000806, -0.00000000025],
    i: [1.849726, -0.0081477, -0.00002255, -0.000000027],
    node: [49.558093, 0.7720959, 0.00001557, 0.000002267],
    peri: [336.060234, 1.8410449, 0.00013477, 0.000000536],
};

const JUPITER: Elements = Elements {
    l: [34.351519, 3034.9056606, -0.00008501, 0.000000016],
    a: [5.202603209, 0.0000001913, 0.0],
    e: [0.04849793, 0.000163225, -0.0000004714, -0.00000000201],
    i: [1.303267, -0.0054965, 0.00000466, -0.000000002],
    node: [100.464407, 1.0209774, 0.00040315, 0.000000404],
    peri: [14.331207, 1.6126352, 0.00103042, -0.000004464],
};

const SATURN: Elements = Elements {
    l: [50.077444, 1222.1138488, 0.00021004, -0.000000046],
    a: [9.554909192, -0.0000021390, 0.000000004],
    e: [0.05554814, -0.000346641, -0.0000006436, 0.0000000340],
    i: [2.488879, -0.0037362, -0.00001519, 0.000000087],
    node: [113.665503, 0.8770880, -0.00012176, -0.000002249],
    peri: [93.057237, 1.9637613, 0.00083753, 0.000004928],
};

fn poly(c: &[f64], t: f64) -> f64 {
    c.iter().rev().fold(0.0, |acc, &k| acc * t + k)
}

/// Julian date of a Unix time (UTC; the 69 s to terrestrial time do not matter here).
fn julian_date(unix_seconds: f64) -> f64 {
    2_440_587.5 + unix_seconds / 86_400.0
}

/// Seconds since 1970-01-01 00:00:00 UTC of a calendar date (proleptic Gregorian), for tests and for the callers that have a
/// date and time of day.
pub fn unix_seconds(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: f64) -> f64 {
    // Days from civil (Hinnant): shifts the year to start in March so the leap day ends it.
    let y = i64::from(year) - i64::from(month <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (i64::from(month) + 9) % 12;
    let doy = (153 * mp + 2) / 5 + i64::from(day) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    days as f64 * 86_400.0 + f64::from(hour) * 3600.0 + f64::from(minute) * 60.0 + second
}

/// The heliocentric position in the ecliptic of the date, in au.
fn heliocentric(el: &Elements, t: f64) -> [f64; 3] {
    let rad = PI / 180.0;
    let (l, a, e) = (poly(&el.l, t), poly(&el.a, t), poly(&el.e, t));
    let (inc, node, peri) = (
        poly(&el.i, t) * rad,
        poly(&el.node, t) * rad,
        poly(&el.peri, t) * rad,
    );
    // Kepler's equation by Newton's method.
    let mean = (l * rad - peri).rem_euclid(2.0 * PI);
    let mut ecc = if e < 0.8 { mean } else { PI };
    for _ in 0..30 {
        let step = (ecc - e * ecc.sin() - mean) / (1.0 - e * ecc.cos());
        ecc -= step;
        if step.abs() < 1e-13 {
            break;
        }
    }
    let nu = 2.0 * (((1.0 + e) / (1.0 - e)).sqrt() * (ecc / 2.0).tan()).atan();
    let r = a * (1.0 - e * ecc.cos());
    let arg = nu + (peri - node);
    let (sn, cn) = node.sin_cos();
    let (su, cu) = arg.sin_cos();
    [
        r * (cn * cu - sn * su * inc.cos()),
        r * (sn * cu + cn * su * inc.cos()),
        r * su * inc.sin(),
    ]
}

impl Planet {
    /// The planet named by `text` ("Jupiter", "jup", "saturn", ...), in any case; `None` if it is none of the three. The first
    /// three letters decide, so file names such as `JUP_2023.ser` work.
    pub fn from_name(text: &str) -> Option<Self> {
        let lower = text.to_ascii_lowercase();
        let find = |key: &str| lower.contains(key);
        if find("jup") {
            Some(Self::Jupiter)
        } else if find("sat") {
            Some(Self::Saturn)
        } else if find("mar") {
            Some(Self::Mars)
        } else {
            None
        }
    }

    /// Flattening `(a - b) / a` of the planet (IAU values).
    pub const fn flattening(self) -> f64 {
        match self {
            Self::Jupiter => 0.06487,
            Self::Saturn => 0.09796,
            Self::Mars => 0.00589,
        }
    }

    /// The rotation period in seconds: Jupiter System II (9 h 55 min 40.6 s), Saturn System III (10 h 33 min 38 s), Mars
    /// (24 h 37 min 22.7 s). The planet turns counter-clockwise seen from its north pole.
    pub const fn rotation_period(self) -> f64 {
        match self {
            Self::Jupiter => 9.0 * 3600.0 + 55.0 * 60.0 + 40.6,
            Self::Saturn => 10.0 * 3600.0 + 33.0 * 60.0 + 38.0,
            Self::Mars => 24.0 * 3600.0 + 37.0 * 60.0 + 22.7,
        }
    }

    /// How far the planet turns in `seconds`, in degrees.
    pub fn rotation_degrees(self, seconds: f64) -> f64 {
        360.0 * seconds / self.rotation_period()
    }

    /// The inner and outer edge of the ring system in equatorial radii of the planet (Saturn: C ring inner edge, A ring outer
    /// edge), if it has one.
    pub const fn ring(self) -> Option<(f64, f64)> {
        match self {
            Self::Saturn => Some((1.239, 2.270)),
            _ => None,
        }
    }

    fn elements(self) -> &'static Elements {
        match self {
            Self::Jupiter => &JUPITER,
            Self::Saturn => &SATURN,
            Self::Mars => &MARS,
        }
    }

    /// Right ascension and declination of the north pole at J2000.0 in degrees, and their change per Julian century (IAU 2009
    /// report; the values of Meeus' physical ephemerides).
    fn pole(self, t: f64) -> (f64, f64) {
        match self {
            Self::Jupiter => (268.056595 - 0.006499 * t, 64.495303 + 0.002413 * t),
            Self::Saturn => (40.589 - 0.036 * t, 83.537 - 0.004 * t),
            Self::Mars => (317.68143 - 0.1061 * t, 52.88650 - 0.0609 * t),
        }
    }

    /// The tilt and the position angle of the axis at `unix_seconds` (UTC).
    pub fn pole_geometry(self, unix_seconds: f64) -> PoleGeometry {
        let rad = PI / 180.0;
        let jd = julian_date(unix_seconds);
        let t = (jd - 2_451_545.0) / 36_525.0;
        let earth = heliocentric(&EARTH, t);

        // The planet as seen at `jd` left it `light_time` days earlier.
        let mut light_time = 0.0;
        let mut geo = [0.0; 3];
        for _ in 0..3 {
            let tp = t - light_time / 36_525.0;
            let p = heliocentric(self.elements(), tp);
            geo = [p[0] - earth[0], p[1] - earth[1], p[2] - earth[2]];
            let distance = (geo[0] * geo[0] + geo[1] * geo[1] + geo[2] * geo[2]).sqrt();
            light_time = 0.005_775_518_3 * distance;
        }
        let distance = (geo[0] * geo[0] + geo[1] * geo[1] + geo[2] * geo[2]).sqrt();

        // Ecliptic of the date to the ecliptic of J2000 (general precession in longitude), then to the equator of J2000, where
        // the pole of the planet is given.
        let precession = (5028.796195 * t + 1.1054348 * t * t) / 3600.0 * rad;
        let (sp, cp) = precession.sin_cos();
        let ecliptic = [
            geo[0] * cp + geo[1] * sp,
            -geo[0] * sp + geo[1] * cp,
            geo[2],
        ];
        let (se, ce) = (23.439_291_f64 * rad).sin_cos();
        let equatorial = [
            ecliptic[0],
            ecliptic[1] * ce - ecliptic[2] * se,
            ecliptic[1] * se + ecliptic[2] * ce,
        ];
        let s = equatorial.map(|v| v / distance);

        let (ra, dec) = self.pole(t);
        let (ra, dec) = (ra * rad, dec * rad);
        let n = [dec.cos() * ra.cos(), dec.cos() * ra.sin(), dec.sin()];

        let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        let cross = |a: [f64; 3], b: [f64; 3]| {
            [
                a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0],
            ]
        };
        // The Earth lies in direction -s as seen from the planet.
        let sub_earth_latitude = (-dot(n, s)).asin() / rad;
        // East and north in the sky at the planet; the pole angle is the direction of the pole axis between them.
        let east_raw = cross([0.0, 0.0, 1.0], s);
        let norm = dot(east_raw, east_raw).sqrt();
        let east = east_raw.map(|v| v / norm);
        let north = cross(s, east);
        let pole_angle = dot(n, east).atan2(dot(n, north)) / rad;
        PoleGeometry {
            sub_earth_latitude,
            pole_angle: pole_angle.rem_euclid(360.0),
            distance_au: distance,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The smallest signed difference of two angles in degrees.
    fn angle_diff(a: f64, b: f64) -> f64 {
        (a - b + 180.0).rem_euclid(360.0) - 180.0
    }

    /// Meeus' worked examples. `D_E`/`B` are Meeus' (the latitude of the Earth-direction vector, "planetocentric"), `P` the
    /// position angle of the pole; JPL Horizons gives the same within 0.05 degrees (table below).
    /// Jupiter, example 43.a: 1992 December 16, 0h. Saturn, example 45.a: the same date. Mars, example 42.a: 1992 November 9.
    #[test]
    fn meeus_worked_examples() {
        let jupiter = Planet::Jupiter.pole_geometry(unix_seconds(1992, 12, 16, 0, 0, 0.0));
        assert!(
            (jupiter.sub_earth_latitude - (-2.48)).abs() < 0.15,
            "Jupiter D_E {}",
            jupiter.sub_earth_latitude
        );
        assert!(
            angle_diff(jupiter.pole_angle, 24.80).abs() < 0.2,
            "Jupiter P {}",
            jupiter.pole_angle
        );
        let saturn = Planet::Saturn.pole_geometry(unix_seconds(1992, 12, 16, 0, 0, 0.0));
        assert!(
            (saturn.sub_earth_latitude - 16.42).abs() < 0.3,
            "Saturn B {}",
            saturn.sub_earth_latitude
        );
        assert!(
            angle_diff(saturn.pole_angle, 6.74).abs() < 0.3,
            "Saturn P {}",
            saturn.pole_angle
        );
        let mars = Planet::Mars.pole_geometry(unix_seconds(1992, 11, 9, 0, 0, 0.0));
        assert!(
            (mars.sub_earth_latitude - 12.43).abs() < 0.3,
            "Mars D_E {}",
            mars.sub_earth_latitude
        );
        assert!(
            angle_diff(mars.pole_angle, 347.64).abs() < 0.3,
            "Mars P {}",
            mars.pole_angle
        );
    }

    /// JPL Horizons (geocentric, "ObsSub-LAT" and "NP.ang"), 0h UT. Horizons gives the planetographic latitude of the disc
    /// centre; `tan(D_E) = (1 - f)^2 tan(latitude)` turns it into the angle of the line of sight above the equator.
    /// Planet, date `(year, month, day)`, ObsSub-LAT, NP.ang.
    type Row = (Planet, (i32, u32, u32), f64, f64);
    const HORIZONS: &[Row] = &[
        (Planet::Jupiter, (1992, 11, 9), -2.489887, 25.2673),
        (Planet::Jupiter, (1992, 12, 16), -2.837473, 24.7971),
        (Planet::Jupiter, (2000, 1, 1), 3.310427, 337.1605),
        (Planet::Jupiter, (2005, 5, 5), -3.121019, 24.9344),
        (Planet::Jupiter, (2010, 6, 15), 2.377204, 334.5250),
        (Planet::Jupiter, (2015, 9, 1), -0.845302, 23.6307),
        (Planet::Jupiter, (2020, 7, 14), -1.441845, 349.0582),
        (Planet::Jupiter, (2023, 3, 10), 2.732389, 335.4139),
        (Planet::Jupiter, (2026, 11, 12), -0.275168, 21.8858),
        (Planet::Jupiter, (2030, 1, 1), -3.560156, 15.9835),
        (Planet::Saturn, (1992, 11, 9), 21.106531, 6.7822),
        (Planet::Saturn, (1992, 12, 16), 19.930632, 6.7149),
        (Planet::Saturn, (2000, 1, 1), -23.026571, 0.2158),
        (Planet::Saturn, (2005, 5, 5), -28.166695, 353.2780),
        (Planet::Saturn, (2010, 6, 15), 2.230695, 355.7143),
        (Planet::Saturn, (2015, 9, 1), 28.982825, 1.9180),
        (Planet::Saturn, (2020, 7, 14), 25.874466, 6.7520),
        (Planet::Saturn, (2023, 3, 10), 12.476208, 6.0286),
        (Planet::Saturn, (2026, 11, 12), -7.759710, 3.4544),
        (Planet::Saturn, (2030, 1, 1), -25.689657, 359.4501),
        (Planet::Mars, (1992, 11, 9), 12.579932, 347.6432),
        (Planet::Mars, (1992, 12, 16), 11.207313, 348.2106),
        (Planet::Mars, (2000, 1, 1), -23.172064, 351.8705),
        (Planet::Mars, (2005, 5, 5), -23.554837, 348.5015),
        (Planet::Mars, (2010, 6, 15), 24.275795, 12.1923),
        (Planet::Mars, (2015, 9, 1), 19.788551, 359.8759),
        (Planet::Mars, (2020, 7, 14), -22.417778, 329.1530),
        (Planet::Mars, (2023, 3, 10), -2.227032, 330.1213),
        (Planet::Mars, (2026, 11, 12), 21.533028, 5.8229),
        (Planet::Mars, (2030, 1, 1), -19.686438, 0.2333),
    ];

    #[test]
    fn agrees_with_jpl_horizons_over_forty_years() {
        let mut worst = (0.0f64, 0.0f64);
        for &(planet, (y, m, d), latitude, pole_angle) in HORIZONS {
            let q = 1.0 - planet.flattening();
            let expected_d = (q * q * latitude.to_radians().tan()).atan().to_degrees();
            let got = planet.pole_geometry(unix_seconds(y, m, d, 0, 0, 0.0));
            let error_d = (got.sub_earth_latitude - expected_d).abs();
            if std::env::var_os("PLANET_VERBOSE").is_some() {
                eprintln!(
                    "{planet:?} {y}-{m}-{d}: dD {:.3}",
                    got.sub_earth_latitude - expected_d
                );
            }
            let error_p = angle_diff(got.pole_angle, pole_angle).abs();
            worst = (worst.0.max(error_d), worst.1.max(error_p));
            assert!(
                error_d < 0.8 && error_p < 0.8,
                "{planet:?} {y}-{m}-{d}: D_E {} (Horizons {expected_d}), P {} (Horizons {pole_angle})",
                got.sub_earth_latitude,
                got.pole_angle
            );
        }
        eprintln!(
            "worst error against Horizons: D_E {:.3}, P {:.3} degrees",
            worst.0, worst.1
        );
        // The tolerance above is what the derotation needs; this keeps the claim in the module text honest.
        assert!(worst.0 < 0.5 && worst.1 < 0.5, "worst error {worst:?}");
    }

    #[test]
    fn rotation_follows_the_period_and_runs_counter_clockwise() {
        let period = Planet::Jupiter.rotation_period();
        assert!((Planet::Jupiter.rotation_degrees(period) - 360.0).abs() < 1e-9);
        // Jupiter turns about 36.3 degrees an hour, Saturn about 34.1, Mars about 14.6.
        assert!((Planet::Jupiter.rotation_degrees(3600.0) - 36.27).abs() < 0.02);
        assert!((Planet::Saturn.rotation_degrees(3600.0) - 34.09).abs() < 0.02);
        assert!((Planet::Mars.rotation_degrees(3600.0) - 14.62).abs() < 0.02);
    }

    #[test]
    fn unix_seconds_matches_known_dates() {
        assert_eq!(unix_seconds(1970, 1, 1, 0, 0, 0.0), 0.0);
        assert_eq!(unix_seconds(2000, 1, 1, 12, 0, 0.0), 946_728_000.0);
        assert_eq!(unix_seconds(2024, 2, 29, 0, 0, 0.0), 1_709_164_800.0);
    }

    #[test]
    fn planets_are_found_by_name() {
        assert_eq!(Planet::from_name("JUPITER.AVI"), Some(Planet::Jupiter));
        assert_eq!(Planet::from_name("2023-sat_ir.ser"), Some(Planet::Saturn));
        assert_eq!(Planet::from_name("Mars"), Some(Planet::Mars));
        assert_eq!(Planet::from_name("moon.ser"), None);
    }
}
