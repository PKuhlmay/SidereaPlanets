---
title: The window at a glance
group: Getting started
order: 3
---
# The window at a glance

The window guides you through the work in seven steps. The names of all buttons and settings are those of the reference program; where they sit is decided by the step they belong to. From top to bottom:

| Part | What it does |
|---|---|
| Toolbar | **Select Recording** opens a recording. The menus Recent Recordings, Flat/Dark/Reference, Tools and Settings sit next to it, followed by **Open** and **Save** for project files. **Cancel** and **Pause** are in the middle; **Help**, **About** and the number of CPUs (threads for the calculations, all cores by default) on the right |
| Step bar | The seven steps **Open · Alignpoints · Align · Limit · Stack · Wavelets · Save**. At the right: file name, codec, size and number of frames |
| Step panel (left) | Shows only the step you are on: what it does, its settings and, at the bottom, the one button that runs it |
| Image area | The image. **Show ▾** at the top right holds the check boxes that switch things on the image on and off; in the Wavelets step **Functions »** sits next to it |
| Functions panel (right) | Wavelets step only: the Functions buttons, Contrast/Brightness and Copy To, Load to, Difference |
| Frame bar | Alignpoints, Align and Limit only: status, Goto Frame and a slider over all frames |
| Status bar | Progress, current action, result, position under the pointer and time |

> The window is being built step by step. Parts that do not work yet are marked **Not available yet** on the page that describes them.

## The seven steps

| Step | What it does | Button at the bottom | Described on |
|---|---|---|---|
| 1 Open | Choose a recording | **Select Recording** | [Select a recording](select-a-recording) |
| 2 Alignpoints | Mark the details the frames are lined up on | **Set Alignpoints** | [Alignpoints, Align and Limit](steps-alignpoints-align-limit) |
| 3 Align | Line up every frame and rate its sharpness | **Align** | [Alignpoints, Align and Limit](steps-alignpoints-align-limit) |
| 4 Limit | Keep the sharpest frames | **Limit** | [Alignpoints, Align and Limit](steps-alignpoints-align-limit) |
| 5 Stack | Add the kept frames into one image | **Stack** | [The Stack step](step-stack) |
| 6 Wavelets | Sharpen the stack | **Do All** (and **Process**) | [Wavelets and Save](step-wavelets-save) |
| 7 Save | Write the finished image to a file | **Save image** | [Wavelets and Save](step-wavelets-save) |

## Before you open a recording

Only the step **Open** is available. The other steps are greyed out, and the image area asks you to press **Select Recording**. Hover over a greyed-out step to see what has to be done first, for example *Run Align first*.

## What a step panel shows

From top to bottom:

1. A line in green with the result of the step before, for example *Alignpoints · 82 set*. It shows when you are on the next step to take.
2. The header: *Step 3 of 7*, the name of the step and one sentence on what it does.
3. The settings you need most.
4. **Advanced**: a line that folds out the remaining settings of the step. Nothing is missing, rarely used settings are only tucked away. The window remembers for each step whether Advanced is open, also after a restart.
5. At the bottom, the button that runs the step, and under it *Then:* and the next step.

## The next step opens by itself

Outside a calculation, exactly one button in the whole window is filled in amber: the button at the bottom of the step panel. When a step has run, the next step opens by itself and its button is the filled one. After **Select Recording** this is Alignpoints, after **Do All** it is Save. While a calculation runs, the button is locked and only **Cancel** is filled.

After **Save image** the Save step stays open and shows the result; instead of *Then:* it offers **Open another recording**.

## The step bar

| Look of a step | Meaning |
|---|---|
| Amber background | The step you are looking at |
| Check mark in green | Done. Click to look at it again; hover to see its result |
| Number | Possible, not done yet. Click to open it |
| Grey | Not possible yet. The tooltip says which step is missing |

During a calculation no step can be clicked.

## Going back to a step

Click a step with a check mark to see its result and settings again. The panel then offers **‹Action› again**, for example **Align again**, and a filled **Continue:** button that takes you to the next step that is still open. When steps after it would be discarded, the panel says *Running it again discards the steps after it.*

**Set Alignpoints**, **Align** and **Limit** can be run again at any time once they have run, also after Stack and Wavelets. This differs from the reference program, where the first two are locked after Align. A new run discards what was built on the old one, and the step bar falls back to the next step to take. See [Align your frames](align-your-frames).

Wavelets counts as done as long as the result of **Do All** matches the current settings. If you change a layer or another setting of the step afterwards, it loses its check mark, and so does Save.

## Help

**Help** in the toolbar opens this documentation in a window of its own, in front of the main window. Your work stays where it is, so you can keep the help open beside it. Press Escape in the help window, or close it, when you are done. On the Mac the menu bar has it: **Help ▸ Siderea Planets Help** (⌘?); see [Menus and settings](menus-and-settings).

## Title of the window

The title shows the name of the file. While a step is running it shows the step and the percent done, for example `Siderea Planets · Align 40%`, so you can follow it in the taskbar.

## Light and dark

The window is dark by default, so it does not tint how you judge the colours of an image. Choose **Settings ▸ Theme ▸ Light** for the light theme. The area around the image stays dark in both themes. The choice is remembered.

More on the parts: [The image area](image-area), [Menus and settings](menus-and-settings), [Progress, Cancel and Pause](progress-cancel-pause).
