use std::process::Command;
use tempfile::TempDir;

#[test]
fn missing_option_values_fail_before_loading_credentials() {
    let home = TempDir::new().unwrap();
    for args in [
        vec!["projects", "list", "--env"],
        vec!["projects", "list", "--query"],
        vec!["issues", "list", "--status", "--json"],
        vec!["configure", "--url"],
        vec!["configure", "--api-key", "--env=qa"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_goodissues"))
            .args(&args)
            .env("HOME", home.path())
            .env("USERPROFILE", home.path())
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1), "{args:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("requires a value"),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!home.path().join(".goodissues.json").exists());
    }
}

#[test]
fn unknown_and_inapplicable_options_are_rejected() {
    let home = TempDir::new().unwrap();
    for args in [
        vec!["projects", "list", "--typo"],
        vec!["issues", "list", "--priority=high"],
        vec!["issues", "update", "id", "--stats=new"],
        vec!["projects", "list", "--json=true"],
        vec!["configure", "show", "--url=http://localhost"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_goodissues"))
            .args(&args)
            .env("HOME", home.path())
            .env("USERPROFILE", home.path())
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1), "{args:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("Unknown option"),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
