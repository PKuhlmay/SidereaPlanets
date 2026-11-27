---
title: The window at a glance
group: Getting started
order: 3
---
# The window at a glance

The window guides you through the work in seven steps; where a setting sits is decided by the step it belongs to. If a term is new to you, look it up in the [Glossary](glossary). From top to bottom:

| Part | What it does |
|---|---|
| Toolbar | **Select Recording…** opens a recording. The menus Recent Recordings and Settings sit next to it, followed by **Open Project…** and **Save Project…** for project files. **Cancel** and **Pause** are in the middle; **Help**, **About** and the number of CPU cores to use (threads for the calculations, all cores by default) on the right |
| Step bar | The seven steps **Open · Alignpoints · Align · Best Frames · Stack · Wavelets · Save**. At the right: file name, size, number of frames and codec |
| Step panel (left) | Shows only the step you are on: what it does, its settings and, at the bottom, the one button that runs it |
| Image area | The image. **Show ▾** at the top right holds the check boxes that switch things on the image on and off; in the Wavelets step **Adjust ›** sits next to it |
| Adjustments panel (right) | Wavelets step only: the Adjustments buttons, Contrast/Brightness and Remember, Restore, Difference |
| Frame bar | Alignpoints, Align and Best Frames only: status, **Go to frame** and a slider over all frames |
| Status bar | At the left the CPU load of the app as a share of all cores (**CPU (12)** means 12 cores) and the memory it holds, which you can hide in Settings; then progress, current action, result, position under the pointer and time; at the right the version, the build and when it was made, for example `0.1.0 · a4e826b · 2026-11-13 13:38` |

> The window is being built step by step. Parts that do not work yet are dimmed and marked **Not available yet**; hover over them to see the note.

## The seven steps

| Step | What it does | Button at the bottom | Described on |
|---|---|---|---|
| 1 Open | Choose a recording | **Select Recording** | [Select a recording](select-a-recording) |
| 2 Alignpoints | Mark the details the frames are lined up on | **Set Alignpoints** | [Alignpoints, Align and Best Frames](steps-alignpoints-align-limit) |
| 3 Align | Line up every frame and rate its sharpness | **Align** | [Alignpoints, Align and Best Frames](steps-alignpoints-align-limit) |
| 4 Best Frames | Keep the sharpest frames | **Keep Best Frames** | [Alignpoints, Align and Best Frames](steps-alignpoints-align-limit) |
| 5 Stack | Add the kept frames into one image | **Stack** | [The Stack step](step-stack) |
| 6 Wavelets | Sharpen the stack | **Do All** (and **Update Preview**) | [Wavelets and Save](step-wavelets-save) |
| 7 Save | Write the finished image to a file | **Save Image** | [Wavelets and Save](step-wavelets-save) |

## Before you open a recording

Only the step **Open** is available. The other steps are grayed out, and the image area asks you to select a recording or drop a video or images on it. Hover over a grayed-out step to see what has to be done first, for example *Run Align first*.

## What a step panel shows

From top to bottom:

1. A line in green with the result of the step before, for example *Alignpoints · 82 points* or *Align · 10 frames rated*. It shows when you are on the next step to take.
2. The header: *Step 3 of 7*, the name of the step and one sentence on what it does.
3. The settings you need most.
4. **Advanced**: a line that folds out the remaining settings of the step. Nothing is missing, rarely used settings are only tucked away. The window remembers for each step whether Advanced is open, also after a restart.
5. At the bottom, the button that runs the step, and under it *Then:* and the next step.

Hover over a button or a setting to see a short explanation. Settings that do nothing yet are dimmed and marked **Not available yet**, so you cannot set them by mistake.

## The next step opens by itself

Outside a calculation, exactly one button in the whole window is filled in amber: the button at the bottom of the step panel. When a step has run, the next step opens by itself and its button is the filled one. After **Select Recording** this is Alignpoints, after **Do All** it is Save. While a calculation runs, the button is locked and only **Cancel** is filled.

After **Save Image** the Save step stays open and shows the result; instead of *Then:* it offers **Select Another Recording…**.

## The step bar

| Look of a step | Meaning |
|---|---|
| Amber background | The step you are looking at |
| Check mark in green | Done. Click to look at it again; hover to see its result |
| Number | Possible, not done yet. Click to open it |
| Gray | Not possible yet. The tooltip says which step is missing |

During a calculation no step can be clicked. **Alt+1** to **Alt+7** (on the Mac **⌥⌘1** to **⌥⌘7**, also in the menu **Run ▸ Go to Step**) open the steps from the keyboard, like a click on them; a gray step does not open. All shortcuts are listed in [Menus and settings](menus-and-settings).

## Going back to a step

Click a step with a check mark to see its result and settings again. The panel then offers **‹Action› Again**, for example **Align Again**, and a filled **Continue:** button that takes you to the next step that is still open. When steps after it would be discarded, the panel says *Running it again discards the results of the later steps.*

**Set Alignpoints**, **Align** and **Keep Best Frames** can be run again at any time once they have run, also after Stack and Wavelets. A new run discards what was built on the old one, and the step bar falls back to the next step to take. See [Align your frames](align-your-frames).

Wavelets counts as done as long as the result of **Do All** matches the current settings. If you change a layer or another setting of the step afterwards, it loses its check mark, and so does Save.

## Help

**Help** in the toolbar opens this documentation in a window of its own, in front of the main window. Your work stays where it is, so you can keep the help open beside it. Press Escape in the help window, or close it, when you are done. On the Mac the menu bar has it as well: **Help ▸ Siderea Planets Help** (⌘?); see [Menus and settings](menus-and-settings).

## Title of the window

The title shows the name of the file. While a step is running it shows the step and the percent done, for example `Siderea Planets · Align 40%`, so you can follow it in the taskbar.

## Light and dark

The window is dark by default, so it does not tint how you judge the colors of an image. Choose **Settings ▸ Theme ▸ Light** for the light theme. The area around the image stays dark in both themes. The choice is remembered. On the Mac the window follows the appearance of macOS instead (System Settings ▸ Appearance), and there is no *Theme* entry.

More on the parts: [The image area](image-area), [Menus and settings](menus-and-settings), [Progress, Cancel and Pause](progress-cancel-pause).
