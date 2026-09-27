/// Match the Zig CLI's flag/value and positional conventions.
pub struct Args<'a>(pub &'a [String]);
impl<'a> Args<'a> {
    pub fn flag(&self, name: &str) -> Option<&'a str> {
        for (i, arg) in self.0.iter().enumerate() {
            if arg == name {
                if let Some(value) = self.0.get(i + 1) {
                    return Some(value);
                }
            } else if let Some(value) = arg.strip_prefix(name).and_then(|s| s.strip_prefix('=')) {
                return Some(value);
            }
        }
        None
    }
    pub fn has(&self, name: &str) -> bool {
        self.0.iter().any(|arg| arg == name)
    }
    pub fn positional(&self, index: usize) -> Option<&'a str> {
        let mut args = self.0.iter();
        let mut count = 0;
        while let Some(arg) = args.next() {
            if arg.starts_with("--") {
                if !arg.contains('=') && arg != "--json" {
                    args.next();
                }
                continue;
            }
            if arg.starts_with('-') && arg.len() > 1 {
                continue;
            }
            if count == index {
                return Some(arg);
            }
            count += 1;
        }
        None
    }
    pub fn required(&self, name: &str) -> Result<&'a str, String> {
        self.flag(name)
            .ok_or_else(|| format!("Error: {name} is required."))
    }
}

impl Args<'_> {
    /// Reject typos and missing values before falling back to a default environment
    /// or sending a request. Literal values starting with `--` use `--flag=value`.
    pub fn validate(&self, command: &str) -> Result<(), String> {
        let operation = self.positional(0).unwrap_or("list");
        let allowed = options(command, operation);
        let mut args = self.0.iter();
        while let Some(arg) = args.next() {
            if !arg.starts_with('-') || arg == "-" {
                continue;
            }
            let (name, inline) = arg
                .split_once('=')
                .map_or((arg.as_str(), None), |(name, value)| (name, Some(value)));
            if name == "--json" && inline.is_none() {
                continue;
            }
            if name != "--env" && !allowed.contains(&name) {
                return Err(format!(
                    "Error: Unknown option: {name}. Run 'goodissues help {command}' for options."
                ));
            }
            if inline.is_none() {
                match args.next() {
                    Some(value)
                        if !value.starts_with("--") && !matches!(value.as_str(), "-h" | "-v") => {}
                    _ => return Err(format!("Error: {name} requires a value.")),
                }
            }
        }
        Ok(())
    }
}

fn options(command: &str, operation: &str) -> &'static [&'static str] {
    match (command, operation) {
        ("configure", "show") => &[],
        ("configure", _) => &["--url", "--api-key"],
        ("projects", "list") => &["--query"],
        ("projects", "create" | "update") => &["--name", "--prefix", "--description", "--body"],
        ("issues", "list") => &[
            "--project",
            "--status",
            "--type",
            "--page",
            "--per-page",
            "--query",
        ],
        ("issues", "create") => &[
            "--project",
            "--title",
            "--type",
            "--status",
            "--priority",
            "--description",
            "--email",
            "--body",
        ],
        ("issues", "update") => &[
            "--title",
            "--type",
            "--status",
            "--priority",
            "--description",
            "--email",
            "--body",
        ],
        ("errors", "list") => &["--status", "--muted", "--page", "--per-page", "--query"],
        ("errors", "search") => &[
            "--module",
            "--function",
            "--file",
            "--page",
            "--per-page",
            "--query",
        ],
        ("errors", "update") => &["--status", "--muted", "--body"],
        ("errors" | "incidents", "report" | "create") | ("incidents", "update") => &["--body"],
        ("incidents", "list") | ("cloud-ip-ranges", "list") => {
            if command == "incidents" {
                &["--query"]
            } else {
                &["--snapshot-id", "--page", "--per-page", "--query"]
            }
        }
        ("checks" | "heartbeats", "list" | "results" | "pings") => &["--project", "--query"],
        ("checks" | "heartbeats", "create" | "update")
        | ("heartbeats", "ping" | "fail" | "start") => &["--project", "--body"],
        ("checks" | "heartbeats", "get" | "delete") => &["--project"],
        _ => &[],
    }
}
