---
title: The Wavelet tab
group: Using the window
order: 7
---
# The Wavelet tab

The third step: bring out fine detail in the stacked image. The tab opens after **Stack**.

## Buttons

| Button | What it does | Status |
|---|---|---|
| Process | Applies the settings to the preview area | Not available yet |
| Do All | Applies the settings to the whole image | Not available yet |
| Save image | Saves the sharpened image | Not available yet |
| Realign with Processed | Aligns again with the sharpened image | Not available yet |
| Stack Again | Stacks again | Locked, as in the reference program |
| Functions » | Shows or hides the Functions panel on the right | Works |

Check boxes: Show Full Image · Show AlignPoints · Show Processing Area. **Before** in the zoom bar compares with the unsharpened stack, see [The image area](image-area).

## Wavelets (left)

| Setting | Default |
|---|---|
| Automatic, Hold Wavelet Setting, Use Linked Wavelets | off |
| Waveletscheme | Linear (or Dyadic (2^n)) |
| Initial Layer | 1 |
| Step Incr. (Step Increment) | 0 |
| Wavelet filter | Gaussian (or Default) |

Six layers, each with a check box, **Denoise** (0.00), **Sharpen** (0.100), a **Preview** button and a slider. The slider runs from −5.0 to 100.0 and starts at 1.0; the Preview button shows its value. **Reset Wavelets** puts all layers back to these values.

**Available schemes**, **Load Scheme** and **Save Scheme** are **Not available yet**.

## Functions (right)

Fifteen buttons in three columns, from Histogram to Cropping Area. Each is a switch for its own panel. The panels themselves are **Not available yet**; the buttons only keep their pressed state.

**Contrast/Brightness** has two sliders and two fields. Contrast starts at 100 and Brightness at 0. **Reset** returns both. **Hold** is a switch; what it holds is **Not available yet**. **Copy To**, **Load to** and **Difference** are **Not available yet**; *Clipboard Image* is locked until something is copied.
