# Alfred Chrome Workflow

This workflow allows you to open a URL in [Google Chrome](https://www.google.com/chrome/) via [Alfred](https://www.alfredapp.com/).

https://sw.cowtech.it/alfred-chrome

## Description

To trigger the workflow, type `chrome` or `cr`. Optionally you can add a URL.

Between the results you will a list of profiles to use to open the URL (or simply to open a new window).

If you hold the `Alt` key while selecting the result item, the window will be open using incognito mode.

# Allow the helper to execute

If you get the message `“alfred-chrome” cannot be opened because the developer cannot be verified.`, then you have to whitelist the execution of the helper:

1. Open Alfred preferences.
2. Go to workflows.
3. Right click (or Alt+Click) on the Alfred Workflow item and select "Open in Finder".
4. Inside finder right click (or Alt+Click) on `alfred-chrome` and then select "Open".
5. You should see a warning from the operating system. Just allow the execution and you're done.

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
