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
| Color Mixing | Builds each color channel from a mix of red, green and blue |
| RGB Balance | Makes a channel stronger or weaker |
| Histogram | Sets the black point and the white point and so stretches the brightness range of the image |
| Gamma | Brightens or darkens the midtones |
| Contrast/Brightness | Spreads the tones and shifts them up or down |
| Denoise/Deringing | Smooths noise and softens the dark ring next to bright edges |
| Masking | Limits the sharpening to the planet, so the sky stays as it is in the stack |
| Remember, Restore, Difference | Keep an image and compare it with the one you are working on |
| View Compare | Shows the stack and the result side by side in one image, with a divider you drag |
| Flip and Rotate | Mirrors the picture, turns it by 90° or by any angle |
| Resize Image | Makes the saved picture bigger or smaller, by a percentage or to a size in pixels |

**Crop** opens a tool of its own, see [Cropping before you save](save-and-export). The other four **Adjustments** buttons (View Zoomed, View Stack Size, Wavelet Filter and Show Line Graph) are dimmed and cannot be pressed: **Not available yet**.

## How the cards work

- An **Adjustments** button stays pressed while its card is open. Press it again, or press the cross in the card, to close the card.
- Closing a card keeps its values. **Reset** in the card sets the values back so that the image is not changed.
- All values are applied to the stacked (or sharpened) image in this order: Deringing, Denoise, Masking, RGB Align, Color Mixing, RGB Balance, Histogram, Gamma, Contrast/Brightness. **Before** shows the image without any of them. Flip and Rotate, Crop and Resize Image come after these, in that order, and only when a file is written (Flip and Rotate is also in the picture on screen).
- The values change what the window shows, and **Save Image** in the Save step writes the image with them applied. **Do All** itself does not include them: the sharpened image stays as it is and the cards apply on top of it.
- RGB Align and RGB Balance need a color image. With a grayscale recording they are grayed out. Color Mixing also works on grayscale images, see below.
- The cards work on the stacked image, or on the sharpened image once there is one. Pressing **Update Preview** or **Do All** puts a new sharpened image in the window; the cards then apply to that image.

## Denoise/Deringing

Sharpening with wavelets brings out detail, but it also makes noise stronger and can leave a dark ring in the sky next to the bright edge of a planet. This card cleans up both. It needs a stack, because it compares the sharpened image with the stack it was made from.

- **Denoise (%)** smooths noise without blurring edges. Each pixel is averaged with its neighbors, but only with those that have nearly the same color, so the edge of the planet and fine bands keep their sharpness. 0 is off; higher values treat more of the difference between neighbors as noise. Start around 20 to 40.
- **Deringing (%)** looks for pixels that sharpening has pushed darker than anything the unsharpened stack had nearby, and lifts them back towards that level. It only acts next to strong edges, so detail on the planet, such as belts, stays as sharp as you made it. 100 puts the ring fully back to the level of the stack; 0 is off.
- **Radius (px)** (1 to 10, default 4) is how far from an edge the ring is searched for. Use about the width of the ring you see.
- **Reset** sets everything back to off.

Deringing only raises pixels and never makes any darker. If the ring is still there, raise the radius first, then the amount.

## Masking

A mask tells the sharpening where it may work. With the mask on, the image is sharpened only where the unsharpened stack is brighter than the **Threshold**; everywhere else, usually the sky, you see the stack as it was. That keeps the noise of the sky out of the result.

1. Press **Show mask**. The part that will be sharpened is tinted blue over the image. This only changes what you see in the window, not the image.
2. Move **Threshold** (0 is black, 255 is white) until the red covers the planet and not the sky.
3. **Feather (px)** softens the edge of the mask over some pixels, so there is no visible step between the sharpened and the unsharpened part. 0 gives a hard edge.
4. Switch on **Use mask**. Switch off **Show mask** to see the result.
5. **Invert** sharpens the dark part instead of the bright part. **Reset** sets the card back to off.

While **Show mask** is on, the image area shows the mask instead of the image. **Before** still shows the stack.

## RGB Align

Air acts like a weak prism: low in the sky, the red and the blue image of a planet are not in the same place, which gives a red edge on one side and a blue edge on the other. RGB Align moves the channels back onto each other.

1. Press **Auto**. Siderea Planets looks for the shift of red and of blue that puts their edges on the edges of green and fills in the fields. **Max shift (px)** (default 5) is how far it searches.
2. Check the limb of the planet. If a fringe is left, change the **X (px)** and **Y (px)** fields of a channel by hand, in steps of 0.1 pixel.
3. **Reset** puts all channels back.

A positive **X (px)** moves a channel to the right, a positive **Y (px)** moves it down. The shift can have decimals, the image keeps its size, and at the edge the last pixel is repeated.

## Color Mixing

Every color of the result is built from the colors of the image. The card has a 3 × 3 grid: for each of the three outputs (**Red output**, **Green output**, **Blue output**) there is one slider and one field per input channel (**Red**, **Green**, **Blue**), in percent from −200 to 200. The output is the sum of the three inputs, each times its percentage.

- The start values are 100 on the diagonal (red in red output, green in green output, blue in blue output) and 0 everywhere else. That leaves the image as it is.
- **Preset** sets all nine values at once: *Neutral* (the start values), *Swap Red and Blue*, *Luminance Gray* (all three outputs show the brightness of the color image) and *Red to All Channels* (for data taken through a single filter, for example an infrared-pass filter).
- **Preserve brightness** scales every pixel after the mixing so that it keeps the brightness it had before. Without it, a mix that adds up to more than 100% makes the image brighter and one that adds up to less makes it darker. A pixel that mixes to black becomes gray at its old brightness.
- Mixing can only add or remove light: a result below 0 becomes 0, and a result above white is cut off. With **Preserve brightness** on, a very bright pixel that is cut off can change its color a little.
- **Reset** puts the start values back.

Typical uses: a color cast that is the same in all channels (lower the share of the channel that is too strong in the other outputs), data from a monochrome camera or an infrared-pass filter where one channel carries the picture (*Red to All Channels*, then give each output its own share for a false-color tint), or a quick trial of a different color look. On a grayscale recording all three inputs are equal, so each output simply gets its own total percentage, which tints the image.

## Flip and Rotate

Planets are usually saved with north up, or in the orientation of a chart or of another picture of the same planet. This card mirrors the picture and turns it, and the image shows the result at once, so you see what will be saved.

- **Flip** has two switches: **Horizontal** mirrors the picture from left to right, **Vertical** from top to bottom. A switch stays pressed while the picture is mirrored; press it again to undo it. Both act on what you see, also after you have turned the picture.
- **Rotate** has **Left 90°**, **Right 90°** and **180°**. Each press turns the picture by that much.
- **Angle (°)** turns the picture by any angle from −180° to 180° in steps of 0.1°. A positive angle turns clockwise, a negative one counterclockwise. The buttons above change this field: Right 90° adds 90°. Turns by 0°, 90°, 180° and 270° only move pixels and lose nothing; any other angle makes new pixels between the old ones.
- **Interpolation** is how those new pixels are calculated. **Bicubic** (the start value) keeps the most detail; **Bilinear** is smoother and cannot add a faint halo next to a bright edge.
- A picture turned by a free angle gets a bigger canvas, so nothing is cut off, and the corners are black. **Crop to fit** cuts the picture to the largest rectangle of the same shape that has no black corners; it has no effect on turns by 90°, 180° or 270°.
- **Picture** shows the size of the picture after flipping and turning. **Reset** puts the card back to no flip and no turn.

The turned picture is the one in the image area, in the steps Wavelets and Save, and also the left side of **View Compare**. The marks that belong to the stack (alignpoints and the preview area) are not drawn on a flipped or turned picture. **Remember**, **Restore** and **Difference** work on the picture as it is before flipping and turning.

Flip and Rotate is applied last: after all other cards, but before the crop. The crop rectangle is therefore set on the picture as you see it. If you flip or turn the picture after you pressed **Done** in the crop tool, the rectangle is removed (the status bar says so). While the crop tool is open, the card cannot be changed and says *Finish or cancel the crop first*. Changing only the **Interpolation** keeps the crop. The values stay for a new stack and are stored in the project file.

## Resize Image

**Resize Image** changes the size of the picture that **Save Image** writes. It does not change what the window shows: the image area stays at the size of the stack (turned and cropped if you did so), and the card tells the result.

- **Percent** scales both sides by the same share, from 10% to 400%. 100% changes nothing.
- **Pixels** scales to a **Width (px)** and a **Height (px)**. With **Lock aspect ratio** (the start value) you type one side and the other follows, so the picture keeps its shape; the fields show both. Without the lock both sides are yours, and the picture is stretched or squeezed. A side is at least 1 and at most 16384 pixels.
- **Method** is how the new pixels are calculated: **Lanczos3** (the start value) keeps the most detail, **Bicubic** is a little softer, **Bilinear** softer still, and **Nearest Neighbor** copies pixels, which is for enlarging pixel by pixel. When the picture becomes smaller, the method averages the pixels it replaces, so the result does not get jagged.
- **Result** shows the size of the file that **Save Image** writes. **Reset** brings back the full size.

Resize Image is the very last thing before a file is written: **Do All** (or the preview) → color and tone cards → Flip and Rotate → Crop → Resize Image. With a crop, 50% means half of the cropped rectangle, not half of the whole picture. The values stay for a new stack and are stored in the project file.

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

## View Compare

**View Compare** splits the image into two halves along a vertical **divider**. Left of it you see the stack as it was before sharpening (**Before**), right of it the image with sharpening and all cards applied (**After**). Both halves have the same zoom and position, so the same detail lines up on both sides of the line.

- Press **View Compare** in the Adjustments panel to turn the split on and off. The button stays pressed while it is on.
- Drag the grip in the middle of the divider to the left or right. The divider cannot leave the image area: a thin strip of each side always stays. Zooming and moving the image do not move the divider. The divider can also be moved like a slider: with VoiceOver on the Mac, with the left and right arrow keys after a click on the grip in Windows.
- The labels **Before** and **After** sit in the lower corners. A label is hidden when the divider is too close to its corner.
- It works in the Wavelets step. In the Save step the finished image is shown in full.
- **Before/After** in the zoom bar shows one of the two images in full. Pressing it turns off **View Compare**, and pressing **View Compare** while *Before* is on shows the result again as a split. The two never run at the same time.
- The button is stored in the project file with the other pressed Adjustments buttons; the position of the divider is not.

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
