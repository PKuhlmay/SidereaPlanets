---
title: Wavelets and Save
group: Using the window
order: 7
---
# Wavelets and Save

Steps 6 and 7: bring out fine detail in the stacked image, then write the result to a file. The Wavelets step opens after **Stack**.

## Buttons

| Button | Where | What it does | Status |
|---|---|---|---|
| Do All | Wavelets, bottom of the panel | Applies the settings to the whole image. Then the Save step opens | Works, with progress; **Cancel** and **Pause** work, see [Progress, Cancel and Pause](progress-cancel-pause) |
| Process | Wavelets, left of Do All | Calculates the preview area with the current settings | Works. **Cancel** drops the result |
| Save image | Save, bottom of the panel | Saves the sharpened image with the current settings, with the colour and tone values applied; runs **Do All** first if the settings changed since the last one (see [Save and export](save-and-export)) | Works |
| Realign with Processed | Wavelets, under Advanced | Aligns again with the sharpened image | Not available yet |
| Stack Again | Wavelets, under Advanced | Stacks again | Locked, as in the reference program |
| Functions » | Top of the image area | Shows or hides the Functions panel on the right | Works |

Save is possible as soon as the stack exists: if **Do All** has not run for the current values, **Save image** runs it first.

## Settings in the Wavelets step

You see first the check boxes **Colour** and **LRGB** and the six layers; the rest is under **Advanced**.

Six layers, each with a check box, **Denoise** (0.00), **Sharpen** (0.100), a **Preview** button and a slider. The slider runs from −5.0 to 100.0 and starts at 1.0; the Preview button shows its value. **Reset Wavelets**, a link next to the heading, puts all layers back to these values.

**Colour** and **LRGB** decide how colour is treated, see [Wavelet sharpening](wavelet-sharpening). Both are locked for grayscale recordings, and LRGB also without Colour.

| Setting under Advanced | Default |
|---|---|
| Automatic, Hold Wavelet Setting, Use Linked Wavelets | off (see [Wavelet sharpening](wavelet-sharpening)) |
| Waveletscheme | Linear (or Dyadic (2^n)) |
| Initial Layer | 1 |
| Step Incr. (Step Increment) | 0 |
| Wavelet filter | Gaussian (or Default) |

The preview updates while you move a slider; how it works and what each setting does is on the page [Wavelet sharpening](wavelet-sharpening). **Load Scheme** and **Save Scheme** (under Advanced) open and save `.rwv` files. The list **Available schemes** is **Not available yet**.

## Check boxes in the Show menu

In the Wavelets and Save steps **Show ▾** at the top right of the image holds: Show Full Image · Show AlignPoints · Show Processing Area. **Before** in the zoom bar compares with the unsharpened stack, see [The image area](image-area).

## Functions (right)

Fifteen buttons in three columns, from Histogram to Cropping Area. Each is a switch for its own card over the image. Histogram, Gamma, RGB Align and RGB Balance have their card, see [Colour and tones](colour-and-tones). The other cards are **Not available yet**; pressing their button says so in the status bar.

**Contrast/Brightness** has two sliders and two fields. Contrast starts at 100 and Brightness at 0. **Reset** returns both. **Hold** is a switch; what it holds is **Not available yet**. **Copy To**, **Load to** and **Difference** keep and compare images, see [Colour and tones](colour-and-tones); *Clipboard Image* is locked until something is copied.

## Save

The Save step writes the sharpened stack with the colour and tone settings. It has no settings of its own; **Save image** opens the list of file types, see [Save and export](save-and-export). If the result of **Do All** does not match the current values, the panel says *Do All runs first with the current values.* After saving, the step shows the file name; **Open another recording** works like **Select**.
