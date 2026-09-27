pub fn text(command: &str) -> &'static str {
    match command {
        "projects" => include_str!("../help/projects.txt"),
        "issues" => include_str!("../help/issues.txt"),
        "errors" => include_str!("../help/errors.txt"),
        "incidents" => include_str!("../help/incidents.txt"),
        "checks" => include_str!("../help/checks.txt"),
        "heartbeats" => include_str!("../help/heartbeats.txt"),
        "cloud-ip-ranges" => include_str!("../help/cloud_ip_ranges.txt"),
        "configure" => include_str!("../help/configure.txt"),
        _ => include_str!("../help/root.txt"),
    }
}
