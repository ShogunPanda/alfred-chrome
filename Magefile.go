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
	"path"
	"path/filepath"
	"strings"

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

func step(message string, args ...interface{}) {
	fmt.Printf("\x1b[33m--- %s\x1b[0m\n", fmt.Sprintf(message, args...))
}

func execute(env map[string]string, args ...string) error {
	step("Executing: %s ...", strings.Join(args, " "))

	_, err := sh.Exec(env, os.Stdout, os.Stderr, args[0], args[1:]...)

	return err
}

// Build the executable.
func Build() error {
	mg.Deps(Clean)

	cmds := [][]string{
		[]string{"go", "build", "-o", target, "-ldflags", "-s -w"},
		[]string{"upx", target},
	}

	for _, cmd := range cmds {
		execute(nil, cmd...)
	}

	return nil
}

func Install() error {
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
			fmt.Println(err)
			continue
		}

		if _, err := plist.Unmarshal(rawProperties, &properties); err == nil {
			if properties.ID == workflowId {
				chromeWorkflow = workflow
			}
		}
	}

	return execute(nil, "cp", target, path.Dir(chromeWorkflow))
}

// Removes the executable.
func Clean() {
	step("Removing file %s ...", target)
	os.Remove(target)
}

// Verifies the code.
func Lint() error {
	return execute(nil, "go", "vet")
}

var Default = Build
