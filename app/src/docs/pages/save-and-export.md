---
title: Save and export
group: Results
order: 8
---
# Save and export

Press **Save image** to write a result to a file. The button is on the Stack tab (the stacked image) and on the Wavelet tab (the sharpened image). The file dialog is the one of your operating system.

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
- Black is the lowest value and white the highest value of the format, so a 16 bit file uses the full range from 0 to 65535.
- A colour image is written with its three channels in one file. A single FITS file holds them as three planes (red, green, blue). Black and white images stay single channel.

> An existing file with the same name is replaced only when the new file was written completely. If saving fails, the old file stays as it was.

## Saving a project

A project remembers your session: the recording and the settings of the steps (alignpoint settings, stacking options, wavelet layers, colour). Save it with **Tools ▸ Save project…** and open it again later with **Tools ▸ Open project…**. Project files end in `.rkproj`.

When you open a project and a recording file has been moved or deleted, SidereaPlanets tells you which file is missing. The images themselves are not stored in the project; Align and Stack run again on the recording.

## Wavelet schemes

**Save Scheme** and **Load Scheme** on the Wavelet tab store the six layer settings in a small `.rwv` file, the same format as the reference program. Schemes made with the reference program can be loaded and the other way round.

## Aligned and integrated videos

**Save Aligned** (the aligned frames) and **Save Integrated** (a moving average over several frames) write a video as uncompressed AVI. Because it is uncompressed, the file gets large; there is no codec and no quality setting. The size limit is 4 GiB.

| Function | Status |
|---|---|
| Save Aligned | Not available yet |
| Save Integrated | Not available yet |
