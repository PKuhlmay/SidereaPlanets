---
title: Save and export
group: Results
order: 8
---
# Save and export

Press **Save image** to write a result to a file. The button is on the Stack tab (the stacked image) and on the Wavelet tab (the sharpened image). The file dialog is the one of your operating system.

On the Wavelet tab the file looks like the image in the window: the values of RGB Align, RGB Balance, Histogram, Gamma and Contrast/Brightness are applied to the sharpened image before it is written, at full precision (not from the 8 bit picture on screen). With all of them at their start values the sharpened image is written unchanged. The Stack tab shows the stack without these values and saves it the same way. The sharpened image is the result of **Do All**; run it before you save.

## Choosing the format

Pressing **Save image** first opens a short list of file types: seven on the Stack tab, five on the Wavelet tab. Pick one and the file dialog of your operating system opens for that type. The file type decides the format, not the ending you type: if the ending does not fit, SidereaPlanets replaces it. The dialog suggests the name of your recording and opens in its folder.

| Format | Ending | Bit depth | Stack tab | Wavelet tab |
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
- A colour image is written with its three channels in one file. A single FITS file holds them as three planes (red, green, blue). Black and white images stay single channel.

> An existing file with the same name is replaced only when the new file was written completely. If saving fails, the old file stays as it was.

## Saving a project

A project remembers your session: the recording and the settings of the steps (alignpoint settings, stacking options, wavelet layers, colour). Save it with **Save** in the toolbar and open it again later with **Open**. Both buttons sit to the right of the menus; **Save** is locked until a recording is open, and both are locked while a calculation runs. Project files end in `.rkproj`.

When you open a project and a recording file has been moved or deleted, SidereaPlanets tells you which file is missing. The images themselves are not stored in the project; Align and Stack run again on the recording.

## Wavelet schemes

**Save Scheme** and **Load Scheme** on the Wavelet tab store the six layer settings in a small `.rwv` file, the same format as the reference program. Schemes made with the reference program can be loaded and the other way round.

## Aligned and integrated videos

On the Stack tab, the panel **Create AVI** has two buttons. Both need the Align step to be done and use the alignpoints and the frames that stay after **Limit** (the checked frames of the Framelist). After pressing one, choose a name in the file dialog; the video is then written with progress, and **Cancel** and **Pause** work as for Stack. A cancelled or failed video leaves no file behind. Both write an uncompressed AVI of the size of the reference frame, so the file gets large; there is no codec and no quality setting (Codec and Compress Qual have no effect). The size limit is 4 GiB; a longer video stops with a message.

| Button | What the video contains |
|---|---|
| Save Aligned | Every frame that stays, moved onto the reference frame the way Stack does it (*Correct geometry*, *Normalization of frame intensity* and *Use Nearestby Alignpoints* count). Parts a frame does not cover are black. *Sort On Quality* puts the best frame first. |
| Save Integrated | A moving average: each video frame is the mean of *Average of n Frames* aligned frames that follow each other, and the window moves on by one frame. From N frames you get N − n + 1 video frames; with fewer than n frames, one frame with their mean. |

The speed of the video is *Framerate* (default 25). Not available yet: *Image size* other than the original size, *Use Prefilters* and the method *Compressed*.

> What the reference program does exactly for Save Integrated was not observed. The moving average above is our reading of its description, so a video from the reference program may differ in length and borders.
