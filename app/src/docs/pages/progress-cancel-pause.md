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
| Status bar, first field | A progress bar with the percent done |
| Status bar, second field | What is happening, for example *Loading frame: 4* |
| Status bar, last field | *Time used* and *Time left* |
| Window title | Step and percent, for example `SidereaPlanets · Align 40%` |
| Step buttons | All locked |

## Cancel and Pause

**Cancel** turns amber while a step runs and stops it; the status bar then says *Cancelled*. **Pause** holds the step and reads **Resume** until you press it again; the time used does not count while it is paused. Outside a step both buttons are locked.

## Messages

The third field of the status bar shows the result of the last step, for example *Alignment finished.*. A problem is shown there too, starting with *Error:*. A button whose function is not built yet says so in that field, for example *Align: not available yet*.

## Do All and Process

**Do All** can be stopped and paused like Align and Stack. It works layer by layer and checks between two layers, so **Cancel** and **Pause** take effect after the layer that is being calculated; on a large image that is a moment. The progress bar counts the layers. A cancelled **Do All** changes nothing: the image keeps the result of the earlier run, and **Save image** still writes that one, not a half-finished result.

**Process** calculates only the Processing Area, which takes a fraction of a second, so it is not stopped in the middle. **Cancel** drops the result when it arrives, and the image stays as it was.
