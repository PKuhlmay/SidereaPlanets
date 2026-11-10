Small AVI fixtures for the compressed codecs (`crates/io/tests/codecs.rs`, and `mjpeg.rs` unit tests). Each `.avi`
has a matching `.rgb`: ffmpeg 8.1's decode of the same file (`-f rawvideo -pix_fmt rgb24`, all frames back to back,
chroma upsampled by replication so it matches the decoder under test). Everything is 2 frames, mostly 48x32.

Regenerate with `bash generate.sh` (needs ffmpeg and python 3). The options per file are in that script and in the
file name: `hfyu_*` = `-c:v huffyuv`, `ffvh_*` = `-c:v ffvhuff`, `mjpg_*` = `-c:v mjpeg`, then pixel format and
predictor. For example:

    ffmpeg -f lavfi -i testsrc2=size=48x32:rate=5 -frames:v 2 -c:v huffyuv -pix_fmt yuv422p -pred median hfyu_yuy2_median.avi
    ffmpeg -i hfyu_yuy2_median.avi -sws_flags neighbor+accurate_rnd+full_chroma_int -f rawvideo -pix_fmt rgb24 hfyu_yuy2_median.rgb

`mjpg_gray.avi` is made by `gray_jpeg.py` because ffmpeg cannot encode single-component JPEG. `mjpg_444.avi` uses the
standard Huffman tables so the unit test can strip the DHT segment and still decode it.
