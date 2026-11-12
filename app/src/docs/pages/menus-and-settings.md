---
title: Menus and settings
group: Using the window
order: 8
---
# Menus and settings

A menu closes with Escape or a click outside. An entry with an arrow opens a submenu to its right.

## Recent Recordings

The recordings you opened last, newest first (up to ten). A click opens the file again. When there are none yet, the menu says *No recent recordings*. The list is kept between runs.

## Flat/Dark/Reference

Load Darkframe, Load Flatfield and Load Referenceframe, and the entries that use them. All of it is **Not available yet**; the entries that need a loaded file are locked.

## Tools

*Show MACRO/Batch window* is **Not available yet**. *Record Processing Information in Filename* is a check box.

## Settings

| Entry | Choices | Default |
|---|---|---|
| Automatic Processing | Align to Wavelet · Align to Stack · Stack to Wavelet | all off |
| Autodetect Color/BW | check box | off |
| Preview area | 2048 · 1024 · 512 · 256 · 128 (pixels per side) | 512 |
| Luminance Setting | Red · Green · Blue, and Reset Luminance | 0.299 · 0.587 · 0.114 |
| Stretch FIT/TIFF | Always Ask · Yes · No | Always Ask |
| Theme | Dark · Light | Dark |
| Show Advanced Controls | check box: opens the *Advanced* section of every step panel when you tick it. You can close and open each section on its own afterwards; unticking leaves them as they are | off |
| Play Sounds, Swap Red/Blue Channel, Wait when loading a single image, Maximize during messages | check boxes | off |
| Save to Subdir | text | empty |
| Bias subtract | number | 0 |

The settings are collected here already; the steps that use them follow with the steps themselves. The check boxes that switch things on the image on and off are not in a menu of the toolbar but in **Show ▾** at the top right of the image, see [The image area](image-area). The **Preview area** is the part of the image that the wavelet preview calculates, see [Wavelet sharpening](wavelet-sharpening).

## Toolbar

**Select Recording** opens a recording, like the button in the step *Open*. **Open Project…** and **Save Project…** next to the menus open and save a project file (see [Save and export](save-and-export)). **Open Project…** is locked while a calculation runs; **Save Project…** is also locked until a recording is open. The field **CPU cores to use** is the number of threads that all calculations use: Set Alignpoints, Align, Stack, Save Aligned Video and Save Averaged Video, the wavelet preview and Do All, the color and tone panels and saving an image. It starts at the number of logical cores of your computer and goes from 1 up to that number; a smaller or larger value is set to the nearest one. So all cores are used from the start; lower the number if the computer should stay free for other work. A change applies to the next calculation, a running one finishes with the number it started with. The value is not saved in the project file, because the number of cores belongs to the computer. **About** is **Not available yet**.

## On the Mac

The Mac app has no toolbar menu *Settings* and no *CPU cores to use* field in the toolbar; it uses the places of a Mac program. **Siderea Planets ▸ Settings… (⌘,)** opens a window with the **CPU cores to use** field: the same number of threads, from 1 up to all cores of the Mac, all cores at the start. **Siderea Planets ▸ About Siderea Planets** shows the version. **Help ▸ Siderea Planets Help (⌘?)** opens this documentation in a window of its own. The theme follows the appearance of macOS, so there is no *Theme* entry. In the **File** menu **Select Recording…** opens a recording; **Open Project…** and **Save Project…** follow below it. **View ▸ Zoom to Fit** fits the image to the window. **Run ▸ Go to Step** (⌥⌘1 to ⌥⌘7) opens a step like a click on the step bar, and **Run ▸ Cancel (⌘.)** and **Pause** do what the buttons in the middle of the toolbar do.

## Number fields

Click in a field and type, or use the small arrows or the up and down keys. The decimal separator is always a point. Denoise shows two decimals, Sharpen three.
