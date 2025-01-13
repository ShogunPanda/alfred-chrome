use std::os::unix::process::CommandExt;
use std::{collections::HashMap, env, process::exit, process::Command};

use fork::{daemon, setsid};
use glob::glob;
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct RawChromeProfileInternal {
  name: String,
}

#[derive(Debug, Deserialize)]
struct RawChromeProfile {
  profile: RawChromeProfileInternal,
}

#[derive(Debug, Deserialize)]
struct Chrome {
  #[serde(rename(deserialize = "CFBundleIdentifier"))]
  id: String,
  #[serde(rename(deserialize = "CFBundleDisplayName"))]
  name: String,
  #[serde(rename(deserialize = "CFBundleExecutable"))]
  executable: String,
  #[serde(rename(deserialize = "CrProductDirName"), default = "default_product_dir_name")]
  product_directory: String,
  #[serde(skip)]
  profiles_root: String,
}

#[derive(Debug, Serialize)]
struct Action {
  #[serde(skip_serializing_if = "Option::is_none")]
  uid: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  title: Option<String>,
  subtitle: String,
  arg: String,
  mods: HashMap<String, Action>,
}

#[derive(Debug, Serialize)]
struct Response {
  items: Vec<Action>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Payload {
  executable: String,
  profile: String,
  url: String,
  incognito: bool,
}

const OPEN_FLAG: &str = "--alfred-chrome-execute";

fn default_product_dir_name() -> String { String::from("") }

fn md5(content: &str) -> String {
  let mut hasher = Md5::new();
  hasher.update(content);
  format!("{:x}", hasher.finalize())
}

fn parse_chrome_information(path: &str) -> Result<Chrome, String> {
  let deserialized: Result<Chrome, plist::Error> =
    plist::from_file(format!("/Applications/{}.app/Contents/Info.plist", path));

  match deserialized {
    Ok(application) => {
      let mut application = application;
      let user_data = "~/Library/Application Support/".replace('~', &std::env::var("HOME").unwrap());

      application.executable = format!("/Applications/{}.app/Contents/MacOS/{}", path, application.executable);

      application.profiles_root = if application.id == "org.chromium.Chromium" {
        user_data + "Chromium"
      } else if !application.product_directory.is_empty() {
        user_data + &application.product_directory
      } else {
        user_data + "Google/Chrome"
      };

      Ok(application)
    }
    Err(e) => {
      eprintln!("{:?}", e);

      if e.is_io() {
        Err(format!("{} cannot be found. Nothing to do! :(", path))
      } else {
        Err(format!("{} informations cannot be read. Nothing to do! :(", path))
      }
    }
  }
}

fn list_chrome_profiles(application: &Chrome) -> Vec<(String, String)> {
  let mut profiles: Vec<(String, String)> = vec![];

  let profiles_root = format!("{}/*/Preferences", application.profiles_root);

  if let Ok(entries) = glob(&profiles_root) {
    for entry in entries {
      if entry.is_err() {
        continue;
      }

      let full_path = entry.unwrap().into_boxed_path();
      let profile_path = full_path.parent().unwrap().file_name().unwrap();

      if profile_path == "System Profile" || profile_path == "Guest Profile" {
        continue;
      }

      if let Ok(profile_content) = std::fs::read_to_string(full_path.as_os_str()) {
        if let Ok(profile) = serde_json::from_str::<RawChromeProfile>(&profile_content) {
          profiles.push((profile.profile.name, String::from(profile_path.to_str().unwrap())));
        }
      }
    }
  }

  profiles
}

fn generate_alfred_actions(
  application: &Chrome,
  profiles: &[(String, String)],
  url: &str,
  display_url: &str,
  display_type: &str,
) -> Vec<Action> {
  let mut actions: Vec<Action> = vec![];

  let helper = env::args().next().unwrap();

  for profile in profiles {
    let uid = format!("chrome-{}-{}", md5(url), md5(profile.1.as_str()));
    let title = format!("Open {} using profile {}", display_type, profile.0);

    let regular_subtitle = format!(
      "Open {} in {} using profile {}",
      display_url, application.name, profile.0
    );
    let incognito_subtitle = format!("{} (in incognito)", regular_subtitle);

    let regular_payload = serde_json::to_string(&Payload {
      executable: application.executable.clone(),
      profile: profile.1.clone(),
      url: url.to_string(),
      incognito: false,
    })
    .unwrap();

    let incognito_payload = serde_json::to_string(&Payload {
      executable: application.executable.clone(),
      profile: profile.1.clone(),
      url: url.to_string(),
      incognito: true,
    })
    .unwrap();

    actions.push(Action {
      uid: Some(uid),
      title: Some(title),
      subtitle: regular_subtitle,
      arg: format!("'{}' {} '{}'", helper, OPEN_FLAG, regular_payload),
      mods: HashMap::from([(
        String::from("alt"),
        Action {
          uid: None,
          title: None,
          subtitle: incognito_subtitle,
          arg: format!("'{}' {} '{}'", helper, OPEN_FLAG, incognito_payload),
          mods: HashMap::new(),
        },
      )]),
    });
  }

  actions
}

// TODO@PI: Open Chrome directly and try to use setsid to detach the process
fn spawn_chrome() {
  if daemon(false, false).is_err() {
    eprintln!("Failed to daemonize Google Chrome.");
    exit(1);
  }

  if setsid().is_err() {
    eprintln!("Failed to detach Google Chrome.");
    exit(1);
  }

  let raw_payload = env::args().skip(2).collect::<Vec<String>>().join(" ");
  let payload: Payload = serde_json::from_str(&raw_payload).unwrap();

  let mut args: Vec<&str> = vec![];

  if !payload.url.is_empty() {
    args.push(&payload.url);
  }

  let profile = format!("--profile-directory={}", payload.profile);
  args.push(&profile);

  if payload.incognito {
    args.push("--incognito");
  }

  Command::new(payload.executable).args(args).exec();
}

fn main() {
  if env::args().nth(1).unwrap_or_else(|| String::from("")) == OPEN_FLAG {
    spawn_chrome();
    return;
  }

  // Gather informations about URL
  let url = String::from(env::args().collect::<Vec<String>>()[1..].join(" ").trim());
  let mut display_url = "a new window";
  let mut display_type = "new window";

  if !url.is_empty() {
    display_url = &url;
    display_type = "URL";
  }

  // Find which Chrom* version we want to use
  let application_path = std::env::var("ALFRED_CHROME_NAME").unwrap_or_else(|_| String::from("Google Chrome"));

  // Parse the plist file in order to find the executable and the application
  // name, then find the application support folder
  let application = parse_chrome_information(application_path.trim()).unwrap_or_else(|e| {
    eprintln!("{}", e);
    exit(1);
  });

  // Scan Chrome/Chromium folder and check for the profiles
  let profiles = list_chrome_profiles(&application);

  if profiles.is_empty() {
    eprintln!("Not suitable profiles found. Nothing to do! :(");
    exit(1);
  }

  // Return the output
  let actions = generate_alfred_actions(&application, &profiles, url.as_str(), display_url, display_type);

  println!(
    "{}",
    serde_json::to_string_pretty(&Response { items: actions }).unwrap()
  );
}
