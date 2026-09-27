use crate::args::Args;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
};

pub const DEFAULT_URL: &str = "http://localhost:4000";
#[derive(Default, Deserialize, Serialize)]
pub struct Config {
    pub default_env: Option<String>,
    pub environments: Option<Vec<Environment>>,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Environment {
    pub name: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
}
fn home() -> Result<PathBuf, String> {
    let value = if cfg!(windows) {
        std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"))
    } else {
        std::env::var_os("HOME")
    };
    value
        .map(PathBuf::from)
        .ok_or_else(|| "Error: home directory is not set.".into())
}
impl Config {
    pub fn load() -> Result<Self, String> {
        Self::load_at(&home()?)
    }
    fn load_at(home: &Path) -> Result<Self, String> {
        match fs::read(home.join(".goodissues.json")) {
            Ok(bytes) if bytes.is_empty() => Ok(Self::default()),
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| e.to_string()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                let text = match fs::read_to_string(home.join(".goodissues/config.yaml")) {
                    Ok(text) => text,
                    Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
                    Err(e) => return Err(e.to_string()),
                };
                let mut url = DEFAULT_URL.to_owned();
                let mut key = None;
                for line in text.lines().map(str::trim) {
                    if line.starts_with('#') {
                        continue;
                    }
                    if let Some((name, raw)) = line.split_once(':') {
                        let raw = raw.trim();
                        let value = if raw.len() >= 2
                            && ((raw.starts_with('\'') && raw.ends_with('\''))
                                || (raw.starts_with('"') && raw.ends_with('"')))
                        {
                            &raw[1..raw.len() - 1]
                        } else {
                            raw
                        };
                        if value.is_empty() {
                            continue;
                        }
                        match name.trim() {
                            "base_url" => url = value.into(),
                            "api_key" => key = Some(value.into()),
                            _ => {}
                        }
                    }
                }
                let mut config = Self::default();
                config.set("default", Some(url), key);
                config.save_at(home)?;
                Ok(config)
            }
            Err(e) => Err(e.to_string()),
        }
    }
    fn save_at(&self, home: &Path) -> Result<(), String> {
        let mut options = fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(home.join(".goodissues.json"))
            .map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(fs::Permissions::from_mode(0o600))
                .map_err(|e| e.to_string())?;
        }
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        file.write_all(&bytes).map_err(|e| e.to_string())
    }
    pub fn get(&self, name: Option<&str>) -> Option<&Environment> {
        let name = name.or(self.default_env.as_deref())?;
        self.environments
            .as_ref()?
            .iter()
            .find(|env| env.name == name)
    }
    fn set(&mut self, name: &str, url: Option<String>, key: Option<String>) {
        let envs = self.environments.get_or_insert_default();
        if let Some(env) = envs.iter_mut().find(|e| e.name == name) {
            if url.is_some() {
                env.base_url = url;
            }
            if key.is_some() {
                env.api_key = key;
            }
        } else {
            envs.push(Environment {
                name: name.into(),
                base_url: url,
                api_key: key,
            });
        }
        if self.default_env.is_none() {
            self.default_env = Some(name.into());
        }
    }
}
fn show(env: &Environment) -> Result<(), String> {
    crate::output::write(format!(
        "Environment: {}\n  URL:     {}\n  API Key: {}\n",
        env.name,
        env.base_url.as_deref().unwrap_or("(not set)"),
        env.api_key
            .as_deref()
            .map(mask)
            .unwrap_or_else(|| "(not set)".into())
    ))
}
fn mask(key: &str) -> String {
    if key.len() <= 8 {
        "****".into()
    } else {
        let first: String = key.chars().take(4).collect();
        let last: String = key
            .chars()
            .rev()
            .take(4)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        format!("{first}****{last}")
    }
}
fn choose(
    typed: Option<&str>,
    prompted: Option<&str>,
    stored: Option<&str>,
    fallback: Option<&str>,
) -> Option<String> {
    [typed, prompted, stored]
        .into_iter()
        .flatten()
        .find(|s| !s.is_empty())
        .or(fallback)
        .map(str::to_owned)
}
pub fn run(args: &Args<'_>) -> Result<(), String> {
    if args.positional(0) == Some("show") {
        let config = Config::load().map_err(|_| "Error: could not load config.")?;
        if let Some(name) = args.flag("--env") {
            show(
                config
                    .get(Some(name))
                    .ok_or_else(|| format!("No environment '{name}' found."))?,
            )?;
        } else {
            crate::output::write(format!(
                "Default: {}\n\n",
                config.default_env.as_deref().unwrap_or("(none)")
            ))?;
            if let Some(envs) = &config.environments {
                for env in envs {
                    show(env)?;
                    crate::output::write("\n")?;
                }
            } else {
                crate::output::write("No environments configured.\n")?;
            }
        }
        return Ok(());
    }
    let name = args.flag("--env").unwrap_or("default");
    let mut config = Config::load().unwrap_or_default();
    let old = config.get(Some(name));
    let mut url_input = String::new();
    let mut key_input = String::new();
    if args.flag("--url").is_none() && args.flag("--api-key").is_none() {
        crate::output::write(format!(
            "Base URL [{}]: ",
            old.and_then(|e| e.base_url.as_deref())
                .unwrap_or(DEFAULT_URL)
        ))?;
        io::stdout().flush().map_err(|e| e.to_string())?;
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        reader
            .read_line(&mut url_input)
            .map_err(|e| e.to_string())?;
        crate::output::write("API Key: ")?;
        io::stdout().flush().map_err(|e| e.to_string())?;
        reader
            .read_line(&mut key_input)
            .map_err(|e| e.to_string())?;
    }
    let url = choose(
        args.flag("--url"),
        Some(url_input.trim()),
        old.and_then(|e| e.base_url.as_deref()),
        Some(DEFAULT_URL),
    );
    let key = choose(
        args.flag("--api-key"),
        Some(key_input.trim()),
        old.and_then(|e| e.api_key.as_deref()),
        None,
    );
    config.set(name, url, key);
    config.save_at(&home()?)?;
    crate::output::write(format!("Configuration saved for environment '{name}'.\n"))?;
    Ok(())
}
