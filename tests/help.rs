use std::process::Command;
#[test]
fn help_and_version_work_without_configuration() {
    for command in [
        "projects",
        "issues",
        "errors",
        "incidents",
        "checks",
        "heartbeats",
        "cloud-ip-ranges",
        "configure",
    ] {
        let a = Command::new(env!("CARGO_BIN_EXE_goodissues"))
            .args(["help", command])
            .output()
            .unwrap();
        let b = Command::new(env!("CARGO_BIN_EXE_goodissues"))
            .args([command, "get", "--help"])
            .output()
            .unwrap();
        assert!(a.status.success());
        assert!(b.status.success());
        assert_eq!(a.stdout, b.stdout);
        assert!(String::from_utf8_lossy(&a.stdout).contains(&format!("goodissues {command}")));
    }
    let version = Command::new(env!("CARGO_BIN_EXE_goodissues"))
        .arg("--version")
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&version.stdout),
        format!("goodissues {}\n", env!("CARGO_PKG_VERSION"))
    );
    let empty = Command::new(env!("CARGO_BIN_EXE_goodissues"))
        .output()
        .unwrap();
    assert_eq!(empty.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&empty.stderr).contains("Usage:"));
}
