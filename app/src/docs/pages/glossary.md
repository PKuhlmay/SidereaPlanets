---
title: Glossary
group: Reference
order: 11
---
# Glossary

The words of planetary photography in plain language, with the name each one has in the window. The terms are in the order you meet them while you work.

| Term | In the window | What it means |
|---|---|---|
| Frame | Frame 5 of 10 | One single picture of the recording. A video of a planet has hundreds or thousands of them. See [Select a recording](select-a-recording) |
| Alignpoint | Alignpoints, Set Alignpoints | A mark on a surface detail of the planet, for example the edge of a cloud band. The program looks for every mark in every frame and so learns how far each frame has moved or been bent by the air. See [Align your frames](align-your-frames) |
| Region of interest | Draw Region, Show Region | A rectangle you draw around the planet. Only this area is used for placing alignpoints and for aligning, so a moon or a dark border cannot confuse the result |
| Reference frame, best frame | Go to Best Frame | The reference frame is the frame all other frames are lined up on; it is the frame that is shown when you press Set Alignpoints. The best frame is the sharpest frame of the recording and makes a good reference. See [Frame quality and Best Frames](frame-quality-and-limit) |
| Align | Align, Align Again | Lines up every frame on the alignpoints and rates how sharp each frame is. Afterwards the frames can be compared and added up |
| Frame quality | Quality, Quality Graph | A number for how sharp a frame is: the more detail and the crisper its edges, the higher it is. Only the order of the frames matters, not the number itself |
| Best Frames, Keep Best Frames | Best Frames, Keep Best Frames, Keep Best Frames Again | The step that chooses the frames that go into the stack. Blurry frames are left out, because they would soften the result. See [Frame quality and Best Frames](frame-quality-and-limit) |
| Stack | Stack | Averages the kept frames into one image. Random noise cancels out, so the stack is much smoother than any single frame and can be sharpened further. See [The Stack step](step-stack) |
| Wavelets, layers | Wavelets, Layer 1 to 6 | A way to split the image into layers of detail. Layer 1 holds the finest detail, the higher layers hold coarser structures. Each layer can be strengthened or weakened on its own. See [Wavelet sharpening](wavelet-sharpening) |
| Denoise | Denoise | Smooths the noise in one layer before it is strengthened |
| Sharpen | Sharpen | Fine adjustment of one layer. The slider of the layer sets the main strength |
| Do All | Do All | Applies the wavelet settings to the whole image. While you move sliders, the preview shows only a small part |
| Preview area | Show Preview Area, Settings ▸ Preview area | The square part of the image that the preview calculates, so that moving a slider stays fast. Its size is set in the settings |
| Update Preview | Update Preview | Calculates the preview area again with the current settings |
| Threshold | Threshold (more points / fewer points) | How strong a detail must be to get an alignpoint. Move the slider left for more points, right for fewer |
| Brightness measure | Brightness measure (Default, 3×3 area, Lowest pixel value) | How the brightness at a point is measured when alignpoints are placed: at the center pixel, as the mean of the 3×3 pixels around it, or as the darkest of those pixels |
| Box size | Box size per point (px) | The edge length of the square around each alignpoint that is searched for in the other frames |
| Output size | Output size | The size of the stacked image. By default it has the size of the frames |
| Correct geometry | Correct geometry | Shifts each part of the image by its own movement, which undoes the wobble of the air. When it is off, every frame moves as a whole |
| Layer spacing | Layer spacing (Linear, Dyadic) | How the wavelet layers grow in size. Linear: evenly. Dyadic: each layer doubles in size |
| Wavelet scheme | Load Scheme…, Save Scheme… (`.rwv`) | The values of all wavelet layers, stored in a file so that you can use them again |
| Adjustments | Adjust ›, Adjustments | Tools for color, tone and cleanup after sharpening: Histogram, Gamma, Color Mixing, RGB Align, RGB Balance, Contrast/Brightness, Denoise/Deringing and Masking, and the split view View Compare. See [Color and tones](color-and-tones) |
| Denoise | Denoise (%) | Smooths the noise of the sharpened image but keeps edges, such as the edge of the planet, sharp. Sharpening makes noise stronger, so this is its counterpart. See [Color and tones](color-and-tones) |
| Ringing, Deringing | Deringing (%), Radius (px) | Strong sharpening can leave a dark ring in the sky next to the bright edge of a planet. Deringing lifts the ring back to the level of the sky. See [Color and tones](color-and-tones) |
| Mask | Masking, Use mask, Threshold, Feather (px), Invert, Show mask | Limits the sharpening to a part of the image, usually the planet, so the sky stays as it is in the stack. The threshold decides how bright a pixel must be, the feather softens the edge. See [Color and tones](color-and-tones) |
| Crop | Crop, Crop… | Cuts the picture to a rectangle when it is saved. You choose the shape (Free, 1:1, 3:2 or 4:3) and drag the rectangle over the image; a small cross marks its center. See [Cropping before you save](save-and-export) |
| Histogram | Histogram, Black point, White point | A graph of how many pixels there are of each brightness. The black point and the white point decide which brightness becomes pure black and pure white |
| Gamma | Gamma | Brightens or darkens the midtones without changing black and white |
| RGB Align | RGB Align | Shifts red, green and blue against each other so that they lie on top of each other. This removes the colored fringes that the air causes on a planet |
| Color Mixing | Color Mixing, Preserve brightness | Builds each color channel (red, green, blue) from a weighted mix of the three channels of the image, in percent. Useful to correct a color cast or to turn data from a single filter into a color image. Preserve brightness keeps the brightness of every pixel while the colors change. See [Color and tones](color-and-tones) |
| View Compare | View Compare, Before, After | Splits the image at a divider you drag: the stack as it was before sharpening on the left, the result with sharpening and all adjustments on the right, both at the same zoom and position. See [Color and tones](color-and-tones) |
| RGB Balance | RGB Balance | Makes red, green or blue stronger or weaker, for example to remove a color cast |
| Contrast, Brightness | Contrast/Brightness | Contrast spreads the tones away from mid gray; Brightness shifts them up or down |
| Remember, Restore | Remember, Restore, Difference | Remember keeps the image you see now so that you can compare it later; Restore brings it back; Difference shows where the two differ |

Back to [The window at a glance](window-at-a-glance).
