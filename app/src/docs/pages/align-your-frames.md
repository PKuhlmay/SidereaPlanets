---
title: Align your frames
group: Using the window
order: 5.1
---
# Align your frames

A planet wobbles from frame to frame: the air makes it dance, the mount drifts. **Set Alignpoints** and **Align** find out how far each frame has moved, and how sharp it is. The next pages use that to pick the best frames and stack them.

## Step by step

1. [Select a recording](select-a-recording). The first frame shows in the image area.
2. Move to a frame that is sharp and shows the planet well, with **Goto Frame** or the slider at the bottom. This frame is the reference for all others.
3. Press **Set Alignpoints**. Red points appear on the planet, and the label above the Threshold slider reads *Alignpoints: 82* or similar.
4. Optional: move the **Threshold** slider towards *strongest* to keep fewer, steadier points, or towards *weakest* to keep more. More on this under *Alignpoints* below.
5. Optional, before Set Alignpoints: press **R.o.Interest** and drag a rectangle over the image, to align on one area only. More on this under *Region of interest* below.
6. Press **Align**. The status bar shows *Loading frame: n* and the progress, the window title shows it as well (`SidereaPlanets · Align 40%`). **Cancel** stops, **Pause** holds and becomes **Resume**.
7. When *Alignment finished.* appears, **Limit** is next: [Frame quality and Limit](frame-quality-and-limit).

After Align, both **Set Alignpoints** and **Align** are locked. To start over, open the recording again.

## Alignpoints

An alignpoint is a spot on the reference frame that SidereaPlanets looks for in every other frame. A point is a small square (the *Alignmentbox size*); the program searches for the same pattern around it, up to *Max Alignpoint movement* away, and finds the position to a fraction of a pixel. Many points across the planet also show how the picture is bent by the air, not only moved.

The points are placed on bright, detailed parts. A straight edge is a poor point, because the pattern can slide along it; a corner or a spot is a good one.

| Setting | What it does | Default |
|---|---|---|
| Minimum distance between | Points keep this far apart. SidereaPlanets adds *Max Alignpoint movement* to it, so that the search areas of two points never overlap: with the defaults, 35 pixels | 30 |
| Min distance from edge | No points near the border of the frame (or of the region of interest) | 20 |
| Intensity selection | Which pixel decides whether a spot is bright enough: **Default** (the centre), **3x3 area** (the mean of 3 by 3 pixels), **Lowest pixelvalue** (the darkest of them) | Default |
| Lo, Hi | Only spots with a brightness between these values (0 to 255) get a point. Dark sky and burnt-out spots stay free | 30, 230 |
| Threshold | Drops the weakest points. The slider runs from 0 up to the number of points that were found; after Set Alignpoints it stands at about a tenth | about 10 % |
| Number of Alignpoints | How many points are in use. Shown only, it cannot be typed | |
| Alignmentbox size | Edge of the square around each point, in pixels | 30 |
| Max Alignpoint movement | How far a point may move between the reference frame and another frame, in pixels | 5 |

Set Alignpoints reads these settings when you press it, and Align reads *Alignmentbox size* and *Max Alignpoint movement* again. If you change a setting that affects the points, press **Set Alignpoints** again before you press **Align**.

> **Frames that drift.** A frame may sit up to about 64 pixels away from the reference. SidereaPlanets finds that drift first, on a reduced picture of the whole frame, and only then looks for the points within *Max Alignpoint movement* of it. So the points need to move 5 pixels at most, however far the whole picture drifted.

## Region of interest

Not everything in the frame is the planet: a moon at the edge, a dark border, hot pixels. **R.o.Interest** lets you draw a rectangle that alignment is based on.

1. Press **R.o.Interest**. The button stays pressed and the pointer becomes a cross.
2. Drag a rectangle over the image, around the planet with a little air around it. A drag that is only a few pixels long is not taken as a region.
3. The rectangle stays in the image as a dashed line. **Show ROI** shows or hides it. Draw again to replace it.
4. To go back to the whole frame, press **R.o.Interest** twice: once to start drawing, once more to drop the region.
5. Press **Set Alignpoints**.

- Set Alignpoints only places points inside the rectangle. *Min distance from edge* counts from the edges of the rectangle.
- The drift between frames is measured inside the rectangle only, so what lies outside does not pull the result away.
- The region can only be changed before **Align**. If it is smaller than twice *Min distance from edge*, there is no room for a point and Set Alignpoints says so.

Without a rectangle, the whole frame counts.

> The rectangle follows SidereaPlanets's own idea of a region of interest. How the reference program treats it was not checked.

## What Align gives you

For every frame Align measures, and shows in the Framelist and the Registrationgraph:

- **Quality**: how sharp the frame is, see [Frame quality and Limit](frame-quality-and-limit).
- **Difference**: how much the points move relative to each other, in pixels. A low value means the frame was only shifted; a high value means the picture was bent.

Behind both is the shift of the whole frame. It is the middle value of all points, so a few badly tracked points do not matter.

A point that could not be found in a frame (the pattern is gone, or the frame is too small) is left out for that frame. Frames of a different size than the reference are not tracked at all.
