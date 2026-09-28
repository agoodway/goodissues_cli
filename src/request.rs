use crate::{
    args::Args,
    commands::{Body, Field, Kind, Method, Operation, Query, Resource, Typed},
    output::Error,
};
use serde_json::{Map, Value};

pub struct Request {
    pub method: Method,
    pub path: String,
    pub body: Option<String>,
    pub expected: &'static [u16],
    pub requires_api_key: bool,
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
fn path_segment(value: &str) -> Result<String, Error> {
    // URL parsers normalize even percent-encoded dot-only segments.
    if matches!(value, "" | "." | "..") {
        return Err("path IDs must be nonempty and cannot be '.' or '..'.".into());
    }
    Ok(encode(value))
}
fn require_any(args: &Args<'_>, flags: &[&str]) -> Result<(), Error> {
    if flags.is_empty() || flags.iter().any(|flag| args.nonempty(flag).is_some()) {
        return Ok(());
    }
    let list = match flags {
        [first, second] => format!("{first} or {second}"),
        [rest @ .., last] if !rest.is_empty() => format!("{}, or {last}", rest.join(", ")),
        _ => flags.concat(),
    };
    Err(format!("at least one of {list} is required.").into())
}
fn typed_value(field: &Field, value: &str) -> Result<Value, Error> {
    match field.kind {
        Kind::Text => Ok(Value::String(value.into())),
        Kind::Bool => match value {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            _ => Err(format!("{} must be true or false.", field.flag).into()),
        },
    }
}
fn query(spec: &Query, args: &Args<'_>) -> Result<String, Error> {
    if let Some(raw) = args.flag("--query") {
        return Ok(raw.into());
    }
    require_any(args, spec.require_any)?;
    let mut parts = Vec::new();
    for field in spec.fields {
        if let Some(value) = args.flag(field.flag) {
            typed_value(field, value)?;
            parts.push(format!("{}={}", field.key, encode(value)));
        }
    }
    Ok(parts.join("&"))
}
fn typed_body(spec: &Typed, args: &Args<'_>) -> Result<String, Error> {
    for (flag, hint) in spec.required {
        if args.nonempty(flag).is_none() {
            return Err(format!("{flag} is required.{hint}").into());
        }
    }
    require_any(args, spec.require_any)?;
    let mut object = Map::new();
    for field in spec.fields {
        if let Some(value) = args.nonempty(field.flag) {
            object.insert(field.key.into(), typed_value(field, value)?);
        }
    }
    for (key, value) in spec.defaults {
        object
            .entry(*key)
            .or_insert_with(|| Value::String((*value).into()));
    }
    Ok(Value::Object(object).to_string())
}
pub fn build(
    resource: &Resource,
    operation: &Operation,
    args: &Args<'_>,
) -> Result<Request, Error> {
    let mut path = String::from("/api/v1");
    if resource.project_scoped {
        path.push_str("/projects/");
        path.push_str(&path_segment(args.required("--project")?)?);
    }
    path.push('/');
    path.push_str(resource.name);
    if operation.takes_id() {
        let id = args.positional(1).ok_or_else(|| {
            format!(
                "{} ID required. Usage: goodissues {} {} <id>",
                resource.singular, resource.name, operation.name
            )
        })?;
        path.push_str(&operation.path.replace("{id}", &path_segment(id)?));
    } else {
        path.push_str(operation.path);
    }
    if let Some(spec) = &operation.query {
        let query = query(spec, args)?;
        if !query.is_empty() {
            path.push('?');
            path.push_str(&query);
        }
    }
    let body = match &operation.body {
        Body::None => None,
        Body::OptionalRaw => args.flag("--body").map(str::to_owned),
        Body::Raw => Some(
            args.flag("--body")
                .ok_or("--body '<json>' is required for this operation.")?
                .to_owned(),
        ),
        Body::Typed(spec) => Some(match args.flag("--body") {
            Some(raw) => raw.to_owned(),
            None => typed_body(spec, args)?,
        }),
    };
    Ok(Request {
        method: operation.method,
        path,
        body,
        expected: operation.expected,
        requires_api_key: operation.requires_api_key,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::find;

    fn build_args(resource: &str, argv: &[&str]) -> Result<Request, Error> {
        let argv: Vec<String> = argv.iter().map(|&arg| arg.into()).collect();
        let args = Args::parse(&argv);
        let resource = find(resource).unwrap();
        let operation = resource
            .operation(args.positional(0).unwrap_or("list"))
            .unwrap();
        build(resource, operation, &args)
    }
    fn message(result: Result<Request, Error>) -> String {
        match result {
            Err(Error::Message(message)) => message,
            _ => panic!("expected an error message"),
        }
    }

    #[test]
    fn dot_only_and_empty_path_parameters_are_rejected() {
        for id in ["", ".", ".."] {
            assert!(build_args("issues", &["get", id]).is_err());
            let project = format!("--project={id}");
            assert!(build_args("checks", &["list", &project]).is_err());
        }
    }

    #[test]
    fn typed_bodies_fill_defaults_and_ignore_empty_values() {
        let request = build_args(
            "issues",
            &[
                "create",
                "--project=p",
                "--title=t",
                "--type=bug",
                "--status=",
            ],
        )
        .unwrap();
        let body: Value = serde_json::from_str(&request.body.unwrap()).unwrap();
        assert_eq!(
            body,
            serde_json::json!({"project_id":"p","title":"t","type":"bug","priority":"medium","status":"new"})
        );
    }

    #[test]
    fn required_and_require_any_messages_name_the_flags() {
        assert_eq!(
            message(build_args("projects", &["create", "--name=n"])),
            "--prefix is required. The API requires a project prefix of 1-10 uppercase letters or numbers."
        );
        assert_eq!(
            message(build_args("errors", &["update", "id"])),
            "at least one of --status or --muted is required."
        );
        assert_eq!(
            message(build_args("errors", &["search"])),
            "at least one of --module, --function, or --file is required."
        );
        assert_eq!(
            message(build_args("errors", &["list", "--muted=yes"])),
            "--muted must be true or false."
        );
    }
}
