use super::*;

/// MSB-first bit writer for hand-made probability headers.
#[derive(Default)]
struct BitWriter {
    bytes: Vec<u8>,
    bits: usize,
}

impl BitWriter {
    fn bit(&mut self, b: bool) {
        if self.bits.is_multiple_of(8) {
            self.bytes.push(0);
        }
        if b {
            *self.bytes.last_mut().unwrap() |= 0x80 >> (self.bits % 8);
        }
        self.bits += 1;
    }

    fn bits(&mut self, value: u32, n: u32) {
        for i in (0..n).rev() {
            self.bit(value >> i & 1 == 1);
        }
    }

    /// Writes a probability value with the Fibonacci prefix code Lagarith uses.
    fn prob(&mut self, value: u32) {
        let series = [1u32, 2, 3, 5, 8, 13, 21];
        let m = value + 1;
        let bits = 31 - m.leading_zeros();
        let mut remaining = bits + 1;
        let mut used = [false; 7];
        for i in (0..7).rev() {
            if series[i] <= remaining {
                used[i] = true;
                remaining -= series[i];
            }
        }
        let top = used.iter().rposition(|&u| u).unwrap();
        for &u in &used[..=top] {
            self.bit(u);
        }
        self.bit(true);
        self.bits(m, bits);
    }
}

/// Probability header for the given symbol frequencies (index = symbol, the rest zero).
fn prob_header(freqs: &[u32]) -> Vec<u8> {
    let mut w = BitWriter::default();
    let mut i = 0;
    while i < 256 {
        let f = freqs.get(i).copied().unwrap_or(0);
        w.prob(f);
        if f == 0 {
            // Zero frequencies are followed by the count of further zeros.
            let mut run = 0;
            while i + 1 + run < 256 && freqs.get(i + 1 + run).copied().unwrap_or(0) == 0 {
                run += 1;
            }
            w.prob(run as u32);
            i += run;
        }
        i += 1;
    }
    w.bytes
}

/// A range coded plane that decodes to the symbol `7` everywhere (then prediction applies).
fn constant_seven_plane() -> Vec<u8> {
    let mut freqs = vec![0u32; 8];
    freqs[7] = 1;
    let mut plane = vec![0u8];
    plane.extend(prob_header(&freqs));
    plane.extend([0; 4]);
    plane
}

fn fmt(width: u32, height: u32, bpp: u16) -> FrameFormat<'static> {
    FrameFormat {
        width,
        height,
        bits_per_pixel: bpp,
        extra: &[],
    }
}

fn bytes(img: &Image) -> Vec<u8> {
    img.data.iter().map(|v| (v * 255.0).round() as u8).collect()
}

/// Header of an arithmetic frame whose planes have the given lengths.
fn arith_frame(kind: u8, planes: &[Vec<u8>]) -> Vec<u8> {
    let mut offset = 9 + 4 * u32::from(kind == 8) as usize;
    let mut offsets = Vec::new();
    for p in planes {
        offsets.push(offset as u32);
        offset += p.len();
    }
    let mut frame = vec![kind];
    frame.extend(offsets[1].to_le_bytes());
    frame.extend(offsets[2].to_le_bytes());
    if kind == 8 {
        frame.extend(offsets[3].to_le_bytes());
    }
    for p in planes {
        frame.extend(p);
    }
    frame
}

#[test]
fn zero_run_lengths_follow_the_sign_folding() {
    assert_eq!(zero_run_len(0), 0);
    assert_eq!(zero_run_len(1), 2);
    assert_eq!(zero_run_len(3), 6);
    assert_eq!(zero_run_len(127), 254);
    assert_eq!(zero_run_len(255), 1);
    assert_eq!(zero_run_len(254), 3);
    assert_eq!(zero_run_len(128), 255);
}

#[test]
fn escape_after_n_zeros_reads_a_run_length() {
    let mut symbols = [5, 0, 0, 3, 9, 0, 7].into_iter();
    let mut out = [1u8; 12];
    fill_plane(&mut out, 2, || symbols.next().unwrap());
    assert_eq!(out, [5, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 7]);
}

#[test]
fn zero_run_is_clipped_at_the_end_of_the_plane() {
    let mut symbols = [4, 0, 0, 3].into_iter();
    let mut out = [1u8; 6];
    fill_plane(&mut out, 2, || symbols.next().unwrap());
    assert_eq!(out, [4, 0, 0, 0, 0, 0]);
}

#[test]
fn without_escape_zeros_are_plain_symbols() {
    let mut symbols = [0, 0, 0, 6].into_iter();
    let mut out = [1u8; 4];
    fill_plane(&mut out, 0, || symbols.next().unwrap());
    assert_eq!(out, [0, 0, 0, 6]);
}

#[test]
fn median_prediction_uses_previous_row_end_as_left_neighbour() {
    // Row 0 is left-predicted and row 1 starts from the pixel above. From row 2 on, the first
    // pixel has the end of the row above as left and the end of the row before as top-left
    // neighbour: row 3 starts with median(3, 2, 3 + 2 - 3) = 2, plus the difference 2.
    let mut plane = [1, 1, 1, 1, 0, 0, 0, 0, 0, 2, 0, 0];
    predict_plane(&mut plane, 3, 4, Predictor::Rgb);
    assert_eq!(plane, [1, 2, 3, 2, 2, 3, 2, 2, 3, 4, 4, 4]);
}

#[test]
fn prediction_wraps_modulo_256() {
    let mut plane = [200, 100, 0, 0];
    predict_plane(&mut plane, 2, 2, Predictor::Rgb);
    // Row 0 is 200, 44; row 1 starts from the pixel above.
    assert_eq!(plane, [200, 44, 200, 44]);
}

#[test]
fn median_gradient_is_not_wrapped_to_a_byte() {
    // Left 0, top 252, top-left 254: the gradient is -2, not 254, so the median is 0.
    let mut plane = [254, 254, 2, 5];
    predict_plane(&mut plane, 2, 2, Predictor::Rgb);
    assert_eq!(plane, [254, 252, 0, 5]);
}

#[test]
fn yuy2_luma_keeps_its_first_sample_and_predicts_from_the_second() {
    let mut luma = [9, 1, 1, 1];
    predict_plane(&mut luma, 4, 1, Predictor::Yuy2Luma);
    assert_eq!(luma, [9, 1, 2, 3]);
    let mut chroma = [9, 1, 1, 1];
    predict_plane(&mut chroma, 4, 1, Predictor::Yuy2Chroma);
    assert_eq!(chroma, [9, 10, 11, 12]);
}

#[test]
fn planar_second_row_starts_from_the_end_of_the_first() {
    let mut plane = [10, 1, 1, 0, 0, 0];
    predict_plane(&mut plane, 3, 2, Predictor::Planar);
    assert_eq!(plane, [10, 11, 12, 12, 12, 12]);
}

#[test]
fn yuy2_gradient_is_wrapped_to_a_byte() {
    // Same neighbours as the unwrapped test above: left 0, top 252, top-left 254. Wrapped, the
    // gradient is 254 and the median 252, so the 5 becomes 1.
    let mut plane = [254, 0, 0, 254, 4, 0, 0, 5];
    predict_plane(&mut plane, 4, 2, Predictor::Yuy2Chroma);
    assert_eq!(plane, [254, 254, 254, 252, 0, 0, 0, 1]);
}

#[test]
fn yuy2_second_row_left_predicts_the_head_then_takes_the_median() {
    let mut luma = [7, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0];
    predict_plane(&mut luma, 6, 2, Predictor::Yuy2Luma);
    assert_eq!(luma, [7, 1, 2, 3, 4, 5, 6, 7, 8, 9, 9, 9]);
}

#[test]
fn probability_table_with_power_of_two_total_is_kept() {
    let table = read_prob_table(&mut BitReader::new(&prob_header(&[2, 2]))).unwrap();
    assert_eq!(table.scale, 2);
    assert_eq!(table.start[..4], [0, 2, 4, 4]);
}

#[test]
fn probability_table_is_scaled_up_to_a_power_of_two() {
    // Three symbols of weight 1: total 3 becomes 4, the spare unit goes to the first symbol.
    let table = read_prob_table(&mut BitReader::new(&prob_header(&[1, 1, 1]))).unwrap();
    assert_eq!(table.scale, 2);
    assert_eq!(table.start[..5], [0, 2, 3, 4, 4]);
    assert_eq!(table.start[256], 4);
}

#[test]
fn probability_table_without_any_weight_is_rejected() {
    assert!(read_prob_table(&mut BitReader::new(&prob_header(&[]))).is_err());
}

#[test]
fn range_decoder_with_two_equal_symbols_reads_the_bit_stream() {
    let table = read_prob_table(&mut BitReader::new(&prob_header(&[1, 1]))).unwrap();
    let data = [0b1010_0101, 0b1100_0011, 0, 0, 0, 0];
    let mut rc = RangeDecoder::new(&data, &table);
    let symbols: Vec<u8> = (0..16).map(|_| rc.next()).collect();
    assert_eq!(symbols, [1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 0, 0, 0, 0, 1, 1]);
}

#[test]
fn empty_frame_is_reported_as_unsupported_repeat() {
    let err = decode(&fmt(2, 2, 24), &[]).unwrap_err();
    assert!(matches!(err, CodecError::Unsupported { .. }));
}

#[test]
fn reduced_resolution_and_old_rgb_frames_are_unsupported() {
    for kind in [7u8, 11] {
        let err = decode(&fmt(2, 2, 24), &[kind, 0, 0, 0, 0, 0, 0, 0, 0]).unwrap_err();
        assert!(matches!(err, CodecError::Unsupported { .. }), "type {kind}");
    }
}

#[test]
fn solid_colour_frame_is_stored_as_bgr() {
    let img = decode(&fmt(3, 2, 24), &[6, 30, 20, 10]).unwrap();
    assert_eq!((img.width, img.height, img.channels), (3, 2, 3));
    assert_eq!(bytes(&img), [10, 20, 30].repeat(6));
}

#[test]
fn solid_rgba_frame_drops_alpha() {
    let img = decode(&fmt(2, 2, 32), &[9, 30, 20, 10, 99]).unwrap();
    assert_eq!(img.channels, 3);
    assert_eq!(bytes(&img), [10, 20, 30].repeat(4));
}

#[test]
fn solid_gray_frame_is_gray_in_rgb_modes() {
    let img = decode(&fmt(2, 1, 24), &[5, 77]).unwrap();
    assert_eq!(bytes(&img), [77; 6]);
}

#[test]
fn uncompressed_rgb_frame_has_padded_bottom_up_rows() {
    // 2x2 at 24 bit: rows are 6 bytes plus 2 padding bytes, bottom row first.
    let frame = [
        1, // frame type
        1, 2, 3, 4, 5, 6, 0, 0, // bottom row (BGR)
        7, 8, 9, 10, 11, 12, 0, 0, // top row
    ];
    let img = decode(&fmt(2, 2, 24), &frame).unwrap();
    assert_eq!(bytes(&img), [9, 8, 7, 12, 11, 10, 3, 2, 1, 6, 5, 4]);
}

#[test]
fn unaligned_rgb24_frame_has_unpadded_bottom_up_rows() {
    let frame = [2, 1, 2, 3, 4, 5, 6];
    let img = decode(&fmt(1, 2, 24), &frame).unwrap();
    assert_eq!(bytes(&img), [6, 5, 4, 3, 2, 1]);
}

#[test]
fn uncompressed_yuy2_frame_is_converted() {
    // Y0 U Y1 V for two white pixels.
    let img = decode(&fmt(2, 1, 16), &[1, 235, 128, 235, 128]).unwrap();
    assert_eq!(bytes(&img), [255; 6]);
}

#[test]
fn rgb_planes_are_added_to_green_and_flipped() {
    // Solid planes carry B-G, G and R-G; no prediction is applied to them.
    let planes = [vec![0xFF, 10], vec![0xFF, 100], vec![0xFF, 20]];
    let img = decode(&fmt(2, 2, 24), &arith_frame(4, &planes)).unwrap();
    assert_eq!(bytes(&img), [120, 100, 110].repeat(4));
}

#[test]
fn rgb_frame_with_range_coded_green_plane_is_predicted_and_flipped() {
    // Green decodes to 7 everywhere, prediction turns that into [7 14 / 14 21] in storage order
    // (bottom row first); R and B are zero differences, so all channels equal green.
    let planes = [vec![0xFF, 0], constant_seven_plane(), vec![0xFF, 0]];
    let img = decode(&fmt(2, 2, 24), &arith_frame(4, &planes)).unwrap();
    assert_eq!(bytes(&img), [14, 14, 14, 21, 21, 21, 7, 7, 7, 14, 14, 14]);
}

#[test]
fn rgba_frame_skips_the_alpha_plane() {
    let planes = [
        vec![0xFF, 10],
        vec![0xFF, 100],
        vec![0xFF, 20],
        vec![0xFF, 255],
    ];
    let img = decode(&fmt(2, 1, 32), &arith_frame(8, &planes)).unwrap();
    assert_eq!(img.channels, 3);
    assert_eq!(bytes(&img), [120, 100, 110].repeat(2));
}

#[test]
fn yuy2_frame_takes_chroma_planes_in_u_v_order() {
    // BT.601 red: Y 81, U 90, V 240. Chroma planes are half as wide.
    let planes = [vec![0xFF, 81], vec![0xFF, 90], vec![0xFF, 240]];
    let img = decode(&fmt(2, 2, 16), &arith_frame(3, &planes)).unwrap();
    for px in img.data.as_chunks::<3>().0.iter() {
        assert!(px[0] > 0.98 && px[1] < 0.02 && px[2] < 0.02, "{px:?}");
    }
}

#[test]
fn yv12_frame_takes_chroma_planes_in_v_u_order() {
    let planes = [vec![0xFF, 81], vec![0xFF, 240], vec![0xFF, 90]];
    let img = decode(&fmt(2, 2, 12), &arith_frame(10, &planes)).unwrap();
    for px in img.data.as_chunks::<3>().0.iter() {
        assert!(px[0] > 0.98 && px[1] < 0.02 && px[2] < 0.02, "{px:?}");
    }
}

#[test]
fn limited_range_bt601_maps_black_and_white() {
    for (got, want) in yuv_to_rgb(16, 128, 128).into_iter().zip([0.0, 0.0, 0.0]) {
        assert!((got - want).abs() < 1e-4, "{got}");
    }
    for (got, want) in yuv_to_rgb(235, 128, 128).into_iter().zip([1.0, 1.0, 1.0]) {
        assert!((got - want).abs() < 1e-4, "{got}");
    }
}

#[test]
fn truncated_frames_are_corrupt_not_a_panic() {
    let planes = [vec![0xFF, 10], vec![0xFF, 100], vec![0xFF, 20]];
    let mut frame = arith_frame(4, &planes);
    frame.truncate(12);
    assert!(matches!(
        decode(&fmt(2, 2, 24), &frame),
        Err(CodecError::Corrupt { .. })
    ));
    assert!(matches!(
        decode(&fmt(2, 2, 24), &[4, 0]),
        Err(CodecError::Corrupt { .. })
    ));
}
