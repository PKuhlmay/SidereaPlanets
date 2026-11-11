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
| Process | Calculates the preview area with the current settings | Works |
| Do All | Applies the settings to the whole image | Works |
| Save image | Saves the sharpened image | Not available yet |
| Realign with Processed | Aligns again with the sharpened image | Not available yet |
| Stack Again | Stacks again | Locked, as in the reference program |
| Functions » | Shows or hides the Functions panel on the right | Works |

Check boxes: Show Full Image · Show AlignPoints · Show Processing Area. **Before** in the zoom bar compares with the unsharpened stack, see [The image area](image-area).

## Wavelets (left)

| Setting | Default |
|---|---|
| Automatic, Hold Wavelet Setting, Use Linked Wavelets | off (see [Wavelet sharpening](wavelet-sharpening)) |
| Waveletscheme | Linear (or Dyadic (2^n)) |
| Initial Layer | 1 |
| Step Incr. (Step Increment) | 0 |
| Wavelet filter | Gaussian (or Default) |

Six layers, each with a check box, **Denoise** (0.00), **Sharpen** (0.100), a **Preview** button and a slider. The slider runs from −5.0 to 100.0 and starts at 1.0; the Preview button shows its value. **Reset Wavelets** puts all layers back to these values.

The preview updates while you move a slider; how it works and what each setting does is on the page [Wavelet sharpening](wavelet-sharpening). **Load Scheme** and **Save Scheme** open and save `.rwv` files. The list **Available schemes** is **Not available yet**.

## Functions (right)

Fifteen buttons in three columns, from Histogram to Cropping Area. Each is a switch for its own card over the image. Histogram, Gamma, RGB Align and RGB Balance have their card, see [Colour and tones](colour-and-tones). The other cards are **Not available yet**; pressing their button says so in the status bar.

**Contrast/Brightness** has two sliders and two fields. Contrast starts at 100 and Brightness at 0. **Reset** returns both. **Hold** is a switch; what it holds is **Not available yet**. **Copy To**, **Load to** and **Difference** keep and compare images, see [Colour and tones](colour-and-tones); *Clipboard Image* is locked until something is copied.
