---
title: Updates
group: Using the window
order: 8.5
---
# Updates

Siderea Planets can look for a new version and install it for you. New versions are published as releases on GitHub; the app finds them there.

## Checking for a new version

- **At the start.** When the app starts it looks for a new version, at most once a day. You switch this off with **Settings ▸ Check for updates automatically**. If the computer is offline, this check says nothing; it tries again at the next start on another day.
- **By hand.** **Check for Updates…** looks at once, whenever you like. On the Mac it is in the menu **Siderea Planets**; on Windows and Linux it is in the **Settings** menu of the toolbar. The result appears in the status bar: *Siderea Planets is up to date*, or a note that the check did not work (for example because you are offline). It is locked while a calculation runs.

## The dialog

When there is a new version, a dialog shows its number and what is new in it (the release notes). You have three choices:

- **Install and Restart** downloads the update, installs it and starts Siderea Planets again. The button is locked while a calculation (Align, Stack, Wavelets …) runs; its tooltip says so. Wait until the calculation has finished, or cancel it. Unsaved work is not saved for you: use **Save Project…** first if you want to keep the session.
- **Later** closes the dialog. The automatic check offers the same version again the next day.
- **Skip This Version** closes the dialog and the automatic check does not mention this version again. A newer version is still offered. **Check for Updates…** always shows the newest version, skipped or not.

The update is checked against a signature before it is installed; a download that was changed on the way is refused.

## What each installation does

| Installation | Updates itself |
|---|---|
| macOS app | yes |
| Windows installer | yes |
| Linux AppImage | yes |
| Linux `.deb` package | no |

The `.deb` package belongs to your package manager, so Siderea Planets cannot replace it. The dialog still tells you about the new version, with the address of the release page, where you download the new `.deb` and install it as before. If you want an app that updates itself on Linux, use the AppImage.
