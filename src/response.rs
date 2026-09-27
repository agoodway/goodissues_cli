// Typed validation preserves the Zig CLI's malformed-response handling.
// Fields are inspected by serde, then the validated JSON is formatted separately.
#![allow(dead_code)]
use serde::Deserialize;
#[derive(Deserialize)]
struct PaginationMeta {
    page: Option<i64>,
    per_page: Option<i64>,
    total: Option<i64>,
    total_pages: Option<i64>,
}
#[derive(Deserialize)]
struct Project {
    id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    otel_service_name: Option<String>,
    retention_days: Option<i64>,
    inserted_at: Option<String>,
    updated_at: Option<String>,
}
#[derive(Deserialize)]
struct Issue {
    id: Option<String>,
    key: Option<String>,
    number: Option<i64>,
    title: Option<String>,
    description: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    r#type: Option<String>,
    project_id: Option<String>,
    submitter_id: Option<String>,
    submitter_email: Option<String>,
    archived_at: Option<String>,
    inserted_at: Option<String>,
    updated_at: Option<String>,
}
#[derive(Deserialize)]
struct StackLine {
    module: Option<String>,
    function: Option<String>,
    arity: Option<i64>,
    file: Option<String>,
    line: Option<i64>,
}
#[derive(Deserialize)]
struct Occurrence {
    reason: Option<String>,
    trace_id: Option<String>,
    inserted_at: Option<String>,
    stacktrace: Option<Stacktrace>,
}
#[derive(Deserialize)]
struct ErrorDetail {
    id: Option<String>,
    issue_id: Option<String>,
    kind: Option<String>,
    reason: Option<String>,
    source_line: Option<String>,
    source_function: Option<String>,
    status: Option<String>,
    muted: Option<bool>,
    fingerprint: Option<String>,
    last_occurrence_at: Option<String>,
    inserted_at: Option<String>,
    updated_at: Option<String>,
    occurrence_count: Option<i64>,
    occurrences: Option<Vec<Occurrence>>,
}
#[derive(Deserialize)]
struct MutationData {
    id: Option<String>,
    title: Option<String>,
    name: Option<String>,
}

#[derive(Deserialize)]
struct Stacktrace {
    lines: Option<Vec<StackLine>>,
}
#[derive(Deserialize)]
struct Detail<T> {
    data: T,
}
#[derive(Deserialize)]
struct List<T> {
    data: Vec<T>,
    meta: Option<PaginationMeta>,
}
pub fn valid(resource: &str, operation: &str, body: &str) -> bool {
    match (resource, operation) {
        ("projects" | "issues", "create" | "update") => {
            serde_json::from_str::<Detail<MutationData>>(body).is_ok()
        }
        ("projects", "list") => serde_json::from_str::<List<Project>>(body).is_ok(),
        ("projects", "get") => serde_json::from_str::<Detail<Project>>(body).is_ok(),
        ("issues", "list") => serde_json::from_str::<List<Issue>>(body).is_ok(),
        ("issues", "get") => serde_json::from_str::<Detail<Issue>>(body).is_ok(),
        ("errors", "get") => serde_json::from_str::<Detail<ErrorDetail>>(body).is_ok(),
        _ => true,
    }
}
