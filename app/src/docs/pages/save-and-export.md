---
title: Save and export
group: Results
order: 8
---
# Save and export

Press **Save Image** to write a result to a file. It is the button of the step *Save* (the sharpened image). For the stacked image, click the step *Stack* in the step bar once the stack is done: **Save Image** then sits under the result. The file dialog is the one of your operating system.

In the step *Save* the file looks like the image in the window: the values of RGB Align, Color Mixing, RGB Balance, Histogram, Gamma and Contrast/Brightness are applied to the sharpened image before it is written, at full precision (not from the 8 bit picture on screen). With all of them at their start values the sharpened image is written unchanged. The step *Stack* saves the stack without these values. The sharpened image is the result of **Do All**. You do not have to run it before you save: see below.

## Choosing the format

Pressing **Save Image** first opens a short list of file types, such as *TIFF 16-bit (.tif)…*: seven for the stacked image (step Stack), five for the sharpened image (step Save). Pick one and the file dialog of your operating system opens for that type. The file type decides the format, not the ending you type: if the ending does not fit, Siderea Planets replaces it. The dialog suggests the name of your recording and opens in its folder.

| Format | Ending | Bit depth | Stack step | Save step |
|---|---|---|---|---|
| BMP | .bmp | 8 bit per channel | Works | Works |
| JPEG | .jpg | 8 bit per channel, lossy | Works | Works |
| TIFF | .tif | 16 bit per channel, uncompressed | Works | Works |
| PNG | .png | 16 bit per channel | Works | Works |
| FITS | .fit | 16 bit integer | Works | Works |
| FITS | .fit | 32 bit integer | Works | Not offered |
| FITS | .fit | 32 bit floating point | Works | Not offered |

- **For further processing** in other programs choose TIFF or PNG (16 bit), or FITS if the program expects astronomy files. They keep the fine tonal steps that sharpening produces.
- **For sharing** choose JPEG. It is smaller, but loses some detail.
- Black is the lowest value and white the highest value of the format, so a 16 bit file uses the full range from 0 to 65535.
- **Save Image** in the Save step writes what the current settings produce. The step can be opened as soon as the stack exists. If you moved a slider, changed a value of a layer, switched **Color** or **Sharpen brightness only**, made a new **Stack**, or have not pressed **Do All** yet, Siderea Planets runs **Do All** with the current values first and then saves. That run shows progress like any other and can be cancelled with **Cancel**; if you cancel it, or it fails, nothing is saved and the status bar says so. If nothing changed since the last **Do All**, the file is written at once. Changing the preview area does not start a new **Do All**, because it only decides what the preview shows. The color and tone cards (RGB Align, Color Mixing, RGB Balance, Histogram, Gamma, Contrast/Brightness) are applied when the file is written, so they never need a new **Do All**. **Update Preview** only changes the preview area and is not saved.
- A color image is written with its three channels in one file. A single FITS file holds them as three planes (red, green, blue). Black and white images stay single channel.

> An existing file with the same name is replaced only when the new file was written completely. If saving fails, the old file stays as it was.

## Cropping before you save

Use **Crop** to save only a part of the sharpened image, for example a planet in the middle of a dark border, in a shape for printing or posting. The crop works for every file type of **Save Image** in the steps *Wavelets* and *Save*.

Open the crop tool in one of two ways: press **Crop** in the Adjustments panel of the Wavelets step, or press **Crop…** in the Save step above **Save Image**. It is the same tool. A rectangle appears over the image, with everything outside it dimmed, and the card **Crop** shows at the top right.

- **Format** is **Free**, **1:1**, **3:2** or **4:3**. With a fixed format the rectangle keeps its shape while you drag it. It stays lying (wider than tall) or standing (taller than wide), as it is; to change that, draw a new rectangle by dragging the other way round. Choosing another format reshapes the rectangle around its center.
- **Move** the rectangle by dragging inside it and **resize** it with the small squares on its edges and corners. It cannot leave the image and cannot be smaller than 8 × 8 pixels. Dragging outside the rectangle draws a new one.
- A **small cross** marks the center of the rectangle, and thin lines divide it in thirds, to help you line up the planet.
- The **size** in pixels (width × height) shows in the corner of the rectangle and in the card.
- **View Compare** is switched off while the crop tool is open, because the rectangle is set on one picture; press it again after **Done** or **Cancel**.
- **Reset** brings back the whole image (the next drag draws a new rectangle). **Cancel**, or the cross of the card, closes the tool and drops your changes. **Done** keeps the rectangle.

After **Done** the image shows the rectangle with a dimmed outside, and the Save step says *Only 800 × 600 px is saved.* **Remove Crop** takes it away again. **Save Image** then writes only what is inside the rectangle, at full precision and with the color and tone values applied; the rectangle is not made smaller or scaled. The rectangle belongs to the stacked image: a new **Stack**, or another recording, removes it. It is not stored in a project file. The stacked image saved from the step *Stack* is always written whole.

## Saving a project

A project remembers your session: the recording and the settings of the steps (alignpoint settings, stacking options, wavelet layers, color). Save it with **Save Project…** in the toolbar and open it again later with **Open Project…**. Both buttons sit to the right of the menus; **Save Project…** is locked until a recording is open, and both are locked while a calculation runs. Project files end in `.rkproj`.

When you open a project and a recording file has been moved or deleted, Siderea Planets tells you which file is missing. The images themselves are not stored in the project; Align and Stack run again on the recording. The Wavelets step stays locked until Stack has run, so the layer values of the project are kept for that first stacked image, even when **Keep layer values for a new stack** is off; you see them again as soon as the Wavelets step opens. A second stacked image puts the layers back to their start values as usual.

## Wavelet schemes

**Save Scheme…** and **Load Scheme…** in the Wavelets step (under **Advanced**) store the six layer settings in a small wavelet scheme file with the ending `.rwv`. Load it again to bring the same layer values back.

## Aligned and averaged videos

In the Stack step, under **Advanced**, the group **Create AVI** has two buttons, **Save Aligned Video…** and **Save Averaged Video…**. Both need the Align step to be done and use the alignpoints and the frames that stay after **Keep Best Frames** (the checked frames of the Frame List). After pressing one, choose a name in the file dialog; the video is then written with progress, and **Cancel** and **Pause** work as for Stack. A cancelled or failed video leaves no file behind. Both write an uncompressed AVI of the size of the reference frame, so the file gets large; there is no codec and no quality setting (*Codec* and *Compression quality* are locked). The size limit is 4 GiB; a longer video stops with a message.

| Button | What the video contains |
|---|---|
| Save Aligned Video… | Every frame that stays, moved onto the reference frame the way Stack does it (*Correct geometry*, *Even out frame brightness* and *Use nearest points* count). Parts a frame does not cover are black. *Sort by quality* puts the best frame first. |
| Save Averaged Video… | A moving average: each video frame is the mean of *Average of n frames* aligned frames that follow each other, and the window moves on by one frame. From N frames you get N − n + 1 video frames; with fewer than n frames, one frame with their mean. |

The speed of the video is *Frame rate (fps)* (default 25). **Not available yet:** *Frame size* other than *Original size* (the size of the source frames), *Use prefilters* and the method *Compressed*.
