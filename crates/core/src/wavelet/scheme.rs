//! Reading and writing wavelet schemes in the `.rwv` format of the reference program (`docs/formats.md`).
//!
//! The clone keeps the original format so that existing schemes stay usable. It is a plain INI file.
//! Sections and keys are case-insensitive, missing keys keep their default, unknown ones are ignored.

use super::settings::{Filter, LAYERS, Scheme, WaveletSettings};
use std::fmt;

/// A `.rwv` file that cannot be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemeError {
    /// 1-based line number.
    pub line: usize,
    pub message: String,
}

impl fmt::Display for SchemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "scheme file, line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for SchemeError {}

/// Writes settings as a `.rwv` file (CRLF line ends, like the original).
pub fn to_rwv(settings: &WaveletSettings) -> String {
    let s = &settings.structure;
    let mut out = String::new();
    let mut line = |text: String| {
        out.push_str(&text);
        out.push_str("\r\n");
    };
    line("[scheme]".into());
    line(format!(
        "itemindex={}",
        match s.scheme {
            Scheme::Dyadic => 0,
            Scheme::Linear => 1,
        }
    ));
    line(format!("initial={}", s.initial));
    line(format!("step={}", s.step));
    line(String::new());
    line("[method]".into());
    line(format!(
        "itemindex={}",
        match s.filter {
            Filter::Default => 0,
            Filter::Gaussian => 1,
        }
    ));
    let per_layer = |line: &mut dyn FnMut(String), section: &str, value: &dyn Fn(usize) -> i64| {
        line(String::new());
        line(format!("[{section}]"));
        for i in 0..LAYERS {
            line(format!("layer{}={}", i + 1, value(i)));
        }
    };
    let l = &settings.layers;
    per_layer(&mut line, "checkbox", &|i| i64::from(l[i].enabled));
    per_layer(&mut line, "slider", &|i| scaled(l[i].gain, 10.0));
    per_layer(&mut line, "Gauss", &|i| scaled(l[i].sharpen, 100.0));
    per_layer(&mut line, "Blur", &|i| scaled(l[i].denoise, 100.0));
    line(String::new());
    out
}

fn scaled(value: f32, factor: f32) -> i64 {
    (f64::from(value) * f64::from(factor)).round() as i64
}

/// Reads a `.rwv` file. Values outside the supported range are clamped ([`WaveletSettings::clamped`]).
pub fn from_rwv(text: &str) -> Result<WaveletSettings, SchemeError> {
    let mut settings = WaveletSettings::default();
    let mut section = String::new();
    for (index, raw) in text.lines().enumerate() {
        let line_no = index + 1;
        let line = raw.trim().trim_start_matches('\u{feff}');
        if line.is_empty() || line.starts_with(';') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = name.trim().to_ascii_lowercase();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(SchemeError {
                line: line_no,
                message: format!("expected key=value, found `{line}`"),
            });
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();
        let number = || -> Result<i64, SchemeError> {
            value.parse().map_err(|_| SchemeError {
                line: line_no,
                message: format!("`{value}` is not a whole number"),
            })
        };
        let layer = key
            .strip_prefix("layer")
            .and_then(|n| n.parse::<usize>().ok())
            .filter(|n| (1..=LAYERS).contains(n))
            .map(|n| n - 1);
        match (section.as_str(), key.as_str(), layer) {
            ("scheme", "itemindex", _) => {
                settings.structure.scheme = if number()? == 0 {
                    Scheme::Dyadic
                } else {
                    Scheme::Linear
                };
            }
            ("scheme", "initial", _) => settings.structure.initial = number()?.max(0) as u32,
            ("scheme", "step", _) => settings.structure.step = number()?.max(0) as u32,
            ("method", "itemindex", _) => {
                settings.structure.filter = if number()? == 0 {
                    Filter::Default
                } else {
                    Filter::Gaussian
                };
            }
            ("checkbox", _, Some(i)) => settings.layers[i].enabled = number()? != 0,
            ("slider", _, Some(i)) => settings.layers[i].gain = number()? as f32 / 10.0,
            ("gauss", _, Some(i)) => settings.layers[i].sharpen = number()? as f32 / 100.0,
            ("blur", _, Some(i)) => settings.layers[i].denoise = number()? as f32 / 100.0,
            _ => {}
        }
    }
    Ok(settings.clamped())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `jupiter-l1-10.rwv` of the reference case R1 (`docs/testdaten.md`).
    const L1_10: &str = "[scheme]\r\nitemindex=1\r\ninitial=1\r\nstep=0\r\n\r\n[method]\r\nitemindex=1\r\n\r\n\
        [checkbox]\r\nlayer1=1\r\nlayer2=1\r\nlayer3=1\r\nlayer4=1\r\nlayer5=1\r\nlayer6=1\r\n\r\n\
        [slider]\r\nlayer1=100\r\nlayer2=10\r\nlayer3=10\r\nlayer4=10\r\nlayer5=10\r\nlayer6=10\r\n\r\n\
        [Gauss]\r\nlayer1=10\r\nlayer2=10\r\nlayer3=10\r\nlayer4=10\r\nlayer5=10\r\nlayer6=10\r\n\r\n\
        [Blur]\r\nlayer1=0\r\nlayer2=0\r\nlayer3=0\r\nlayer4=0\r\nlayer5=0\r\nlayer6=0\r\n\r\n";

    #[test]
    fn reads_the_scheme_of_the_original() {
        let s = from_rwv(L1_10).unwrap();
        assert_eq!(s.layers[0].gain, 10.0);
        assert!(s.layers[1..].iter().all(|l| l.gain == 1.0));
        let mut expected = WaveletSettings::default();
        expected.layers[0].gain = 10.0;
        assert_eq!(s, expected);
    }

    #[test]
    fn writes_the_same_file_as_the_original() {
        let mut s = WaveletSettings::default();
        s.layers[0].gain = 10.0;
        assert_eq!(to_rwv(&s), L1_10);
    }

    #[test]
    fn roundtrip_keeps_all_values() {
        let mut s = WaveletSettings::default();
        s.structure.scheme = Scheme::Dyadic;
        s.structure.filter = Filter::Default;
        s.structure.initial = 2;
        s.structure.step = 3;
        s.layers[1].enabled = false;
        s.layers[2].gain = -2.5;
        s.layers[3].denoise = 1.25;
        s.layers[4].sharpen = 0.375;
        assert_eq!(from_rwv(&to_rwv(&s)).unwrap(), s);
    }

    #[test]
    fn missing_keys_keep_their_defaults() {
        let s = from_rwv("[slider]\nlayer3=50\n").unwrap();
        assert_eq!(s.layers[2].gain, 5.0);
        let mut expected = WaveletSettings::default();
        expected.layers[2].gain = 5.0;
        assert_eq!(s, expected);
    }

    #[test]
    fn names_are_case_insensitive_and_unknown_keys_are_ignored() {
        let s = from_rwv("[SLIDER]\nLayer1=30\nlayer9=1\n[other]\nfoo=bar\n").unwrap();
        assert_eq!(s.layers[0].gain, 3.0);
    }

    #[test]
    fn out_of_range_values_are_clamped() {
        let s = from_rwv("[slider]\nlayer1=99999\nlayer2=-900\n").unwrap();
        assert_eq!(s.layers[0].gain, 100.0);
        assert_eq!(s.layers[1].gain, -5.0);
    }

    #[test]
    fn reports_the_line_of_a_broken_value() {
        let err = from_rwv("[slider]\nlayer1=abc\n").unwrap_err();
        assert_eq!(err.line, 2);
        let err = from_rwv("[slider]\njunk\n").unwrap_err();
        assert_eq!(err.line, 2);
    }
}
