---
title: The image area
group: Using the window
order: 4
---
# The image area

The image sits on a dark background in the middle of the window. It shows the frame you are on (Align tab and, until the stack exists, Stack tab), the stacked image (Stack tab, after Stack) and the sharpened image (Wavelet tab).

## Zoom and move

| Action | How |
|---|---|
| Zoom | Mouse wheel, or **−** and **+** in the zoom bar. The wheel zooms around the pointer |
| Move | Drag the image |
| Fit | **Fit** in the zoom bar shows the whole image |
| 100 % | Click the scale in the zoom bar. It shows the zoom in percent of the image pixels |

A recording starts fitted and follows the size of the window until you zoom or move. Beyond 200 % single pixels are shown as squares.

## Before and after

On the Wavelet tab the zoom bar has a **Before** switch. While it is pressed the window shows the stack as it was before sharpening. Press it again to see the sharpened image. Both views differ as soon as you have pressed **Process** or **Do All**.

## Marks on the image

| Mark | When |
|---|---|
| Red dots | Alignpoints after Set Alignpoints, with **Show Alignpoints** on the Align tab |
| Yellow rings with a green stroke | Alignpoints after Limit, with **Show AlignSections** (Stack tab) or **Show AlignPoints** (Wavelet tab). The stroke shows the shift of the point, enlarged |
| Dashed amber square | The Processing Area on the Wavelet tab, with **Show Processing Area**. Its size comes from **Settings ▸ Processing Area** |

The marks move and zoom with the image.

## Cards over the image

Framelist, Registrationgraph and Alignmentpoints are small cards in the upper right corner of the image, one below the other. A card is shown while its check mark in the action row is set. The cross in the card removes the check mark. Cards cannot be moved.

## Position

On the Wavelet tab the status bar shows the pixel under the pointer, for example `X=200 Y=201 Stack=10`.
