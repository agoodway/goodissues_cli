mod args;
mod client;
mod commands;
mod config;
mod format;
mod help;
mod output;
mod request;

use output::Error;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = run(&args) {
        output::error(&error);
        std::process::exit(1);
    }
}
fn run(argv: &[String]) -> Result<(), Error> {
    let Some(command) = argv.first().map(String::as_str) else {
        return Err(Error::Usage(help::text("").trim_end().into()));
    };
    if matches!(command, "help" | "--help" | "-h") {
        return output::write(help::text(argv.get(1).map_or("", String::as_str)));
    }
    if matches!(command, "--version" | "-v") {
        return output::write(format!("goodissues {}\n", env!("CARGO_PKG_VERSION")));
    }
    if argv.len() >= 2 && matches!(argv.last().map(String::as_str), Some("--help" | "-h")) {
        return output::write(help::text(command));
    }
    let args = args::Args::parse(&argv[1..]);
    if command == "configure" {
        return config::run(&args);
    }
    let resource = commands::find(command).ok_or_else(|| {
        format!(
            "Unknown command: {command}\n\n{}",
            help::text("").trim_end()
        )
    })?;
    let name = args.positional(0).unwrap_or("list");
    let operation = resource.operation(name).ok_or_else(|| {
        format!("Unknown {command} subcommand: {name}. Run 'goodissues help {command}' for usage.")
    })?;
    args.validate(
        command,
        |flag| operation.accepts(resource, flag),
        1 + usize::from(operation.takes_id()),
    )?;
    let request = request::build(resource, operation, &args)?;
    let body = client::execute(&request, &args)?;
    output::write(format::render(operation.view, &body, args.json())?)
}
