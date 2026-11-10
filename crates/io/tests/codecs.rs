//! Compressed AVI codecs against ffmpeg's decode of the same files (see `tests/data/README.md`).

use planets_io::avi::AviReader;
use std::path::PathBuf;

fn data_path(file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data")
        .join(file)
}

/// Decodes every frame of `name.avi` and compares with ffmpeg's `name.rgb` (rgb24, frames back to
/// back). Returns the largest sample difference in 1/255 steps and the PSNR over all samples.
fn compare(name: &str, expect_channels: u8) -> (u8, f64) {
    let mut reader = AviReader::open(&data_path(&format!("{name}.avi"))).unwrap();
    let reference = std::fs::read(data_path(&format!("{name}.rgb"))).unwrap();
    let info = reader.info().clone();
    let pixels = info.width as usize * info.height as usize;
    assert_eq!(reference.len(), pixels * 3 * info.frame_count as usize);

    let mut max_diff = 0u8;
    let mut squares = 0f64;
    let mut samples = 0f64;
    for index in 0..info.frame_count {
        let img = reader.read_frame(index).unwrap();
        assert_eq!(img.channels, expect_channels, "{name} channels");
        assert_eq!(img.data.len(), pixels * usize::from(expect_channels));
        let frame = &reference[index as usize * pixels * 3..][..pixels * 3];
        for (i, &v) in img.data.iter().enumerate() {
            assert!((0.0..=1.0).contains(&v), "{name}: sample {i} out of range");
            let got = (v * 255.0).round() as i32;
            let px = i / usize::from(expect_channels);
            let ch = i % usize::from(expect_channels);
            // A single channel image is compared with the red channel (r = g = b for gray).
            let want = i32::from(frame[px * 3 + ch]);
            let diff = (got - want).abs();
            max_diff = max_diff.max(diff as u8);
            squares += f64::from(diff * diff);
            samples += 1.0;
        }
    }
    let psnr = if squares == 0.0 {
        f64::INFINITY
    } else {
        10.0 * (255.0 * 255.0 / (squares / samples)).log10()
    };
    (max_diff, psnr)
}

fn assert_exact(name: &str, channels: u8) {
    let (max_diff, _) = compare(name, channels);
    assert_eq!(max_diff, 0, "{name} must match ffmpeg exactly");
}

fn assert_close(name: &str, channels: u8, max: u8) {
    let (max_diff, psnr) = compare(name, channels);
    assert!(
        max_diff <= max,
        "{name}: max diff {max_diff} > {max} (PSNR {psnr:.1} dB)"
    );
}

#[test]
fn huffyuv_rgb24_left_is_exact() {
    assert_exact("hfyu_bgr24_left", 3);
}

#[test]
fn huffyuv_rgb24_gradient_is_exact() {
    assert_exact("hfyu_bgr24_plane", 3);
}

#[test]
fn huffyuv_rgb24_odd_size_is_exact() {
    assert_exact("hfyu_bgr24_odd", 3);
}

#[test]
fn huffyuv_rgb24_interlaced_gradient_is_exact() {
    assert_exact("hfyu_bgr24_ilace_plane", 3);
}

#[test]
fn huffyuv_rgb32_drops_alpha() {
    assert_exact("hfyu_bgra_left", 3);
}

#[test]
fn huffyuv_yuy2_left() {
    assert_close("hfyu_yuy2_left", 3, 2);
}

#[test]
fn huffyuv_yuy2_gradient() {
    assert_close("hfyu_yuy2_plane", 3, 2);
}

#[test]
fn huffyuv_yuy2_median() {
    assert_close("hfyu_yuy2_median", 3, 2);
}

#[test]
fn huffyuv_yuy2_interlaced_gradient() {
    assert_close("hfyu_yuy2_ilace_plane", 3, 2);
}

#[test]
fn huffyuv_yuy2_interlaced_median() {
    assert_close("hfyu_yuy2_ilace_median", 3, 2);
}

#[test]
fn ffvhuff_yuv420_left() {
    assert_close("ffvh_420_left", 3, 2);
}

#[test]
fn ffvhuff_yuv420_median() {
    assert_close("ffvh_420_median", 3, 2);
}

#[test]
fn ffvhuff_yuv422_gradient() {
    assert_close("ffvh_422_plane", 3, 2);
}

#[test]
fn ffvhuff_yuv422_with_per_frame_tables() {
    assert_close("ffvh_422_context", 3, 2);
}

#[test]
fn ffvhuff_rgb32_gradient() {
    assert_exact("ffvh_bgra_plane", 3);
}

#[test]
fn ffvhuff_planar_rgb_is_exact() {
    assert_exact("ffvh_gbrp_left", 3);
}

#[test]
fn ffvhuff_planar_yuv444_median() {
    assert_close("ffvh_444_median", 3, 2);
}

#[test]
fn ffvhuff_gray_gives_one_channel() {
    assert_exact("ffvh_gray_left", 1);
}

#[test]
fn ffvhuff_gray_gradient() {
    assert_exact("ffvh_gray_plane", 1);
}

#[test]
fn mjpeg_yuv444_matches_ffmpeg() {
    assert_close("mjpg_444", 3, 3);
}

#[test]
fn mjpeg_yuv420_matches_ffmpeg() {
    let (max_diff, psnr) = compare("mjpg_420", 3);
    assert!(psnr >= 40.0, "PSNR {psnr:.1} dB, max diff {max_diff}");
}

#[test]
fn mjpeg_gray_gives_one_channel() {
    assert_close("mjpg_gray", 1, 3);
}
