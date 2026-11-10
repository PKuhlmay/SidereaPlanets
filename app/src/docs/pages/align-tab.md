---
title: The Align tab
group: Using the window
order: 5
---
# The Align tab

The first step: find the planet in every frame and measure how far each frame has to move.

## Steps

| Button | What it does | Status |
|---|---|---|
| Set Alignpoints | Sets points on the current frame | Not available yet |
| Align | Aligns all frames and rates their quality | Not available yet |
| Limit | Keeps the best frames, then opens the Stack tab | Not available yet |

The buttons already follow the order of the reference program: **Set Alignpoints** is the first step, **Align** is possible after it, **Limit** after Align.

## Check boxes in the action row

Colour · LRGB · Show Full Image · Show ROI (locked) · Show Alignpoints (on) · Show Aligndata · Show Framelist · Show Registrationgraph · Show Prefilter. Colour is set from the recording when you open it.

## Settings on the left

| Group | Setting | Default |
|---|---|---|
| Set Alignpoint parameters | Minimum distance between | 30 |
| | Min distance from edge | 20 |
| | Intensity selection | Default (or 3x3 area, Lowest pixelvalue) |
| | Lo / Hi | 30 / 230 |
| | Threshold | Slider, from **weakest** to **strongest**. Locked until the points are set; then the label reads *Alignpoints: n* |
| | Number of Alignpoints | Shows the count, cannot be typed |
| Alignment setup | Alignmentbox size | 30 |
| | Max Alignpoint movement | 5 |
| | No Align · R.o.Interest · Scan Frames | Scan Frames opens after Align. None of the three works yet |
| Limit Setup | Lowest Quality (%) · Best Frames (%) · Frames/Apoint · Best Frames | Lowest Quality (%) |
| | Percent value | 80 |
| | Number of frames | 200, used by Frames/Apoint and Best Frames |

*Keep Alignpoints inside ScanFrame*, *Align by Centre of gravity*, *Lum. Threshold* and *Estimate Rotation* are locked, as they are in the reference program.

## Frame bar

At the bottom: the state (*Loaded*, after Align *Aligned*), **Goto Frame**, a slider over all frames and the position, for example `Frame (1): 1/10`. Moving the slider shows that frame in the image.

## Framelist and Registrationgraph

**Show Framelist** lists every frame with a check mark; after Align the best frame comes first, with its quality in percent. **Check all** and **Check none** work, **Empty List** is **Not available yet**. **Show Registrationgraph** draws Quality and Difference per frame as two lines; before Align it asks you to run Align.
