///////////////////////////////////////////
// API types and client for GoodIssues REST API.
// Based on OpenAPI spec at app/openapi.json.
// Regenerate types: openapi2zig generate -i ../app/openapi.json -o src/generated.zig
// Then manually fix nested types and client functions for Zig 0.15.2.
///////////////////////////////////////////

const std = @import("std");

// --- Models ---

pub const PaginationMeta = struct {
    page: ?i64 = null,
    per_page: ?i64 = null,
    total: ?i64 = null,
    total_pages: ?i64 = null,
};

pub const Project = struct {
    id: ?[]const u8 = null,
    name: ?[]const u8 = null,
    description: ?[]const u8 = null,
    inserted_at: ?[]const u8 = null,
    updated_at: ?[]const u8 = null,
};

pub const ProjectListResponse = struct {
    data: []const Project,
    meta: ?PaginationMeta = null,
};

pub const ProjectDetailResponse = struct {
    data: Project,
};

// Using []const u8 for enums since JSON comes as strings
pub const Issue = struct {
    id: ?[]const u8 = null,
    key: ?[]const u8 = null,
    number: ?i64 = null,
    title: ?[]const u8 = null,
    description: ?[]const u8 = null,
    status: ?[]const u8 = null,
    priority: ?[]const u8 = null,
    type: ?[]const u8 = null,
    project_id: ?[]const u8 = null,
    submitter_id: ?[]const u8 = null,
    submitter_email: ?[]const u8 = null,
    archived_at: ?[]const u8 = null,
    inserted_at: ?[]const u8 = null,
    updated_at: ?[]const u8 = null,
};

pub const IssueListResponse = struct {
    data: []const Issue,
    meta: ?PaginationMeta = null,
};

pub const IssueDetailResponse = struct {
    data: Issue,
};

// --- API Client ---

pub const Client = struct {
    allocator: std.mem.Allocator,
    base_url: []const u8,
    api_key: []const u8,

    pub fn init(allocator: std.mem.Allocator, base_url: []const u8, api_key: []const u8) Client {
        return .{ .allocator = allocator, .base_url = base_url, .api_key = api_key };
    }

    pub const RawResponse = struct {
        status: std.http.Status,
        body: []const u8,
    };

    // --- Projects ---

    /// GET /api/v1/projects
    pub fn listProjects(self: *const Client) !RawResponse {
        return self.request(.GET, "/api/v1/projects", null);
    }

    /// GET /api/v1/projects/{id}
    pub fn getProject(self: *const Client, id: []const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/projects/{s}", .{id});
        defer self.allocator.free(path);
        return self.request(.GET, path, null);
    }

    /// POST /api/v1/projects
    pub fn createProject(self: *const Client, body: []const u8) !RawResponse {
        return self.request(.POST, "/api/v1/projects", body);
    }

    /// DELETE /api/v1/projects/{id}
    pub fn deleteProject(self: *const Client, id: []const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/projects/{s}", .{id});
        defer self.allocator.free(path);
        return self.request(.DELETE, path, null);
    }

    // --- Issues ---

    /// GET /api/v1/issues
    pub fn listIssues(self: *const Client) !RawResponse {
        return self.request(.GET, "/api/v1/issues", null);
    }

    /// GET /api/v1/issues/{id}
    pub fn getIssue(self: *const Client, id: []const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/issues/{s}", .{id});
        defer self.allocator.free(path);
        return self.request(.GET, path, null);
    }

    /// POST /api/v1/issues
    pub fn createIssue(self: *const Client, body: []const u8) !RawResponse {
        return self.request(.POST, "/api/v1/issues", body);
    }

    /// DELETE /api/v1/issues/{id}
    pub fn deleteIssue(self: *const Client, id: []const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/issues/{s}", .{id});
        defer self.allocator.free(path);
        return self.request(.DELETE, path, null);
    }

    fn request(self: *const Client, method: std.http.Method, path: []const u8, body: ?[]const u8) !RawResponse {
        const url = try std.fmt.allocPrint(self.allocator, "{s}{s}", .{ self.base_url, path });
        defer self.allocator.free(url);

        const auth_header = try std.fmt.allocPrint(self.allocator, "Bearer {s}", .{self.api_key});
        defer self.allocator.free(auth_header);

        var http_client: std.http.Client = .{ .allocator = self.allocator };
        defer http_client.deinit();

        var aw: std.Io.Writer.Allocating = .init(self.allocator);
        defer aw.deinit();

        const result = try http_client.fetch(.{
            .location = .{ .url = url },
            .method = method,
            .payload = body,
            .response_writer = &aw.writer,
            .extra_headers = &.{
                .{ .name = "Authorization", .value = auth_header },
                .{ .name = "Content-Type", .value = "application/json" },
            },
            .headers = .{
                .accept_encoding = .omit,
            },
        });

        var al = aw.toArrayList();
        const response_body = try al.toOwnedSlice(self.allocator);

        return .{ .status = result.status, .body = response_body };
    }
};
