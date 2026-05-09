const std = @import("std");
const config = @import("config");
const help = @import("help.zig");
const projects_cmd = @import("commands/projects.zig");
const issues_cmd = @import("commands/issues.zig");
const configure_cmd = @import("commands/configure.zig");

const File = std.fs.File;

pub fn main() !void {
    const allocator = std.heap.page_allocator;

    const args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, args);

    if (args.len < 2) {
        try File.stderr().writeAll(help.root_help);
        std.process.exit(1);
    }

    const cmd = args[1];

    if (std.mem.eql(u8, cmd, "help") or std.mem.eql(u8, cmd, "--help") or std.mem.eql(u8, cmd, "-h")) {
        if (args.len >= 3) {
            try dispatchHelp(args[2..]);
        } else {
            try File.stdout().writeAll(help.root_help);
        }
        return;
    }
    if (std.mem.eql(u8, cmd, "--version") or std.mem.eql(u8, cmd, "-v")) {
        try File.stdout().writeAll("goodissues " ++ config.version ++ "\n");
        return;
    }

    // Check for trailing --help on any command
    if (args.len >= 3 and (std.mem.eql(u8, args[args.len - 1], "--help") or std.mem.eql(u8, args[args.len - 1], "-h"))) {
        try dispatchHelp(args[1 .. args.len - 1]);
        return;
    }

    if (std.mem.eql(u8, cmd, "projects")) {
        try projects_cmd.run(allocator, args[2..]);
    } else if (std.mem.eql(u8, cmd, "issues")) {
        try issues_cmd.run(allocator, args[2..]);
    } else if (std.mem.eql(u8, cmd, "configure")) {
        try configure_cmd.run(allocator, args[2..]);
    } else {
        try writeErr(allocator, "Unknown command: {s}\n\n", .{cmd});
        try File.stderr().writeAll(help.root_help);
        std.process.exit(1);
    }
}

fn dispatchHelp(args: []const []const u8) !void {
    if (args.len == 0) {
        try File.stdout().writeAll(help.root_help);
        return;
    }
    const cmd = args[0];
    if (std.mem.eql(u8, cmd, "projects")) {
        return File.stdout().writeAll(help.projects_help);
    }
    if (std.mem.eql(u8, cmd, "issues")) {
        return File.stdout().writeAll(help.issues_help);
    }
    if (std.mem.eql(u8, cmd, "configure")) {
        return File.stdout().writeAll(help.configure_help);
    }
    try File.stdout().writeAll(help.root_help);
}

// --- Output helpers ---

pub fn writeOut(allocator: std.mem.Allocator, comptime fmt: []const u8, args: anytype) !void {
    const msg = try std.fmt.allocPrint(allocator, fmt, args);
    defer allocator.free(msg);
    try File.stdout().writeAll(msg);
}

pub fn writeErr(allocator: std.mem.Allocator, comptime fmt: []const u8, args: anytype) !void {
    const msg = try std.fmt.allocPrint(allocator, fmt, args);
    defer allocator.free(msg);
    try File.stderr().writeAll(msg);
}

// --- Arg parsing helpers ---

pub fn getFlag(args: []const []const u8, name: []const u8) ?[]const u8 {
    var i: usize = 0;
    while (i < args.len) : (i += 1) {
        const arg = args[i];
        if (std.mem.startsWith(u8, arg, name)) {
            if (arg.len > name.len and arg[name.len] == '=') {
                return arg[name.len + 1 ..];
            }
            if (std.mem.eql(u8, arg, name)) {
                if (i + 1 < args.len) return args[i + 1];
            }
        }
    }
    return null;
}

pub fn hasFlag(args: []const []const u8, name: []const u8) bool {
    for (args) |arg| {
        if (std.mem.eql(u8, arg, name)) return true;
    }
    return false;
}

pub fn getPositional(args: []const []const u8) ?[]const u8 {
    var i: usize = 0;
    while (i < args.len) : (i += 1) {
        const arg = args[i];
        if (std.mem.startsWith(u8, arg, "--")) {
            if (std.mem.indexOf(u8, arg, "=") == null) i += 1;
            continue;
        }
        if (std.mem.startsWith(u8, arg, "-")) continue;
        return arg;
    }
    return null;
}

test "getFlag" {
    const a = &[_][]const u8{ "--env", "dev", "--url", "http://localhost" };
    try std.testing.expectEqualStrings("dev", getFlag(a, "--env").?);
    try std.testing.expect(getFlag(a, "--missing") == null);
}

test "getFlag with equals" {
    const a = &[_][]const u8{"--env=prod"};
    try std.testing.expectEqualStrings("prod", getFlag(a, "--env").?);
}

test "hasFlag" {
    const a = &[_][]const u8{ "--json", "--env", "dev" };
    try std.testing.expect(hasFlag(a, "--json"));
    try std.testing.expect(!hasFlag(a, "--force"));
}

test "getPositional" {
    const a = &[_][]const u8{ "--env", "dev", "42" };
    try std.testing.expectEqualStrings("42", getPositional(a).?);
}
