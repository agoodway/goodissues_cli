const std = @import("std");
const main_mod = @import("../main.zig");
const common = @import("common.zig");

pub fn run(allocator: std.mem.Allocator, args: []const []const u8) !void {
    const subcmd = main_mod.getPositional(args) orelse "batch";
    const client = try common.getClient(allocator, args);

    if (std.mem.eql(u8, subcmd, "batch")) {
        try common.printResponse(allocator, try client.createEventBatch(try common.requireBody(allocator, args)), &.{.created});
    } else {
        try main_mod.writeErr(allocator, "Unknown events subcommand: {s}\n", .{subcmd});
        std.process.exit(1);
    }
}
