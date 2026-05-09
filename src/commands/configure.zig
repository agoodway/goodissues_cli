const std = @import("std");
const main_mod = @import("../main.zig");
const cfg_mod = @import("../config.zig");

const File = std.fs.File;

pub fn run(allocator: std.mem.Allocator, args: []const []const u8) !void {
    const subcmd = main_mod.getPositional(args);

    if (subcmd != null and std.mem.eql(u8, subcmd.?, "show")) {
        try showConfig(allocator, args);
        return;
    }

    const env_name = main_mod.getFlag(args, "--env") orelse "default";
    const url = main_mod.getFlag(args, "--url");
    const api_key = main_mod.getFlag(args, "--api-key");

    if (url == null and api_key == null) {
        try File.stderr().writeAll("Error: at least one of --url or --api-key is required.\n");
        try File.stderr().writeAll("Usage: goodissues configure --url <url> --api-key <key>\n");
        std.process.exit(1);
    }

    var cfg = cfg_mod.load(allocator) catch Config: {
        break :Config cfg_mod.Config{};
    };

    try cfg_mod.setEnv(allocator, &cfg, env_name, url, api_key);
    try cfg_mod.save(allocator, cfg);

    try main_mod.writeOut(allocator, "Configuration saved for environment '{s}'.\n", .{env_name});
}

fn showConfig(allocator: std.mem.Allocator, args: []const []const u8) !void {
    const cfg = cfg_mod.load(allocator) catch {
        try File.stderr().writeAll("Error: could not load config.\n");
        std.process.exit(1);
    };

    const env_name = main_mod.getFlag(args, "--env");

    if (env_name) |name| {
        const env = cfg_mod.getEnv(cfg, name) orelse {
            try main_mod.writeErr(allocator, "No environment '{s}' found.\n", .{name});
            std.process.exit(1);
        };
        try printEnv(allocator, env);
        return;
    }

    // Show all environments
    try main_mod.writeOut(allocator, "Default: {s}\n\n", .{cfg.default_env orelse "(none)"});
    const envs = cfg.environments orelse {
        try File.stdout().writeAll("No environments configured.\n");
        return;
    };
    for (envs) |env| {
        try printEnv(allocator, env);
        try File.stdout().writeAll("\n");
    }
}

fn printEnv(allocator: std.mem.Allocator, env: cfg_mod.EnvEntry) !void {
    try main_mod.writeOut(allocator, "Environment: {s}\n", .{env.name});
    try main_mod.writeOut(allocator, "  URL:     {s}\n", .{env.base_url orelse "(not set)"});
    if (env.api_key) |key| {
        const masked = try cfg_mod.maskKey(allocator, key);
        try main_mod.writeOut(allocator, "  API Key: {s}\n", .{masked});
    } else {
        try File.stdout().writeAll("  API Key: (not set)\n");
    }
}
