const std = @import("std");

pub const Table = struct {
    headers: []const []const u8,
    rows: std.ArrayList([]const []const u8),
    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator, headers: []const []const u8) Table {
        return .{ .headers = headers, .rows = .{}, .allocator = allocator };
    }

    pub fn addRow(self: *Table, row: []const []const u8) !void {
        try self.rows.append(self.allocator, row);
    }

    pub fn render(self: *const Table) ![]const u8 {
        const allocator = self.allocator;
        var widths = try allocator.alloc(usize, self.headers.len);
        defer allocator.free(widths);

        for (self.headers, 0..) |h, i| widths[i] = h.len;
        for (self.rows.items) |row| {
            for (row, 0..) |cell, i| {
                if (i < widths.len and cell.len > widths[i]) widths[i] = cell.len;
            }
        }

        var buf: std.ArrayList(u8) = .{};
        for (self.headers, 0..) |h, i| {
            try buf.appendSlice(allocator, h);
            if (i < self.headers.len - 1) try buf.appendNTimes(allocator, ' ', widths[i] - h.len + 2);
        }
        try buf.append(allocator, '\n');

        for (self.rows.items) |row| {
            for (row, 0..) |cell, i| {
                try buf.appendSlice(allocator, cell);
                if (i < row.len - 1) {
                    const w = if (i < widths.len) widths[i] else cell.len;
                    try buf.appendNTimes(allocator, ' ', w - cell.len + 2);
                }
            }
            try buf.append(allocator, '\n');
        }
        return try buf.toOwnedSlice(allocator);
    }
};
