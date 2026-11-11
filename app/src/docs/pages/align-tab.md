---
title: The Align tab
group: Using the window
order: 5
---
# The Align tab

The first step: find the planet in every frame and measure how far each frame has to move. This page names the parts of the tab; how the steps work is in [Align your frames](align-your-frames) and [Frame quality and Limit](frame-quality-and-limit).

## Steps

| Button | What it does |
|---|---|
| Set Alignpoints | Sets points on the current frame |
| Align | Aligns all frames to it and rates their quality |
| Limit | Keeps the best frames, then opens the Stack tab |

The buttons follow the order of the reference program: **Set Alignpoints** is the first step, **Align** is possible after it, **Limit** after Align. A step that is done carries a check mark. **Set Alignpoints** and **Align** stay available once they have run, also after Limit, Stack and Wavelet. Pressing either again starts the alignment over and discards what was built on the old one: the Limit choice, the stack, the sharpened image and the colour adjustment. See [Align your frames](align-your-frames).

## Check boxes in the action row

Colour · LRGB · Show Full Image · Show ROI · Show Alignpoints (on) · Show Aligndata · Show Framelist · Show Registrationgraph · Show Prefilter. Colour is set from the recording when you open it. **Show ROI** is locked until a region of interest is drawn.

## Settings on the left

| Group | What is in it |
|---|---|
| Set Alignpoint parameters | Minimum distance between, Min distance from edge, Intensity selection with Lo and Hi, the Threshold slider and Number of Alignpoints. See [Align your frames](align-your-frames) |
| Alignment setup | **No Align**, **R.o.Interest**, **Scan Frames**, Alignmentbox size and Max Alignpoint movement. **R.o.Interest** works, see [Align your frames](align-your-frames). **No Align** and **Scan Frames** are **Not available yet** |
| Limit Setup | Lowest Quality (%), Best Frames (%), Frames/Apoint and Best Frames with their fields. See [Frame quality and Limit](frame-quality-and-limit) |

*Keep Alignpoints inside ScanFrame*, *Align by Centre of gravity*, *Lum. Threshold* and *Estimate Rotation* are locked, as they are in the reference program.

## Frame bar

At the bottom: the state (*Loaded*, after Align *Aligned*), **Goto Frame**, a slider over all frames and the position, for example `Frame (1): 1/10`. Moving the slider shows that frame in the image.

## Framelist and Registrationgraph

**Show Framelist** lists every frame with a check mark; after Align the best frame comes first, with its quality in percent of the best frame. **Check all** and **Check none** work, **Empty List** is **Not available yet**. **Show Registrationgraph** draws Quality and Difference per frame as two lines; before Align it asks you to run Align.
