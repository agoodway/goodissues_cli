//! Readable output for the views in the command table. Reads whose response
//! has an unexpected shape fail; writes that already succeeded print the raw
//! response instead.

use crate::{
    commands::{Record, View},
    output::Error,
};
use serde::{Deserialize, de::DeserializeOwned};
use std::fmt::Write;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const MISSING: &str = "-";

#[derive(Deserialize)]
struct Envelope<T> {
    data: T,
}

#[derive(Deserialize)]
struct Project {
    id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    inserted_at: Option<String>,
    updated_at: Option<String>,
}

#[derive(Deserialize)]
struct Issue {
    id: Option<String>,
    key: Option<String>,
    title: Option<String>,
    description: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    r#type: Option<String>,
    project_id: Option<String>,
    submitter_email: Option<String>,
    inserted_at: Option<String>,
    updated_at: Option<String>,
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
    occurrence_count: Option<u64>,
    occurrences: Option<Vec<Occurrence>>,
}

#[derive(Deserialize)]
struct Occurrence {
    reason: Option<String>,
    trace_id: Option<String>,
    inserted_at: Option<String>,
    stacktrace: Option<Stacktrace>,
}

#[derive(Deserialize)]
struct Stacktrace {
    lines: Option<Vec<StackLine>>,
}

#[derive(Deserialize)]
struct StackLine {
    module: Option<String>,
    function: Option<String>,
    arity: Option<i64>,
    file: Option<String>,
    line: Option<i64>,
}

trait Printable: DeserializeOwned {
    const NOUN: &'static str;
    const COLUMNS: &'static [&'static str];
    fn row(&self) -> Vec<&str>;
    fn fields(&self) -> Vec<(&'static str, Option<&str>)>;
    fn id(&self) -> Option<&str>;
    /// The name or title shown after a write.
    fn label(&self) -> Option<&str>;
}

impl Printable for Project {
    const NOUN: &'static str = "project";
    const COLUMNS: &'static [&'static str] = &["ID", "NAME", "DESCRIPTION"];
    fn row(&self) -> Vec<&str> {
        [&self.id, &self.name, &self.description]
            .map(|value| value.as_deref().unwrap_or(MISSING))
            .into()
    }
    fn fields(&self) -> Vec<(&'static str, Option<&str>)> {
        vec![
            ("ID", self.id.as_deref()),
            ("Name", self.name.as_deref()),
            ("Description", self.description.as_deref()),
            ("Created", self.inserted_at.as_deref()),
            ("Updated", self.updated_at.as_deref()),
        ]
    }
    fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }
    fn label(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

impl Printable for Issue {
    const NOUN: &'static str = "issue";
    const COLUMNS: &'static [&'static str] = &["KEY", "TITLE", "STATUS", "PRIORITY", "TYPE"];
    fn row(&self) -> Vec<&str> {
        let key = self.key.as_deref().or(self.id.as_deref());
        [
            key,
            self.title.as_deref(),
            self.status.as_deref(),
            self.priority.as_deref(),
            self.r#type.as_deref(),
        ]
        .map(|value| value.unwrap_or(MISSING))
        .into()
    }
    fn fields(&self) -> Vec<(&'static str, Option<&str>)> {
        vec![
            ("Key", self.key.as_deref()),
            ("ID", self.id.as_deref()),
            ("Title", self.title.as_deref()),
            ("Status", self.status.as_deref()),
            ("Priority", self.priority.as_deref()),
            ("Type", self.r#type.as_deref()),
            ("Description", self.description.as_deref()),
            ("Project", self.project_id.as_deref()),
            ("Email", self.submitter_email.as_deref()),
            ("Created", self.inserted_at.as_deref()),
            ("Updated", self.updated_at.as_deref()),
        ]
    }
    fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }
    fn label(&self) -> Option<&str> {
        self.title.as_deref()
    }
}

/// The response body unchanged, ending in exactly one added newline at most.
fn raw(body: &[u8]) -> Vec<u8> {
    let mut out = body.to_vec();
    if !out.is_empty() && !out.ends_with(b"\n") {
        out.push(b'\n');
    }
    out
}

fn parse<T: DeserializeOwned>(body: &[u8]) -> Option<T> {
    serde_json::from_slice::<Envelope<T>>(body)
        .ok()
        .map(|envelope| envelope.data)
}

fn read<T: DeserializeOwned>(body: &[u8]) -> Result<T, Error> {
    parse(body).ok_or_else(|| Error::Response("unexpected API response".into(), body.to_vec()))
}

fn table_cell(value: &str) -> String {
    const MAX_WIDTH: usize = 60;
    let clean: String = value
        .chars()
        .map(|c| {
            if c.is_control() || c.is_whitespace() {
                ' '
            } else {
                c
            }
        })
        .collect();
    if clean.width() <= MAX_WIDTH {
        return clean;
    }
    let mut result = String::new();
    let mut width = 0;
    for grapheme in clean.graphemes(true) {
        let next_width = grapheme.width();
        if width + next_width >= MAX_WIDTH {
            break;
        }
        result.push_str(grapheme);
        width += next_width;
    }
    result.push('…');
    result
}

fn table<R: Printable>(rows: &[R]) -> String {
    let mut cells: Vec<Vec<String>> = vec![R::COLUMNS.iter().map(|&h| h.to_owned()).collect()];
    cells.extend(
        rows.iter()
            .map(|row| row.row().into_iter().map(table_cell).collect()),
    );
    let widths: Vec<usize> = (0..R::COLUMNS.len())
        .map(|i| cells.iter().map(|row| row[i].width()).max().unwrap_or(0))
        .collect();
    let mut output = String::new();
    for row in cells {
        let last = row.len() - 1;
        for (i, cell) in row.iter().enumerate() {
            output.push_str(cell);
            if i < last {
                output.push_str(&" ".repeat(widths[i] - cell.width() + 2));
            }
        }
        output.push('\n');
    }
    output
}

fn labeled(out: &mut String, width: usize, label: &str, value: Option<&str>) {
    let _ = writeln!(
        out,
        "{:<width$}{}",
        format!("{label}:"),
        value.unwrap_or(MISSING)
    );
}

fn detail<R: Printable>(record: &R) -> String {
    let mut out = String::new();
    for (label, value) in record.fields() {
        labeled(&mut out, 13, label, value);
    }
    out
}

fn written<R: Printable>(verb: &str, body: &[u8]) -> Vec<u8> {
    match parse::<R>(body) {
        Some(record) => format!(
            "{verb} {}: {} ({})\n",
            R::NOUN,
            record.label().unwrap_or(MISSING),
            record.id().unwrap_or(MISSING)
        )
        .into_bytes(),
        None => raw(body),
    }
}

fn error_report(error: &ErrorDetail) -> String {
    const WIDTH: usize = 17;
    let mut out = String::new();
    let field = Option::<String>::as_deref;
    for (label, value) in [
        ("ID", &error.id),
        ("Issue ID", &error.issue_id),
        ("Kind", &error.kind),
        ("Reason", &error.reason),
    ] {
        labeled(&mut out, WIDTH, label, field(value));
    }
    let source = format!(
        "{} in {}",
        field(&error.source_function).unwrap_or(MISSING),
        field(&error.source_line).unwrap_or(MISSING)
    );
    labeled(&mut out, WIDTH, "Source", Some(&source));
    labeled(&mut out, WIDTH, "Status", field(&error.status));
    let muted = error.muted.unwrap_or(false).to_string();
    labeled(&mut out, WIDTH, "Muted", Some(&muted));
    for (label, value) in [
        ("Fingerprint", &error.fingerprint),
        ("Last Occurrence", &error.last_occurrence_at),
        ("Created", &error.inserted_at),
        ("Updated", &error.updated_at),
    ] {
        labeled(&mut out, WIDTH, label, field(value));
    }
    let occurrences = error.occurrences.as_deref().unwrap_or_default();
    if occurrences.is_empty() {
        return out;
    }
    let count = error.occurrence_count.unwrap_or(occurrences.len() as u64);
    let _ = writeln!(out, "\nOccurrences ({count}):");
    for (i, occurrence) in occurrences.iter().enumerate() {
        let _ = writeln!(
            out,
            "\n  [{}] {}\n      Reason: {}",
            i + 1,
            field(&occurrence.inserted_at).unwrap_or(MISSING),
            field(&occurrence.reason).unwrap_or(MISSING)
        );
        if let Some(trace) = field(&occurrence.trace_id).filter(|t| !t.is_empty()) {
            let _ = writeln!(out, "      Trace:  {trace}");
        }
        let lines = occurrence
            .stacktrace
            .as_ref()
            .and_then(|s| s.lines.as_deref())
            .unwrap_or_default();
        if lines.is_empty() {
            continue;
        }
        out.push_str("      Stacktrace:\n");
        for line in lines.iter().take(5) {
            let _ = writeln!(
                out,
                "        {}.{}/{} ({}:{})",
                field(&line.module).unwrap_or(MISSING),
                field(&line.function).unwrap_or(MISSING),
                line.arity.unwrap_or(0),
                field(&line.file).unwrap_or(MISSING),
                line.line.unwrap_or(0)
            );
        }
        if lines.len() > 5 {
            let _ = writeln!(out, "        ... and {} more lines", lines.len() - 5);
        }
    }
    out
}

fn record<R: Printable>(view: View, body: &[u8]) -> Result<Vec<u8>, Error> {
    Ok(match view {
        View::Table(_) => table(&read::<Vec<R>>(body)?).into_bytes(),
        View::Detail(_) => detail(&read::<R>(body)?).into_bytes(),
        View::Created(_) => written::<R>("Created", body),
        View::Updated(_) => written::<R>("Updated", body),
        View::Deleted(_) => {
            let mut noun = R::NOUN.to_owned();
            noun[..1].make_ascii_uppercase();
            format!("{noun} deleted.\n").into_bytes()
        }
        View::Raw | View::ErrorReport => raw(body),
    })
}

pub fn render(view: View, body: &[u8], json: bool) -> Result<Vec<u8>, Error> {
    // A delete has no body worth printing, even as JSON.
    if json && !matches!(view, View::Deleted(_)) {
        return Ok(raw(body));
    }
    match view {
        View::Raw => Ok(raw(body)),
        View::ErrorReport => Ok(error_report(&read(body)?).into_bytes()),
        View::Table(r)
        | View::Detail(r)
        | View::Created(r)
        | View::Updated(r)
        | View::Deleted(r) => match r {
            Record::Project => record::<Project>(view, body),
            Record::Issue => record::<Issue>(view, body),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(result: Result<Vec<u8>, Error>) -> String {
        String::from_utf8(result.unwrap()).unwrap()
    }

    #[test]
    fn tables_align_unicode_display_columns() {
        let body = serde_json::json!({"data": [
            {"id":"1", "name":"café", "description":"one"},
            {"id":"2", "name":"👩‍💻", "description":"two"},
            {"id":"3", "name":"e\u{301}", "description":"three"}
        ]});
        assert_eq!(
            text(render(
                View::Table(Record::Project),
                body.to_string().as_bytes(),
                false
            )),
            "ID  NAME  DESCRIPTION\n1   café  one\n2   👩‍💻    two\n3   e\u{301}     three\n"
        );
    }

    #[test]
    fn tables_keep_cells_single_line_and_bounded() {
        let body = serde_json::json!({"data": [{"id":"1", "name":"line\nbreak\t\u{1b}", "description":"x".repeat(100)}]});
        let result = text(render(
            View::Table(Record::Project),
            body.to_string().as_bytes(),
            false,
        ));
        assert_eq!(result.lines().count(), 2);
        assert!(!result.contains(['\t', '\u{1b}']));
        assert!(result.contains(&format!("{}…", "x".repeat(59))));
    }

    #[test]
    fn issue_tables_fall_back_to_the_id_without_a_key() {
        let body = br#"{"data":[{"id":"i1","title":"T"}]}"#;
        let result = text(render(View::Table(Record::Issue), body, false));
        assert!(result.lines().nth(1).unwrap().starts_with("i1   T"));
    }

    #[test]
    fn malformed_reads_fail_with_the_body_and_writes_fall_back_to_raw() {
        let body = br#"{"data":{"id":123}}"#;
        for view in [View::Detail(Record::Project), View::ErrorReport] {
            let Err(Error::Response(_, echoed)) = render(view, body, false) else {
                panic!("{view:?} should fail");
            };
            assert_eq!(echoed, body);
        }
        assert_eq!(
            render(View::Created(Record::Issue), body, false).unwrap(),
            b"{\"data\":{\"id\":123}}\n"
        );
    }

    #[test]
    fn raw_output_adds_at_most_one_newline() {
        assert_eq!(raw(b""), b"");
        assert_eq!(raw(b"{}"), b"{}\n");
        assert_eq!(raw(b"{}\n"), b"{}\n");
    }
}
