---
title: Menus and settings
group: Using the window
order: 8
---
# Menus and settings

A menu closes with Escape or a click outside. An entry with an arrow opens a submenu to its right.

## Recent Recordings

The recordings you opened last, newest first (up to ten). A click opens the file again. When there are none yet, the menu says *No recent recordings*. The list is kept between runs.

## Settings

| Entry | Choices | Default |
|---|---|---|
| Preview Area | 2048 × 2048 px · 1024 × 1024 px · 512 × 512 px · 256 × 256 px · 128 × 128 px | 512 × 512 px |
| Theme | Dark · Light (not on the Mac, which follows macOS) | Dark |
| Show Advanced Controls | check box: opens the *Advanced* section of every step panel when you tick it. You can close and open each section on its own afterwards; unticking leaves them as they are | off |
| Show CPU and Memory Usage | check box: the CPU load of the app as a share of all cores and the memory it holds, at the left end of the status bar. It is measured once a second while the window is visible; unticked, the field goes and nothing is measured | on |

Below them, under *Not available yet*, the menu lists dimmed entries that no step uses yet: Automatic Processing, Autodetect Color/BW, Luminance Setting, Stretch FIT/TIFF, Play Sounds, Swap Red/Blue Channel, Wait When Loading a Single Image, Maximize During Messages, Save to Subdir and Bias Subtract. The check boxes that switch things on the image on and off are not in a menu of the toolbar but in **Show ▾** at the top right of the image, see [The image area](image-area). The **Preview Area** is the part of the image that the wavelet preview calculates, see [Wavelet sharpening](wavelet-sharpening).

## Toolbar

**Select Recording…** opens a recording, like the button in the step *Open*; you can also drop a video or images onto the window. **Open Project…** and **Save Project…** next to the menus open and save a project file (see [Save and export](save-and-export)). **Open Project…** is locked while a calculation runs; **Save Project…** is also locked until a recording is open. The field **CPUs** (CPU cores to use) is the number of threads that all calculations use: Set Alignpoints, Align, Stack, Save Aligned Video and Save Averaged Video, the wavelet preview and Do All, the color and tone panels and saving an image. It starts at the number of logical cores of your computer and goes from 1 up to that number; a smaller or larger value is set to the nearest one. So all cores are used from the start; lower the number if the computer should stay free for other work. A change applies to the next calculation, a running one finishes with the number it started with. The value is not saved in the project file, because the number of cores belongs to the computer. **About** is **Not available yet**.

## On the Mac

The Mac has a menu bar as well as the toolbar; every item does what the same button of the window does and is dimmed when the button is locked.

- **Siderea Planets ▸ About Siderea Planets** shows the version. **Settings… (⌘,)** opens the *Settings* menu of the toolbar. The theme follows the appearance of macOS, light or dark, also while the app runs, so there is no *Theme* entry. The field **CPUs** stays in the toolbar.
- **File** has **Select Recording… (⌘O)**, **Open Recent** (the list of *Recent Recordings*), **Open Project…**, **Save Image… (⌘S)** and **Save Project…**. **Save Image…** writes the stacked or sharpened image in the file type chosen last (TIFF 16-bit at the start), without asking for the type again; it is dimmed until the stack exists.
- **Edit** has Undo, Redo, Cut, Copy, Paste and Select All for the number and text fields.
- **View** has **Zoom In (⌘+)**, **Zoom Out (⌘-)**, **Zoom to Fit (⌘0)** and **Actual Size (⌘1)** for the image, and full screen.
- **Run** has **Cancel (⌘.)**, **Pause** (it reads **Resume** while paused) and **Go to Step** with the seven steps (⌥⌘1 to ⌥⌘7), which open a step like a click on the step bar.
- **Window** has Minimize and Zoom. **Help ▸ Siderea Planets Help (⌘?)** opens this documentation.

Double-clicking a project file (`.sidereaplanets`) in Finder, or choosing **Open With ▸ Siderea Planets**, opens the project. Dropping a recording or a project file on the window opens it too.

## Keyboard shortcuts

The shortcuts use **⌘ (Cmd)** on the Mac and **Ctrl** on Windows and Linux. The step shortcuts differ because Alt and a digit is free on Windows only.

| What | Mac | Windows and Linux |
|---|---|---|
| Select Recording… | ⌘O | Ctrl+O |
| Save Image… (in the file type chosen last) | ⌘S | Ctrl+S |
| Settings menu | ⌘, | Ctrl+, |
| Cancel a calculation | ⌘. | Ctrl+. |
| Open step 1 to 7 | ⌥⌘1 to ⌥⌘7 | Alt+1 to Alt+7 |
| Previous and next frame | ← → or ⌘[ and ⌘] | ← → or Ctrl+[ and Ctrl+] |
| Zoom in, zoom out | ⌘+ and ⌘- | Ctrl+plus and Ctrl+minus |
| Fit the image, show it at 100% | ⌘0 and ⌘1 | Ctrl+0 and Ctrl+1 |

A shortcut that does not fit the moment does nothing: during a calculation only Cancel works, and the arrow keys leave a text or number field to its cursor. The bracket keys are found by their place on the keyboard, so they work with a German keyboard too (the keys right of the letter P).

## Number fields

Click in a field and type, or use the small arrows or the up and down keys. The decimal separator is always a point. Denoise shows two decimals, Sharpen three.
