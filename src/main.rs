mod args;
mod client;
mod config;
mod format;
mod help;
mod output;
mod request;
mod response;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = run(&args) {
        output::error(&error);
        std::process::exit(1);
    }
}
fn run(argv: &[String]) -> Result<(), output::Error> {
    let Some(command) = argv.first().map(String::as_str) else {
        return Err(help::text("").trim_end_matches('\n').into());
    };
    if matches!(command, "help" | "--help" | "-h") {
        output::write(help::text(argv.get(1).map(String::as_str).unwrap_or("")))?;
        return Ok(());
    }
    if matches!(command, "--version" | "-v") {
        output::write(format!("goodissues {}\n", env!("CARGO_PKG_VERSION")))?;
        return Ok(());
    }
    if argv.len() >= 2 && matches!(argv.last().map(String::as_str), Some("--help" | "-h")) {
        output::write(help::text(command))?;
        return Ok(());
    }
    let args = args::Args(&argv[1..]);
    args.validate(command)?;
    if command == "configure" {
        return config::run(&args).map_err(Into::into);
    }
    if ![
        "projects",
        "issues",
        "errors",
        "incidents",
        "checks",
        "heartbeats",
        "cloud-ip-ranges",
    ]
    .contains(&command)
    {
        return Err(format!(
            "Unknown command: {command}\n\n{}",
            help::text("").trim_end()
        )
        .into());
    }
    let request = request::build(command, &args)?;
    let body = client::execute(&request, &args)?;
    output::write(format::response_bytes(
        command,
        &request.operation,
        &body,
        args.has("--json"),
    )?)?;
    Ok(())
}
