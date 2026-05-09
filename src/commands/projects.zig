const std = @import("std");
const main_mod = @import("../main.zig");
const cfg_mod = @import("../config.zig");
const gen = @import("../generated.zig");
const table_mod = @import("../table.zig");

const File = std.fs.File;

pub fn run(allocator: std.mem.Allocator, args: []const []const u8) !void {
    const subcmd = main_mod.getPositional(args) orelse "list";

    if (std.mem.eql(u8, subcmd, "list")) {
        try listProjects(allocator, args);
    } else if (std.mem.eql(u8, subcmd, "get")) {
        try getProject(allocator, args);
    } else if (std.mem.eql(u8, subcmd, "create")) {
        try createProject(allocator, args);
    } else if (std.mem.eql(u8, subcmd, "delete")) {
        try deleteProject(allocator, args);
    } else {
        try main_mod.writeErr(allocator, "Unknown projects subcommand: {s}\n", .{subcmd});
        std.process.exit(1);
    }
}

fn getClient(allocator: std.mem.Allocator, args: []const []const u8) !gen.Client {
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

fn listProjects(allocator: std.mem.Allocator, args: []const []const u8) !void {
    const client = try getClient(allocator, args);
    const resp = try client.listProjects();

    if (resp.status != .ok) {
        try main_mod.writeErr(allocator, "Error: API returned {d}\n{s}\n", .{ @intFromEnum(resp.status), resp.body });
        std.process.exit(1);
    }

    if (main_mod.hasFlag(args, "--json")) {
        try File.stdout().writeAll(resp.body);
        try File.stdout().writeAll("\n");
        return;
    }

    const parsed = try std.json.parseFromSlice(gen.ProjectListResponse, allocator, resp.body, .{
        .ignore_unknown_fields = true,
        .allocate = .alloc_always,
    });

    var tbl = table_mod.Table.init(allocator, &.{ "ID", "NAME", "DESCRIPTION" });
    for (parsed.value.data) |project| {
        const row = try allocator.alloc([]const u8, 3);
        row[0] = project.id orelse "-";
        row[1] = project.name orelse "-";
        row[2] = project.description orelse "-";
        try tbl.addRow(row);
    }

    const output = try tbl.render();
    try File.stdout().writeAll(output);
}

fn getProject(allocator: std.mem.Allocator, args: []const []const u8) !void {
    // The ID is the second positional arg (first is "get")
    const id = blk: {
        var i: usize = 0;
        var count: usize = 0;
        while (i < args.len) : (i += 1) {
            const arg = args[i];
            if (std.mem.startsWith(u8, arg, "--")) {
                if (std.mem.indexOf(u8, arg, "=") == null) i += 1;
                continue;
            }
            if (std.mem.startsWith(u8, arg, "-")) continue;
            count += 1;
            if (count == 2) break :blk arg;
        }
        try File.stderr().writeAll("Error: project ID required. Usage: goodissues projects get <id>\n");
        std.process.exit(1);
    };

    const client = try getClient(allocator, args);
    const resp = try client.getProject(id);

    if (resp.status != .ok) {
        try main_mod.writeErr(allocator, "Error: API returned {d}\n{s}\n", .{ @intFromEnum(resp.status), resp.body });
        std.process.exit(1);
    }

    if (main_mod.hasFlag(args, "--json")) {
        try File.stdout().writeAll(resp.body);
        try File.stdout().writeAll("\n");
        return;
    }

    const parsed = try std.json.parseFromSlice(gen.ProjectDetailResponse, allocator, resp.body, .{
        .ignore_unknown_fields = true,
        .allocate = .alloc_always,
    });
    const p = parsed.value.data;

    try main_mod.writeOut(allocator, "ID:          {s}\n", .{p.id orelse "-"});
    try main_mod.writeOut(allocator, "Name:        {s}\n", .{p.name orelse "-"});
    try main_mod.writeOut(allocator, "Description: {s}\n", .{p.description orelse "-"});
    try main_mod.writeOut(allocator, "Created:     {s}\n", .{p.inserted_at orelse "-"});
    try main_mod.writeOut(allocator, "Updated:     {s}\n", .{p.updated_at orelse "-"});
}

fn createProject(allocator: std.mem.Allocator, args: []const []const u8) !void {
    const name = main_mod.getFlag(args, "--name") orelse {
        try File.stderr().writeAll("Error: --name is required. Usage: goodissues projects create --name <name>\n");
        std.process.exit(1);
    };
    const description = main_mod.getFlag(args, "--description");

    const body = if (description) |desc|
        try std.fmt.allocPrint(allocator, "{{\"project\":{{\"name\":\"{s}\",\"description\":\"{s}\"}}}}", .{ name, desc })
    else
        try std.fmt.allocPrint(allocator, "{{\"project\":{{\"name\":\"{s}\"}}}}", .{name});
    defer allocator.free(body);

    const client = try getClient(allocator, args);
    const resp = try client.createProject(body);

    if (resp.status != .created) {
        try main_mod.writeErr(allocator, "Error: API returned {d}\n{s}\n", .{ @intFromEnum(resp.status), resp.body });
        std.process.exit(1);
    }

    if (main_mod.hasFlag(args, "--json")) {
        try File.stdout().writeAll(resp.body);
        try File.stdout().writeAll("\n");
        return;
    }

    try main_mod.writeOut(allocator, "Project created.\n", .{});
}

fn deleteProject(allocator: std.mem.Allocator, args: []const []const u8) !void {
    const id = blk: {
        var i: usize = 0;
        var count: usize = 0;
        while (i < args.len) : (i += 1) {
            const arg = args[i];
            if (std.mem.startsWith(u8, arg, "--")) {
                if (std.mem.indexOf(u8, arg, "=") == null) i += 1;
                continue;
            }
            if (std.mem.startsWith(u8, arg, "-")) continue;
            count += 1;
            if (count == 2) break :blk arg;
        }
        try File.stderr().writeAll("Error: project ID required. Usage: goodissues projects delete <id>\n");
        std.process.exit(1);
    };

    const client = try getClient(allocator, args);
    const resp = try client.deleteProject(id);

    if (resp.status != .no_content and resp.status != .ok) {
        try main_mod.writeErr(allocator, "Error: API returned {d}\n{s}\n", .{ @intFromEnum(resp.status), resp.body });
        std.process.exit(1);
    }

    try main_mod.writeOut(allocator, "Project deleted.\n", .{});
}
