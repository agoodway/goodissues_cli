///////////////////////////////////////////
// API types and client for GoodIssues REST API.
// Based on OpenAPI spec at openapi.json.
// Regenerate types: openapi2zig generate -i openapi.json -o src/generated.zig
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

pub const ErrorItem = struct {
    id: ?[]const u8 = null,
    fingerprint: ?[]const u8 = null,
    kind: ?[]const u8 = null,
    reason: ?[]const u8 = null,
    status: ?[]const u8 = null,
    muted: ?bool = null,
    issue_id: ?[]const u8 = null,
    source_function: ?[]const u8 = null,
    source_line: ?[]const u8 = null,
    last_occurrence_at: ?[]const u8 = null,
    inserted_at: ?[]const u8 = null,
    updated_at: ?[]const u8 = null,
};

pub const ErrorListResponse = struct {
    data: []const ErrorItem,
    meta: ?PaginationMeta = null,
};

pub const ErrorDetailResponse = struct {
    data: ErrorItem,
};

pub const Incident = struct {
    id: ?[]const u8 = null,
    fingerprint: ?[]const u8 = null,
    title: ?[]const u8 = null,
    severity: ?[]const u8 = null,
    source: ?[]const u8 = null,
    status: ?[]const u8 = null,
    muted: ?bool = null,
    issue_id: ?[]const u8 = null,
    last_occurrence_at: ?[]const u8 = null,
    inserted_at: ?[]const u8 = null,
    updated_at: ?[]const u8 = null,
};

pub const IncidentListResponse = struct {
    data: []const Incident,
    meta: ?PaginationMeta = null,
};

pub const IncidentDetailResponse = struct {
    data: Incident,
};

pub const Check = struct {
    id: ?[]const u8 = null,
    project_id: ?[]const u8 = null,
    name: ?[]const u8 = null,
    url: ?[]const u8 = null,
    method: ?[]const u8 = null,
    expected_status: ?i64 = null,
    keyword: ?[]const u8 = null,
    keyword_absence: ?bool = null,
    interval_seconds: ?i64 = null,
    failure_threshold: ?i64 = null,
    consecutive_failures: ?i64 = null,
    paused: ?bool = null,
    status: ?[]const u8 = null,
    current_issue_id: ?[]const u8 = null,
    last_checked_at: ?[]const u8 = null,
    inserted_at: ?[]const u8 = null,
    updated_at: ?[]const u8 = null,
};

pub const CheckListResponse = struct {
    data: []const Check,
    meta: ?PaginationMeta = null,
};

pub const CheckDetailResponse = struct {
    data: Check,
};

pub const CheckResult = struct {
    id: ?[]const u8 = null,
    check_id: ?[]const u8 = null,
    issue_id: ?[]const u8 = null,
    status: ?[]const u8 = null,
    status_code: ?i64 = null,
    response_ms: ?i64 = null,
    @"error": ?[]const u8 = null,
    checked_at: ?[]const u8 = null,
};

pub const CheckResultListResponse = struct {
    data: []const CheckResult,
    meta: ?PaginationMeta = null,
};

pub const Heartbeat = struct {
    id: ?[]const u8 = null,
    project_id: ?[]const u8 = null,
    name: ?[]const u8 = null,
    interval_seconds: ?i64 = null,
    grace_seconds: ?i64 = null,
    failure_threshold: ?i64 = null,
    consecutive_failures: ?i64 = null,
    paused: ?bool = null,
    status: ?[]const u8 = null,
    current_issue_id: ?[]const u8 = null,
    last_ping_at: ?[]const u8 = null,
    next_due_at: ?[]const u8 = null,
    ping_token: ?[]const u8 = null,
    ping_url: ?[]const u8 = null,
    inserted_at: ?[]const u8 = null,
    updated_at: ?[]const u8 = null,
};

pub const HeartbeatListResponse = struct {
    data: []const Heartbeat,
    meta: ?PaginationMeta = null,
};

pub const HeartbeatDetailResponse = struct {
    data: Heartbeat,
};

pub const HeartbeatPing = struct {
    id: ?[]const u8 = null,
    heartbeat_id: ?[]const u8 = null,
    issue_id: ?[]const u8 = null,
    kind: ?[]const u8 = null,
    duration_ms: ?i64 = null,
    exit_code: ?i64 = null,
    pinged_at: ?[]const u8 = null,
};

pub const HeartbeatPingListResponse = struct {
    data: []const HeartbeatPing,
    meta: ?PaginationMeta = null,
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
    pub fn listProjects(self: *const Client, query: ?[]const u8) !RawResponse {
        return self.requestWithQuery(.GET, "/api/v1/projects", query, null);
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

    /// PATCH /api/v1/projects/{id}
    pub fn updateProject(self: *const Client, id: []const u8, body: []const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/projects/{s}", .{id});
        defer self.allocator.free(path);
        return self.request(.PATCH, path, body);
    }

    /// DELETE /api/v1/projects/{id}
    pub fn deleteProject(self: *const Client, id: []const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/projects/{s}", .{id});
        defer self.allocator.free(path);
        return self.request(.DELETE, path, null);
    }

    // --- Issues ---

    /// GET /api/v1/issues
    pub fn listIssues(self: *const Client, query: ?[]const u8) !RawResponse {
        return self.requestWithQuery(.GET, "/api/v1/issues", query, null);
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

    /// PATCH /api/v1/issues/{id}
    pub fn updateIssue(self: *const Client, id: []const u8, body: []const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/issues/{s}", .{id});
        defer self.allocator.free(path);
        return self.request(.PATCH, path, body);
    }

    /// DELETE /api/v1/issues/{id}
    pub fn deleteIssue(self: *const Client, id: []const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/issues/{s}", .{id});
        defer self.allocator.free(path);
        return self.request(.DELETE, path, null);
    }

    // --- Errors ---

    pub fn listErrors(self: *const Client, query: ?[]const u8) !RawResponse {
        return self.requestWithQuery(.GET, "/api/v1/errors", query, null);
    }

    pub fn searchErrors(self: *const Client, query: ?[]const u8) !RawResponse {
        return self.requestWithQuery(.GET, "/api/v1/errors/search", query, null);
    }

    pub fn getError(self: *const Client, id: []const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/errors/{s}", .{id});
        defer self.allocator.free(path);
        return self.request(.GET, path, null);
    }

    pub fn reportError(self: *const Client, body: []const u8) !RawResponse {
        return self.request(.POST, "/api/v1/errors", body);
    }

    pub fn updateError(self: *const Client, id: []const u8, body: []const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/errors/{s}", .{id});
        defer self.allocator.free(path);
        return self.request(.PATCH, path, body);
    }

    // --- Events ---

    pub fn createEventBatch(self: *const Client, body: []const u8) !RawResponse {
        return self.request(.POST, "/api/v1/events/batch", body);
    }

    // --- Incidents ---

    pub fn listIncidents(self: *const Client, query: ?[]const u8) !RawResponse {
        return self.requestWithQuery(.GET, "/api/v1/incidents", query, null);
    }

    pub fn getIncident(self: *const Client, id: []const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/incidents/{s}", .{id});
        defer self.allocator.free(path);
        return self.request(.GET, path, null);
    }

    pub fn reportIncident(self: *const Client, body: []const u8) !RawResponse {
        return self.request(.POST, "/api/v1/incidents", body);
    }

    pub fn updateIncident(self: *const Client, id: []const u8, body: []const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/incidents/{s}", .{id});
        defer self.allocator.free(path);
        return self.request(.PATCH, path, body);
    }

    pub fn resolveIncident(self: *const Client, id: []const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/incidents/{s}/resolve", .{id});
        defer self.allocator.free(path);
        return self.request(.POST, path, null);
    }

    // --- Checks ---

    pub fn listChecks(self: *const Client, project_id: []const u8, query: ?[]const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/projects/{s}/checks", .{project_id});
        defer self.allocator.free(path);
        return self.requestWithQuery(.GET, path, query, null);
    }

    pub fn createCheck(self: *const Client, project_id: []const u8, body: []const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/projects/{s}/checks", .{project_id});
        defer self.allocator.free(path);
        return self.request(.POST, path, body);
    }

    pub fn getCheck(self: *const Client, project_id: []const u8, check_id: []const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/projects/{s}/checks/{s}", .{ project_id, check_id });
        defer self.allocator.free(path);
        return self.request(.GET, path, null);
    }

    pub fn updateCheck(self: *const Client, project_id: []const u8, check_id: []const u8, body: []const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/projects/{s}/checks/{s}", .{ project_id, check_id });
        defer self.allocator.free(path);
        return self.request(.PATCH, path, body);
    }

    pub fn deleteCheck(self: *const Client, project_id: []const u8, check_id: []const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/projects/{s}/checks/{s}", .{ project_id, check_id });
        defer self.allocator.free(path);
        return self.request(.DELETE, path, null);
    }

    pub fn listCheckResults(self: *const Client, project_id: []const u8, check_id: []const u8, query: ?[]const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/projects/{s}/checks/{s}/results", .{ project_id, check_id });
        defer self.allocator.free(path);
        return self.requestWithQuery(.GET, path, query, null);
    }

    // --- Heartbeats ---

    pub fn listHeartbeats(self: *const Client, project_id: []const u8, query: ?[]const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/projects/{s}/heartbeats", .{project_id});
        defer self.allocator.free(path);
        return self.requestWithQuery(.GET, path, query, null);
    }

    pub fn createHeartbeat(self: *const Client, project_id: []const u8, body: []const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/projects/{s}/heartbeats", .{project_id});
        defer self.allocator.free(path);
        return self.request(.POST, path, body);
    }

    pub fn getHeartbeat(self: *const Client, project_id: []const u8, heartbeat_id: []const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/projects/{s}/heartbeats/{s}", .{ project_id, heartbeat_id });
        defer self.allocator.free(path);
        return self.request(.GET, path, null);
    }

    pub fn updateHeartbeat(self: *const Client, project_id: []const u8, heartbeat_id: []const u8, body: []const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/projects/{s}/heartbeats/{s}", .{ project_id, heartbeat_id });
        defer self.allocator.free(path);
        return self.request(.PATCH, path, body);
    }

    pub fn deleteHeartbeat(self: *const Client, project_id: []const u8, heartbeat_id: []const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/projects/{s}/heartbeats/{s}", .{ project_id, heartbeat_id });
        defer self.allocator.free(path);
        return self.request(.DELETE, path, null);
    }

    pub fn listHeartbeatPings(self: *const Client, project_id: []const u8, heartbeat_id: []const u8, query: ?[]const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/projects/{s}/heartbeats/{s}/pings", .{ project_id, heartbeat_id });
        defer self.allocator.free(path);
        return self.requestWithQuery(.GET, path, query, null);
    }

    pub fn pingHeartbeat(self: *const Client, project_id: []const u8, heartbeat_token: []const u8, body: ?[]const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/projects/{s}/heartbeats/{s}/ping", .{ project_id, heartbeat_token });
        defer self.allocator.free(path);
        return self.request(.POST, path, body);
    }

    pub fn failHeartbeat(self: *const Client, project_id: []const u8, heartbeat_token: []const u8, body: ?[]const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/projects/{s}/heartbeats/{s}/ping/fail", .{ project_id, heartbeat_token });
        defer self.allocator.free(path);
        return self.request(.POST, path, body);
    }

    pub fn startHeartbeat(self: *const Client, project_id: []const u8, heartbeat_token: []const u8, body: ?[]const u8) !RawResponse {
        const path = try std.fmt.allocPrint(self.allocator, "/api/v1/projects/{s}/heartbeats/{s}/ping/start", .{ project_id, heartbeat_token });
        defer self.allocator.free(path);
        return self.request(.POST, path, body);
    }

    fn requestWithQuery(self: *const Client, method: std.http.Method, path: []const u8, query: ?[]const u8, body: ?[]const u8) !RawResponse {
        if (query) |q| {
            if (q.len == 0) return self.request(method, path, body);
            const full_path = try std.fmt.allocPrint(self.allocator, "{s}?{s}", .{ path, q });
            defer self.allocator.free(full_path);
            return self.request(method, full_path, body);
        }
        return self.request(method, path, body);
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
