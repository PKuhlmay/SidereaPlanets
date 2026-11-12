---
title: Save and export
group: Results
order: 8
---
# Save and export

Press **Save image** to write a result to a file. It is the button of the step *Save* (the sharpened image). For the stacked image, click the step *Stack* in the step bar once the stack is done: **Save image** then sits under the result. The file dialog is the one of your operating system.

In the step *Save* the file looks like the image in the window: the values of RGB Align, RGB Balance, Histogram, Gamma and Contrast/Brightness are applied to the sharpened image before it is written, at full precision (not from the 8 bit picture on screen). With all of them at their start values the sharpened image is written unchanged. The step *Stack* saves the stack without these values. The sharpened image is the result of **Do All**. You do not have to run it before you save: see below.

## Choosing the format

Pressing **Save image** first opens a short list of file types: seven for the stacked image (step Stack), five for the sharpened image (step Save). Pick one and the file dialog of your operating system opens for that type. The file type decides the format, not the ending you type: if the ending does not fit, SidereaPlanets replaces it. The dialog suggests the name of your recording and opens in its folder.

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
- Black is the lowest value and white the highest value of the format, so a 16 bit file uses the full range from 0 to 65535. the reference program writes 16 bit files as its 8 bit image times 256 (at most 65280); if you compare files of both programs, divide the the reference program values by 256.
- **Save image** in the Save step writes what the current settings produce. The step can be opened as soon as the stack exists. If you moved a slider, changed a value of a layer, switched **Colour** or **LRGB**, made a new **Stack**, or have not pressed **Do All** yet, SidereaPlanets runs **Do All** with the current values first and then saves. That run shows progress like any other and can be cancelled with **Cancel**; if you cancel it, or it fails, nothing is saved and the status bar says so. If nothing changed since the last **Do All**, the file is written at once. Changing the Processing Area does not start a new **Do All**, because it only decides what the preview shows. The colour and tone cards (RGB Align, RGB Balance, Histogram, Gamma, Contrast/Brightness) are applied when the file is written, so they never need a new **Do All**. **Process** only changes the preview area and is not saved.
- A colour image is written with its three channels in one file. A single FITS file holds them as three planes (red, green, blue). Black and white images stay single channel.

> An existing file with the same name is replaced only when the new file was written completely. If saving fails, the old file stays as it was.

## Saving a project

A project remembers your session: the recording and the settings of the steps (alignpoint settings, stacking options, wavelet layers, colour). Save it with **Save** in the toolbar and open it again later with **Open**. Both buttons sit to the right of the menus; **Save** is locked until a recording is open, and both are locked while a calculation runs. Project files end in `.rkproj`.

When you open a project and a recording file has been moved or deleted, SidereaPlanets tells you which file is missing. The images themselves are not stored in the project; Align and Stack run again on the recording. The Wavelets step stays locked until Stack has run, so the layer values of the project are kept for that first stacked image, even when **Hold Wavelet Setting** is off; you see them again as soon as the Wavelets step opens. A second stacked image puts the layers back to their start values as usual.

## Wavelet schemes

**Save Scheme** and **Load Scheme** in the Wavelets step (under **Advanced**) store the six layer settings in a small `.rwv` file, the same format as the reference program. Schemes made with the reference program can be loaded and the other way round.

## Aligned and integrated videos

In the Stack step, under **Advanced**, the group **Create AVI** has two buttons. Both need the Align step to be done and use the alignpoints and the frames that stay after **Limit** (the checked frames of the Framelist). After pressing one, choose a name in the file dialog; the video is then written with progress, and **Cancel** and **Pause** work as for Stack. A cancelled or failed video leaves no file behind. Both write an uncompressed AVI of the size of the reference frame, so the file gets large; there is no codec and no quality setting (Codec and Compress Qual have no effect). The size limit is 4 GiB; a longer video stops with a message.

| Button | What the video contains |
|---|---|
| Save Aligned | Every frame that stays, moved onto the reference frame the way Stack does it (*Correct geometry*, *Normalization of frame intensity* and *Use Nearestby Alignpoints* count). Parts a frame does not cover are black. *Sort On Quality* puts the best frame first. |
| Save Integrated | A moving average: each video frame is the mean of *Average of n Frames* aligned frames that follow each other, and the window moves on by one frame. From N frames you get N − n + 1 video frames; with fewer than n frames, one frame with their mean. |

The speed of the video is *Framerate* (default 25). Not available yet: *Image size* other than the original size, *Use Prefilters* and the method *Compressed*.

> What the reference program does exactly for Save Integrated was not observed. The moving average above is our reading of its description, so a video from the reference program may differ in length and borders.
