---
title: Wavelet sharpening
group: Processing
order: 40
---
# Wavelet sharpening

The Wavelets step brings out fine detail in the stacked image. It splits the image into six layers, from the finest structures (layer 1) to coarser ones (layer 6), and lets you strengthen or weaken each layer on its own. This is the step that turns a soft stack into a sharp planet. New to a term? See the [Glossary](glossary).

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

The layers are at the top of the step. The group *Wavelet Settings* with **Layer spacing**, **Wavelet filter**, **First layer size**, **Size step**, **Run Do All automatically**, **Keep layer values for a new stack** and **Move all sliders together** is under **Advanced**.

| Control | What it does | Start value |
|---|---|---|
| Checkbox of a layer | Off leaves the layer untouched, whatever its slider says | On |
| Slider | Strength of the layer. Its value stands as text to the right of the slider. The **Preview** button of the layer (an eye) is **Not available yet** | 1.0 |
| **Denoise** | Removes weak detail of the layer, which is mostly noise, before it is strengthened. 0 is off | 0.00 |
| **Sharpen** | Fine adjustment of the layer: scales how strongly the slider acts. At 0.100 the slider acts exactly as shown | 0.100 |
| **Layer spacing** | How the layers grow in size: Linear (evenly) or Dyadic (2^n: doubling from layer to layer) | Linear |
| **Wavelet filter** | The smoothing the layers are built from: Default or Gaussian (smoother) | Gaussian |
| **First layer size**, **Size step** | Size of the finest layer and how much each further layer grows | 1 and 0 |
| **Reset Wavelets** | A link next to the heading of the layers. Puts everything back to the start values | |

Values outside the sliders' range are cut back to it. Denoise and Sharpen accept 0 to 100.

## A slightly darker image

Even with every slider at 1.0, **Do All** makes the image a little darker, by about one percent in the midtones: the result is run through a fixed tone curve. The preview shows the curve too, and so does the saved image. **Before** (in the **Before/After** switch) shows the stack without it.

## Color and Sharpen brightness only

The two check boxes **Color** and **Sharpen brightness only** decide how color is treated, in the preview and in **Do All**. They sit at the top of the Wavelets step, above the layers:

| Color | Sharpen brightness only | Result |
|---|---|---|
| On | Off | Red, green and blue are sharpened each on its own |
| On | On | Only the brightness is sharpened. Hue and saturation stay as in the stack, which keeps color noise from growing with the detail |
| Off | | The image is reduced to gray first and the result is grayscale |

*Sharpen brightness only* only counts together with Color. Both are grayed out for grayscale recordings, where nothing changes.

## Update Preview and Do All

Moving a slider updates the preview at once. Only the *preview area* is calculated, a square of 512×512 pixels by default that you can change under Settings > Preview area and show with **Show Preview Area**. The status bar tells how long the last preview took, for example *Preview calculated in 85 ms*. The preview only runs in the step Wavelets: when you open Save, or come back to Wavelets after **Do All** with the same values, the image from **Do All** stays as it is. That keeps the preview fast, even for large images. The area is calculated together with a small border around it, so what you see matches what the whole image will look like.

**Update Preview** calculates the preview area once more with the current settings. **Do All**, the button at the bottom of the step, applies the settings to the whole image and opens the Save step (you can cancel or pause it, see [Progress, Cancel and Pause](progress-cancel-pause)); after it, moving a slider shows the preview of the area on the stacked image again. **Before** in the **Before/After** switch shows the stacked image without any change, to compare.

> With Denoise, the preview judges the amount of noise from the preview area, **Do All** from the whole image. The two can differ slightly.

## Run Do All automatically, Keep layer values and Move all sliders together

These three check boxes sit under **Advanced**, in the group *Wavelet Settings*:

| Check box | What it does |
|---|---|
| Run Do All automatically | Once you stop moving the sliders for about half a second, **Do All** runs by itself |
| Keep layer values for a new stack | Keeps the layer values when a new stacked image arrives. Without it, the layers go back to their start values. Layer values that come from a project file are kept for the first stacked image after **Open Project…** in any case |
| Move all sliders together | Moving one slider moves all six by the same amount |

## Wavelet schemes

**Save Scheme…** and **Load Scheme…** ask for a file and store or read the settings of the panel. A wavelet scheme is a small `.rwv` file, so schemes you made once can be used again in any later session. The status bar then says *Scheme saved* or *Scheme loaded* with the file name, or the error if the file could not be written or read. After **Load Scheme…** the preview is calculated with the new values, and the message stays. The list *Saved Schemes* is **Not available yet**.

## What works today

| Part | Status |
|---|---|
| Six layers with strength, Denoise and Sharpen, Layer spacing Linear and Dyadic, both filters | Works |
| Preview of the preview area, **Update Preview**, **Do All** and **Before/After** | Works |
| **Color** and **Sharpen brightness only** in the preview and in **Do All** | Works |
| **Save Scheme…** and **Load Scheme…** | Works |
| List **Saved Schemes** | **Not available yet** |
| **Preview** button of a layer | Dimmed: **Not available yet** |
| **Realign with Processed** | Dimmed: **Not available yet** |
| **Stack Again** | Dimmed: **Not available yet** |
