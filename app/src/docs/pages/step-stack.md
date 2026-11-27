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

1. Every frame that is checked in the Frame List is shifted back onto the reference frame, the frame you set the alignpoints on. With **De-rotate images** on, each frame is first turned back by the rotation of the planet (see below).
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
| De-rotate images | off | Works. Turns every frame back to the middle of the recording (see *De-rotate images* below) |
| Ignore edge pixels (px) | 0 | Works. Leaves this border of every frame out of the stack |
| Resampling | off | Not available yet. The check box is locked; the method (Bell, Bspline, Lanczos, Mitchell) and the factor belong to it |
| Drizzling | off | Not available yet. The check box is locked; the factor belongs to it |

The stack always keeps the size of the frames. **Output size** other than Default, **Stretch histogram after stacking**, **Resampling** and **Drizzling** are locked and marked *Not available yet*. **Resampling** and **Drizzling** would enlarge the result.

## De-rotate images

A planet turns while you record it: Jupiter once in about 9 hours 56 minutes, which is 36° an hour, Saturn once in 10 hours 34 minutes, Mars once in 24 hours 37 minutes. Over a few minutes the details towards the edge of the disc move by several pixels and blur in the stack, and the alignpoints can only follow the small shifts of the whole frame. **De-rotate images** turns every frame back to one moment, the middle of the recording, before it is stacked. Details at the edge of the disc then stay sharp. You do not need data from the telescope.

It pays off for recordings that last more than a minute or two. When the planet turns by less than 1° in the frames you keep, the panel says *Too short to matter*, and the result hardly changes.

Settings, shown below the check box when it is on:

| Setting | What it is |
|---|---|
| Planet | Jupiter, Saturn or Mars. Siderea Planets suggests the planet when its name is in the file name. It decides how fast the planet turns and how flattened it is |
| Start time | When the first frame was taken. It is only asked for when the file has no time for each frame (most AVI videos; SER files from most capture programs do have the times and the field stays away). The field shows the time of your computer; below it you see the same time in UTC. The suggestion is the time the file was last saved minus the length of the recording, and the line says so: check it. Only the date and the time of day to within a few minutes matter, because they decide how the axis of the planet is tilted towards the Earth |
| Camera angle (°) | 0 when celestial north is up in your picture (east is then on the left). If you turned the camera, enter how far, in degrees counter-clockwise on the screen |
| Mirrored image | On when your picture is a mirror image, as a single mirror in the light path makes it: east is on the right when north is up. Wrong settings turn the planet the wrong way and make the stack worse, so check the outline (below) |

Below the settings a line tells *Span 3 min 12 s · rotation 5.8°*: the time from the first to the last kept frame and how far the planet turns in it.

In the Stack step the picture shows the reference frame with an orange outline: the planet as Siderea Planets measured it, a dashed outline for the outer edge of Saturn's rings, and an **N** where the north pole of the planet points. If the outline does not sit on the planet, or **N** points the wrong way, correct the planet, the camera angle or the mirror setting.

How it works: the planet is measured in the reference frame (its center and its radius, from the edge of the disc; a Mars that is not full or the rings of Saturn do not matter). The tilt and the direction of its axis in the sky are worked out from the date and time, to about one degree. Every pixel of the disc is traced to the surface of the flattened sphere, turned by the rotation between the time of the frame and the middle of the recording, and read from the frame there. Pixels off the disc stay as they are. The rings of Saturn do not change when the planet turns, so where they pass in front of the globe nothing is moved.

Good to know:

- The first and the last frame of a long recording are turned the most. A feature that has just turned round the edge of the disc has nothing to be read from, the pixels at the edge stay as they are.
- Clouds that drift against the surface (wind) and the shadows of the rings are not taken into account.
- The measured planet needs to lie fully inside the picture in the reference frame. Use the sharpest frame as the reference (*Go to Best Frame*).
- **Save Aligned Video…** and **Save Averaged Video…** do not de-rotate.
- The settings are stored in the project file in a section of their own.

## Check boxes in the Show menu

In this step **Show ▾** at the top right of the image holds: Show Full Image · Show Alignpoints (shows the card *Alignpoints Used for Stacking* with one check box per point) · Show Stack Graph · Show Stack List. Show Full Image, Show Stack Graph and Show Stack List are **Not available yet**: they sit under a heading of that name in the menu and cannot be chosen.

## Create AVI

Under **Advanced** in the Stack step. It saves the aligned frames as a video, for example for animations or other software.

| Setting | Default |
|---|---|
| Codec | DIB (locked: the video is always uncompressed) |
| Compression quality | 7500 (locked, no effect) |
| Frame rate (fps) | 25 |
| Frame size (aligned video) | Source size. Smallest common area, Largest area and Region are locked: **Not available yet** |
| Sort by quality | off |
| Use prefilters | off (locked: **Not available yet**) |
| Method (averaged video) | Moving average |
| Average of … frames | 5 |

**Save Aligned Video…** writes the frames that stay after Best Frames as a video, **Save Averaged Video…** a moving average over several of them. Both are described on the page [Save and export](save-and-export). They need the Align step to be done. *Source size* here means the size of the source frames.
