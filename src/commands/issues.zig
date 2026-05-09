const std = @import("std");
const main_mod = @import("../main.zig");
const cfg_mod = @import("../config.zig");
const gen = @import("../generated.zig");
const table_mod = @import("../table.zig");

const File = std.fs.File;

pub fn run(allocator: std.mem.Allocator, args: []const []const u8) !void {
    const subcmd = main_mod.getPositional(args) orelse "list";

    if (std.mem.eql(u8, subcmd, "list")) {
        try listIssues(allocator, args);
    } else if (std.mem.eql(u8, subcmd, "get")) {
        try getIssue(allocator, args);
    } else if (std.mem.eql(u8, subcmd, "create")) {
        try createIssue(allocator, args);
    } else if (std.mem.eql(u8, subcmd, "delete")) {
        try deleteIssue(allocator, args);
    } else {
        try main_mod.writeErr(allocator, "Unknown issues subcommand: {s}\n", .{subcmd});
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

fn listIssues(allocator: std.mem.Allocator, args: []const []const u8) !void {
    const client = try getClient(allocator, args);
    const resp = try client.listIssues();

    if (resp.status != .ok) {
        try main_mod.writeErr(allocator, "Error: API returned {d}\n{s}\n", .{ @intFromEnum(resp.status), resp.body });
        std.process.exit(1);
    }

    if (main_mod.hasFlag(args, "--json")) {
        try File.stdout().writeAll(resp.body);
        try File.stdout().writeAll("\n");
        return;
    }

    const parsed = try std.json.parseFromSlice(gen.IssueListResponse, allocator, resp.body, .{
        .ignore_unknown_fields = true,
        .allocate = .alloc_always,
    });

    var tbl = table_mod.Table.init(allocator, &.{ "KEY", "TITLE", "STATUS", "PRIORITY", "TYPE" });
    for (parsed.value.data) |issue| {
        const row = try allocator.alloc([]const u8, 5);
        row[0] = issue.key orelse issue.id orelse "-";
        row[1] = issue.title orelse "-";
        row[2] = issue.status orelse "-";
        row[3] = issue.priority orelse "-";
        row[4] = issue.type orelse "-";
        try tbl.addRow(row);
    }

    const output = try tbl.render();
    try File.stdout().writeAll(output);
}

fn getIssue(allocator: std.mem.Allocator, args: []const []const u8) !void {
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
        try File.stderr().writeAll("Error: issue ID required. Usage: goodissues issues get <id>\n");
        std.process.exit(1);
    };

    const client = try getClient(allocator, args);
    const resp = try client.getIssue(id);

    if (resp.status != .ok) {
        try main_mod.writeErr(allocator, "Error: API returned {d}\n{s}\n", .{ @intFromEnum(resp.status), resp.body });
        std.process.exit(1);
    }

    if (main_mod.hasFlag(args, "--json")) {
        try File.stdout().writeAll(resp.body);
        try File.stdout().writeAll("\n");
        return;
    }

    const parsed = try std.json.parseFromSlice(gen.IssueDetailResponse, allocator, resp.body, .{
        .ignore_unknown_fields = true,
        .allocate = .alloc_always,
    });
    const issue = parsed.value.data;

    try main_mod.writeOut(allocator, "Key:         {s}\n", .{issue.key orelse "-"});
    try main_mod.writeOut(allocator, "ID:          {s}\n", .{issue.id orelse "-"});
    try main_mod.writeOut(allocator, "Title:       {s}\n", .{issue.title orelse "-"});
    try main_mod.writeOut(allocator, "Status:      {s}\n", .{issue.status orelse "-"});
    try main_mod.writeOut(allocator, "Priority:    {s}\n", .{issue.priority orelse "-"});
    try main_mod.writeOut(allocator, "Type:        {s}\n", .{issue.type orelse "-"});
    try main_mod.writeOut(allocator, "Description: {s}\n", .{issue.description orelse "-"});
    try main_mod.writeOut(allocator, "Project:     {s}\n", .{issue.project_id orelse "-"});
    try main_mod.writeOut(allocator, "Created:     {s}\n", .{issue.inserted_at orelse "-"});
    try main_mod.writeOut(allocator, "Updated:     {s}\n", .{issue.updated_at orelse "-"});
}

fn createIssue(allocator: std.mem.Allocator, args: []const []const u8) !void {
    const project_id = main_mod.getFlag(args, "--project") orelse {
        try File.stderr().writeAll("Error: --project is required.\n");
        std.process.exit(1);
    };
    const title = main_mod.getFlag(args, "--title") orelse {
        try File.stderr().writeAll("Error: --title is required.\n");
        std.process.exit(1);
    };
    const issue_type = main_mod.getFlag(args, "--type") orelse {
        try File.stderr().writeAll("Error: --type is required (bug, incident, feature_request).\n");
        std.process.exit(1);
    };
    const priority = main_mod.getFlag(args, "--priority") orelse "medium";
    const status = main_mod.getFlag(args, "--status") orelse "new";
    const description = main_mod.getFlag(args, "--description");

    var body_parts: std.ArrayList(u8) = .{};
    try body_parts.appendSlice(allocator, "{\"issue\":{");
    try body_parts.appendSlice(allocator, try std.fmt.allocPrint(allocator, "\"project_id\":\"{s}\",", .{project_id}));
    try body_parts.appendSlice(allocator, try std.fmt.allocPrint(allocator, "\"title\":\"{s}\",", .{title}));
    try body_parts.appendSlice(allocator, try std.fmt.allocPrint(allocator, "\"type\":\"{s}\",", .{issue_type}));
    try body_parts.appendSlice(allocator, try std.fmt.allocPrint(allocator, "\"priority\":\"{s}\",", .{priority}));
    try body_parts.appendSlice(allocator, try std.fmt.allocPrint(allocator, "\"status\":\"{s}\"", .{status}));
    if (description) |desc| {
        try body_parts.appendSlice(allocator, try std.fmt.allocPrint(allocator, ",\"description\":\"{s}\"", .{desc}));
    }
    try body_parts.appendSlice(allocator, "}}");
    const body = try body_parts.toOwnedSlice(allocator);

    const client = try getClient(allocator, args);
    const resp = try client.createIssue(body);

    if (resp.status != .created) {
        try main_mod.writeErr(allocator, "Error: API returned {d}\n{s}\n", .{ @intFromEnum(resp.status), resp.body });
        std.process.exit(1);
    }

    if (main_mod.hasFlag(args, "--json")) {
        try File.stdout().writeAll(resp.body);
        try File.stdout().writeAll("\n");
        return;
    }

    try main_mod.writeOut(allocator, "Issue created.\n", .{});
}

fn deleteIssue(allocator: std.mem.Allocator, args: []const []const u8) !void {
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
        try File.stderr().writeAll("Error: issue ID required. Usage: goodissues issues delete <id>\n");
        std.process.exit(1);
    };

    const client = try getClient(allocator, args);
    const resp = try client.deleteIssue(id);

    if (resp.status != .no_content and resp.status != .ok) {
        try main_mod.writeErr(allocator, "Error: API returned {d}\n{s}\n", .{ @intFromEnum(resp.status), resp.body });
        std.process.exit(1);
    }

    try main_mod.writeOut(allocator, "Issue deleted.\n", .{});
}
