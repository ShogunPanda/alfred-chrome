#/usr/bin/env ruby
#
# This file is part of alfred-chrome. Copyright (C) 2016 and above Shogun <shogun@cowtech.it>.
# Licensed under the MIT license, which can be found at http://www.opensource.org/licenses/mit-license.php.
#

require "json"

desc "Build the main executable."
task :build do
  orig = Dir.pwd
  Dir.chdir("src/alfred-chrome/")
  system("swift build -c release")
  FileUtils.mv(".build/release/alfred-chrome", "../..", verbose: true)
  Dir.chdir(orig)
end

desc "Cleans the build directories."
task :clean do
  FileUtils.rm_rf(["alfred-chrome", "src/alfred-chrome/Sources/.build"], verbose: true)
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

task default: ["build"]
