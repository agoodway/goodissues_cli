#!/usr/bin/env python3
"""Compare the Rust and Zig processes against the same isolated HTTP fixture.

Usage: python3 scripts/parity.py /path/to/rust/goodissues /path/to/zig/goodissues
No live API or existing user configuration is used.
"""
import http.server
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import threading

RUST, ZIG = [str(Path(p).resolve()) for p in sys.argv[1:]]
RESOURCE = {"id": "id", "key": "APP-1", "name": "API", "title": "Login é", "description": "line one", "status": "new", "priority": "medium", "type": "bug", "project_id": "proj", "submitter_email": "a@b.co", "inserted_at": "2026-01-01", "updated_at": "2026-01-02", "issue_id": "issue", "kind": "exception", "reason": "boom", "source_function": "run", "source_line": "app.ex:9", "muted": False, "fingerprint": "fp", "last_occurrence_at": "2026-01-02", "occurrence_count": 8, "occurrences": [{"reason": "boom", "trace_id": "trace", "inserted_at": "2026-01-02", "stacktrace": {"lines": [{"module": "M", "function": "f", "arity": 1, "file": "app.ex", "line": i} for i in range(7)]}}]}

class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        body = self.rfile.read(int(self.headers.get("Content-Length", 0))).decode()
        try:
            body = json.loads(body)
        except ValueError:
            pass
        self.server.requests.append((self.command, self.path, self.headers.get("Authorization"), self.headers.get("Content-Type"), body))
        response = self.server.fixture.encode()
        self.send_response(self.server.status)
        self.send_header("Content-Length", str(len(response)))
        self.end_headers()
        self.wfile.write(response)
    do_POST = do_PATCH = do_DELETE = do_GET
    def log_message(self, *args):
        pass

count = 0

def compare(args, status=200, data=None, fixture=None, configured=True, input="", exact_error=True):
    global count
    if fixture is None:
        fixture = json.dumps({"data": RESOURCE if data is None else data}) if status != 204 else ""
    with http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler) as server, tempfile.TemporaryDirectory() as tmp:
        server.fixture, server.status, server.requests = fixture, status, []
        thread = threading.Thread(target=server.serve_forever, kwargs={"poll_interval": 0.01}, daemon=True)
        thread.start()
        outputs = []
        for index, binary in enumerate([RUST, ZIG]):
            home = Path(tmp) / str(index)
            home.mkdir()
            if configured:
                (home / ".goodissues.json").write_text(json.dumps({"default_env": "fixture", "environments": [{"name": "fixture", "base_url": f"http://127.0.0.1:{server.server_port}", "api_key": "sk_test"}]}))
            env = dict(os.environ, HOME=str(home), USERPROFILE=str(home), NO_PROXY="127.0.0.1")
            result = subprocess.run([binary, *args], input=input, capture_output=True, text=True, env=env, timeout=10)
            outputs.append((result.returncode, result.stdout, result.stderr if exact_error else bool(result.stderr)))
        server.shutdown()
        thread.join()
        # Zig 0.15.2 panics before sending POSTs with no payload. Rust must
        # implement these documented operations, not reproduce the crash.
        bodiless_post = (args[:2] == ["incidents", "resolve"] or
                         (len(args) >= 2 and args[0] == "heartbeats" and args[1] in ["ping", "fail", "start"] and not any(a.startswith("--body") for a in args)))
        if bodiless_post and "sendBodilessUnflushed" in str(outputs[1][2]):
            assert outputs[0][0] == 0 and len(server.requests) == 1, (args, outputs)
            assert server.requests[0][0] == "POST", server.requests
            count += 1
            return
        help_command = args[1] if args[:1] == ["help"] and len(args) > 1 else args[0] if args and args[-1] == "--help" else None
        if help_command in ["heartbeats", "errors", "incidents", "cloud-ip-ranges"]:
            old_help = outputs[1][1]
            if help_command == "cloud-ip-ranges":
                old_help = old_help.replace("abc123", "00000000-0000-4000-8000-000000000001")
            assert outputs[0][0] == outputs[1][0] == 0 and outputs[0][1].startswith(old_help), (args, outputs)
        else:
            assert outputs[0] == outputs[1], (args, outputs)
        assert len(server.requests) in (0, 2), (args, server.requests)
        if server.requests:
            assert server.requests[0] == server.requests[1], (args, server.requests)
        count += 1

for resource in ["projects", "issues", "errors", "incidents", "checks", "heartbeats", "cloud-ip-ranges"]:
    scoped = ["--project=proj"] if resource in ["checks", "heartbeats"] else []
    for json_args in [[], ["--json"]]:
        for operation in [[], ["list"]]:
            compare([resource, *operation, *scoped, *json_args], data=[RESOURCE])
        compare([resource, "list", *scoped, "--query=page=2", *json_args], data=[])
        if resource == "cloud-ip-ranges":
            compare([resource, "sync-state", *json_args])
            continue
        compare([resource, "get", *json_args, "id", *scoped])
        for operation in ["create", "update"]:
            compare([resource, operation, *([] if operation == "create" else ["id"]), *scoped, '--body={"name":"Hi"}', *json_args], status=201 if operation == "create" else 200)
        if resource in ["projects", "issues", "checks", "heartbeats"]:
            for status in [200, 204]:
                compare([resource, "delete", "id", *scoped, *json_args], status=status)
        else:
            compare([resource, "report", "--body={}", *json_args], status=200)
    compare([resource, "list", *scoped], status=403, fixture='{"error":"denied"}')
    compare([resource, "list", *scoped], status=500, fixture='server error')
    compare([resource, "list", *scoped, "--env=missing"])
    compare([resource, "list", *scoped], configured=False)

for operation in ["ping", "fail", "start"]:
    for status in [200, 204]:
        for body in [[], ["--body={}"]]:
            compare(["heartbeats", operation, "token", "--project=proj", *body], status=status)
for resource, operation in [("checks", "results"), ("heartbeats", "pings")]:
    compare([resource, operation, "id", "--project=proj", "--query=page=2"], data=[])
compare(["incidents", "resolve", "id"])
for args in [
    ["issues", "list", "--project=a b", "--status=new", "--type=bug", "--page=2", "--per-page=10"],
    ["errors", "list", "--status=unresolved", "--muted=false", "--page=3", "--per-page=20"],
    ["errors", "search", "--module=My App", "--function=f/1", "--file=lib/é.ex", "--page=2", "--per-page=9"],
    ["errors", "search", "--query=module=M"],
    ["errors", "list", "--muted=no", "--query=muted=true"],
    ["cloud-ip-ranges", "list", "--snapshot-id=snap", "--page=2", "--per-page=100"],
]:
    compare(args, data=[])
for args, status in [
    (["projects", "create", '--name=Say "hello"', "--prefix=APP", "--description=é\nnext"], 201),
    (["issues", "create", "--project=p", "--title=Hi", "--type=bug"], 201),
    (["issues", "create", "--project=p", "--title=Hi", "--type=bug", "--status=archived", "--priority=high", "--email=a@b.co", "--description=test"], 201),
    (["projects", "update", "id", "--name=New", "--prefix=NEW", "--description=Changed"], 200),
    (["issues", "update", "id", "--title=New", "--status=archived", "--priority=high", "--type=incident", "--email=a@b.co", "--description=Changed"], 200),
    (["errors", "update", "id", "--status=resolved", "--muted=false"], 200),
]:
    compare(args, status=status)
for args in [
    ["projects", "create", "--name=Hi"], ["projects", "update", "id"],
    ["issues", "create"], ["issues", "update", "id"],
    ["errors", "search"], ["errors", "list", "--muted=no"], ["errors", "update", "id", "--muted=1"],
    ["errors", "update", "id"], ["checks", "list"], ["heartbeats", "ping", "token"],
]:
    compare(args)
for resource in ["projects", "issues", "errors", "incidents", "checks", "heartbeats", "cloud-ip-ranges", "configure"]:
    compare(["help", resource], configured=False)
    compare([resource, "--help"], configured=False)
for args in [[], ["--help"], ["-h"], ["--version"], ["-v"], ["help", "unknown"]]:
    compare(args, configured=False)
print(f"{count} differential cases passed")
