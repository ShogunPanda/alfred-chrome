#!/usr/bin/env ruby
# encoding: utf-8
#
# This file is part of the alfred-chrome workflow. Copyright (C) 2013 and above Shogun <shogun@cowtech.it>.
# Licensed under the MIT license, which can be found at http://www.opensource.org/licenses/mit-license.php.
#

require "rubygems"
require "strscan"
require "oj"

def parse_request
  scanner = StringScanner.new(ARGV.join(" ").strip)

  # Get the user
  user = scanner.scan(/as \S+\s*/i)
  user = user.gsub(/^as /, "").strip if user
  incognito = !!scanner.scan(/\s*in incognito\s*/i)
  url = scanner.rest.strip
  url = nil if url.empty?
  profiles = load_profiles
  profile = profiles[user]
  user = nil if !profile

  {:user => user, :incognito => incognito, :url => url, :profiles => profiles, :profile => profile}
end

def load_profiles
  Dir.glob(File.expand_path("~/Library/Application\ Support/Google/Chrome/**/Preferences")).reduce({}) do |accu, profile_data|
    profile = Oj.load_file(profile_data)
    folder = File.basename(File.dirname(profile_data))
    name = profile.fetch("profile", {}).fetch("name", nil)
    accu[name] = folder if name
    accu
  end
end

def chrome_running?
  `ps aux` =~ /Google Chrome/
end

def add_to_chrome(request)
  new_user = "tell application \"System Events\" to tell process \"Google Chrome\" to click menu item \"#{request[:user]}\" of menu 9 of menu bar 1\ndelay 1.5" 
  script = <<-EOSCRIPT
    tell application "Google Chrome"
      activate
      
      # Get the window mode and the original frontmost window
      set original_front to the frontmost
      set window_mode to "#{request[:incognito] ? "incognito" : "normal"}"
      set new_user to #{request[:user] ? true : false}

      # Switch the user if needed
      #{request[:user] ? new_user : nil}

      # Get the new frontmost window
      set current_front to the frontmost

      if window_mode is "incognito" then
        make new window with properties {mode: "incognito"}
        set URL of active tab of window 1 to "#{request[:url]}"
      else
        if original_front is not false and current_front is original_front then
          make new tab at the end of window 1 with properties {URL: "#{request[:url]}"}
        else
          set URL of active tab of window 1 to "#{request[:url]}"
        end if
      end if
    end tell
  EOSCRIPT

  IO.popen("osascript", "w") {|pipe| pipe.write(script) }
end

def open_chrome(request)
  args = []
  args << " --profile-directory=\"#{request[:profile]}\"" if request[:profile]
  args << " --incognito" if request[:incognito]
  args << " --new-window " + request[:url]
  `open -a "Google Chrome" --args#{args}`
end

if chrome_running? then
  add_to_chrome(parse_request)
else
  open_chrome(parse_request)
end