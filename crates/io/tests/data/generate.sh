#!/bin/bash
# Regenerates every fixture in this directory (needs ffmpeg 8.x and python 3). Run from here.
cd "$(dirname "$0")" || exit 1

# gen NAME SOURCE ENCODER-ARGS...: writes NAME.avi (2 frames) and NAME.rgb, ffmpeg's rgb24 decode of it.
# The reference conversion upsamples chroma by replication, like the decoder under test.
gen() {
  name=$1; src=$2; shift 2
  ffmpeg -v error -y -f lavfi -i "$src" -frames:v 2 "$@" "$name.avi"
  ffmpeg -v error -y -i "$name.avi" -sws_flags "${REF:-neighbor+accurate_rnd+full_chroma_int}" \
    -f rawvideo -pix_fmt rgb24 "$name.rgb"
}

T="testsrc2=size=48x32:rate=5"
gen hfyu_bgr24_left   "$T" -c:v huffyuv -pix_fmt bgr24 -pred left
gen hfyu_bgr24_plane  "$T" -c:v huffyuv -pix_fmt bgr24 -pred plane
# Odd size, for rows that are not a multiple of anything.
gen hfyu_bgr24_odd    "nullsrc=size=35x19:rate=5,format=gbrp,geq=r='mod(X*7+N*5,256)':g='mod(Y*13,256)':b='mod(X*Y+N*9,256)'" -c:v huffyuv -pix_fmt bgr24 -pred left
gen hfyu_bgra_left    "$T" -c:v huffyuv -pix_fmt bgra -pred left
gen hfyu_yuy2_left    "$T" -c:v huffyuv -pix_fmt yuv422p -pred left
gen hfyu_yuy2_plane   "$T" -c:v huffyuv -pix_fmt yuv422p -pred plane
gen hfyu_yuy2_median  "$T" -c:v huffyuv -pix_fmt yuv422p -pred median
# Interlaced prediction (the row two above is the predictor).
gen hfyu_yuy2_ilace_plane  "$T" -c:v huffyuv -pix_fmt yuv422p -pred plane -flags +ilme
gen hfyu_yuy2_ilace_median "$T" -c:v huffyuv -pix_fmt yuv422p -pred median -flags +ilme
gen hfyu_bgr24_ilace_plane "$T" -c:v huffyuv -pix_fmt bgr24 -pred plane -flags +ilme
gen ffvh_420_left     "$T" -c:v ffvhuff -pix_fmt yuv420p -pred left
gen ffvh_420_median   "$T" -c:v ffvhuff -pix_fmt yuv420p -pred median
gen ffvh_422_plane    "$T" -c:v ffvhuff -pix_fmt yuv422p -pred plane
gen ffvh_422_context  "$T" -c:v ffvhuff -pix_fmt yuv422p -pred median -context 1
gen ffvh_bgra_plane   "$T" -c:v ffvhuff -pix_fmt bgra -pred plane
# ffvhuff stores these as planar streams (format version 3).
gen ffvh_gbrp_left    "$T" -c:v ffvhuff -pix_fmt gbrp -pred left
gen ffvh_444_median   "$T" -c:v ffvhuff -pix_fmt yuv444p -pred median
gen ffvh_gray_left    "$T" -c:v ffvhuff -pix_fmt gray -pred left
gen ffvh_gray_plane   "$T" -c:v ffvhuff -pix_fmt gray -pred plane

# MJPEG: a smooth source, so the decoder's chroma upsampling does not matter for the comparison.
G="gradients=size=48x32:rate=5:seed=3"
REF=bilinear gen mjpg_420 "$G" -c:v mjpeg -pix_fmt yuvj420p -q:v 3
# mjpg_444 uses the standard Huffman tables (the others use ffmpeg's default per-frame optimal ones).
gen mjpg_444 "$G" -c:v mjpeg -pix_fmt yuvj444p -q:v 3 -huffman default

# Grayscale JPEG: ffmpeg cannot encode it, so gray_jpeg.py writes the frames.
tmp=$(mktemp -d)
python gray_jpeg.py "$tmp"
ffmpeg -v error -y -framerate 5 -i "$tmp/frame%d.jpg" -c copy mjpg_gray.avi
ffmpeg -v error -y -i mjpg_gray.avi -f rawvideo -pix_fmt rgb24 mjpg_gray.rgb
rm -r "$tmp"
