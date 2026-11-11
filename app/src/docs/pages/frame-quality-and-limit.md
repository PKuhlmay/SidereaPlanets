---
title: Frame quality and Limit
group: Using the window
order: 5.2
---
# Frame quality and Limit

Not all frames are equally good. After [Align](align-your-frames) every frame has a quality, and **Limit** keeps only the best ones for stacking.

## Quality

The quality of a frame says how sharp it is. It is measured inside the boxes around the alignpoints, where the planet has detail: the sharper the edges and markings there, the higher the value. It is related to the brightness of the frame, so a frame that was exposed longer does not win only for being brighter.

Only the order and the ratios between frames matter. The value is not a percentage of anything, and it cannot be compared between two recordings.

> SidereaPlanets ranks the frames close to the reference program, but not identically. On the test recording both agreed on the best frame and on the two worst ones; the frames in between came in a different order. Frames of almost equal sharpness are easily swapped.

## Framelist and Registrationgraph

Tick **Show Framelist** to see all frames with a check mark each. After Align the best frame comes first, with its quality in percent of the best frame's. Take a check mark away to leave a frame out of the stack. **Check all** and **Check none** switch all frames at once.

Tick **Show Registrationgraph** to see **Quality** and **Difference** as two lines over all frames, in the order of the recording. A dip in Quality is a blurry moment. A high Difference means the picture was bent, not only moved. Before Align, the card asks you to run Align first.

## Limit

Choose in *Limit Setup* what to keep, then press **Limit**. At least one frame always stays.

| Mode | What stays | Field |
|---|---|---|
| Lowest Quality (%) | Frames whose quality is at least this share of the best frame's. With 80, a frame stays when it is at least 80 % as sharp as the best one | Percent value, default 80 |
| Best Frames (%) | This share of all frames, the best ones, rounded up. With 10 frames and 30 %, three frames stay | Percent value |
| Best Frames | This many frames, the best ones | Number of frames, default 200 |
| Frames/Apoint | Every alignpoint keeps its own best frames, this many: the frames in which that point is sharpest. A frame stays when at least one point keeps it | Number of frames |

The number of frames is only free in the two modes that count frames, **Best Frames** and **Frames/Apoint**. If the number is larger than the number of frames, all frames stay.

**Frames/Apoint** suits a planet whose parts are sharp in different frames: the left side may be best in one frame and the right side in another. Each point judges only its own box. The Stack step takes the frames that stay for the whole image.

> the reference program offers the same four modes. How it handles Frames/Apoint was not checked, SidereaPlanets works as described here.

## After Limit

The status bar reports how many frames stay, for example *Processing limited to 10 frames*, and the window moves on to the Stack tab. The alignpoints now show as yellow rings. The green stroke on each ring shows how far that point sits from where the movement of the whole frame puts it, on average over the frames that stay, enlarged four times so that it can be seen. A short stroke means a steady picture, a long one a picture that is bent there. You can go back to the Align tab, change the Limit mode and press **Limit** again; that discards an existing stack. You can also press **Set Alignpoints** or **Align** again, which discards the Limit choice as well, see [Align your frames](align-your-frames).
