use crate::args::Args;
use serde_json::{Map, Value};

pub struct Request {
    pub method: &'static str,
    pub path: String,
    pub body: Option<String>,
    pub expected: &'static [u16],
    pub operation: String,
}
fn encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || b"-_.~".contains(&byte) {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}
fn query(args: &Args<'_>, fields: &[(&str, &str)]) -> Option<String> {
    if let Some(raw) = args.flag("--query") {
        return Some(raw.into());
    }
    let parts: Vec<_> = fields
        .iter()
        .filter_map(|(flag, key)| {
            args.flag(flag)
                .map(|value| format!("{key}={}", encode(value)))
        })
        .collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("&"))
    }
}
fn muted(value: &str) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err("Error: --muted must be true or false.".into()),
    }
}
fn body(resource: &str, operation: &str, args: &Args<'_>) -> Result<String, String> {
    if let Some(raw) = args.flag("--body") {
        return Ok(raw.into());
    }
    let mut object = Map::new();
    let fields: &[(&str, &str)] = match resource {
        "projects" => &[
            ("--name", "name"),
            ("--prefix", "prefix"),
            ("--description", "description"),
        ],
        "issues" => &[
            ("--title", "title"),
            ("--description", "description"),
            ("--type", "type"),
            ("--status", "status"),
            ("--priority", "priority"),
            ("--email", "submitter_email"),
        ],
        "errors" if operation == "update" => &[("--status", "status")],
        _ => return Err("Error: --body '<json>' is required for this operation.".into()),
    };
    for (flag, key) in fields {
        if let Some(value) = args.flag(flag).filter(|v| !v.is_empty()) {
            object.insert((*key).into(), Value::String(value.into()));
        }
    }
    if resource == "errors"
        && let Some(value) = args.flag("--muted")
    {
        object.insert("muted".into(), Value::Bool(muted(value)?));
    }
    if operation == "create" {
        if resource == "projects" {
            object.insert(
                "name".into(),
                Value::String(args.required("--name")?.into()),
            );
            if !object.contains_key("prefix") {
                return Err("Error: --prefix is required. The API requires a project prefix of 1-10 uppercase letters or numbers.".into());
            }
        } else if resource == "issues" {
            for (flag, key) in [
                ("--project", "project_id"),
                ("--title", "title"),
                ("--type", "type"),
            ] {
                object.insert(key.into(), Value::String(args.required(flag)?.into()));
            }
            object.insert(
                "priority".into(),
                Value::String(args.flag("--priority").unwrap_or("medium").into()),
            );
            object.insert(
                "status".into(),
                Value::String(args.flag("--status").unwrap_or("new").into()),
            );
        }
    } else if object.is_empty() {
        return Err(match resource {
            "projects" => "Error: at least one of --name, --description, or --prefix is required.",
            "errors" => "Error: at least one of --status or --muted is required.",
            _ => "Error: at least one update flag is required.",
        }
        .into());
    }
    Ok(Value::Object(object).to_string())
}
pub fn build(resource: &str, args: &Args<'_>) -> Result<Request, String> {
    let operation = args.positional(0).unwrap_or("list");
    let allowed: &[&str] = match resource {
        "projects" | "issues" => &["list", "get", "create", "update", "delete"],
        "errors" => &["list", "get", "search", "report", "create", "update"],
        "incidents" => &["list", "get", "report", "create", "update", "resolve"],
        "checks" => &["list", "get", "create", "update", "delete", "results"],
        "heartbeats" => &[
            "list", "get", "create", "update", "delete", "pings", "ping", "fail", "start",
        ],
        "cloud-ip-ranges" => &["list", "sync-state"],
        _ => return Err(format!("Unknown command: {resource}")),
    };
    if !allowed.contains(&operation) {
        return Err(format!("Unknown {resource} subcommand: {operation}"));
    }
    let base = if matches!(resource, "checks" | "heartbeats") {
        format!(
            "/api/v1/projects/{}/{resource}",
            args.required("--project")?
        )
    } else {
        format!("/api/v1/{resource}")
    };
    let mut path = base;
    if matches!(
        operation,
        "get" | "update" | "delete" | "resolve" | "results" | "pings" | "ping" | "fail" | "start"
    ) {
        let id = args.positional(1).ok_or_else(|| {
            format!(
                "Error: {} ID required. Usage: goodissues {resource} {operation} <id>",
                resource.trim_end_matches('s')
            )
        })?;
        path.push('/');
        path.push_str(id);
    }
    match operation {
        "search" | "sync-state" | "resolve" | "results" | "pings" | "ping" => {
            path.push('/');
            path.push_str(operation);
        }
        "fail" | "start" => {
            path.push_str("/ping/");
            path.push_str(operation);
        }
        _ => {}
    }
    if matches!(operation, "list" | "search" | "results" | "pings") {
        if resource == "errors" && args.flag("--query").is_none() {
            if operation == "list"
                && let Some(value) = args.flag("--muted")
            {
                muted(value)?;
            }
            if operation == "search"
                && ["--module", "--function", "--file"]
                    .iter()
                    .all(|flag| args.flag(flag).is_none())
            {
                return Err("Error: at least one search filter is required (--module, --function, or --file)".into());
            }
        }
        let fields: &[(&str, &str)] = match (resource, operation) {
            ("issues", "list") => &[
                ("--project", "project_id"),
                ("--status", "status"),
                ("--type", "type"),
                ("--page", "page"),
                ("--per-page", "per_page"),
            ],
            ("errors", "list") => &[
                ("--status", "status"),
                ("--muted", "muted"),
                ("--page", "page"),
                ("--per-page", "per_page"),
            ],
            ("errors", "search") => &[
                ("--module", "module"),
                ("--function", "function"),
                ("--file", "file"),
                ("--page", "page"),
                ("--per-page", "per_page"),
            ],
            ("cloud-ip-ranges", "list") => &[
                ("--snapshot-id", "snapshot_id"),
                ("--page", "page"),
                ("--per-page", "per_page"),
            ],
            _ => &[],
        };
        if let Some(query) = query(args, fields).filter(|q| !q.is_empty()) {
            path.push('?');
            path.push_str(&query);
        }
    }
    let (method, expected): (&str, &[u16]) = match operation {
        "create" | "report" => (
            "POST",
            if matches!(resource, "errors" | "incidents") {
                &[200, 201]
            } else {
                &[201]
            },
        ),
        "update" => ("PATCH", &[200]),
        "delete" => ("DELETE", &[204, 200]),
        "resolve" => ("POST", &[200]),
        "ping" | "fail" | "start" => ("POST", &[204, 200]),
        _ => ("GET", &[200]),
    };
    let body = match operation {
        "create" | "report" | "update" => Some(body(resource, operation, args)?),
        "ping" | "fail" | "start" => args.flag("--body").map(str::to_owned),
        _ => None,
    };
    Ok(Request {
        method,
        path,
        body,
        expected,
        operation: operation.into(),
    })
}
