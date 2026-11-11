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

> Process and Do All cannot be stopped once they run: **Cancel** then drops the result, and the status bar says *Cancelled*.
