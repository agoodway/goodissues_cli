# CLAUDE.md — goodissues (Zig CLI)

## What This Is

CLI for GoodIssues bug and feature request tracking. Built in Zig with zero external dependencies. Single static binary (~1MB).

## Commands

zig build                  # Build debug binary
zig build test             # Run tests
zig build run -- <args>    # Build and run
just release               # Build optimized native binary
just dist                  # Cross-compile all 6 platform binaries

## Project Structure

src/
  main.zig              # CLI entry point, arg parsing, command routing
  help.zig              # All help text (agent-readable, one const per command)
  config.zig            # Per-environment config (~/.goodissues.json, Windows: %USERPROFILE%\.goodissues.json)
  generated.zig         # API types and HTTP client (projects, issues)
  table.zig             # Column-aligned table printer
  commands/
    projects.zig        # projects list|get|create|delete
    issues.zig          # issues list|get|create|delete
    configure.zig       # configure [show] --url --api-key --env

## Adding a New Command

When adding a new command, you MUST update THREE files:

1. **src/main.zig** — Add routing in `main()` and `dispatchHelp()`
2. **src/help.zig** — Add a help constant with usage, flags, arguments, behavior, exit codes, and examples. Also update root_help to list the new command.
3. **src/commands/<name>.zig** — Implementation

## API Client (generated.zig)

Types and client live in `src/generated.zig`. Based on the OpenAPI spec at `../app/openapi.json`.

Regenerate after API changes:
  ~/.local/bin/openapi2zig generate -i ../app/openapi.json -o src/generated.zig
  # Then manually fix nested types, function names, and Zig 0.15.2 API calls

The client uses named methods (e.g. `listProjects`, `getIssue`, `createIssue`) that map
1:1 to REST endpoints. Commands import via `const gen = @import("../generated.zig");`.

## Version Management

Version is defined once in `build.zig.zon` and derived everywhere else:
- `build.zig` reads it via `@import("build.zig.zon")` and passes it as a comptime build option
- `src/main.zig` imports it via `@import("config").version`
- `justfile` extracts it with `grep | head -1 | sed`

To bump: `just bump` (patch), `just bump minor`, or `just bump major`.
To release: `just publish` (runs tests, builds all platforms, tags, pushes, creates GitHub release).

## Zig 0.15.2 Gotchas

- No `std.io.getStdOut()` — Use `std.fs.File.stdout()`
- No `ArrayList.init(allocator)` — Use `var list: std.ArrayList(u8) = .{};` + pass allocator to methods
- No `std.json.stringify` — Use `std.json.fmt()` with `std.fmt.allocPrint("{f}", .{...})`
- Table rows in loops must be heap-allocated (not `&.{...}`)
- Use `std.heap.page_allocator` for CLI processes
