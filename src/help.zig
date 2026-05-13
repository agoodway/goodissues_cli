// Comprehensive help text for all commands and subcommands.
// Designed to be machine-readable by AI coding agents — every flag,
// argument, and behavior is documented in plain text.
//
// IMPORTANT: When adding a new command, you must:
// 1. Add a help constant here (e.g. pub const my_cmd_help)
// 2. Update root_help to list the new command
// 3. Add dispatch in main.zig dispatchHelp()

pub const root_help =
    \\goodissues — CLI for GoodIssues bug and feature request tracking
    \\
    \\Usage:
    \\  goodissues <command> [options]
    \\  goodissues help <command>
    \\
    \\Commands:
    \\  projects     List, create, and manage projects
    \\  issues       List, create, and manage issues
    \\  errors       List, search, report, and update error groups
    \\  incidents    List, report, update, and resolve incidents
    \\  checks       Manage project HTTP checks
    \\  heartbeats   Manage project heartbeat monitors and pings
    \\  events       Submit event batches
    \\  configure    Set up API connection (URL, API key)
    \\  help         Show help for any command
    \\
    \\Global Options:
    \\  --help, -h       Show help for the current command
    \\  --version, -v    Print version and exit
    \\  --env <name>     Use a named environment from config
    \\  --json           Output raw JSON instead of formatted tables
    \\
    \\Run 'goodissues help <command>' for details on a specific command.
    \\
;

pub const projects_help =
    \\goodissues projects — Manage projects.
    \\
    \\Usage:
    \\  goodissues projects list [--json] [--env <name>]
    \\  goodissues projects get <id> [--json] [--env <name>]
    \\  goodissues projects create --name <name> [--description <desc>] [--env <name>]
    \\  goodissues projects update <id> --body '<json>' [--env <name>]
    \\  goodissues projects delete <id> [--env <name>]
    \\
    \\Subcommands:
    \\  list       List all projects (default if no subcommand given)
    \\  get        Show a single project by ID
    \\  create     Create a new project (requires sk_* key)
    \\  update     Update a project with a raw ProjectRequest JSON body
    \\  delete     Delete a project (requires sk_* key)
    \\
    \\Options:
    \\  --name <name>           Project name (required for create)
    \\  --description <desc>    Project description
    \\  --body <json>           Raw JSON request body for update
    \\  --query <query>         Raw query string for list (without '?')
    \\  --json                  Output raw JSON
    \\  --env <name>            Use named environment
    \\
    \\Behavior:
    \\  - 'list' is the default when no subcommand is provided
    \\  - Create and delete require a private API key (sk_*)
    \\  - Read operations work with public keys (pk_*)
    \\
    \\Exit Codes:
    \\  0    Success
    \\  1    Error (API error, missing config, invalid arguments)
    \\
    \\Examples:
    \\  goodissues projects list
    \\  goodissues projects get abc123
    \\  goodissues projects create --name "My Project" --description "A project"
    \\  goodissues projects update abc123 --body '{"name":"Renamed"}'
    \\  goodissues projects delete abc123
    \\
;

pub const issues_help =
    \\goodissues issues — Manage issues.
    \\
    \\Usage:
    \\  goodissues issues list [--project <id>] [--status <status>] [--json] [--env <name>]
    \\  goodissues issues get <id> [--json] [--env <name>]
    \\  goodissues issues create --project <id> --title <title> --type <type> [options] [--env <name>]
    \\  goodissues issues update <id> --body '<json>' [--env <name>]
    \\  goodissues issues delete <id> [--env <name>]
    \\
    \\Subcommands:
    \\  list       List issues (default if no subcommand given)
    \\  get        Show a single issue by ID
    \\  create     Create a new issue (requires sk_* key)
    \\  update     Update an issue with a raw IssueUpdateRequest JSON body
    \\  delete     Delete an issue (requires sk_* key)
    \\
    \\Options:
    \\  --project <id>          Filter by project ID (list) or set project (create)
    \\  --title <title>         Issue title (required for create)
    \\  --type <type>           Issue type: bug, incident, feature_request (required for create)
    \\  --priority <priority>   Priority: low, medium, high, critical (default: medium)
    \\  --status <status>       Status: new, in_progress, archived (default: new)
    \\  --description <desc>    Issue description
    \\  --body <json>           Raw JSON request body for update
    \\  --query <query>         Raw query string for list (without '?')
    \\  --json                  Output raw JSON
    \\  --env <name>            Use named environment
    \\
    \\Behavior:
    \\  - 'list' is the default when no subcommand is provided
    \\  - Create and delete require a private API key (sk_*)
    \\  - Issues belong to a project (project_id required for create)
    \\  - Each issue has a key (e.g. PROJ-1) and sequential number
    \\
    \\Exit Codes:
    \\  0    Success
    \\  1    Error (API error, missing config, invalid arguments)
    \\
    \\Examples:
    \\  goodissues issues list
    \\  goodissues issues list --project abc123 --status new
    \\  goodissues issues get def456
    \\  goodissues issues create --project abc123 --title "Fix login" --type bug --priority high
    \\  goodissues issues update def456 --body '{"status":"in_progress"}'
    \\  goodissues issues delete def456
    \\
;

pub const errors_help =
    \\goodissues errors — Manage error groups.
    \\
    \\Usage:
    \\  goodissues errors list [--query <query>] [--env <name>]
    \\  goodissues errors search --query <query> [--env <name>]
    \\  goodissues errors get <id> [--env <name>]
    \\  goodissues errors report --body '<json>' [--env <name>]
    \\  goodissues errors update <id> --body '<json>' [--env <name>]
    \\
    \\Options:
    \\  --query <query>    Raw query string without '?'
    \\  --body <json>      Raw ErrorReportRequest or ErrorUpdateRequest JSON
    \\  --env <name>       Use named environment
    \\
;

pub const incidents_help =
    \\goodissues incidents — Manage incidents.
    \\
    \\Usage:
    \\  goodissues incidents list [--query <query>] [--env <name>]
    \\  goodissues incidents get <id> [--env <name>]
    \\  goodissues incidents report --body '<json>' [--env <name>]
    \\  goodissues incidents update <id> --body '<json>' [--env <name>]
    \\  goodissues incidents resolve <id> [--env <name>]
    \\
    \\Options:
    \\  --query <query>    Raw query string without '?'
    \\  --body <json>      Raw IncidentReportRequest or IncidentUpdateRequest JSON
    \\  --env <name>       Use named environment
    \\
;

pub const checks_help =
    \\goodissues checks — Manage project HTTP checks.
    \\
    \\Usage:
    \\  goodissues checks list --project <id> [--query <query>] [--env <name>]
    \\  goodissues checks get <id> --project <id> [--env <name>]
    \\  goodissues checks create --project <id> --body '<json>' [--env <name>]
    \\  goodissues checks update <id> --project <id> --body '<json>' [--env <name>]
    \\  goodissues checks delete <id> --project <id> [--env <name>]
    \\  goodissues checks results <id> --project <id> [--query <query>] [--env <name>]
    \\
    \\Options:
    \\  --project <id>     Project ID containing the check
    \\  --query <query>    Raw query string without '?'
    \\  --body <json>      Raw CheckRequest or CheckUpdateRequest JSON
    \\  --env <name>       Use named environment
    \\
;

pub const heartbeats_help =
    \\goodissues heartbeats — Manage project heartbeat monitors.
    \\
    \\Usage:
    \\  goodissues heartbeats list --project <id> [--query <query>] [--env <name>]
    \\  goodissues heartbeats get <id> --project <id> [--env <name>]
    \\  goodissues heartbeats create --project <id> --body '<json>' [--env <name>]
    \\  goodissues heartbeats update <id> --project <id> --body '<json>' [--env <name>]
    \\  goodissues heartbeats delete <id> --project <id> [--env <name>]
    \\  goodissues heartbeats pings <id> --project <id> [--query <query>] [--env <name>]
    \\  goodissues heartbeats ping <token> --project <id> [--body '<json>'] [--env <name>]
    \\  goodissues heartbeats start <token> --project <id> [--body '<json>'] [--env <name>]
    \\  goodissues heartbeats fail <token> --project <id> [--body '<json>'] [--env <name>]
    \\
    \\Options:
    \\  --project <id>     Project ID containing the heartbeat
    \\  --query <query>    Raw query string without '?'
    \\  --body <json>      Raw HeartbeatRequest, HeartbeatUpdateRequest, or PingPayload JSON
    \\  --env <name>       Use named environment
    \\
;

pub const events_help =
    \\goodissues events — Submit event batches.
    \\
    \\Usage:
    \\  goodissues events batch --body '<json>' [--env <name>]
    \\
    \\Options:
    \\  --body <json>      Raw batch event JSON request body
    \\  --env <name>       Use named environment
    \\
;

pub const configure_help =
    \\goodissues configure — Set up API connection.
    \\
    \\Usage:
    \\  goodissues configure [--env <name>] [--url <url>] [--api-key <key>]
    \\  goodissues configure show [--env <name>]
    \\
    \\Subcommands:
    \\  (none)     Set configuration values (interactive if flags omitted)
    \\  show       Display current configuration (API key masked)
    \\
    \\Options:
    \\  --env <name>        Environment name (default: "default")
    \\  --url <url>         Base URL of GoodIssues server
    \\  --api-key <key>     API key (pk_* for read-only, sk_* for read/write)
    \\
    \\Behavior:
    \\  - Config stored at ~/.goodissues.json (Windows: %USERPROFILE%\.goodissues.json)
    \\  - Multiple environments supported (--env production, --env staging)
    \\  - First configured environment becomes the default
    \\  - 'show' masks API keys (shows first/last 4 chars)
    \\
    \\Exit Codes:
    \\  0    Success
    \\  1    Error (file write failure, no config found)
    \\
    \\Examples:
    \\  goodissues configure --url http://localhost:4000 --api-key sk_test123
    \\  goodissues configure --env production --url https://issues.example.com --api-key sk_live456
    \\  goodissues configure show
    \\  goodissues configure show --env production
    \\
;
