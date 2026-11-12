---
title: Frame quality and Best Frames
group: Using the window
order: 5.2
---
# Frame quality and Best Frames

Not all frames are equally good. After [Align](align-your-frames) every frame has a quality, and the step **Best Frames** keeps only the best ones for stacking. Its button is **Keep Best Frames**. New to a term? See the [Glossary](glossary).

## Quality

The quality of a frame says how sharp it is. It is measured inside the boxes around the alignpoints, where the planet has detail: the sharper the edges and markings there, the higher the value. It is related to the brightness of the frame, so a frame that was exposed longer does not win only for being brighter.

Only the order and the ratios between frames matter. The value is not a percentage of anything, and it cannot be compared between two recordings.

Frames of almost equal sharpness are easily swapped in the ranking.

## Frame List and Quality Graph

Both are in the **Show ▾** menu at the top right of the image (steps Alignpoints, Align and Best Frames).

Tick **Show Frame List** to see all frames with a check mark each. After Align the best frame comes first, with its quality in percent of the best frame's. Take a check mark away to leave a frame out of the stack. **Check All** and **Check None** switch all frames at once.

Tick **Show Quality Graph** to see **Quality** and **Difference** as two lines over all frames, in the order of the recording. A dip in Quality is a blurry moment. A high Difference means the picture was bent, not only moved. Before Align, the card asks you to run Align first.

## Best Frames

Choose in the step *Best Frames* what to keep, then press **Keep Best Frames**. At least one frame always stays. While it runs, the status bar says for example *Keeping 8 of 10 frames*.

Under the settings the step shows the quality of all frames as bars, the sharpest frame on the left and the blurriest on the right. Kept frames are amber, dropped ones gray, and a dashed line marks the limit: horizontal at the chosen quality for At least % of the best quality, vertical after the last kept frame for the other modes. The chart follows every change of the value at once, so you can see how many frames stay before you press **Keep Best Frames**.

| Mode | What stays | Field |
|---|---|---|
| At least % of the best quality | Frames whose quality is at least this share of the best frame's. With 80, a frame stays when it is at least 80% as sharp as the best one | Percent value, default 80 |
| Best % of frames | This share of all frames, the sharpest ones, rounded up. With 10 frames and 30%, three frames stay | Percent value |
| Best number of frames | This many frames, the sharpest ones | Number of frames, default 200 |
| Best frames per point | Every alignpoint keeps its own sharpest frames, this many: the frames in which that point is sharpest. A frame stays when at least one point keeps it | Number of frames |

The number of frames is only free in the two modes that count frames, **Best number of frames** and **Best frames per point**. If the number is larger than the number of frames, all frames stay.

**Best frames per point** suits a planet whose parts are sharp in different frames: the left side may be best in one frame and the right side in another. Each point judges only its own box. The Stack step takes the frames that stay for the whole image.

## Use the best frame as reference

Set Alignpoints and Align use the frame that is shown when you press **Set Alignpoints** as the reference for all others. A sharp reference gives better points, and Align has just told you which frame is the sharpest. Press **Go to Best Frame** to use it:

1. Run Align. **Go to Best Frame** is grayed out until then.
2. Press **Go to Best Frame**, under the quality chart in the step *Best Frames* or in the Frame List card. The best frame, the first of the ranking, shows in the image and in the frame bar, and the step *Alignpoints* opens.
3. The panel names the frame, for example *Best frame: 295. Run Set Alignpoints and Align again to line all frames up on this one.* The number is the one the frame bar shows.
4. Press **Set Alignpoints Again** and then **Align Again**, and run **Keep Best Frames** and **Stack** once more.

Nothing is discarded by the jump: the rating, the choice of frames and the stack stay as they are until you run **Set Alignpoints** again. The new alignment then replaces them, see [Align your frames](align-your-frames). If you move to another frame before you press **Set Alignpoints**, that frame becomes the reference instead.

## After Keep Best Frames

The status bar reports how many frames will be stacked and how good the weakest one is, for example *8 frames will be stacked · weakest kept: 72%* (the quality of the weakest frame that stays, in percent of the best one). The step *Stack* opens. The alignpoints now show as yellow rings. The green stroke on each ring shows how far that point sits from where the movement of the whole frame puts it, on average over the frames that stay, enlarged four times so that it can be seen. A short stroke means a steady picture, a long one a picture that is bent there. You can click the step *Best Frames* again, change the mode and press **Keep Best Frames Again**; that discards an existing stack. You can also run **Set Alignpoints** or **Align** again, which discards the choice of frames as well, see [Align your frames](align-your-frames).
