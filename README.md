# goodissues

CLI client for [goodissues.dev](https://goodissues.dev) — manage projects and track bugs and feature requests from the command line.

Built in Zig with zero external dependencies. Single static binary (~1MB).

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
zig build -Doptimize=ReleaseSafe
cp zig-out/bin/goodissues /usr/local/bin/
```

## Quick Start

```sh
# Configure your API key
goodissues configure --url https://goodissues.dev --api-key sk_your_api_key

# List all projects
goodissues projects list

# Create a project
goodissues projects create --name "My App"

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
goodissues projects create --name "Backend API"
goodissues projects create --name "Mobile App" --description "iOS and Android client"

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

# Combine filters
goodissues issues list --project <project-id> --status new

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

# Delete an issue
goodissues issues delete <issue-id>
```

**Issue types:** `bug`, `incident`, `feature_request`

**Priorities:** `low`, `medium` (default), `high`, `critical`

**Statuses:** `new` (default), `in_progress`, `archived`

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

## Build from Source

Requires [Zig](https://ziglang.org/) 0.15.2 or later.

```sh
zig build                     # Debug build
zig build test                # Run tests
zig build run -- --help       # Build and run
just release                  # Optimized native binary
just dist                     # Cross-compile for all 6 platforms
```

## Releasing

Version is defined in `build.zig.zon` and derived everywhere else automatically.

```sh
# Bump version (defaults to patch)
just bump              # 0.1.0 -> 0.1.1
just bump minor        # 0.1.0 -> 0.2.0
just bump major        # 0.1.0 -> 1.0.0

# Publish a release (runs tests, builds all platforms, tags, pushes, creates GitHub release)
just publish
```

`just publish` will:
1. Run `zig build test`
2. Cross-compile binaries for macOS, Linux, and Windows (amd64 + arm64)
3. Generate SHA-256 checksums
4. Create and push a git tag (`v0.1.0`)
5. Create a GitHub release with all binaries attached
