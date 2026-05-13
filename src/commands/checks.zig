const std = @import("std");
const main_mod = @import("../main.zig");
const common = @import("common.zig");

const File = std.fs.File;

pub fn run(allocator: std.mem.Allocator, args: []const []const u8) !void {
    const subcmd = main_mod.getPositional(args) orelse "list";
    const project_id = try common.requireFlag(allocator, args, "--project");
    const client = try common.getClient(allocator, args);

    if (std.mem.eql(u8, subcmd, "list")) {
        try common.printResponse(allocator, try client.listChecks(project_id, common.query(args)), &.{.ok});
    } else if (std.mem.eql(u8, subcmd, "get")) {
        const id = common.positionalAt(args, 1) orelse return missing("check ID", "goodissues checks get <id> --project <project_id>");
        try common.printResponse(allocator, try client.getCheck(project_id, id), &.{.ok});
    } else if (std.mem.eql(u8, subcmd, "create")) {
        try common.printResponse(allocator, try client.createCheck(project_id, try common.requireBody(allocator, args)), &.{.created});
    } else if (std.mem.eql(u8, subcmd, "update")) {
        const id = common.positionalAt(args, 1) orelse return missing("check ID", "goodissues checks update <id> --project <project_id> --body '<json>'");
        try common.printResponse(allocator, try client.updateCheck(project_id, id, try common.requireBody(allocator, args)), &.{.ok});
    } else if (std.mem.eql(u8, subcmd, "delete")) {
        const id = common.positionalAt(args, 1) orelse return missing("check ID", "goodissues checks delete <id> --project <project_id>");
        try common.printResponse(allocator, try client.deleteCheck(project_id, id), &.{ .no_content, .ok });
    } else if (std.mem.eql(u8, subcmd, "results")) {
        const id = common.positionalAt(args, 1) orelse return missing("check ID", "goodissues checks results <id> --project <project_id>");
        try common.printResponse(allocator, try client.listCheckResults(project_id, id, common.query(args)), &.{.ok});
    } else {
        try main_mod.writeErr(allocator, "Unknown checks subcommand: {s}\n", .{subcmd});
        std.process.exit(1);
    }
}

fn missing(what: []const u8, usage: []const u8) !void {
    try File.stderr().writeAll("Error: ");
    try File.stderr().writeAll(what);
    try File.stderr().writeAll(" required. Usage: ");
    try File.stderr().writeAll(usage);
    try File.stderr().writeAll("\n");
    std.process.exit(1);
}
