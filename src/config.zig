const std = @import("std");
const builtin = @import("builtin");

pub const EnvEntry = struct {
    name: []const u8,
    base_url: ?[]const u8 = null,
    api_key: ?[]const u8 = null,
};

pub const Config = struct {
    default_env: ?[]const u8 = null,
    environments: ?[]const EnvEntry = null,
};

const config_filename = ".goodissues.json";

pub fn configPath(allocator: std.mem.Allocator) ![]const u8 {
    const home = if (builtin.os.tag == .windows)
        std.process.getEnvVarOwned(allocator, "USERPROFILE") catch
            std.process.getEnvVarOwned(allocator, "HOME") catch
            return error.NoHomeDir
    else
        std.process.getEnvVarOwned(allocator, "HOME") catch return error.NoHomeDir;
    defer allocator.free(home);
    const sep: []const u8 = if (builtin.os.tag == .windows) "\\" else "/";
    return std.fmt.allocPrint(allocator, "{s}{s}{s}", .{ home, sep, config_filename });
}

pub fn load(allocator: std.mem.Allocator) !Config {
    const path = try configPath(allocator);
    defer allocator.free(path);

    const file = std.fs.openFileAbsolute(path, .{}) catch |err| switch (err) {
        error.FileNotFound => return Config{},
        else => return err,
    };
    defer file.close();

    const stat = try file.stat();
    if (stat.size == 0) return Config{};

    const content = try file.readToEndAlloc(allocator, 1024 * 1024);
    defer allocator.free(content);

    const parsed = try std.json.parseFromSlice(Config, allocator, content, .{
        .ignore_unknown_fields = true,
        .allocate = .alloc_always,
    });
    return parsed.value;
}

pub fn save(allocator: std.mem.Allocator, cfg: Config) !void {
    const path = try configPath(allocator);
    defer allocator.free(path);

    const json_str = try std.fmt.allocPrint(allocator, "{f}", .{std.json.fmt(cfg, .{ .whitespace = .indent_2 })});
    defer allocator.free(json_str);

    const file = try std.fs.createFileAbsolute(path, .{ .truncate = true });
    defer file.close();
    try file.writeAll(json_str);
}

pub fn getEnv(cfg: Config, name: ?[]const u8) ?EnvEntry {
    const env_name = name orelse cfg.default_env orelse return null;
    const envs = cfg.environments orelse return null;
    for (envs) |entry| {
        if (std.mem.eql(u8, entry.name, env_name)) return entry;
    }
    return null;
}

pub fn setEnv(allocator: std.mem.Allocator, cfg: *Config, name: []const u8, base_url: ?[]const u8, api_key: ?[]const u8) !void {
    var envs: std.ArrayList(EnvEntry) = .{};

    if (cfg.environments) |existing| {
        for (existing) |entry| {
            if (std.mem.eql(u8, entry.name, name)) {
                try envs.append(allocator, .{
                    .name = try allocator.dupe(u8, name),
                    .base_url = if (base_url) |u| try allocator.dupe(u8, u) else entry.base_url,
                    .api_key = if (api_key) |k| try allocator.dupe(u8, k) else entry.api_key,
                });
            } else {
                try envs.append(allocator, entry);
            }
        }
    }

    var found = false;
    if (cfg.environments) |existing| {
        for (existing) |entry| {
            if (std.mem.eql(u8, entry.name, name)) {
                found = true;
                break;
            }
        }
    }
    if (!found) {
        try envs.append(allocator, .{
            .name = try allocator.dupe(u8, name),
            .base_url = if (base_url) |u| try allocator.dupe(u8, u) else null,
            .api_key = if (api_key) |k| try allocator.dupe(u8, k) else null,
        });
    }

    cfg.environments = envs.items;
    if (cfg.default_env == null) cfg.default_env = try allocator.dupe(u8, name);
}

pub fn maskKey(allocator: std.mem.Allocator, key: []const u8) ![]const u8 {
    if (key.len <= 8) return "****";
    return std.fmt.allocPrint(allocator, "{s}****{s}", .{ key[0..4], key[key.len - 4 ..] });
}
