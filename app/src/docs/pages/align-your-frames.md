---
title: Align your frames
group: Using the window
order: 5.1
---
# Align your frames

A planet wobbles from frame to frame: the air makes it dance, the mount drifts. **Set Alignpoints** and **Align** find out how far each frame has moved, and how sharp it is. New to a term? See the [Glossary](glossary). The next pages use that to pick the best frames and stack them.

## Step by step

1. [Select a recording](select-a-recording). The first frame shows in the image area and the step *Alignpoints* opens.
2. Move to a frame that is sharp and shows the planet well, with **Go to frame** or the slider at the bottom. This frame is the reference for all others. After a first Align, **Go to Best Frame** takes you to the sharpest one, see [Frame quality and Best Frames](frame-quality-and-limit).
3. Press **Set Alignpoints**. Red points appear on the planet, and the label of the Threshold slider reads *Threshold · 82 points* or similar. The step *Align* opens.
4. Optional: open the step *Alignpoints* again and move the **Threshold** slider towards *fewer points* to keep fewer, steadier points, or towards *more points* to keep more. More on this under *Alignpoints* below.
5. Optional: before **Set Alignpoints**, press **Draw Region** in the step *Alignpoints* and drag a rectangle over the image, to align on one area only. The points then land inside the rectangle. More on this under *Region of interest* below.
6. Press **Align**. The status bar shows *Aligning frame: n* and the progress, the window title shows it as well (`Siderea Planets · Align 40%`). **Cancel** stops, **Pause** holds and becomes **Resume**.
7. When *Alignment finished* appears, the step *Best Frames* opens: [Frame quality and Best Frames](frame-quality-and-limit).

## Aligning again

**Set Alignpoints** and **Align** stay available after they have run, also after Best Frames, Stack and Wavelets. Use it to try other points, a region of interest or another box size without opening the recording again.

Click the step *Alignpoints* or *Align* in the step bar; the panel then offers **Set Alignpoints Again** or **Align Again**, with the note *Running it again discards the results of the later steps.* Running either starts the alignment over. Everything that was built on the old alignment is discarded at once, so that no old image stays on screen and **Save Image** cannot write one:

- the frame quality and the choice of frames from Best Frames (all frames are checked again),
- the stack and the sharpened image,
- the color and tone adjustment that was shown on them,
- the steps Best Frames, Stack, Wavelets and Save lock again, and the next step to take opens (after Set Alignpoints this is Align, after Align it is Best Frames).

Run Keep Best Frames and Stack again afterwards. Your slider values, the wavelet settings and the remembered image stay as they are. If you cancel **Align**, the old alignment is gone as well: the points are still set, and Align has to run again.

## Alignpoints

An alignpoint is a spot on the reference frame that Siderea Planets looks for in every other frame. A point is a small square (the *Box size per point*); the program searches for the same pattern around it, up to *Max. point shift* away, and finds the position to a fraction of a pixel. Many points across the planet also show how the picture is bent by the air, not only moved.

The points are placed on bright, detailed parts. A straight edge is a poor point, because the pattern can slide along it; a corner or a spot is a good one.

| Setting | What it does | Default |
|---|---|---|
| Min. distance between points (px) | Points keep this far apart. Siderea Planets adds *Max. point shift* to it, so that the search areas of two points never overlap: with the defaults, 35 pixels | 30 |
| Min. distance from image edge (px) | No points near the border of the frame (or of the region) | 20 |
| Brightness measure | Which pixel decides whether a spot is bright enough: **Default** (the center), **3×3 area** (the mean of 3 by 3 pixels), **Lowest pixel value** (the darkest of them) | Default |
| Min. brightness (0–255), Max. brightness (0–255) | Only spots with a brightness between these values get a point. Dark sky and burnt-out spots stay free | 30, 230 |
| Threshold | Drops the weakest points. Move it left for more points, right for fewer. The slider runs from 0 up to the number of points that were found; after Set Alignpoints it stands at about a tenth | about 10% |
| Number of alignpoints | How many points are in use. Shown only, it cannot be typed | |
| Box size per point (px) | Edge of the square around each point, in pixels | 30 |
| Max. point shift (px) | How far a point may move between the reference frame and another frame, in pixels | 5 |

Threshold, Min. distance between points and Min. distance from image edge are in the step *Alignpoints*; Brightness measure, Min. brightness, Max. brightness and Number of alignpoints are under **Advanced** there, in the group *Point Placement*. Box size per point and Max. point shift are in the step *Align*.

Set Alignpoints reads these settings when you press it, and Align reads *Box size per point* and *Max. point shift* again. If you change a setting that affects the points, press **Set Alignpoints Again** before you press **Align**.

> **Frames that drift.** A frame may sit up to about 64 pixels away from the reference. Siderea Planets finds that drift first, on a reduced picture of the whole frame, and only then looks for the points within *Max. point shift* of it. So the points need to move 5 pixels at most, however far the whole picture drifted.

## Align by center of gravity

If the planet jumps further than about 64 pixels between frames (strong seeing, a mount that does not track well), the search above loses it. **Align by center of gravity** under **Advanced** in the step *Align* finds the drift another way: from the centre of the planet's brightness, like the middle of a bright coin on a dark table. That works however far the planet moves, as long as it stays whole in the frame. The points then look within *Max. point shift* of that centre as before.

1. In the step *Align*, open **Advanced** and turn on **Align by center of gravity**.
2. **Brightness threshold** (0–255, 20 to start with): pixels up to this brightness do not count, so the dark sky and its noise do not pull the centre towards the middle of the frame. Raise it if a moon or a bright edge pulls the planet off; keep it below the faint edge of the planet.
3. Press **Align** (or **Align Again**). Stack uses the same setting when it lines up the frames.

- Use it for a whole planet against a dark sky. For the Moon, or the Sun in close-up where the surface fills the frame, leave it off: there is no centre to follow, and the search finds the drift from the details.
- The centre is measured in the whole frame, also when a region is drawn: the region stays where it is, while the planet may jump out of it.
- If no pixel is brighter than the threshold, Siderea Planets falls back to the search on the reduced picture.

## Region of interest

Not everything in the frame is the planet: a moon at the edge, a dark border, hot pixels. **Draw Region** lets you draw a rectangle that alignment is based on. The rectangle is called a region of interest.

1. In the step *Alignpoints*, press **Draw Region**. The button stays pressed and the pointer becomes a cross.
2. Drag a rectangle over the image, around the planet with a little air around it. A drag that is only a few pixels long is not taken as a region.
3. The rectangle stays in the image as a dashed line. **Show Region** in the **Show ▾** menu shows or hides it. Draw again to replace it.
4. To go back to the whole frame, press **Draw Region** twice: once to start drawing, once more to remove the region.
5. Press **Set Alignpoints** (or **Set Alignpoints Again**) in the step *Alignpoints*.

- Set Alignpoints only places points inside the rectangle. *Min. distance from image edge* counts from the edges of the rectangle.
- The drift between frames is measured inside the rectangle only, so what lies outside does not pull the result away.
- The region can only be changed before **Align**. If it is smaller than twice *Min. distance from image edge*, there is no room for a point and Set Alignpoints says so.

Without a rectangle, the whole frame counts.

## What Align gives you

For every frame Align measures, and shows in the Frame List and the Quality Graph (**Show Frame List** and **Show Quality Graph** in the **Show ▾** menu):

- **Quality**: how sharp the frame is, see [Frame quality and Best Frames](frame-quality-and-limit).
- **Difference**: how much the points move relative to each other, in pixels. A low value means the frame was only shifted; a high value means the picture was bent.

Behind both is the shift of the whole frame. It is the middle value of all points, so a few badly tracked points do not matter.

A point that could not be found in a frame (the pattern is gone, or the frame is too small) is left out for that frame. Frames of a different size than the reference are not tracked at all.
