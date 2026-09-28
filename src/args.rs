use crate::output::Error;

/// Arguments after the command name. Every option except `--json` takes a
/// value, written `--flag value` or `--flag=value`; values beginning with `--`
/// must use the `=` form. When an option repeats, the last value wins.
pub struct Args<'a> {
    positionals: Vec<&'a str>,
    options: Vec<(&'a str, Option<&'a str>)>,
    json: bool,
}
impl<'a> Args<'a> {
    pub fn parse(argv: &'a [String]) -> Self {
        let mut args = Self {
            positionals: Vec::new(),
            options: Vec::new(),
            json: false,
        };
        let mut argv = argv.iter().map(String::as_str).peekable();
        while let Some(arg) = argv.next() {
            if arg == "--json" {
                args.json = true;
            } else if !arg.starts_with('-') || arg == "-" {
                args.positionals.push(arg);
            } else if let Some((name, value)) = arg.split_once('=') {
                args.options.push((name, Some(value)));
            } else {
                let value = argv
                    .next_if(|value| !value.starts_with("--") && !matches!(*value, "-h" | "-v"));
                args.options.push((arg, value));
            }
        }
        args
    }
    /// Reject typos, missing values, and stray arguments before falling back to
    /// a default environment or sending a request.
    pub fn validate(
        &self,
        command: &str,
        allowed: impl Fn(&str) -> bool,
        max_positionals: usize,
    ) -> Result<(), Error> {
        for (name, value) in &self.options {
            if *name != "--env" && !allowed(name) {
                return Err(format!(
                    "Unknown option: {name}. Run 'goodissues help {command}' for options."
                )
                .into());
            }
            if value.is_none() {
                return Err(format!("{name} requires a value.").into());
            }
        }
        if let Some(extra) = self.positionals.get(max_positionals) {
            return Err(format!(
                "Unexpected argument: {extra}. Run 'goodissues help {command}' for usage."
            )
            .into());
        }
        Ok(())
    }
    pub fn flag(&self, name: &str) -> Option<&'a str> {
        self.options
            .iter()
            .rev()
            .find(|(option, _)| *option == name)
            .and_then(|(_, value)| *value)
    }
    /// A flag's value, treating an empty value as absent.
    pub fn nonempty(&self, name: &str) -> Option<&'a str> {
        self.flag(name).filter(|value| !value.is_empty())
    }
    pub fn required(&self, name: &str) -> Result<&'a str, Error> {
        self.nonempty(name)
            .ok_or_else(|| format!("{name} is required.").into())
    }
    pub fn positional(&self, index: usize) -> Option<&'a str> {
        self.positionals.get(index).copied()
    }
    pub fn json(&self) -> bool {
        self.json
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(args: &[&str]) -> Vec<String> {
        args.iter().map(|&arg| arg.into()).collect()
    }

    #[test]
    fn values_positionals_and_json_are_separated() {
        let argv = strings(&["get", "--env", "qa", "id", "--json", "--page=-1", "-"]);
        let args = Args::parse(&argv);
        assert_eq!(args.positional(0), Some("get"));
        assert_eq!(args.positional(1), Some("id"));
        assert_eq!(args.positional(2), Some("-"));
        assert_eq!(args.flag("--env"), Some("qa"));
        assert_eq!(args.flag("--page"), Some("-1"));
        assert!(args.json());
    }

    #[test]
    fn the_last_repeated_option_wins() {
        let argv = strings(&["--status", "new", "--status=archived"]);
        assert_eq!(Args::parse(&argv).flag("--status"), Some("archived"));
    }

    #[test]
    fn empty_values_do_not_satisfy_required_flags() {
        let argv = strings(&["--name="]);
        let args = Args::parse(&argv);
        assert_eq!(args.flag("--name"), Some(""));
        assert!(args.required("--name").is_err());
    }

    #[test]
    fn validation_reports_unknown_options_before_missing_values() {
        let argv = strings(&["--typo"]);
        let Err(Error::Message(message)) = Args::parse(&argv).validate("x", |_| false, 1) else {
            panic!("expected an error");
        };
        assert!(message.starts_with("Unknown option: --typo"));
        let argv = strings(&["--status", "--json"]);
        let Err(Error::Message(message)) = Args::parse(&argv).validate("x", |_| true, 1) else {
            panic!("expected an error");
        };
        assert_eq!(message, "--status requires a value.");
    }

    #[test]
    fn validation_rejects_extra_positionals() {
        let argv = strings(&["get", "a", "b"]);
        let args = Args::parse(&argv);
        assert!(args.validate("x", |_| true, 2).is_err());
        assert!(args.validate("x", |_| true, 3).is_ok());
    }
}
