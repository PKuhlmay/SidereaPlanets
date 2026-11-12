---
title: Alignpoints, Align and Limit
group: Using the window
order: 5
---
# Alignpoints, Align and Limit

Steps 2 to 4: find the planet in every frame, measure how far each frame has to move and keep the sharpest ones. This page names the parts of the three steps; how they work is in [Align your frames](align-your-frames) and [Frame quality and Limit](frame-quality-and-limit).

## The three steps

| Step | Button | What it does | Then |
|---|---|---|---|
| 2 Alignpoints | Set Alignpoints | Sets points on the current frame | Align |
| 3 Align | Align | Aligns all frames to it and rates their quality | Limit |
| 4 Limit | Limit | Keeps the best frames | Stack |

The steps follow the order of the reference program: **Align** is possible after **Set Alignpoints**, **Limit** after Align. Each button runs its step and the next step opens by itself. A step that is done carries a check mark; click it to see its result again. **Set Alignpoints**, **Align** and **Limit** stay available once they have run, also after Stack and Wavelets; the panel then offers **Set Alignpoints again**, **Align again** or **Limit again**. Running one again discards what was built on the old result: for Set Alignpoints and Align the Limit choice, the stack, the sharpened image and the colour adjustment, for Limit the stack. See [Align your frames](align-your-frames).

## Settings in the step panel

| Step | Settings you see first | Under Advanced |
|---|---|---|
| Alignpoints | The **Threshold** slider (*weakest* to *strongest*, afterwards *Alignpoints: 82* or similar), Minimum distance between, Min distance from edge, **R.o.Interest** | The rest of *Set Alignpoint parameters*: Intensity selection with Lo and Hi, Number of Alignpoints, Keep Alignpoints inside ScanFrame. See [Align your frames](align-your-frames) |
| Align | **No Align**, **Scan Frames**, Alignmentbox size, Max Alignpoint movement | The rest of *Alignment setup* |
| Limit | *Limit Setup* with Lowest Quality (%), Best Frames (%), Frames/Apoint and Best Frames and their fields, and the quality chart. See [Frame quality and Limit](frame-quality-and-limit) | Nothing |

**R.o.Interest** works, see [Align your frames](align-your-frames). **No Align** and **Scan Frames** are greyed out and marked **Not available yet**.

*Keep Alignpoints inside ScanFrame*, *Align by Centre of gravity*, *Lum. Threshold* and *Estimate Rotation* are locked, as they are in the reference program.

## Quality chart

The Limit step shows the quality of the frames as bars, one bar per frame, the best frame on the left. Kept frames are amber, dropped ones grey. A dashed line marks the limit: for Lowest Quality (%) a horizontal line at that quality, for the other modes a vertical line after the last frame that stays. With more frames than pixels, one bar stands for several frames. The chart follows every change of the value at once.

## Check boxes in the Show menu

In these three steps **Show ▾** at the top right of the image holds: Show Full Image · Show ROI · Show Alignpoints (on) · Show Aligndata · Show Framelist · Show Registrationgraph · Show Prefilter. **Show ROI** is locked until a region of interest is drawn. Their effect is described in [The image area](image-area).

## Frame bar

At the bottom: the state (*Loaded*, after Align *Aligned*), **Goto Frame**, a slider over all frames and the position, for example `Frame (1): 1/10`. Moving the slider shows that frame in the image. The frame bar is there in Alignpoints, Align and Limit only.

## Framelist and Registrationgraph

**Show Framelist** lists every frame with a check mark; after Align the best frame comes first, with its quality in percent of the best frame. **Check all** and **Check none** work, **Empty List** is **Not available yet**. **Go to best frame** in the card (and under the quality chart in the step Limit) shows the best frame and opens the step Alignpoints, so that you can use it as reference; it is greyed out until Align has run. See [Frame quality and Limit](frame-quality-and-limit). **Show Registrationgraph** draws Quality and Difference per frame as two lines; before Align it asks you to run Align.
