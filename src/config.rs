use crate::{args::Args, output::Error};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, BufRead, IsTerminal, Write},
    path::{Path, PathBuf},
};

pub const DEFAULT_URL: &str = "http://localhost:4000";
const FILE: &str = ".goodissues.json";
const LEGACY_FILE: &str = ".goodissues/config.yaml";

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
fn home() -> Result<PathBuf, Error> {
    let value = if cfg!(windows) {
        std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"))
    } else {
        std::env::var_os("HOME")
    };
    value
        .map(PathBuf::from)
        .ok_or_else(|| "home directory is not set.".into())
}
/// The `base_url` and `api_key` of a legacy YAML config. Only flat
/// `key: value` lines are supported, optionally quoted.
fn parse_legacy(text: &str) -> (Option<&str>, Option<&str>) {
    let (mut url, mut key) = (None, None);
    for line in text.lines().map(str::trim) {
        if line.starts_with('#') {
            continue;
        }
        let Some((name, raw)) = line.split_once(':') else {
            continue;
        };
        let raw = raw.trim();
        let value = ['\'', '"']
            .iter()
            .find_map(|&q| raw.strip_prefix(q)?.strip_suffix(q))
            .unwrap_or(raw);
        if value.is_empty() {
            continue;
        }
        match name.trim() {
            "base_url" => url = Some(value),
            "api_key" => key = Some(value),
            _ => {}
        }
    }
    (url, key)
}
impl Config {
    pub fn load() -> Result<Self, Error> {
        Self::load_at(&home()?)
    }
    fn load_at(home: &Path) -> Result<Self, Error> {
        let path = home.join(FILE);
        match fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map_err(|e| format!("could not parse {}: {e}", path.display()).into()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Self::import_legacy(home),
            Err(e) => Err(format!("could not read {}: {e}", path.display()).into()),
        }
    }
    /// One-time migration: when only the legacy YAML config exists, convert it
    /// and save it as JSON so later runs, and later edits, use the JSON file.
    fn import_legacy(home: &Path) -> Result<Self, Error> {
        let path = home.join(LEGACY_FILE);
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(format!("could not read {}: {e}", path.display()).into()),
        };
        let (url, key) = parse_legacy(&text);
        let mut config = Self::default();
        config.set(
            "default",
            Some(url.unwrap_or(DEFAULT_URL).into()),
            key.map(Into::into),
        );
        config.save_at(home)?;
        Ok(config)
    }
    fn save_at(&self, home: &Path) -> Result<(), Error> {
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        let path = home.join(FILE);
        let failed = |e: io::Error| format!("could not save {}: {e}", path.display());
        // A same-directory temporary file keeps replacement on one filesystem.
        // NamedTempFile creates private files and removes them on failure.
        let mut file = tempfile::NamedTempFile::new_in(home).map_err(failed)?;
        file.write_all(&bytes).map_err(failed)?;
        file.as_file().sync_all().map_err(failed)?;
        file.persist(&path).map_err(|e| failed(e.error))?;
        #[cfg(unix)]
        fs::File::open(home)
            .and_then(|dir| dir.sync_all())
            .map_err(failed)?;
        Ok(())
    }

    pub fn get(&self, name: Option<&str>) -> Option<&Environment> {
        let name = name.or(self.default_env.as_deref())?;
        self.environments
            .as_ref()?
            .iter()
            .find(|env| env.name == name)
    }
    fn set(&mut self, name: &str, url: Option<String>, key: Option<String>) {
        let url = url.map(|url| url.trim_end_matches('/').to_owned());
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
fn show(env: &Environment) -> Result<(), Error> {
    crate::output::write(format!(
        "Environment: {}\n  URL:     {}\n  API Key: {}\n",
        env.name,
        env.base_url.as_deref().unwrap_or("(not set)"),
        env.api_key
            .as_deref()
            .map_or_else(|| "(not set)".into(), mask)
    ))
}
fn mask(key: &str) -> String {
    let chars: Vec<char> = key.chars().collect();
    if chars.len() <= 8 {
        return "****".into();
    }
    let first: String = chars[..4].iter().collect();
    let last: String = chars[chars.len() - 4..].iter().collect();
    format!("{first}****{last}")
}
/// The first nonempty value in precedence order: flag, prompt, stored, fallback.
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
fn prompt(label: &str) -> Result<(), Error> {
    crate::output::write(label)
}
/// Read the API key without echoing it when typed at a terminal.
fn read_secret(stdin: &mut impl BufRead) -> Result<String, Error> {
    if io::stdin().is_terminal() {
        return Ok(rpassword::read_password()?);
    }
    let mut line = String::new();
    stdin.read_line(&mut line)?;
    Ok(line)
}
fn show_all(args: &Args<'_>) -> Result<(), Error> {
    let config = Config::load()?;
    if let Some(name) = args.flag("--env") {
        let env = config
            .get(Some(name))
            .ok_or_else(|| format!("no environment '{name}' found."))?;
        return show(env);
    }
    crate::output::write(format!(
        "Default: {}\n\n",
        config.default_env.as_deref().unwrap_or("(none)")
    ))?;
    let Some(envs) = &config.environments else {
        return crate::output::write("No environments configured.\n");
    };
    for env in envs {
        show(env)?;
        crate::output::write("\n")?;
    }
    Ok(())
}
pub fn run(args: &Args<'_>) -> Result<(), Error> {
    let show = match args.positional(0) {
        Some("show") => true,
        None => false,
        Some(other) => {
            return Err(format!(
                "Unknown configure subcommand: {other}. Run 'goodissues help configure' for usage."
            )
            .into());
        }
    };
    args.validate(
        "configure",
        |flag| !show && matches!(flag, "--url" | "--api-key"),
        1,
    )?;
    if show {
        return show_all(args);
    }
    let name = args.flag("--env").unwrap_or("default");
    let mut config = Config::load()?;
    let old = config.get(Some(name));
    let (mut url_input, mut key_input) = (String::new(), String::new());
    if args.flag("--url").is_none() && args.flag("--api-key").is_none() {
        let mut stdin = io::stdin().lock();
        prompt(&format!(
            "Base URL [{}]: ",
            old.and_then(|e| e.base_url.as_deref())
                .unwrap_or(DEFAULT_URL)
        ))?;
        stdin.read_line(&mut url_input)?;
        prompt("API Key: ")?;
        key_input = read_secret(&mut stdin)?;
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
    crate::output::write(format!("Configuration saved for environment '{name}'.\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_replacement_cleans_up_temporary_file() {
        let home = tempfile::tempdir().unwrap();
        let path = home.path().join(FILE);
        fs::create_dir(&path).unwrap();
        fs::write(path.join("sentinel"), "keep").unwrap();
        assert!(Config::default().save_at(home.path()).is_err());
        assert_eq!(fs::read_to_string(path.join("sentinel")).unwrap(), "keep");
        assert_eq!(fs::read_dir(home.path()).unwrap().count(), 1);
    }

    #[test]
    fn empty_existing_config_is_an_error() {
        let home = tempfile::tempdir().unwrap();
        fs::write(home.path().join(FILE), "").unwrap();
        assert!(Config::load_at(home.path()).is_err());
    }

    #[test]
    fn saving_replaces_file_without_truncating_existing_readers() {
        use std::io::Read;
        let home = tempfile::tempdir().unwrap();
        let path = home.path().join(FILE);
        let original = br#"{"default_env":"original"}"#;
        fs::write(&path, original).unwrap();
        let mut reader = fs::File::open(&path).unwrap();
        Config::default().save_at(home.path()).unwrap();
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, original);
        assert!(Config::load_at(home.path()).unwrap().default_env.is_none());
    }

    #[test]
    fn legacy_yaml_values_are_unquoted_and_comments_skipped() {
        let text =
            "# base_url: http://ignored\nbase_url: \"http://a:1\"\napi_key: 'sk_x'\nother: y\n";
        assert_eq!(parse_legacy(text), (Some("http://a:1"), Some("sk_x")));
        assert_eq!(parse_legacy("api_key: ''\n"), (None, None));
    }

    #[test]
    fn keys_are_masked_by_character() {
        assert_eq!(mask("sk_12345"), "****");
        assert_eq!(mask("sk_1234567890"), "sk_1****7890");
        assert_eq!(mask("ключ_секрет_ab"), "ключ****т_ab");
    }

    #[test]
    fn choose_prefers_nonempty_values_in_order() {
        assert_eq!(
            choose(Some(""), Some("p"), Some("s"), None).as_deref(),
            Some("p")
        );
        assert_eq!(
            choose(None, Some(""), Some("s"), None).as_deref(),
            Some("s")
        );
        assert_eq!(choose(None, None, None, Some("f")).as_deref(), Some("f"));
        assert_eq!(choose(None, None, None, None), None);
    }
}
