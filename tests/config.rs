use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
};
use tempfile::TempDir;
fn cli(home: &TempDir, args: &[&str], input: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_goodissues"))
        .args(args)
        .env("HOME", home.path())
        .env("USERPROFILE", home.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}
#[test]
fn named_config_preserves_default_and_masks_keys() {
    let h = TempDir::new().unwrap();
    let out = cli(
        &h,
        &[
            "configure",
            "--env=dev",
            "--url",
            "http://localhost:9999",
            "--api-key",
            "sk_1234567890",
        ],
        "",
    );
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "Configuration saved for environment 'dev'.\n"
    );
    assert!(
        cli(
            &h,
            &["configure", "--env", "prod", "--api-key", "sk_other"],
            ""
        )
        .status
        .success()
    );
    let data: serde_json::Value =
        serde_json::from_slice(&fs::read(h.path().join(".goodissues.json")).unwrap()).unwrap();
    assert_eq!(data["default_env"], "dev");
    assert_eq!(data["environments"][0]["api_key"], "sk_1234567890");
    let show = cli(&h, &["configure", "show"], "");
    let text = String::from_utf8_lossy(&show.stdout);
    assert!(text.contains("Default: dev\n"));
    assert!(text.contains("sk_1****7890"));
    assert!(!text.contains("sk_1234567890"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(h.path().join(".goodissues.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}
#[test]
fn interactive_config_reads_both_lines_and_keeps_existing_values() {
    let h = TempDir::new().unwrap();
    assert!(
        cli(&h, &["configure"], "http://localhost:8123\nsk_example\n")
            .status
            .success()
    );
    assert!(cli(&h, &["configure"], "\n\n").status.success());
    let data: serde_json::Value =
        serde_json::from_slice(&fs::read(h.path().join(".goodissues.json")).unwrap()).unwrap();
    assert_eq!(data["environments"][0]["base_url"], "http://localhost:8123");
    assert_eq!(data["environments"][0]["api_key"], "sk_example");
}
#[test]
fn imports_legacy_config_only_when_json_missing() {
    let h = TempDir::new().unwrap();
    fs::create_dir(h.path().join(".goodissues")).unwrap();
    fs::write(
        h.path().join(".goodissues/config.yaml"),
        "base_url: 'http://localhost:4444'\napi_key: sk_old\n",
    )
    .unwrap();
    let out = cli(&h, &["configure", "show"], "");
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("http://localhost:4444"));
    fs::write(
        h.path().join(".goodissues/config.yaml"),
        "api_key: sk_new\n",
    )
    .unwrap();
    assert!(cli(&h, &["configure", "show"], "").status.success());
    let data: serde_json::Value =
        serde_json::from_slice(&fs::read(h.path().join(".goodissues.json")).unwrap()).unwrap();
    assert_eq!(data["environments"][0]["api_key"], "sk_old");
}
