use std::{collections::HashMap, env, path::Path, process::exit, process::Command};

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

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ProfileMetadata {
  name: String,
  gaia_given_name: String,
  gaia_name: String,
  enterprise_label: String,
  is_using_default_name: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ProfileCache {
  info_cache: HashMap<String, ProfileMetadata>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct LocalState {
  profile: ProfileCache,
}

#[derive(Debug, Deserialize)]
struct Chrome {
  #[serde(rename(deserialize = "CFBundleIdentifier"))]
  id: String,
  #[serde(rename(deserialize = "CFBundleDisplayName"))]
  name: String,
  #[serde(skip)]
  application_path: String,
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
  application_path: String,
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

      application.application_path = format!("/Applications/{}.app", path);

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

fn profile_name(metadata: &ProfileMetadata, fallback: &str) -> String {
  let local_name = if !metadata.enterprise_label.is_empty() {
    &metadata.enterprise_label
  } else if !metadata.name.is_empty() {
    &metadata.name
  } else {
    fallback
  };
  let account_name = if metadata.gaia_given_name.is_empty() {
    &metadata.gaia_name
  } else {
    &metadata.gaia_given_name
  };

  // Chrome replaces default names with the account name, but preserves custom
  // labels.
  if account_name.is_empty() || account_name.eq_ignore_ascii_case(local_name) {
    local_name.to_string()
  } else if metadata.is_using_default_name && metadata.enterprise_label.is_empty() {
    account_name.clone()
  } else {
    format!("{} ({})", account_name, local_name)
  }
}

fn list_chrome_profiles(application: &Chrome) -> Vec<(String, String)> {
  let mut profiles: Vec<(String, String)> = vec![];

  let root = Path::new(&application.profiles_root);
  let state = std::fs::read_to_string(root.join("Local State"))
    .ok()
    .and_then(|content| serde_json::from_str::<LocalState>(&content).ok())
    .unwrap_or_default();
  let mut names: HashMap<String, String> = HashMap::new();
  let profiles_root = format!("{}/*/Preferences", glob::Pattern::escape(&application.profiles_root));

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
          names.insert(profile_path.to_string_lossy().into_owned(), profile.profile.name);
        }
      }
    }
  }

  // Local State can describe a profile even when its Preferences file cannot be
  // read.
  for directory in state.profile.info_cache.keys() {
    if Path::new(directory).components().count() == 1
      && !Path::new(directory).is_absolute()
      && directory != "."
      && directory != ".."
      && root.join(directory).is_dir()
    {
      names.entry(directory.clone()).or_insert_with(|| directory.clone());
    }
  }

  for (directory, fallback) in names {
    if directory == "System Profile" || directory == "Guest Profile" {
      continue;
    }
    let name = state
      .profile
      .info_cache
      .get(&directory)
      .map_or_else(|| fallback.clone(), |metadata| profile_name(metadata, &fallback));
    profiles.push((if name.is_empty() { directory.clone() } else { name }, directory));
  }

  let mut counts = HashMap::new();
  for (name, _) in &profiles {
    *counts.entry(name.clone()).or_insert(0) += 1;
  }
  for (name, directory) in &mut profiles {
    if counts[name] > 1 {
      *name = format!("{} ({})", name, directory);
    }
  }
  profiles.sort_by(|a, b| a.1.cmp(&b.1));
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

  for profile in profiles {
    let uid = format!("chrome-{}-{}", md5(url), md5(profile.1.as_str()));
    let title = format!("Open {} using profile {}", display_type, profile.0);

    let regular_subtitle = format!(
      "Open {} in {} using profile {}",
      display_url, application.name, profile.0
    );
    let incognito_subtitle = format!("{} (in incognito)", regular_subtitle);

    let regular_payload = serde_json::to_string(&Payload {
      application_path: application.application_path.clone(),
      profile: profile.1.clone(),
      url: url.to_string(),
      incognito: false,
    })
    .unwrap();

    let incognito_payload = serde_json::to_string(&Payload {
      application_path: application.application_path.clone(),
      profile: profile.1.clone(),
      url: url.to_string(),
      incognito: true,
    })
    .unwrap();

    actions.push(Action {
      uid: Some(uid),
      title: Some(title),
      subtitle: regular_subtitle,
      arg: regular_payload,
      mods: HashMap::from([(
        String::from("alt"),
        Action {
          uid: None,
          title: None,
          subtitle: incognito_subtitle,
          arg: incognito_payload,
          mods: HashMap::new(),
        },
      )]),
    });
  }

  actions
}

fn chrome_command(payload: &Payload) -> Command {
  let mut command = Command::new("/usr/bin/open");
  // Launch Services assigns Chrome its own macOS application identity. A new
  // launch delivers arguments even when Chrome is running; Chrome forwards them
  // to its existing instance using its own singleton mechanism.
  command.args(["-n", "-a", &payload.application_path, "--args"]);
  command.arg(format!("--profile-directory={}", payload.profile));
  if payload.incognito {
    command.arg("--incognito");
  }
  if payload.url.is_empty() {
    command.arg("--new-window");
  } else {
    command.args(["--", &payload.url]);
  }
  command
}

fn spawn_chrome() -> Result<(), String> {
  let raw_payload = env::args().nth(2).ok_or("Missing Chrome launch payload.")?;
  let payload: Payload =
    serde_json::from_str(&raw_payload).map_err(|error| format!("Invalid Chrome launch payload: {}", error))?;
  let status = chrome_command(&payload)
    .status()
    .map_err(|error| format!("Failed to launch Chrome: {}", error))?;
  if !status.success() {
    return Err(format!("Failed to launch Chrome: {}", status));
  }
  Ok(())
}

fn main() {
  if env::args().nth(1).unwrap_or_else(|| String::from("")) == OPEN_FLAG {
    if let Err(error) = spawn_chrome() {
      eprintln!("{}", error);
      exit(1);
    }
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

  // Read the application metadata and locate its profile storage.
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
