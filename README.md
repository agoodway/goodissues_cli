# goodissues

CLI client for [goodissues.dev](https://goodissues.dev) — manage projects and track bugs and feature requests from the command line.

Built in Rust. Distributed as a single executable for Linux, macOS, and Windows (amd64 and arm64). HTTPS uses Rustls; no system OpenSSL installation is required.

## Install

**macOS / Linux:**

```sh
curl -fsSL https://raw.githubusercontent.com/agoodway/goodissues_cli/main/install.sh | sh
```

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/agoodway/goodissues_cli/main/install.ps1 | iex
```

**From source:**

```sh
cargo build --release --locked
cp target/release/goodissues /usr/local/bin/
```

## Quick Start

```sh
# Configure your API key
goodissues configure --url https://goodissues.dev --api-key sk_your_api_key

# List all projects
goodissues projects list

# Create a project (prefix is required by the API)
goodissues projects create --name "My App" --prefix MA

# File a bug
goodissues issues create --project <project-id> --title "Login broken on Safari" --type bug --priority high

# See all issues
goodissues issues list
```

## Configuration

Config is stored at `~/.goodissues.json` (Windows: `%USERPROFILE%\.goodissues.json`).

```sh
# Set up your default environment
goodissues configure --url https://goodissues.dev --api-key sk_live_abc123

# Add a local dev environment
goodissues configure --env dev --url http://localhost:4000 --api-key sk_test_xyz

# Add a staging environment
goodissues configure --env staging --url https://staging.goodissues.dev --api-key sk_staging_789

# Show all configured environments (keys are masked)
goodissues configure show

# Show a specific environment
goodissues configure show --env production
```

The first configured environment becomes the default. Use `--env <name>` on any command to switch environments.

## Commands

### projects

Manage projects within your account.

```sh
# List all projects
goodissues projects list

# List projects as JSON (useful for scripting)
goodissues projects list --json

# Get a single project by ID
goodissues projects get <project-id>

# Create a new project
goodissues projects create --name "Backend API" --prefix API
goodissues projects create --name "Mobile App" --prefix MOB --description "iOS and Android client"

# Delete a project
goodissues projects delete <project-id>
```

### issues

Track bugs, incidents, and feature requests.

```sh
# List all issues across projects
goodissues issues list

# Filter issues by project
goodissues issues list --project <project-id>

# Filter issues by status
goodissues issues list --status new
goodissues issues list --status in_progress
goodissues issues list --status archived

# Combine filters, including type
goodissues issues list --project <project-id> --status new --type bug

# Get a single issue by ID
goodissues issues get <issue-id>

# Create a bug report
goodissues issues create \
  --project <project-id> \
  --title "Crash on file upload" \
  --type bug \
  --priority critical \
  --description "App crashes when uploading files larger than 10MB"

# Create a feature request
goodissues issues create \
  --project <project-id> \
  --title "Add dark mode support" \
  --type feature_request \
  --priority medium

# Report an incident
goodissues issues create \
  --project <project-id> \
  --title "API returning 503 errors" \
  --type incident \
  --priority critical

# Update an issue
goodissues issues update <issue-id> --status in_progress

# Delete an issue
goodissues issues delete <issue-id>
```

**Issue types:** `bug`, `incident`, `feature_request`

**Priorities:** `low`, `medium` (default), `high`, `critical`

**Statuses:** `new` (default), `in_progress`, `archived`

### errors

List, search, report, and update error groups.

```sh
# List unresolved errors
goodissues errors list --status unresolved --muted false

# Search by stacktrace. At least one of these filters is required.
goodissues errors search --module MyApp.Worker --function do_work --file lib/my_app/worker.ex

# Get a readable report, including the first five stack frames
goodissues errors get <id>

# Report an error
goodissues errors report --body '{"project_id":"<id>","kind":"exception","reason":"NullPointerException","fingerprint":"abc"}'

# Update an error group
goodissues errors update <id> --status resolved --muted true
```

### incidents

List, report, update, and resolve incidents.

```sh
# List all incidents
goodissues incidents list

# Get a single incident by ID
goodissues incidents get <id>

# Report an incident
goodissues incidents report --body '{"project_id":"<id>","title":"API returning 503 errors","severity":"critical"}'

# Update an incident
goodissues incidents update <id> --body '{"severity":"major"}'

# Resolve an incident
goodissues incidents resolve <id>
```

### configure

Set up API connection and manage environments.

```sh
# Set URL and API key in one command
goodissues configure --url https://goodissues.dev --api-key sk_live_abc123

# Update just the API key for an existing environment
goodissues configure --api-key sk_new_key_456

# Set up a named environment
goodissues configure --env production --url https://goodissues.dev --api-key sk_live_abc123

# Show current configuration
goodissues configure show

# Show a specific environment
goodissues configure show --env dev
```

## Global Options

All commands support these options:

| Flag | Description |
|------|-------------|
| `--env <name>` | Use a specific configured environment |
| `--json` | Output raw JSON response |
| `--help`, `-h` | Show help for the current command |
| `--version`, `-v` | Print version and exit |

## API Keys

- `sk_*` keys are **read-write** (required for create and delete operations)
- `pk_*` keys are **read-only** (sufficient for listing and viewing)

Get your API key at [goodissues.dev](https://goodissues.dev).

## JSON Output

Add `--json` to any command to get raw JSON instead of formatted tables. Useful for piping to `jq` or other tools:

```sh
# Get all projects as JSON
goodissues projects list --json

# Pipe to jq for filtering
goodissues issues list --json | jq '.data[] | select(.priority == "critical")'

# Get a single issue as JSON
goodissues issues get <issue-id> --json
```

## Multiple Environments

Manage separate configurations for dev, staging, and production:

```sh
# Set up environments
goodissues configure --env dev --url http://localhost:4000 --api-key sk_test_local
goodissues configure --env staging --url https://staging.goodissues.dev --api-key sk_staging_abc
goodissues configure --env production --url https://goodissues.dev --api-key sk_live_xyz

# Use a specific environment for any command
goodissues projects list --env production
goodissues issues create --env staging --project <id> --title "Test issue" --type bug

# Check which environments are configured
goodissues configure show
```

## Checks and heartbeats

These resources require `--project <project-id>` on every command.

```sh
goodissues checks list --project <project-id>
goodissues checks get <id> --project <project-id>
goodissues checks create --project <project-id> --body '{"name":"API","url":"https://example.com"}'
goodissues checks update <id> --project <project-id> --body '{"name":"API health"}'
goodissues checks delete <id> --project <project-id>
goodissues checks results <id> --project <project-id> --query 'page=2'
goodissues heartbeats list --project <project-id>
goodissues heartbeats get <id> --project <project-id>
goodissues heartbeats create --project <project-id> --body '<json>'
goodissues heartbeats update <id> --project <project-id> --body '<json>'
goodissues heartbeats delete <id> --project <project-id>
goodissues heartbeats pings <id> --project <project-id>
goodissues heartbeats ping <token> --project <project-id>
goodissues heartbeats start <token> --project <project-id> --body '{}'
goodissues heartbeats fail <token> --project <project-id>
```

Signals accept an optional `--body`. Lists and results accept `--query`.

## Cloud IP ranges

```sh
goodissues cloud-ip-ranges list --snapshot-id <id> --page 2 --per-page 100
goodissues cloud-ip-ranges sync-state
```

## Compatibility

The Rust CLI preserves the Zig command set, typed flags, raw `--body` and
`--query` overrides, readable output, and raw JSON output. `create` is also
an alias for `report` on errors and incidents. Run `goodissues help <command>`
for every option, including project updates and issue pagination.

Existing `.goodissues.json` files work unchanged. When it is absent, the
legacy `~/.goodissues/config.yaml` is imported once. Run `goodissues configure`
without flags for interactive setup. Unix configuration files use mode 0600.

Intentional fixes: bodyless heartbeat signals and incident resolution send
POST requests successfully; the Zig 0.15.2 implementation panics on these.
Cloud range filter values are percent-encoded to preserve special characters.

## Build from Source

Install the stable [Rust toolchain](https://rustup.rs/), then:

```sh
cargo build --locked
cargo test --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo build --release --locked
cargo run -- --help
```

Tests use temporary configuration directories and local HTTP servers. No API
key or live service is needed. In the monorepo, compare against Zig:

```sh
cd cli-zig && zig build && cd ..
cargo build --manifest-path cli-rust/Cargo.toml
python3 cli-rust/scripts/parity.py cli-rust/target/debug/goodissues cli-zig/zig-out/bin/goodissues
```

## Subtree publishing

`cli-rust/` in `agoodway/goodissues` is the source of the standalone
`agoodway/goodissues_cli` repository. Commit changes in the monorepo, then run
`just sync` from `cli-rust/` (or `bash cli-rust/scripts/sync-subtree.sh`). This
uses a normal fast-forward subtree push and preserves the repository history.
If standalone main has diverged, reconcile those commits before retrying;
do not force-push. The Zig publish recipe is retired.

## Releasing

In a standalone clone, update the version in `Cargo.toml`, run `cargo check`
to update `Cargo.lock`, commit and push main, then run `just publish vX.Y.Z`
(or `bash scripts/publish.sh vX.Y.Z`). The tag must match the package version.
A pushed tag triggers `.github/workflows/release.yml`, which tests and builds:

| Asset | GitHub runner |
|---|---|
| `goodissues-linux-amd64` | `ubuntu-latest` |
| `goodissues-linux-arm64` | `ubuntu-24.04-arm` |
| `goodissues-darwin-arm64` | `macos-latest` |
| `goodissues-darwin-amd64` | `macos-15-intel` |
| `goodissues-windows-amd64.exe` | `windows-latest` |
| `goodissues-windows-arm64.exe` | `windows-11-arm` |

All six builds must succeed before a GitHub release is created with the
binaries and `checksums.txt`. Main pushes, pull requests and manual workflow
runs build the same artifacts without publishing a release. Linux builds
use the runner's glibc; they are not musl/static binaries. Installers retain
the existing release asset names and environment overrides.
