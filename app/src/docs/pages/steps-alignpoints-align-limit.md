---
title: Alignpoints, Align and Best Frames
group: Using the window
order: 5
---
# Alignpoints, Align and Best Frames

Steps 2 to 4: find the planet in every frame, measure how far each frame has to move and keep the sharpest ones. This page names the parts of the three steps; how they work is in [Align your frames](align-your-frames) and [Frame quality and Best Frames](frame-quality-and-limit).

## The three steps

| Step | Button | What it does | Then |
|---|---|---|---|
| 2 Alignpoints | Set Alignpoints | Places alignpoints on the frame that is shown | Align |
| 3 Align | Align | Lines up all frames on them and rates how sharp each frame is | Best Frames |
| 4 Best Frames | Keep Best Frames | Keeps the sharpest frames | Stack |

The steps follow each other in this order: **Align** is possible after **Set Alignpoints**, **Keep Best Frames** after Align. Each button runs its step and the next step opens by itself. A step that is done carries a check mark; click it to see its result again. After a run the panel shows the result in one line: *82 points* for Alignpoints, *10 frames rated* for Align, *8 of 10 frames kept* for Best Frames. **Set Alignpoints**, **Align** and **Keep Best Frames** stay available once they have run, also after Stack and Wavelets; the panel then offers **Set Alignpoints Again**, **Align Again** or **Keep Best Frames Again**. Running one again discards what was built on the old result: for Set Alignpoints and Align the choice of frames, the stack, the sharpened image and the color adjustment, for Keep Best Frames the stack. See [Align your frames](align-your-frames).

## Settings in the step panel

| Step | Settings you see first | Under Advanced |
|---|---|---|
| Alignpoints | The **Threshold** slider (*more points* to *fewer points*, afterwards *Threshold · 82 points* or similar), Min. distance between points (px), Min. distance from image edge (px), **Draw Region** | The group *Point Placement*: Brightness measure with Min. brightness (0–255) and Max. brightness (0–255), Number of alignpoints, Keep points inside the scan frame. See [Align your frames](align-your-frames) |
| Align | Box size per point (px), Max. point shift (px) | *Alignment mode* with **No Align** and **Scan Frames**, and the rest of the alignment settings: Show Alignment, Align by center of gravity, Brightness threshold, Estimate rotation |
| Best Frames | The four modes At least % of the best quality, Best % of frames, Best number of frames and Best frames per point, each with its field, and the quality chart. See [Frame quality and Best Frames](frame-quality-and-limit) | Nothing |

**Draw Region** works, see [Align your frames](align-your-frames). **No Align** and **Scan Frames** are dimmed and marked **Not available yet**.

*Keep points inside the scan frame*, *Show Alignment* and *Estimate rotation* are also **Not available yet**. *Align by center of gravity* with its *Brightness threshold* works in the Mac version (locked in the Windows version for now), see [Align your frames](align-your-frames).

## Quality chart

The Best Frames step shows the quality of the frames as bars, one bar per frame, the sharpest frame on the left and the blurriest on the right. Kept frames are amber, dropped ones gray. A dashed line marks the limit: for At least % of the best quality a horizontal line at that quality, for the other modes a vertical line after the last frame that stays. With more frames than pixels, one bar stands for several frames. The chart follows every change of the value at once.

## Check boxes in the Show menu

In these three steps **Show ▾** at the top right of the image holds: Show Full Image · Show Region · Show Alignpoints (on) · Show Align Data · Show Frame List · Show Quality Graph · Show Prefilter. **Show Region** is locked until a region is drawn. **Show Full Image**, **Show Align Data** and **Show Prefilter** are **Not available yet**: they sit under a heading of that name in the menu and cannot be chosen. The others are described in [The image area](image-area).

## Frame bar

At the bottom: the state (*Loaded*, after Align *Aligned*), **Go to frame**, a slider over all frames and the position, for example `Frame 5 of 10`. Moving the slider shows that frame in the image. The frame bar is there in Alignpoints, Align and Best Frames only.

## Frame List and Quality Graph

**Show Frame List** lists every frame with a check mark; after Align the best frame comes first, with its quality in percent of the best frame. **Check All** and **Check None** work, **Clear List** is **Not available yet**. **Go to Best Frame** in the card (and under the quality chart in the step Best Frames) shows the best frame and opens the step Alignpoints, so that you can line all frames up on it; it is grayed out until Align has run. See [Frame quality and Best Frames](frame-quality-and-limit). **Show Quality Graph** draws Quality and Difference per frame as two lines; before Align it asks you to run Align.

New to a term? See the [Glossary](glossary).
