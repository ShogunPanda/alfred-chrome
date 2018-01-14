/*
 * This file is part of alfred-chrome. Copyright (C) 2016 and above Shogun <shogun@cowtech.it>.
 * Licensed under the MIT license, which can be found at http://www.opensource.org/licenses/mit-license.php.
 */

package main

import (
	"crypto/md5"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"os"
	"path"
	"path/filepath"
	"strings"

	"howett.net/plist"
)

// RawChromeProfile represents a Chrom* parsed profile
type RawChromeProfile struct {
	Profile struct {
		Name string
	}
}

// Application represents a OSX Bundle information
type Application struct {
	ID           string `plist:"CFBundleIdentifier"`
	Name         string `plist:"CFBundleDisplayName"`
	Executable   string `plist:"CFBundleExecutable"`
	ProductDir   string `plist:"CrProductDirName"`
	ProfilesRoot string
}

// ChromeProfile represents a Chrom* profile
type ChromeProfile struct {
	Name string
	Path string
}

// AlfredAction is a Alfred action
type AlfredAction struct {
	UID      string                  `json:"uid,omitempty"`
	Title    string                  `json:"title,omitempty"`
	Subtitle string                  `json:"subtitle"`
	Arg      string                  `json:"arg"`
	Mods     map[string]AlfredAction `json:"mods,omitempty"`
}

// AlfredResponse is a Alfred response
type AlfredResponse struct {
	Items []AlfredAction `json:"items"`
}

func resolveChromeProfilesRoot(application Application) string {
	userData := strings.Replace("~/Library/Application Support/", "~", os.Getenv("HOME"), -1)

	if application.ID == "org.chromium.Chromium" {
		userData += "Chromium"
	} else if application.ProductDir != "" {
		userData += application.ProductDir
	} else {
		userData += "Google/Chrome"
	}

	return userData
}

func parseChromeInformation(applicationPath string) Application {
	var application Application
	rawApplication, err := ioutil.ReadFile(fmt.Sprintf("/Applications/%s.app/Contents/Info.plist", applicationPath))

	if err != nil {
		fmt.Fprintf(os.Stderr, "%s cannot be found. Nothing to do! :(\n", applicationPath)
		os.Exit(1)
	}

	if _, err := plist.Unmarshal(rawApplication, &application); err != nil {
		fmt.Println(err)
		fmt.Fprintf(os.Stderr, "%s informations cannot be read. Nothing to do! :(\n", applicationPath)
		os.Exit(1)
	} else {
		application.Executable = fmt.Sprintf("/Applications/%s.app/Contents/MacOS/%s", applicationPath, application.Executable)
	}

	application.ProfilesRoot = resolveChromeProfilesRoot(application)
	return application
}

func listChromeProfiles(application Application) []ChromeProfile {
	var profiles []ChromeProfile

	profilesPaths, _ := filepath.Glob(fmt.Sprintf("%s/*/Preferences", application.ProfilesRoot))
	for _, profileFullPath := range profilesPaths {
		profilePath := path.Base(path.Dir(profileFullPath))

		if profilePath == "System Profile" || profilePath == "Guest Profile" {
			continue
		}

		rawProfile, err := ioutil.ReadFile(profileFullPath)

		if err != nil {
			continue
		}

		var parsedProfile RawChromeProfile
		err = json.Unmarshal(rawProfile, &parsedProfile)

		// Filter out System Profile and Guest Profile
		profiles = append(profiles, ChromeProfile{parsedProfile.Profile.Name, profilePath})
	}

	return profiles
}

func generateAlfredActions(url, displayURL, displayType string, application Application, profiles []ChromeProfile) []AlfredAction {
	var actions []AlfredAction

	for _, profile := range profiles {
		uid := fmt.Sprintf("chrome-%x-%x", md5.Sum([]byte(url)), md5.Sum([]byte(profile.Path)))

		title := fmt.Sprintf("Open %s using profile %s", displayType, profile.Name)
		subtitle := fmt.Sprintf("Open %s in %s using profile %s", displayURL, application.Name, profile.Name)
		arg := fmt.Sprintf("\"%s\" %s --profile-directory=\"%s\"", application.Executable, url, profile.Path)

		incognitoSubtitle := fmt.Sprintf("%s (in incognito)", subtitle)
		incognitoArg := fmt.Sprintf("%s --incognito", arg)

		incognito := AlfredAction{Subtitle: incognitoSubtitle, Arg: incognitoArg}
		actions = append(actions, AlfredAction{UID: uid, Title: title, Subtitle: subtitle, Arg: arg, Mods: map[string]AlfredAction{"alt": incognito}})
	}

	return actions
}

func main() {
	// Gather informations about URL
	url := strings.TrimSpace(strings.Join(os.Args[1:], " "))
	displayURL := "a new window"
	displayType := "new window"

	if len(url) > 0 {
		displayURL = url
		displayType = "URL"
	}

	// Find which Chrom* version we want to use
	applicationPath, applicationSet := os.LookupEnv("ALFRED_CHROME_NAME")
	if !applicationSet {
		applicationPath = "Google Chrome"
	} else {
		applicationPath = strings.TrimSpace(string(applicationPath))
	}

	// Parse the plist file in order to find the executable and the application name, then find the application support folder
	application := parseChromeInformation(applicationPath)

	// Scan Chrome/Chromium folder and check for the profiles
	profiles := listChromeProfiles(application)

	if len(profiles) == 0 {
		fmt.Fprintln(os.Stderr, "Not suitable profiles found. Nothing to do! :(")
		os.Exit(1)
	}

	// Return the output
	actions := generateAlfredActions(url, displayURL, displayType, application, profiles)

	json, _ := json.Marshal(AlfredResponse{Items: actions})
	fmt.Println(string(json))
}
