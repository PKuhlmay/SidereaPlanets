---
title: Wavelets and Save
group: Using the window
order: 7
---
# Wavelets and Save

Steps 6 and 7: bring out fine detail in the stacked image, then write the result to a file. The Wavelets step opens after **Stack**. New to a term? See the [Glossary](glossary).

## Buttons

| Button | Where | What it does | Status |
|---|---|---|---|
| Do All | Wavelets, bottom of the panel | Applies the settings to the whole image. Then the Save step opens | Works, with progress; **Cancel** and **Pause** work, see [Progress, Cancel and Pause](progress-cancel-pause) |
| Update Preview | Wavelets, left of Do All | Calculates the preview area with the current settings | Works. **Cancel** drops the result |
| Save Image | Save, bottom of the panel | Saves the sharpened image with the current settings, with the color and tone values applied; runs **Do All** first if the settings changed since the last one (see [Save and export](save-and-export)) | Works |
| Realign with Processed | Wavelets, under Advanced | Aligns again with the sharpened image | Dimmed: Not available yet |
| Stack Again | Wavelets, under Advanced | Stacks again | Dimmed: Not available yet |
| Adjust › | Top of the image area | Shows or hides the Adjustments panel on the right | Works |
| Crop | Adjustments panel | Opens the crop tool: choose the part of the image that is saved, see [Cropping before you save](save-and-export) | Works |
| Crop… | Save, in the panel | The same crop tool | Works |

Save is possible as soon as the stack exists: if **Do All** has not run for the current values, **Save Image** runs it first.

## Settings in the Wavelets step

You see first the check boxes **Color** and **Sharpen brightness only** and the six layers; the rest is under **Advanced**.

Six layers, each with a check box, **Denoise** (0.00), **Sharpen** (0.100) and a slider; the value of the slider stands as text to its right. The slider runs from −5.0 to 100.0 and starts at 1.0. The **Preview** button of a layer (an eye) is dimmed: **Not available yet**. **Reset Wavelets**, a link next to the heading, puts all layers back to these values.

**Color** and **Sharpen brightness only** decide how color is treated, see [Wavelet sharpening](wavelet-sharpening). Both are locked for grayscale recordings, and *Sharpen brightness only* also without Color.

| Setting under Advanced (group *Wavelet Settings*) | Default |
|---|---|
| Run Do All automatically, Keep layer values for a new stack, Move all sliders together | off (see [Wavelet sharpening](wavelet-sharpening)) |
| Layer spacing | Linear (or Dyadic (2^n)) |
| First layer size | 1 |
| Size step | 0 |
| Wavelet filter | Gaussian (or Default) |

The preview updates while you move a slider; how it works and what each setting does is on the page [Wavelet sharpening](wavelet-sharpening). **Load Scheme…** and **Save Scheme…** (under Advanced) open and save wavelet scheme files (`.rwv`). The list *Saved Schemes* is **Not available yet**.

## Check boxes in the Show menu

In the Wavelets and Save steps **Show ▾** at the top right of the image holds: Show Full Image · Show Alignpoints · Show Preview Area. **Show Full Image** is **Not available yet**. **Before/After** in the zoom bar compares with the unsharpened stack, see [The image area](image-area).

## Adjustments (right)

Fifteen buttons in three columns, from Histogram to Crop. Histogram, Gamma, Color Mixing, RGB Align, RGB Balance, Denoise/Deringing and Masking work: each is a switch for its own card over the image, see [Color and tones](color-and-tones). **View Compare** is a switch for the split view of stack and result. **Crop** opens the crop tool, see [Cropping before you save](save-and-export). The other six (View Zoomed, View Stack Size, Flip and Rotate, Resize Image, Wavelet Filter and Show Line Graph) are dimmed and cannot be pressed: **Not available yet**.

**Contrast/Brightness** has two sliders, each with its label and a field above it. Contrast starts at 100 and Brightness at 0. **Reset** returns both. **Keep** is **Not available yet**. **Remember**, **Restore** and **Difference** keep and compare images, see [Color and tones](color-and-tones); *Remembered image* is locked until something is remembered.

## Save

The Save step saves the sharpened stack with your color and tone settings. Above **Save Image** is the group **Crop** with **Crop…**: it cuts the picture to a rectangle when it is saved, see [Cropping before you save](save-and-export). **Save Image** opens the list of file types, see [Save and export](save-and-export). If the result of **Do All** does not match the current values, the panel says that your latest changes are applied to the whole image first (Do All). After saving, the step shows the file name; **Select Another Recording…** works like **Select Recording**.
