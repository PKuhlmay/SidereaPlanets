---
title: Progress, Cancel and Pause
group: Using the window
order: 9
---
# Progress, Cancel and Pause

Align, Stack and Do All can take a while. While one of them runs, the window shows how far it is and lets you stop it.

## While a step runs

| Where | What you see |
|---|---|
| Status bar, first field | A progress bar with the percent done. It is empty again once the step is over, whether it finished, was cancelled or failed |
| Status bar, second field | What is happening, for example *Aligning frame: 4* |
| Status bar, last field | *Time used* and *Time left* |
| Window title | Step and percent, for example `Siderea Planets · Align 40%` |
| Step bar | No step can be opened |
| Button at the bottom of the step panel | Locked. Only **Cancel** is filled |

## Cancel and Pause

**Cancel** turns amber while a step runs and stops it (**⌘.**, Windows and Linux **Ctrl+.**); the status bar then says *Cancelled*. **Pause** pauses the step and reads **Resume** until you press it again; the time used does not count while it is paused. Outside a step both buttons are locked.

## Messages

The third field of the status bar shows the result of the last step, for example *Alignment finished*. A problem is shown there too, in the form *‹Step› failed: ‹reason›*, for example *Align failed: …*. A control whose function is not built yet is dimmed and says *Not available yet* in its tooltip.

When a run ends successfully, the next step opens by itself. If you cancel, the step stays where it is.

## Do All and Update Preview

**Do All** (Wavelets step) can be stopped and paused like Align and Stack. It works layer by layer and checks between two layers, so **Cancel** and **Pause** take effect after the layer that is being calculated; on a large image that is a moment. The progress bar counts the layers. A cancelled **Do All** changes nothing: the image keeps the result of the earlier run, and **Save Image** still writes that one, not a half-finished result.

**Update Preview** calculates only the preview area, which takes a fraction of a second, so it is not stopped in the middle. **Cancel** drops the result when it arrives, and the image stays as it was.
