use serde_json::Value;
use std::fmt::Write;
fn string<'a>(value: &'a Value, key: &str, fallback: &'a str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or(fallback)
}
fn table(resource: &str, data: &Value) -> Result<String, String> {
    let rows = data
        .as_array()
        .ok_or("Error: expected a data array in API response.")?;
    let fields: &[(&str, &str)] = if resource == "projects" {
        &[
            ("ID", "id"),
            ("NAME", "name"),
            ("DESCRIPTION", "description"),
        ]
    } else {
        &[
            ("KEY", "key"),
            ("TITLE", "title"),
            ("STATUS", "status"),
            ("PRIORITY", "priority"),
            ("TYPE", "type"),
        ]
    };
    let mut cells: Vec<Vec<&str>> = vec![fields.iter().map(|(h, _)| *h).collect()];
    for row in rows {
        cells.push(
            fields
                .iter()
                .map(|(_, key)| {
                    string(
                        row,
                        key,
                        if *key == "key" {
                            string(row, "id", "-")
                        } else {
                            "-"
                        },
                    )
                })
                .collect(),
        );
    }
    let widths: Vec<usize> = (0..fields.len())
        .map(|i| cells.iter().map(|row| row[i].len()).max().unwrap_or(0))
        .collect();
    let mut output = String::new();
    for row in cells {
        for (i, cell) in row.iter().enumerate() {
            output.push_str(cell);
            if i < row.len() - 1 {
                output.push_str(&" ".repeat(widths[i] - cell.len() + 2));
            }
        }
        output.push('\n');
    }
    Ok(output)
}
fn detail(resource: &str, data: &Value) -> String {
    let fields: &[(&str, &str)] = if resource == "projects" {
        &[
            ("ID", "id"),
            ("Name", "name"),
            ("Description", "description"),
            ("Created", "inserted_at"),
            ("Updated", "updated_at"),
        ]
    } else {
        &[
            ("Key", "key"),
            ("ID", "id"),
            ("Title", "title"),
            ("Status", "status"),
            ("Priority", "priority"),
            ("Type", "type"),
            ("Description", "description"),
            ("Project", "project_id"),
            ("Email", "submitter_email"),
            ("Created", "inserted_at"),
            ("Updated", "updated_at"),
        ]
    };
    let mut output = String::new();
    for (label, key) in fields {
        writeln!(
            output,
            "{:<13}{}",
            format!("{label}:"),
            string(data, key, "-")
        )
        .unwrap();
    }
    output
}
fn error_detail(data: &Value) -> String {
    let mut out = String::new();
    for (label, key) in [
        ("ID", "id"),
        ("Issue ID", "issue_id"),
        ("Kind", "kind"),
        ("Reason", "reason"),
    ] {
        writeln!(out, "{:<17}{}", format!("{label}:"), string(data, key, "")).unwrap();
    }
    writeln!(
        out,
        "Source:          {} in {}",
        string(data, "source_function", ""),
        string(data, "source_line", "")
    )
    .unwrap();
    writeln!(
        out,
        "Status:          {}\nMuted:           {}",
        string(data, "status", ""),
        data["muted"].as_bool().unwrap_or(false)
    )
    .unwrap();
    for (label, key) in [
        ("Fingerprint", "fingerprint"),
        ("Last Occurrence", "last_occurrence_at"),
        ("Created", "inserted_at"),
        ("Updated", "updated_at"),
    ] {
        writeln!(out, "{:<17}{}", format!("{label}:"), string(data, key, "")).unwrap();
    }
    if let Some(occurrences) = data["occurrences"].as_array().filter(|v| !v.is_empty()) {
        writeln!(
            out,
            "\nOccurrences ({}):",
            data["occurrence_count"]
                .as_i64()
                .unwrap_or(occurrences.len() as i64)
        )
        .unwrap();
        for (i, occurrence) in occurrences.iter().enumerate() {
            writeln!(
                out,
                "\n  [{}] {}\n      Reason: {}",
                i + 1,
                string(occurrence, "inserted_at", ""),
                string(occurrence, "reason", "")
            )
            .unwrap();
            let trace = string(occurrence, "trace_id", "");
            if !trace.is_empty() {
                writeln!(out, "      Trace:  {trace}").unwrap();
            }
            if let Some(lines) = occurrence["stacktrace"]["lines"]
                .as_array()
                .filter(|v| !v.is_empty())
            {
                out.push_str("      Stacktrace:\n");
                for line in lines.iter().take(5) {
                    writeln!(
                        out,
                        "        {}.{}/{} ({}:{})",
                        string(line, "module", ""),
                        string(line, "function", ""),
                        line["arity"].as_i64().unwrap_or(0),
                        string(line, "file", ""),
                        line["line"].as_i64().unwrap_or(0)
                    )
                    .unwrap();
                }
                if lines.len() > 5 {
                    writeln!(out, "        ... and {} more lines", lines.len() - 5).unwrap();
                }
            }
        }
    }
    out
}
pub fn response(resource: &str, operation: &str, body: &str, json: bool) -> Result<String, String> {
    let pretty = matches!(resource, "projects" | "issues");
    if pretty && operation == "delete" {
        return Ok(if resource == "projects" {
            "Project deleted.\n"
        } else {
            "Issue deleted.\n"
        }
        .into());
    }
    let mutation = pretty && matches!(operation, "create" | "update");
    let raw = || {
        if mutation {
            format!("{body}{}", if body.ends_with('\n') { "" } else { "\n" })
        } else if body.is_empty() && !pretty {
            String::new()
        } else {
            format!("{body}\n")
        }
    };
    if json {
        return Ok(raw());
    }
    if pretty || (resource == "errors" && operation == "get") {
        if !crate::response::valid(resource, operation, body) {
            return if mutation || resource == "errors" {
                Ok(raw())
            } else {
                Err("Error: invalid API response.".into())
            };
        }
        let parsed = serde_json::from_str::<Value>(body);
        let data = parsed.as_ref().ok().and_then(|v| v.get("data"));
        if mutation || resource == "errors" {
            let Some(data) = data.filter(|d| d.is_object()) else {
                return Ok(raw());
            };
            if resource == "errors" {
                return Ok(error_detail(data));
            }
            return Ok(format!(
                "{} {}: {} ({})\n",
                if operation == "create" {
                    "Created"
                } else {
                    "Updated"
                },
                resource.trim_end_matches('s'),
                string(
                    data,
                    if resource == "projects" {
                        "name"
                    } else {
                        "title"
                    },
                    "-"
                ),
                string(data, "id", "-")
            ));
        }
        let data = data.ok_or("Error: expected data in API response.")?;
        return match operation {
            "list" => table(resource, data),
            "get" => Ok(detail(resource, data)),
            _ => Ok(raw()),
        };
    }
    Ok(raw())
}

/// Raw output must not decode arbitrary upstream response bytes lossily.
pub fn response_bytes(
    resource: &str,
    operation: &str,
    body: &[u8],
    json: bool,
) -> Result<Vec<u8>, String> {
    if let Ok(text) = std::str::from_utf8(body) {
        return response(resource, operation, text, json).map(String::into_bytes);
    }
    let pretty = matches!(resource, "projects" | "issues");
    if pretty && operation == "delete" {
        return response(resource, operation, "", json).map(String::into_bytes);
    }
    let mutation = pretty && matches!(operation, "create" | "update");
    if pretty && !mutation && !json {
        return Err("Error: invalid API response.".into());
    }
    let mut raw = body.to_vec();
    if !mutation || !raw.ends_with(b"\n") {
        raw.push(b'\n');
    }
    Ok(raw)
}
