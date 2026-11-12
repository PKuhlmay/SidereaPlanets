---
title: Supported formats
group: Reference
order: 10
---
# Supported formats

What Siderea Planets can read today. Everything else that is planned is marked **Not available yet**.

## SER

Raw video from astronomy cameras. Bit depths from 8 to 16 bit, mono as well as RGB and BGR color. Bayer data is kept as raw data together with its pattern.

## AVI

| Variant | Status |
|---|---|
| Uncompressed (8 bit with palette, 24 and 32 bit), including large files in several segments | Works |
| Lagarith | Works |
| MJPEG | Works |
| HuffYUV | Works |
| MPEG (.mpg, .mpeg) | Not available yet |

## Single images

Still frames in BMP, JPEG, TIFF, PNG and FITS format, opened as a sequence: **Not available yet**.

## Wavelet scheme files

The settings of the wavelet layers can be stored in small `.rwv` files, see [Wavelet sharpening](wavelet-sharpening). Project files end in `.rkproj`, see [Save and export](save-and-export).
