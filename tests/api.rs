use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    process::Command,
    thread,
    time::{Duration, Instant},
};
use tempfile::TempDir;

fn request(args: &[&str], status: u16, response: &str) -> (std::process::Output, String) {
    request_bytes(args, status, response.as_bytes())
}
fn request_bytes(args: &[&str], status: u16, response: &[u8]) -> (std::process::Output, String) {
    request_with_key(args, status, response, Some("sk_fixture"))
}
fn request_with_key(
    args: &[&str],
    status: u16,
    response: &[u8],
    key: Option<&str>,
) -> (std::process::Output, String) {
    let home = TempDir::new().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    fs::write(home.path().join(".goodissues.json"), serde_json::json!({"default_env":"test","environments":[{"name":"test","base_url":format!("http://{}",listener.local_addr().unwrap()),"api_key":key}]}).to_string()).unwrap();
    let response = response.to_vec();
    let server = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut stream = loop {
            if let Ok((s, _)) = listener.accept() {
                break s;
            }
            if Instant::now() > deadline {
                return String::new();
            }
            thread::sleep(Duration::from_millis(5));
        };
        // Windows and BSD/macOS can inherit the listener's nonblocking mode.
        // Only accept() is polled; request reads must block with a timeout.
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut bytes = Vec::new();
        let mut buf = [0; 4096];
        loop {
            let n = stream.read(&mut buf).unwrap();
            if n == 0 {
                break;
            }
            bytes.extend_from_slice(&buf[..n]);
            if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&bytes[..end]).to_lowercase();
                let length = headers
                    .lines()
                    .find_map(|l| l.strip_prefix("content-length: "))
                    .unwrap_or("0")
                    .parse::<usize>()
                    .unwrap();
                if bytes.len() >= end + 4 + length {
                    break;
                }
            }
        }
        write!(stream, "HTTP/1.1 {status} OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n", response.len()).unwrap();
        stream.write_all(&response).unwrap();
        String::from_utf8(bytes).unwrap()
    });
    let output = Command::new(env!("CARGO_BIN_EXE_goodissues"))
        .args(args)
        .env("HOME", home.path())
        .env("USERPROFILE", home.path())
        .env("NO_PROXY", "127.0.0.1")
        .output()
        .unwrap();
    (output, server.join().unwrap())
}
#[test]
fn every_api_operation_uses_compatible_route_and_method() {
    let mut cases: Vec<(Vec<String>, String, &str, u16)> = Vec::new();
    for resource in [
        "projects",
        "issues",
        "errors",
        "incidents",
        "checks",
        "heartbeats",
    ] {
        let scoped = matches!(resource, "checks" | "heartbeats");
        let base = if scoped {
            format!("/api/v1/projects/proj/{resource}")
        } else {
            format!("/api/v1/{resource}")
        };
        let mut operations = vec![
            ("list", "GET", "", 200),
            ("get", "GET", "/id", 200),
            ("create", "POST", "", 201),
            ("update", "PATCH", "/id", 200),
        ];
        if !matches!(resource, "errors" | "incidents") {
            operations.push(("delete", "DELETE", "/id", 204));
        }
        if matches!(resource, "errors" | "incidents") {
            operations.push(("report", "POST", "", 200));
        }
        if resource == "errors" {
            operations.push(("search", "GET", "/search", 200));
        }
        if resource == "incidents" {
            operations.push(("resolve", "POST", "/id/resolve", 200));
        }
        if resource == "checks" {
            operations.push(("results", "GET", "/id/results", 200));
        }
        if resource == "heartbeats" {
            operations.extend([
                ("pings", "GET", "/id/pings", 200),
                ("ping", "POST", "/id/ping", 204),
                ("fail", "POST", "/id/ping/fail", 204),
                ("start", "POST", "/id/ping/start", 204),
            ]);
        }
        for (operation, method, suffix, status) in operations {
            let mut args = vec![resource.to_string(), operation.to_string(), "--json".into()];
            if suffix.contains("/id") {
                args.push("id".into());
            }
            if scoped {
                args.extend(["--project".into(), "proj".into()]);
            }
            if matches!(operation, "create" | "report" | "update") {
                args.extend(["--body".into(), "{\"title\":\"raw\"}".into()]);
            }
            if operation == "search" {
                args.extend(["--query".into(), "module=M".into()]);
            }
            cases.push((
                args,
                format!(
                    "{base}{suffix}{}",
                    if operation == "search" {
                        "?module=M"
                    } else {
                        ""
                    }
                ),
                method,
                status,
            ));
        }
    }
    for (op, suffix) in [("list", ""), ("sync-state", "/sync-state")] {
        cases.push((
            vec!["cloud-ip-ranges".into(), op.into()],
            format!("/api/v1/cloud-ip-ranges{suffix}"),
            "GET",
            200,
        ));
    }
    for (args, path, method, status) in cases {
        let argv: Vec<_> = args.iter().map(String::as_str).collect();
        let (out, wire) = request(
            &argv,
            status,
            if status == 204 { "" } else { "{\"data\":{}}" },
        );
        assert!(
            out.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            wire.starts_with(&format!("{method} {path} HTTP/1.1\r\n")),
            "{args:?}: {wire}"
        );
        assert!(
            wire.to_lowercase()
                .contains("authorization: bearer sk_fixture\r\n")
        );
        assert!(
            wire.to_lowercase()
                .contains("content-type: application/json\r\n")
        );
        if argv.contains(&"--body") {
            assert!(wire.ends_with("{\"title\":\"raw\"}"));
        }
    }
}
#[test]
fn typed_fields_defaults_encoding_and_raw_overrides() {
    let cases: &[(&[&str], &str)] = &[
        (
            &[
                "issues",
                "list",
                "--project",
                "a b",
                "--type=bug",
                "--status=new",
                "--page=2",
                "--per-page=10",
                "--json",
            ],
            "/api/v1/issues?project_id=a%20b&status=new&type=bug&page=2&per_page=10",
        ),
        (
            &[
                "errors",
                "search",
                "--module=M A",
                "--function=f",
                "--file=lib/x.ex",
            ],
            "/api/v1/errors/search?module=M%20A&function=f&file=lib%2Fx.ex",
        ),
        (
            &[
                "errors",
                "list",
                "--muted=invalid",
                "--query=status=resolved",
            ],
            "/api/v1/errors?status=resolved",
        ),
        (
            &[
                "cloud-ip-ranges",
                "--snapshot-id=snap",
                "--page=2",
                "--per-page=50",
            ],
            "/api/v1/cloud-ip-ranges?snapshot_id=snap&page=2&per_page=50",
        ),
    ];
    for (args, path) in cases {
        let (out, wire) = request(args, 200, "{\"data\":[]}");
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(wire.starts_with(&format!("GET {path} HTTP/1.1")), "{wire}");
    }
    let (out, wire) = request(
        &[
            "issues",
            "create",
            "--project=p",
            "--title=Hi \"there\"",
            "--type=bug",
            "--json",
        ],
        201,
        "{}",
    );
    assert!(out.status.success());
    let body: serde_json::Value =
        serde_json::from_str(wire.split_once("\r\n\r\n").unwrap().1).unwrap();
    assert_eq!(
        body,
        serde_json::json!({"project_id":"p", "title":"Hi \"there\"", "type":"bug", "priority":"medium", "status":"new"})
    );
    let (out, wire) = request(&["errors", "update", "id", "--muted=false"], 200, "{}");
    assert!(out.status.success());
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(wire.split_once("\r\n\r\n").unwrap().1).unwrap(),
        serde_json::json!({"muted":false})
    );
}
#[test]
fn api_failure_reports_status_and_body() {
    let (out, _) = request(&["issues", "list"], 403, "{\"error\":\"denied\"}");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "Error: API returned 403\n{\"error\":\"denied\"}\n"
    );
}
#[test]
fn readable_outputs_and_raw_json_are_preserved() {
    let (out, _) = request(
        &["projects", "list"],
        200,
        r#"{"data":[{"id":"p1","name":"API","description":null}]}"#,
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "ID  NAME  DESCRIPTION\np1  API   -\n"
    );
    let (out, _) = request(
        &["issues", "create", "--body={}"],
        201,
        r#"{"data":{"id":"i1","title":"Login"}}"#,
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "Created issue: Login (i1)\n"
    );
    let (out, _) = request(&["issues", "delete", "i1", "--json"], 204, "");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "Issue deleted.\n");
    let (out, _) = request(
        &["projects", "update", "p1", "--name=New", "--json"],
        200,
        "{ \"data\": {} }\n",
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "{ \"data\": {} }\n");
    let frames: Vec<_> = (1..=7).map(|n| serde_json::json!({"module":"M", "function":"f", "arity":1, "file":"a.ex", "line":n})).collect();
    let data = serde_json::json!({"data":{"id":"err1","occurrence_count":9,"occurrences":[{"trace_id":"t1", "stacktrace":{"lines":frames}}]}});
    let (out, _) = request(&["errors", "get", "err1"], 200, &data.to_string());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("ID:              err1\n"));
    assert!(text.contains("Occurrences (9):\n"));
    assert!(text.contains("M.f/1 (a.ex:5)\n"));
    assert!(!text.contains("M.f/1 (a.ex:6)\n"));
    assert!(text.contains("... and 2 more lines\n"));
}
#[test]
fn raw_json_does_not_strip_a_utf8_bom() {
    let body = "\u{feff}{\"data\":[]}";
    let (out, _) = request(&["projects", "list", "--json"], 200, body);
    assert!(out.status.success());
    assert_eq!(out.stdout, format!("{body}\n").as_bytes());
}

#[test]
fn raw_responses_preserve_non_utf8_bytes() {
    let (out, _) = request_bytes(&["issues", "list", "--json"], 200, b"raw\xff");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"raw\xff\n");
    let (out, _) = request_bytes(&["issues", "list", "--json"], 500, b"raw\xff");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(out.stderr, b"Error: API returned 500\nraw\xff\n");
}
#[test]
#[cfg(unix)]
fn output_write_failures_exit_without_panicking() {
    use std::{
        os::{fd::OwnedFd, unix::net::UnixStream},
        process::Stdio,
    };
    let (writer, reader) = UnixStream::pair().unwrap();
    drop(reader);
    let fd: OwnedFd = writer.into();
    let out = Command::new(env!("CARGO_BIN_EXE_goodissues"))
        .arg("--help")
        .stdout(Stdio::from(fd))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(!String::from_utf8_lossy(&out.stderr).contains("panicked"));
}

#[test]
fn malformed_success_payloads_keep_zig_failure_and_fallback_behavior() {
    for body in [r#"{"data":null}"#, r#"{"data":{"id":123}}"#] {
        let (out, _) = request(&["projects", "get", "id"], 200, body);
        assert_eq!(out.status.code(), Some(1));
    }
    let body = r#"{"data":{"id":123}}"#;
    let (out, _) = request(&["errors", "get", "id"], 200, body);
    assert!(out.status.success());
    assert_eq!(out.stdout, format!("{body}\n").as_bytes());
}

#[test]
fn heartbeat_signals_use_the_url_token_without_an_api_key() {
    for operation in ["ping", "start", "fail"] {
        let (out, wire) = request_with_key(
            &["heartbeats", operation, "token", "--project=proj"],
            204,
            b"",
            None,
        );
        assert!(
            out.status.success(),
            "{operation}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(wire.starts_with("POST /api/v1/projects/proj/heartbeats/token/ping"));
        assert!(!wire.to_lowercase().contains("authorization:"));
    }
}

#[test]
fn authenticated_operations_still_require_an_api_key() {
    let (out, wire) = request_with_key(&["heartbeats", "list", "--project=proj"], 200, b"{}", None);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("no API key configured"));
    assert!(wire.is_empty());
}
