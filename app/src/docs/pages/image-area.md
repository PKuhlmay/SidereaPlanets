---
title: The image area
group: Using the window
order: 4
---
# The image area

The image sits on a dark background in the middle of the window. It shows the frame you are on (steps Alignpoints, Align, Best Frames and, until the stack exists, Stack), the stacked image (Stack, after Stack) and the sharpened image (Wavelets and Save). At the top left a label says which one it is: *Frame 3 of 10*, *Stack of 8 frames* or *Sharpened stack*. Before a recording is open the area says *Select a recording or drop a video or images here*; when there is nothing to show yet it says *No image to show yet*.

## Show menu

**Show ▾** at the top right of the image holds the check boxes that decide what is shown on the image. Which ones it holds depends on the step:

| Steps | Check boxes |
|---|---|
| Alignpoints, Align, Best Frames | Show Full Image · Show Region · Show Alignpoints (on) · Show Align Data · Show Frame List · Show Quality Graph · Show Prefilter |
| Stack | Show Full Image · Show Alignpoints · Show Stack Graph · Show Stack List |
| Wavelets, Save | Show Full Image · Show Alignpoints · Show Preview Area |
| Open | No menu |

**Show Full Image**, **Show Align Data**, **Show Prefilter**, **Show Stack Graph** and **Show Stack List** are **Not available yet**: they sit under a heading of that name at the end of the menu and cannot be chosen. **Show Region** is locked until you have drawn a region.

In the Wavelets step **Adjust ›** sits to the left of **Show ▾**. It hides and shows the Adjustments panel on the right; the image then uses the space that becomes free.

## Zoom and move

| Action | How |
|---|---|
| Zoom | Mouse wheel, or **−** and **+** in the zoom bar. The wheel zooms around the pointer |
| Move | Drag the image |
| Fit | **Fit** in the zoom bar shows the whole image |
| 100% | Click the scale in the zoom bar. It shows the zoom in percent of the image pixels |

A recording starts fitted and follows the size of the window until you zoom or move (a click on the image without dragging does not count). Each time you go to another step the image is fitted again, because the space around it changes from step to step. Beyond 200% single pixels are shown as squares.

## Before and after

In the steps Wavelets and Save the zoom bar has a **Before/After** switch. In the position *Before* the window shows the stack as it was before sharpening; switch back to see the sharpened image. Both views differ as soon as you have pressed **Update Preview** or **Do All**.

## Marks on the image

| Mark | When |
|---|---|
| Red dots | Alignpoints after Set Alignpoints, with **Show Alignpoints** in the Show menu |
| Yellow rings with a green stroke | Alignpoints after Keep Best Frames, with **Show Alignpoints** (Stack, Wavelets, Save). The stroke shows the shift of the point, enlarged |
| Dashed amber square | The preview area in the Wavelets and Save steps, with **Show Preview Area**. Its size comes from **Settings ▸ Preview area** |

The marks move and zoom with the image.

## Cards over the image

Frame List, Quality Graph and Alignpoints Used for Stacking are small cards in the upper right corner of the image, below the **Show ▾** button, one below the other. A card is shown while its check mark in the Show menu is set. The cross in the card removes the check mark. Cards cannot be moved.

## Position

In the Wavelets and Save steps the status bar shows the pixel under the pointer, for example `X=200 Y=201 Stack=10`.
