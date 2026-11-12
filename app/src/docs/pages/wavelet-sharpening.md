---
title: Wavelet sharpening
group: Processing
order: 40
---
# Wavelet sharpening

The Wavelets step brings out fine detail in the stacked image. It splits the image into six layers, from the finest structures (layer 1) to coarser ones (layer 6), and lets you strengthen or weaken each layer on its own. This is the step that turns a soft stack into a sharp planet.

> Needs a stacked image, so it works as soon as the Stack step has run. The buttons of the step are described on [Wavelets and Save](step-wavelets-save).

## The six layers

Each layer has a slider from -5.0 to 100.0. The slider value is a factor for that layer:

| Slider | Effect |
|---|---|
| 1.0 | The layer stays as it is. This is the start value, so a fresh image is not changed |
| Above 1.0 | The detail of the layer gets stronger, the image looks sharper |
| Between 0 and 1.0 | The detail gets weaker |
| 0 | The layer is removed |
| Below 0 | The layer is inverted. Rarely useful |

Start with layers 1 to 3 and raise them a little at a time. Too much on layer 1 makes the noise of the image grow along with the detail.

## What you can set

The layers are at the top of the step. **Waveletscheme**, **Wavelet filter**, **Initial Layer**, **Step Increment**, **Automatic**, **Hold Wavelet Setting** and **Use Linked Wavelets** are under **Advanced**.

| Control | What it does | Start value |
|---|---|---|
| Checkbox of a layer | Off leaves the layer untouched, whatever its slider says | On |
| Slider and **Preview** button | Strength of the layer. The button shows the value | 1.0 |
| **Denoise** | Removes weak detail of the layer, which is mostly noise, before it is strengthened. 0 is off | 0.00 |
| **Sharpen** | Scales how strongly the slider acts. At 0.100 the slider acts exactly as shown | 0.100 |
| **Waveletscheme** | How the layers grow in size: Linear (evenly) or Dyadic (doubling from layer to layer) | Linear |
| **Wavelet filter** | The smoothing the layers are built from: Default or Gaussian | Gaussian |
| **Initial Layer**, **Step Increment** | Size of the first layer and how much each further layer grows | 1 and 0 |
| **Reset Wavelets** | A link next to the heading of the layers. Puts everything back to the start values | |

Values outside the sliders' range are cut back to it. Denoise and Sharpen accept 0 to 100.

## A slightly darker image, like in the reference program

Even with every slider at 1.0, **Do All** makes the image a little darker, by about one percent in the midtones. the reference program does the same, and so does this program: the result is run through the same tone curve, so the same stack gives the same picture. The preview shows the curve too, and so does the saved image. **Before** shows the stack without it.

## Colour and LRGB

The two check boxes **Colour** and **LRGB** decide how colour is treated, in the preview and in **Do All**. They sit at the top of the Wavelets step, above the layers:

| Colour | LRGB | Result |
|---|---|---|
| On | Off | Red, green and blue are sharpened each on its own |
| On | On | Only the brightness is sharpened. Hue and saturation stay as in the stack, which keeps colour noise from growing with the detail |
| Off | | The image is reduced to gray first and the result is grayscale |

LRGB only counts together with Colour. Both are greyed out for grayscale recordings, where nothing changes.

## Preview, Process and Do All

Moving a slider updates the preview at once. Only the *Processing Area* is calculated, a square of 512 pixels by default that you can change under Settings > Processing Area and show with **Show Processing Area**. The status bar tells how long the last preview took. That keeps the preview fast, even for large images. The area is calculated together with a small border around it, so what you see matches what the whole image will look like.

**Process** calculates the Processing Area once more with the current settings. **Do All**, the button at the bottom of the step, applies the settings to the whole image and opens the Save step (you can cancel or pause it, see [Progress, Cancel and Pause](progress-cancel-pause)); after it, moving a slider shows the preview of the area on the stacked image again. **Before** shows the stacked image without any change, to compare.

> With Denoise, the preview judges the amount of noise from the Processing Area, **Do All** from the whole image. The two can differ slightly.

## Automatic, Hold and Linked

These three check boxes (under **Advanced**) are not fully known from the reference program; here they do the following:

| Check box | What it does here |
|---|---|
| Automatic | Once you stop moving the sliders for about half a second, **Do All** runs by itself |
| Hold Wavelet Setting | Keeps the layer values when a new stacked image arrives. Without it, the layers go back to their start values |
| Use Linked Wavelets | Moving one slider moves all six by the same amount |

## Schemes

**Save Scheme** and **Load Scheme** ask for a file and store or read the settings of the panel. Schemes use the same `.rwv` files as the reference program, so schemes you already have keep working, and schemes saved here open in the reference program.

## What works today

| Part | Status |
|---|---|
| Six layers with strength, Denoise and Sharpen, Linear and Dyadic scheme, both filters | Works |
| Preview of the Processing Area, **Process**, **Do All** and **Before** | Works |
| **Colour** and **LRGB** in the preview and in **Do All** | Works. What the original does with LRGB is not known; the behaviour above is our reading of the name |
| **Save Scheme** and **Load Scheme** | Works |
| List **Available schemes** | **Not available yet** |
| **Preview** button of a layer | **Not available yet** |
| **Realign with Processed** | **Not available yet** |
| Matching the reference program exactly | With the start values and with layer 1 at 10.0 the result matches the original on the Jupiter test image. The tone curve is measured only up to about 60 percent of full brightness; above it, it is an extension. Layers 2 to 6, Dyadic, the Default filter, Denoise, Automatic, Hold and Linked are built from observation and guesses and not yet compared |
