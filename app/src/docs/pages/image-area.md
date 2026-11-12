---
title: The image area
group: Using the window
order: 4
---
# The image area

The image sits on a dark background in the middle of the window. It shows the frame you are on (steps Alignpoints, Align, Limit and, until the stack exists, Stack), the stacked image (Stack, after Stack) and the sharpened image (Wavelets and Save). At the top left a label says which one it is: *Frame 3 of 10*, *Stack of 8 frames* or *Sharpened stack*. Before a recording is open the area asks you to press **Select**.

## Show menu

**Show ▾** at the top right of the image holds the check boxes that decide what is shown on the image. Which ones it holds depends on the step:

| Steps | Check boxes |
|---|---|
| Alignpoints, Align, Limit | Show Full Image · Show ROI · Show Alignpoints (on) · Show Aligndata · Show Framelist · Show Registrationgraph · Show Prefilter |
| Stack | Show Full Image · Show AlignSections · Show Stackgraph · Show Stacklist |
| Wavelets, Save | Show Full Image · Show AlignPoints · Show Processing Area |
| Open | No menu |

In the Wavelets step **Functions »** sits to the left of **Show ▾**. It hides and shows the Functions panel on the right; the image then uses the space that becomes free.

## Zoom and move

| Action | How |
|---|---|
| Zoom | Mouse wheel, or **−** and **+** in the zoom bar. The wheel zooms around the pointer |
| Move | Drag the image |
| Fit | **Fit** in the zoom bar shows the whole image |
| 100 % | Click the scale in the zoom bar. It shows the zoom in percent of the image pixels |

A recording starts fitted and follows the size of the window until you zoom or move. Beyond 200 % single pixels are shown as squares.

## Before and after

In the steps Wavelets and Save the zoom bar has a **Before** switch. While it is pressed the window shows the stack as it was before sharpening. Press it again to see the sharpened image. Both views differ as soon as you have pressed **Process** or **Do All**.

## Marks on the image

| Mark | When |
|---|---|
| Red dots | Alignpoints after Set Alignpoints, with **Show Alignpoints** in the Show menu |
| Yellow rings with a green stroke | Alignpoints after Limit, with **Show AlignSections** (Stack) or **Show AlignPoints** (Wavelets, Save). The stroke shows the shift of the point, enlarged |
| Dashed amber square | The Processing Area in the Wavelets and Save steps, with **Show Processing Area**. Its size comes from **Settings ▸ Processing Area** |

The marks move and zoom with the image.

## Cards over the image

Framelist, Registrationgraph and Alignmentpoints are small cards in the upper right corner of the image, below the **Show ▾** button, one below the other. A card is shown while its check mark in the Show menu is set. The cross in the card removes the check mark. Cards cannot be moved.

## Position

In the Wavelets and Save steps the status bar shows the pixel under the pointer, for example `X=200 Y=201 Stack=10`.
