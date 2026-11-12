---
title: Menus and settings
group: Using the window
order: 8
---
# Menus and settings

A menu closes with Escape or a click outside. An entry with an arrow opens a submenu to its right.

## MRU

The files you opened last, newest first (up to ten). A click opens the file again. The list is kept between runs.

## Flat/Dark/Reference

Load Darkframe, Load Flatfield and Load Referenceframe, and the entries that use them. All of it is **Not available yet**; the entries that need a loaded file are locked.

## Tools

*Show MACRO/Batch window* is **Not available yet**. *Record Processing Information in Filename* is a check box.

## Settings

| Entry | Choices | Default |
|---|---|---|
| Automatic Processing | Align to Wavelet · Align to Stack · Stack to Wavelet | all off |
| Autodetect Colour/BW | check box | off |
| Processing Area | 2048 · 1024 · 512 · 256 · 128 | 512 |
| Luminance Setting | Red · Green · Blue, and Reset Luminance | 0.299 · 0.587 · 0.114 |
| Stretch FIT/TIFF | Always Ask · Yes · No | Always Ask |
| Theme | Dark · Light | Dark |
| Show Advanced Controls, Play Sounds, Swap Red/Blue Channel, Wait when loading a single image, Maximize during messages | check boxes | off |
| Save to Subdir | text | empty |
| Bias subtract | number | 0 |

The settings are collected here already; the steps that use them follow with the steps themselves. The check boxes that switch things on the image on and off are not in a menu of the toolbar but in **Show ▾** at the top right of the image, see [The image area](image-area). The entries *Check for the reference program Update*, *Layouts* and *Set Skin* of the original do not exist. **Theme** replaces *Set Skin*.

## Toolbar

**Select** opens a recording, like the button in the step *Open*. **Open** and **Save** next to the menus open and save a project file (see Save and export). **Open** is locked while a calculation runs; **Save** is also locked until a recording is open. **CPUs** is the number of threads for calculations (default 1). **About** is **Not available yet**.

## Number fields

Click in a field and type, or use the small arrows or the up and down keys. The decimal separator is always a point. Denoise shows two decimals, Sharpen three.
