---
title: Colour and tones
group: Wavelet tab
order: 40
---
# Colour and tones

After stacking, the Wavelet tab has a set of tools to fix the colour and the brightness of the result. They sit in the **Functions** panel on the right: each button opens a small card above the image, and the image follows every change at once.

| Tool | What it does |
|---|---|
| RGB Align | Moves the red, green and blue channels against each other |
| RGB Balance | Makes a channel stronger or weaker |
| Histogram | Stretches the brightness range of the image |
| Gamma | Brightens or darkens the midtones |
| Contrast/Brightness | Spreads the tones and shifts them up or down |
| Copy To, Load to, Difference | Keep an image and compare it with the one you are working on |

The other **Functions** buttons have no card yet. Pressing one says so in the status bar.

## How the cards work

- A **Functions** button stays pressed while its card is open. Press it again, or press the cross in the card, to close the card.
- Closing a card keeps its values. **Reset** in the card sets the values back so that the image is not changed.
- All values are applied to the stacked (or sharpened) image in this order: RGB Align, RGB Balance, Histogram, Gamma, Contrast/Brightness. **Before** shows the image without any of them.
- The values count for **Do All** and **Save image** as well.
- RGB Align and RGB Balance need a colour image. With a grayscale recording they are greyed out.
- The cards work on the stacked image, or on the sharpened image once there is one. Pressing **Process** or **Do All** puts a new sharpened image in the window; the cards then apply to that image.

## RGB Align

Air acts like a weak prism: low in the sky, the red and the blue image of a planet are not in the same place, which gives a red edge on one side and a blue edge on the other. RGB Align moves the channels back onto each other.

1. Press **Auto**. SidereaPlanets looks for the shift of red and of blue that puts their edges on the edges of green and fills in the fields. **Max shift** (default 5 pixels) is how far it searches.
2. Check the limb of the planet. If a fringe is left, change the **X** and **Y** fields of a channel by hand, in steps of 0.1 pixel.
3. **Reset** puts all channels back.

A positive **X** moves a channel to the right, a positive **Y** moves it down. The shift can have decimals, the image keeps its size, and at the edge the last pixel is repeated.

## RGB Balance

One slider per channel, from 0 to 200 percent, 100 is unchanged. Lower red to remove a red cast, or raise it to warm the image. Values that get brighter than white are cut off.

## Histogram

The graph shows how many pixels there are of each brightness, from black on the left to white on the right. Red, green and blue are the coloured lines, the grey area is the brightness of the whole image. The height is logarithmic, so that the few bright pixels of a planet are still visible next to the black sky.

- **Black** and **White** are the two markers under the graph. Everything darker than Black becomes black, everything brighter than White becomes white, and the range between is spread out. Drag the markers or type the values (0 to 255). The stretch is the same for all channels, so the colours do not shift.
- **Auto** sets both markers so that the darkest and the brightest **Clip %** of the pixels (default 0.1) are cut off. Use it as a first step and fine-tune by hand.
- **Input** shows the histogram before the stretch, which is the image the markers sit on. **Result** shows it after all steps.

## Gamma

One slider and one field, default 1.00, from 0.10 to 5.00. A value above 1 brightens the midtones, a value below 1 darkens them. Black and white stay where they are, so it is the gentle way to lift a dark planet without clipping the bright parts.

## Contrast/Brightness

- **Contrast** 100 is unchanged. Higher values move the tones away from mid grey, lower values towards it.
- **Brightness** 0 is unchanged. The slider runs from −512 to 512, which is the whole range from black to white.
- The Contrast field shows the slider position divided by 2.5 (250 gives 100). That ratio is a guess, so the slider may feel a little different from the reference program.
- **Hold** keeps its place in the panel; what it holds is **Not available yet**.

## Copy To, Load to and Difference

Use them to compare two versions of your image, for example before and after a stretch.

1. Press **Copy To** to keep the image you see now, with all the cards applied. It is called the *Clipboard Image*.
2. Change the image, then switch between **Current Image** and **Clipboard Image** under *Toggle* to compare.
3. **Difference** shows how much the two differ, four times amplified so that small changes show. Black means no change. It follows the current image while it is shown.
4. **Load to** makes the kept image your current image. The cards and Contrast/Brightness go back to unchanged, because their effect is already in the kept image.

*Load to* and *Difference* need a copy first, and *Difference* needs both images to have the same size.

## Colour and LRGB

**Colour** (Align and Stack tab) processes the recording in colour; without it the result is grayscale. **LRGB** needs Colour and is meant to sharpen only the brightness, while hue and saturation come from the stack. That keeps the colours calm when the wavelets would otherwise bring out colour noise. Both are greyed out for grayscale recordings.

| Part | Status |
|---|---|
| The two check boxes and their rules | Works |
| Their effect on **Do All** | Not available yet |

> SidereaPlanets follows the reference program in what the tools are called and where they are. The exact scale of the sliders and the way the original applies them has not been compared yet, so the same numbers can look a little different in the two programs.
