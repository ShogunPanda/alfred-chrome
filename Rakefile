#/usr/bin/env ruby
#
# This file is part of alfred-chrome. Copyright (C) 2016 and above Shogun <shogun@cowtech.it>.
# Licensed under the MIT license, which can be found at https://choosealicense.com/licenses/mit.
#

require "json"

desc "Build the main executable."
task :build do
  # Compile the executable
  system("go build -ldflags='-s -w'")
  system("upx alfred-chrome") unless ENV["DEBUG"]
end

desc "Cleans the build directories."
task :clean do
  FileUtils.rm_rf(["alfred-chrome"], verbose: true)
end

desc "Install the executable in the workflow directory for distribution."
task :install => :build do
  # Detecting the right workflow
  workflow = Dir.glob(File.expand_path("~/Library/Application\ Support/Alfred\ 3/Alfred.alfredpreferences/workflows/*/info.plist")).find { |plist|
    begin
      JSON.parse(`plutil -convert json -o - "#{plist}"`)["bundleid"] == "it.cowtech.alfred.chrome"
    rescue
    end
  }

  FileUtils.cp("alfred-chrome", File.dirname(workflow), verbose: true)
end

desc "Verifies the code."
task :lint do
  Kernel.exec("go vet")
end

task default: ["build"]