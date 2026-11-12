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

Both are in the **Show ▾** menu at the top right of the image (steps Alignpoints, Align and Limit).

Tick **Show Framelist** to see all frames with a check mark each. After Align the best frame comes first, with its quality in percent of the best frame's. Take a check mark away to leave a frame out of the stack. **Check all** and **Check none** switch all frames at once.

Tick **Show Registrationgraph** to see **Quality** and **Difference** as two lines over all frames, in the order of the recording. A dip in Quality is a blurry moment. A high Difference means the picture was bent, not only moved. Before Align, the card asks you to run Align first.

## Limit

Choose in *Limit Setup* what to keep, then press **Limit**. At least one frame always stays.

Under the settings the step shows the quality of all frames as bars, the best frame on the left. Kept frames are amber, dropped ones grey, and a dashed line marks the limit: horizontal at the chosen quality for Lowest Quality (%), vertical after the last kept frame for the other modes. The chart follows every change of the value at once, so you can see how many frames stay before you press **Limit**.

| Mode | What stays | Field |
|---|---|---|
| Lowest Quality (%) | Frames whose quality is at least this share of the best frame's. With 80, a frame stays when it is at least 80 % as sharp as the best one | Percent value, default 80 |
| Best Frames (%) | This share of all frames, the best ones, rounded up. With 10 frames and 30 %, three frames stay | Percent value |
| Best Frames | This many frames, the best ones | Number of frames, default 200 |
| Frames/Apoint | Every alignpoint keeps its own best frames, this many: the frames in which that point is sharpest. A frame stays when at least one point keeps it | Number of frames |

The number of frames is only free in the two modes that count frames, **Best Frames** and **Frames/Apoint**. If the number is larger than the number of frames, all frames stay.

**Frames/Apoint** suits a planet whose parts are sharp in different frames: the left side may be best in one frame and the right side in another. Each point judges only its own box. The Stack step takes the frames that stay for the whole image.

> the reference program offers the same four modes. How it handles Frames/Apoint was not checked, SidereaPlanets works as described here.

## Use the best frame as reference

Set Alignpoints and Align use the frame that is shown when you press **Set Alignpoints** as the reference for all others. A sharp reference gives better points, and Align has just told you which frame is the sharpest. Press **Go to best frame** to use it:

1. Run Align. **Go to best frame** is greyed out until then.
2. Press **Go to best frame**, under the quality chart in the step *Limit* or in the Framelist card. The best frame, the first of the ranking, shows in the image and in the frame bar, and the step *Alignpoints* opens.
3. The panel names the frame, for example *Best frame: 295 — run Set Alignpoints and Align again to use it as reference.* The number is the one the frame bar shows.
4. Press **Set Alignpoints again** and then **Align** again, and run **Limit** and **Stack** once more.

Nothing is discarded by the jump: the rating, the choice of frames and the stack stay as they are until you run **Set Alignpoints** again. The new alignment then replaces them, see [Align your frames](align-your-frames). If you move to another frame before you press **Set Alignpoints**, that frame becomes the reference instead.

## After Limit

The status bar reports how many frames stay, for example *Processing limited to 10 frames*, and in its result field the *Stacksize* with *Low quality*, the quality of the weakest frame that stays in percent of the best one (for example *Stacksize: 8  Low quality: 87.20%*). The step *Stack* opens. The alignpoints now show as yellow rings. The green stroke on each ring shows how far that point sits from where the movement of the whole frame puts it, on average over the frames that stay, enlarged four times so that it can be seen. A short stroke means a steady picture, a long one a picture that is bent there. You can click the step *Limit* again, change the Limit mode and press **Limit again**; that discards an existing stack. You can also run **Set Alignpoints** or **Align** again, which discards the Limit choice as well, see [Align your frames](align-your-frames).
