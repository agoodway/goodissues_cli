const std = @import("std");
const main_mod = @import("../main.zig");
const cfg_mod = @import("../config.zig");
const gen = @import("../generated.zig");

const File = std.fs.File;

pub fn getClient(allocator: std.mem.Allocator, args: []const []const u8) !gen.Client {
    const cfg = cfg_mod.load(allocator) catch {
        try File.stderr().writeAll("Error: could not load config. Run 'goodissues configure' first.\n");
        std.process.exit(1);
    };
    const env_name = main_mod.getFlag(args, "--env");
    const env = cfg_mod.getEnv(cfg, env_name) orelse {
        try File.stderr().writeAll("Error: no environment configured. Run 'goodissues configure' first.\n");
        std.process.exit(1);
    };
    const base_url = env.base_url orelse {
        try File.stderr().writeAll("Error: no base URL configured. Run 'goodissues configure --url <url>'.\n");
        std.process.exit(1);
    };
    const api_key = env.api_key orelse {
        try File.stderr().writeAll("Error: no API key configured. Run 'goodissues configure --api-key <key>'.\n");
        std.process.exit(1);
    };
    return gen.Client.init(allocator, base_url, api_key);
}

pub fn positionalAt(args: []const []const u8, index: usize) ?[]const u8 {
    var i: usize = 0;
    var count: usize = 0;
    while (i < args.len) : (i += 1) {
        const arg = args[i];
        if (std.mem.startsWith(u8, arg, "--")) {
            if (std.mem.indexOf(u8, arg, "=") == null) i += 1;
            continue;
        }
        if (std.mem.startsWith(u8, arg, "-")) continue;
        if (count == index) return arg;
        count += 1;
    }
    return null;
}

pub fn requireFlag(allocator: std.mem.Allocator, args: []const []const u8, flag: []const u8) ![]const u8 {
    return main_mod.getFlag(args, flag) orelse {
        try main_mod.writeErr(allocator, "Error: {s} is required.\n", .{flag});
        std.process.exit(1);
    };
}

pub fn requireBody(allocator: std.mem.Allocator, args: []const []const u8) ![]const u8 {
    return main_mod.getFlag(args, "--body") orelse {
        try main_mod.writeErr(allocator, "Error: {s} is required for this operation.\n", .{"--body '<json>'"});
        std.process.exit(1);
    };
}

pub fn optionalBody(args: []const []const u8) ?[]const u8 {
    return main_mod.getFlag(args, "--body");
}

pub fn query(args: []const []const u8) ?[]const u8 {
    return main_mod.getFlag(args, "--query");
}

pub fn printResponse(allocator: std.mem.Allocator, resp: gen.Client.RawResponse, expected: []const std.http.Status) !void {
    for (expected) |status| {
        if (resp.status == status) {
            if (resp.body.len > 0) {
                try File.stdout().writeAll(resp.body);
                try File.stdout().writeAll("\n");
            }
            return;
        }
    }

    try main_mod.writeErr(allocator, "Error: API returned {d}\n{s}\n", .{ @intFromEnum(resp.status), resp.body });
    std.process.exit(1);
}
