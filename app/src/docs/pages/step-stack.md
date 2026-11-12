---
title: The Stack step
group: Using the window
order: 6
---
# The Stack step

Step 5: add up the best frames into one sharper image. The step opens after **Limit**.

## Button

| Button | What it does | Status |
|---|---|---|
| Stack | Adds up the frames that stay after Limit. Then the Wavelets step opens | Works |
| Save image | Saves the stacked image. It appears under the result when you open the done step again (see [Save and export](save-and-export)) | Works |

## How the stack is made

1. Every frame that is checked in the Framelist is shifted back onto the reference frame, the frame you set the alignpoints on.
2. The shift is not the same everywhere. A planet that wobbles in the seeing is moved a little differently in each part of the picture, so SidereaPlanets takes the movement of the alignpoints **nearest** to each pixel and blends it. That straightens the frame locally before it is added. The movements of the points are only followed as far as neighbouring points agree with each other; on soft, blurred frames, where the points scatter, the whole frame moves by one value instead.
3. The shifted frames are averaged. The brightness stays that of the frames, it is not added up beyond the range.

Only alignpoints with a check mark in the card *Alignmentpoints* are used (see *Show AlignSections*).

The sum is kept at floating point precision, so even thousands of 16 bit frames add up without overflow or visible rounding. Frames are read and added one after the other, so a long video does not have to fit into memory.

While **Stack** runs, the status bar shows the progress and the time. **Cancel** stops it, **Pause** holds it. When it is done the status bar reads for example "Stack size (n=10) - (1024x680)" and the Wavelets step opens.

At the edge of the image some frames do not reach, because they were shifted. There each pixel is the average of the frames that do cover it, so the brightness stays right, only the noise is a little higher.

## Settings in the step panel

You see first: StackSize, Use Nearestby Alignpoints and Correct geometry. Everything else is under **Advanced**: the other stacking options, **Resampling**, **Drizzling** and the group *Create AVI*.

| Setting | Default | Status |
|---|---|---|
| StackSize | Default (or Minimized, Maximized, R.of Interest) | Not available yet |
| Use Nearestby Alignpoints | 3 | Works. How many of the nearest alignpoints decide the shift of a pixel: more give a smoother correction, fewer follow the local movement more closely |
| Correct geometry | on | Works. Shifts each part of the picture by its own movement; off, the whole frame moves by one value |
| Normalization of frame intensity | off | Works. Brightens or dims each frame to the brightness of the reference frame before adding, useful when clouds pass by |
| Stretch histogram after stacking | off | Not available yet |
| De-rotate images | off | Not available yet |
| Do not stack pixels at the frameedge | 0 | Works. Leaves this border of every frame out of the stack |
| Resampling | off | Not available yet. The check box is locked; the method (Bell, Bspline, Lanczos, Mitchell) and the factor belong to it |
| Drizzling | off | Not available yet. The check box is locked; the factor belongs to it |

The stack always keeps the size of the frames. **Resampling** and **Drizzling**, which would enlarge the result, are under **Advanced**, but they are locked and marked *Not available yet*, so they cannot be switched on by mistake.

## Check boxes in the Show menu

In this step **Show ▾** at the top right of the image holds: Show Full Image · Show AlignSections (shows the card *Alignmentpoints* with one check box per point) · Show Stackgraph · Show Stacklist. Stackgraph and Stacklist are **Not available yet**: they can be set, but nothing is shown.

## Create AVI

Under **Advanced** in the Stack step.

| Setting | Default |
|---|---|
| Codec | DIB (no effect, the video is always uncompressed) |
| Compress Qual | 7500 (no effect) |
| Framerate | 25 |
| Image size (aligned sequence) | Original image size (the other choices have no effect yet) |
| Sort On Quality | off |
| Use Prefilters | off (no effect yet) |
| Method (integrated sequence) | Moving average |
| Average of … Frames | 5 |

**Save Aligned** writes the frames that stay after Limit as a video, **Save Integrated** a moving average over several of them. Both are described on the page [Save and export](save-and-export). They need the Align step to be done.

## Compared with the reference program

The stack is meant to look like the one the reference program makes from the same frames. With the practice video JUPITER.AVI the result differs from the the reference program stack by 54.9 dB (PSNR), well above the 45 dB aimed for, and is about 0.4 % brighter.
