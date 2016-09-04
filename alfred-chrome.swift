import Foundation
import AppKit
import Regex

// First of all, gather the query and check if Chrome Canary is requested
let query = Process.arguments.dropFirst().joinWithSeparator(" ")
let useCanary = NSProcessInfo.processInfo().environment["alfred_chrome_canary"] == "YES"
let name = "Chrome\(useCanary ? " Canary" : "")"

// Check if the browser is installed, otherwise nothing to do
let application = NSWorkspace.sharedWorkspace().absolutePathForAppBundleWithIdentifier("com.google.Chrome\(useCanary ? ".canary" : "")")

if application == nil {
  print("Google \(name) not installed. Doing nothing!")
  exit(0)
}

// Now scan the list of valid profiles for the browser
let fsManager = NSFileManager.defaultManager()
let base = "\(fsManager.URLsForDirectory(.ApplicationSupportDirectory, inDomains: .UserDomainMask)[0].path!)/Google/\(name)"

let profiles = try fsManager.contentsOfDirectoryAtPath(base)
  .filter({ $0 != "System Profile" && $0 != "Guest Profile" && fsManager.fileExistsAtPath(base + "/\($0)/Preferences") })
  .reduce([String: String]()) { accu, entry in
    guard 
      let preferences = NSData(contentsOfFile: base + "/\(entry)/Preferences"), 
      let profileInfo = try? NSJSONSerialization.JSONObjectWithData(preferences, options: NSJSONReadingOptions()) as! NSDictionary, 
      let profile = profileInfo["profile"]!["name"] as? String
    else {
      return accu
    }

    var newAccu = accu // Needed since accu is let and var usage is deprecated in Swift 3
    newAccu[profile] = entry
    return newAccu
  }

// Parse the query
guard 
  let matcher = try? Regex(pattern: "^(as\\s+(.+?)\\s+)?((in\\s+)?incognito\\s+)?(\\S+)$"), 
  let match = matcher.findFirst(in: query) 
else { 
  exit(0) 
}

// Prepare executable path and its arguments
let executable = NSBundle(path: application!)!.executablePath!
var arguments = [match.group(at: 5)!]  

if let user = match.group(at: 2), let path = profiles[user] {
  arguments.append("--profile-directory=\(path)")
}

if match.group(at: 3) != nil {
  arguments.append("--incognito")
}

// Finally Launch the browser
print("Launching: \(executable) \(arguments.joinWithSeparator(" "))")
NSTask.launchedTaskWithLaunchPath(executable, arguments: arguments)