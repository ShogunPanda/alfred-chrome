// +build mage

/*
 * This file is part of alfred-chrome. Copyright (C) 2016 and above Shogun <shogun@cowtech.it>.
 * Licensed under the MIT license, which can be found at https://choosealicense.com/licenses/mit.
 */

package main

import (
	"fmt"
	"io/ioutil"
	"os"
	"path/filepath"
	"strings"
	"syscall"

	"github.com/magefile/mage/mg"
	"github.com/magefile/mage/sh"
	"howett.net/plist"
)

const target = "alfred-chrome"
const workflowId = "it.cowtech.alfred.chrome"
const workflows = "~/Library/Application Support/Alfred 3/Alfred.alfredpreferences/workflows/*/info.plist"

type WorkflowProperties struct {
	ID string `plist:"bundleid"`
}

// Build the executable.
func Build() error {
	mg.Deps(Clean)

	cmds := [][]string{
		[]string{"go", "build", "-o", target, "-ldflags", "-s -w"},
		[]string{"upx", target},
	}

	for _, cmd := range cmds {
		fmt.Printf("Executing: %s ...\n", strings.Join(cmd, " "))

		err := sh.Run(cmd[0], cmd[1:]...)

		if err != nil {
			return err
		}
	}

	return nil
}

func Install() error {
	mg.Deps(Build)

	// Detect the right workflow
	allWorkflows, err := filepath.Glob(strings.Replace(workflows, "~", os.Getenv("HOME"), -1))

	if err != nil {
		return err
	}

	var chromeWorkflow string
	for _, workflow := range allWorkflows {
		var properties WorkflowProperties
		rawProperties, err := ioutil.ReadFile(workflow)

		if err != nil {
			continue
		}

		if _, err := plist.Unmarshal(rawProperties, &properties); err == nil {
			if properties.ID == workflowId {
				chromeWorkflow = workflow
			}
		}
	}

	fmt.Printf("Copying file %s to %s ...\n", target, chromeWorkflow)
	return sh.Run("cp", target, chromeWorkflow)
}

// Removes the executable.
func Clean() {
	fmt.Printf("Removing file %s ...\n", target)
	os.Remove(target)
}

// Verifies the code.
func Lint() error {
	gopath, err := sh.Output("which", "go")

	if err != nil {
		return err
	}

	return syscall.Exec(gopath, []string{"", "vet"}, os.Environ())
}

var Default = Build
