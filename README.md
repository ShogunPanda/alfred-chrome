# Alfred Chrome Workflow

[![Release](https://img.shields.io/github/release/ShogunPanda/alfred-chrome.svg)](https://github.com/ShogunPanda/alfred-chrome/releases/latest)
[![Packal](https://img.shields.io/badge/packal-alfred--chrome-green)](http://www.packal.org/workflow/alfred-chrome)
[![Forum](https://img.shields.io/badge/forum-alfred--chrome-yellowgreen)](https://www.alfredforum.com/topic/9473-alfred-chrome-open-url-in-google-chrome-also-with-specific-profile)
[![License](https://img.shields.io/github/license/ShogunPanda/alfred-chrome.svg)](https://github.com/ShogunPanda/alfred-chrome/blob/master/LICENSE.md)

This workflow allows you to open a URL in [Google Chrome](https://www.google.com/chrome/) via [Alfred](https://www.alfredapp.com/).

https://sw.cowtech.it/alfred-chrome

## Description

To trigger the workflow, type `chrome` or `cr`. Optionally you can add a URL.

Between the results you will a list of profiles to use to open the URL (or simply to open a new window).

If you hold the `Alt` key while selecting the result item, the window will be open using incognito mode.

## Compatibility

This workflow has been tested on Alfred 3, 4 and 5.

## Installation

Double click (or open in Alfred settings) the `Alfred Chrome.alfredworkflow` file.

### Allow the helper to execute

If macOS says the developer cannot be verified or Apple cannot verify that `alfred-chrome` is free of malware, allow the helper through System Settings. On recent macOS versions the warning only offers **Move to Trash** and **Done**; right-clicking **Open** does not bypass it.

1. Install the workflow from this repository's releases and trigger `cr` in Alfred once.
2. Choose **Done** to dismiss the warning.
3. Open **System Settings → Privacy & Security**, then scroll to **Security**.
4. Click **Open Anyway** next to the message about `alfred-chrome`, authenticate if prompted, and confirm **Open**.
5. Trigger `cr` again in Alfred.

If the entry is missing, open Alfred preferences, select the workflow, and choose **Open in Finder** from its context menu. Try opening the `alfred-chrome` helper, then return to Privacy & Security. On older macOS versions, the equivalent controls are under **System Preferences → Security & Privacy → General**. Managed Macs may require assistance from the administrator.

See [Apple's instructions for opening an unidentified developer's app](https://support.apple.com/en-us/102445) for the current Gatekeeper exception procedure.

### Chrome permissions and profile names

Chrome is launched through macOS Launch Services so camera, microphone, and network permissions are associated with Chrome. After upgrading from an older workflow, fully quit Chrome and reopen it with `cr` before checking these permissions.

Profile names use Chrome's `Local State` metadata, including account and custom names, with per-profile preferences as a fallback. Duplicate display names include the profile directory to distinguish them.

### No profiles found: Alfred file permissions

If `cr` shows no profiles and the workflow debugger reports `Not suitable profiles found. Nothing to do! :(`, check that Alfred has permission to access Chrome's profile files.

1. Open **System Settings → Privacy & Security → Files & Folders** and review Alfred's file-access permissions.
2. Enable the relevant file-access permissions for Alfred. If macOS prompts to allow Alfred to access or modify files from other apps, allow access to Chrome's data. The wording varies by macOS version.
3. Quit and reopen Alfred, then try `cr` again.

This can affect Alfred even when the helper works from Terminal, because macOS grants file access separately to each application. If the problem persists, confirm Chrome has an existing profile and `ALFRED_CHROME_NAME` identifies the intended browser.

## Uninstallation

Remove the workflow via Alfred settings.

## Specifying a different flavor of Chrome

If you use Canary version of Chrome or similar applications, like Chromium, you can instruct the workflow to use it.

To do it:

- In the Workflows settings, select Alfred Chrome workflow
- Open the `Configure workflow and variables` by clicking on the second icon on right section of the workflow header.
- Under the `Workflow Environment Variables` section, create a variable called `ALFRED_CHROME_NAME` and enter the same name you see in your Applications folder.
- Click on `Save` and you're good to go! :)

## Copyright

Copyright (C) 2013 and above Shogun (shogun@cowtech.it).

Licensed under the MIT license, which can be found at https://choosealicense.com/licenses/mit.
