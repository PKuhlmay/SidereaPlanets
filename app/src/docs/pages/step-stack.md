---
title: The Stack step
group: Using the window
order: 6
---
# The Stack step

Step 5: add up the best frames into one sharper image. The step opens after **Keep Best Frames**. New to a term? See the [Glossary](glossary).

## Button

| Button | What it does | Status |
|---|---|---|
| Stack | Adds up the frames that stay after Best Frames. Then the Wavelets step opens | Works |
| Save Image | Saves the stacked image. It appears under the result when you open the done step again (see [Save and export](save-and-export)) | Works |

## How the stack is made

1. Every frame that is checked in the Frame List is shifted back onto the reference frame, the frame you set the alignpoints on.
2. The shift is not the same everywhere. A planet that wobbles in the seeing is moved a little differently in each part of the picture, so Siderea Planets takes the movement of the alignpoints **nearest** to each pixel and blends it. That straightens the frame locally before it is added. The movements of the points are only followed as far as neighbouring points agree with each other; on soft, blurred frames, where the points scatter, the whole frame moves by one value instead.
3. The shifted frames are averaged. The brightness stays that of the frames, it is not added up beyond the range.

Only alignpoints with a check mark in the card *Alignpoints Used for Stacking* are used (see *Show Alignpoints*).

The sum is kept at floating point precision, so even thousands of 16 bit frames add up without overflow or visible rounding. Frames are read and added one after the other, so a long video does not have to fit into memory.

While **Stack** runs, the status bar shows the progress and the time. **Cancel** stops it, **Pause** holds it. When it is done the status bar reads for example *Stacked 10 frames · 1024×680*, the value range of the result is shown as *Value range 0–255*, and the Wavelets step opens.

At the edge of the image some frames do not reach, because they were shifted. There each pixel is the average of the frames that do cover it, so the brightness stays right, only the noise is a little higher.

## Settings in the step panel

You see first: Output size, Use nearest points and Correct geometry. Everything else is under **Advanced**: the other stacking options, **Resampling**, **Drizzling** and the group *Create AVI*.

Options that do nothing yet are locked: they are dimmed, cannot be switched on, show the tooltip *Not available yet* and say so at the right of the row. Locked options do not change the result, so you cannot set them by mistake.

| Setting | Default | Status |
|---|---|---|
| Output size: Default | selected | Works. The stack has the size of the frames |
| Output size: Smallest common area | locked | Not available yet |
| Output size: Largest area | locked | Not available yet |
| Output size: Region | locked | Not available yet |
| Use nearest points | 3 | Works. How many of the nearest alignpoints decide the shift of a pixel: more give a smoother correction, fewer follow the local movement more closely |
| Correct geometry | on | Works. Shifts each part of the picture by its own movement; off, the whole frame moves by one value |
| Even out frame brightness | off | Works. Brightens or dims each frame to the brightness of the reference frame before adding, useful when clouds pass by |
| Stretch histogram after stacking | off | Not available yet. The check box is locked |
| De-rotate images | off | Not available yet. The check box is locked |
| Ignore edge pixels (px) | 0 | Works. Leaves this border of every frame out of the stack |
| Resampling | off | Not available yet. The check box is locked; the method (Bell, Bspline, Lanczos, Mitchell) and the factor belong to it |
| Drizzling | off | Not available yet. The check box is locked; the factor belongs to it |

The stack always keeps the size of the frames. **Output size** other than Default, **Stretch histogram after stacking**, **De-rotate images**, **Resampling** and **Drizzling** are locked and marked *Not available yet*. **Resampling** and **Drizzling** would enlarge the result.

## Check boxes in the Show menu

In this step **Show ▾** at the top right of the image holds: Show Full Image · Show Alignpoints (shows the card *Alignpoints Used for Stacking* with one check box per point) · Show Stack Graph · Show Stack List. Show Full Image, Show Stack Graph and Show Stack List are **Not available yet**: they sit under a heading of that name in the menu and cannot be chosen.

## Create AVI

Under **Advanced** in the Stack step. It saves the aligned frames as a video, for example for animations or other software.

| Setting | Default |
|---|---|
| Codec | DIB (locked: the video is always uncompressed) |
| Compression quality | 7500 (locked, no effect) |
| Frame rate (fps) | 25 |
| Frame size (aligned video) | Original size. Smallest common area, Largest area and Region are locked: **Not available yet** |
| Sort by quality | off |
| Use prefilters | off (locked: **Not available yet**) |
| Method (averaged video) | Moving average |
| Average of … frames | 5 |

**Save Aligned Video…** writes the frames that stay after Best Frames as a video, **Save Averaged Video…** a moving average over several of them. Both are described on the page [Save and export](save-and-export). They need the Align step to be done. *Original size* here means the size of the source frames.
