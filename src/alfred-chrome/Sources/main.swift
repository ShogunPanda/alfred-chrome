/*
 * This file is part of alfred-chrome. Copyright (C) 2016 and above Shogun <shogun@cowtech.it>.
 * Licensed under the MIT license, which can be found at http://www.opensource.org/licenses/mit-license.php.
 */

import Foundation
import AppKit.NSWorkspace
let fsManager = FileManager()

// Gather informations about URL and app
let url = CommandLine.arguments.dropFirst().joined(separator: " ")
let name = fsManager.fileExists(atPath: "./application") ?
    String(data: fsManager.contents(atPath: "./application")!, encoding: String.Encoding.utf8)!.trimmingCharacters(in: CharacterSet.whitespacesAndNewlines) :
    "Google Chrome"

// Check if the browser is installed, otherwise nothing to do
guard
  let application = NSWorkspace.shared().fullPath(forApplication: name), let executable = Bundle(path: application)!.executablePath,
  let data = fsManager.contents(atPath: "\(application)/Contents/Info.plist"),
  let plist = try PropertyListSerialization.propertyList(from: data, options: [], format: nil) as? [String: Any],
  let supportDirectory = plist["CrProductDirName"]
else {
  print("\(name) cannot be found.\nNothing to do! :(\n")
  exit(1)
}

// Now scan the list of valid profiles for the browser
let base = "\(fsManager.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0].path)/\(supportDirectory)"
let profiles = try fsManager.contentsOfDirectory(atPath: base)
  .filter({ $0 != "System Profile" && $0 != "Guest Profile" && fsManager.fileExists(atPath: base + "/\($0)/Preferences") })
  .reduce([String: String]()) { accu, entry in
    guard
      let data = fsManager.contents(atPath: "\(base)/\(entry)/Preferences"),
      let parsed = try JSONSerialization.jsonObject(with: data, options: JSONSerialization.ReadingOptions()) as? [String: Any],
      let profile = (parsed["profile"] as! [String: Any])["name"] as? String
    else {
      return accu
    }

    var newAccu = accu // Needed since accu is let and var usage is deprecated in Swift 3
    newAccu[profile] = entry
    return newAccu
  }

let items: [[String: Any]] = profiles.keys.map { key in
  let profile = profiles[key]!
  let displayUrl = url.characters.count > 0 ? url : "a new window"
  let title = "Open \(url.characters.count > 0 ? "URL" : "a new window") using profile \(key)"

  return [
    "uid": "chrome-\(name.lowercased().replacingOccurrences(of: " ", with: "-"))-profile-\(key.lowercased().replacingOccurrences(of: " ", with: "-"))",
    "title": title,
    "subtitle": "Open \(displayUrl) in \(name) using profile \(key) ...",
    "arg": "\"\(executable)\" \(url) --profile-directory=\"\(profile)\"",
    "mods": [
      "alt": [
        "title": "\(title) (in incognito)",
        "subtitle": "Open \(displayUrl) in \(name) (in incognito) using profile \(key) ...",
        "arg": "\"\(executable)\" \(url) --profile-directory=\"\(profile)\" --incognito",
      ]
    ]
  ]
}

let results = try JSONSerialization.data(withJSONObject: ["items": items], options: [])
print(String(data: results, encoding: String.Encoding.utf8)!)
