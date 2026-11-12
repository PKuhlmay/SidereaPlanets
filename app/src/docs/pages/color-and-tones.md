---
title: Color and tones
group: Processing
order: 41
---
# Color and tones

After stacking, the Wavelets step has a set of tools to fix the color and the brightness of the result. They sit in the **Adjustments** panel on the right (show or hide it with **Adjust ›** at the top of the image area): each button opens a small card above the image, and the image follows every change at once. New to a term? See the [Glossary](glossary).

| Tool | What it does |
|---|---|
| RGB Align | Moves the red, green and blue channels against each other |
| RGB Balance | Makes a channel stronger or weaker |
| Histogram | Sets the black point and the white point and so stretches the brightness range of the image |
| Gamma | Brightens or darkens the midtones |
| Contrast/Brightness | Spreads the tones and shifts them up or down |
| Remember, Restore, Difference | Keep an image and compare it with the one you are working on |

The other eleven **Adjustments** buttons (Color Mixing, View Zoomed, View Compare, View Stack Size, Flip and Rotate, Resize Image, Denoise/Deringing, Wavelet Filter, Masking, Show Line Graph and Cropping Area) are dimmed and cannot be pressed: **Not available yet**.

## How the cards work

- An **Adjustments** button stays pressed while its card is open. Press it again, or press the cross in the card, to close the card.
- Closing a card keeps its values. **Reset** in the card sets the values back so that the image is not changed.
- All values are applied to the stacked (or sharpened) image in this order: RGB Align, RGB Balance, Histogram, Gamma, Contrast/Brightness. **Before** shows the image without any of them.
- The values change what the window shows, and **Save Image** in the Save step writes the image with them applied. **Do All** itself does not include them: the sharpened image stays as it is and the cards apply on top of it.
- RGB Align and RGB Balance need a color image. With a grayscale recording they are grayed out.
- The cards work on the stacked image, or on the sharpened image once there is one. Pressing **Update Preview** or **Do All** puts a new sharpened image in the window; the cards then apply to that image.

## RGB Align

Air acts like a weak prism: low in the sky, the red and the blue image of a planet are not in the same place, which gives a red edge on one side and a blue edge on the other. RGB Align moves the channels back onto each other.

1. Press **Auto**. Siderea Planets looks for the shift of red and of blue that puts their edges on the edges of green and fills in the fields. **Max shift (px)** (default 5) is how far it searches.
2. Check the limb of the planet. If a fringe is left, change the **X (px)** and **Y (px)** fields of a channel by hand, in steps of 0.1 pixel.
3. **Reset** puts all channels back.

A positive **X (px)** moves a channel to the right, a positive **Y (px)** moves it down. The shift can have decimals, the image keeps its size, and at the edge the last pixel is repeated.

## RGB Balance

One slider per channel, from 0 to 200%, 100 is unchanged; the fields carry the % sign. Lower red to remove a red cast, or raise it to warm the image. Values that get brighter than white are cut off.

## Histogram

The graph shows how many pixels there are of each brightness, from black on the left to white on the right. Red, green and blue are the colored lines, the gray area is the brightness of the whole image. The height is logarithmic, so that the few bright pixels of a planet are still visible next to the black sky.

- **Black point** and **White point** are the two markers under the graph. Everything darker than the black point becomes black, everything brighter than the white point becomes white, and the range between is spread out. Drag the markers or type the values (0 to 255). The stretch is the same for all channels, so the colors do not shift.
- **Auto** sets both markers so that the darkest and the brightest **Clip (%)** of the pixels (default 0.1) are cut off. Use it as a first step and fine-tune by hand.
- **Input** shows the histogram before the stretch, which is the image the markers sit on. **Result** shows it after all steps.

## Gamma

One slider and one field, default 1.00, from 0.10 to 5.00. A value above 1 brightens the midtones, a value below 1 darkens them. Black and white stay where they are, so it is the gentle way to lift a dark planet without clipping the bright parts.

## Contrast/Brightness

- **Contrast** 100 is unchanged. Higher values move the tones away from mid gray, lower values towards it.
- **Brightness** 0 is unchanged. The slider runs from −512 to 512, which is the whole range from black to white.
- The Contrast field shows the slider position divided by 2.5 (250 gives 100).
- **Keep** is dimmed: **Not available yet**.

## Remember, Restore and Difference

Use them to compare two versions of your image, for example before and after a stretch.

1. Press **Remember** to keep the image you see now, with all the cards applied. It is called the *Remembered image*.
2. Change the image, then switch between **Current image** and **Remembered image** under *Show:* to compare.
3. **Difference** shows how much the two differ, four times amplified so that small changes show. Black means no change. It follows the current image while it is shown.
4. **Restore** makes the remembered image your current image. The cards and Contrast/Brightness go back to unchanged, because their effect is already in the remembered image.

*Restore* and *Difference* need a remembered image first (the status bar says *Restore: nothing remembered yet* otherwise), and *Difference* needs both images to have the same size.

## Color and Sharpen brightness only

**Color** (at the top of the Wavelets step, above the layers) processes the recording in color; without it the result is grayscale. **Sharpen brightness only** needs Color and sharpens only the brightness, while hue and saturation come from the stack. That keeps the colors calm when the wavelets would otherwise bring out color noise. Both act on the preview of the Wavelets step and on **Do All**, see [Wavelet sharpening](wavelet-sharpening). Both are grayed out for grayscale recordings.

| Part | Status |
|---|---|
| The two check boxes and their rules | Works |
| Their effect on the wavelet preview and **Do All** | Works |
