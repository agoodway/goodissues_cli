const std = @import("std");
const main_mod = @import("../main.zig");
const common = @import("common.zig");

const File = std.fs.File;

pub fn run(allocator: std.mem.Allocator, args: []const []const u8) !void {
    const subcmd = main_mod.getPositional(args) orelse "list";
    const client = try common.getClient(allocator, args);

    if (std.mem.eql(u8, subcmd, "list")) {
        try common.printResponse(allocator, try client.listIncidents(common.query(args)), &.{.ok});
    } else if (std.mem.eql(u8, subcmd, "get")) {
        const id = common.positionalAt(args, 1) orelse return missing("incident ID", "goodissues incidents get <id>");
        try common.printResponse(allocator, try client.getIncident(id), &.{.ok});
    } else if (std.mem.eql(u8, subcmd, "report") or std.mem.eql(u8, subcmd, "create")) {
        try common.printResponse(allocator, try client.reportIncident(try common.requireBody(allocator, args)), &.{ .ok, .created });
    } else if (std.mem.eql(u8, subcmd, "update")) {
        const id = common.positionalAt(args, 1) orelse return missing("incident ID", "goodissues incidents update <id> --body '<json>'");
        try common.printResponse(allocator, try client.updateIncident(id, try common.requireBody(allocator, args)), &.{.ok});
    } else if (std.mem.eql(u8, subcmd, "resolve")) {
        const id = common.positionalAt(args, 1) orelse return missing("incident ID", "goodissues incidents resolve <id>");
        try common.printResponse(allocator, try client.resolveIncident(id), &.{.ok});
    } else {
        try main_mod.writeErr(allocator, "Unknown incidents subcommand: {s}\n", .{subcmd});
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
