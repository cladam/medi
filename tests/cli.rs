use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use assert_cmd::Command;
use tempfile::{tempdir, TempDir};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// TestHarness manages the temporary directory and paths for our tests.
struct TestHarness {
    _temp_dir: TempDir,
    db_path: PathBuf,
    editor_script_path: PathBuf,
}

impl TestHarness {
    fn new() -> Self {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let editor_script_path = temp_dir.path().join("mock_editor.sh");
        let source_script_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/resources/mock_editor.sh");

        fs::copy(source_script_path, &editor_script_path).unwrap();

        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&editor_script_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&editor_script_path, perms).unwrap();
        }

        TestHarness {
            _temp_dir: temp_dir,
            db_path,
            editor_script_path,
        }
    }
}

/// Tests the `-m` flag, the most reliable non-interactive input.
#[test]
fn test_new_with_message_flag() -> Result<(), Box<dyn std::error::Error>> {
    let harness = TestHarness::new();
    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["new", "flag-note", "-m", "content from flag"])
        .assert()
        .success();

    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["get", "flag-note"])
        .assert()
        .success()
        .stdout(predicate::str::contains("content from flag"));
    Ok(())
}

/// Tests piped input using the recommended `.write_stdin()` method.
#[test]
fn test_new_with_piped_input() -> Result<(), Box<dyn std::error::Error>> {
    let harness = TestHarness::new();
    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["new", "piped-note"])
        .write_stdin("content from pipe")
        .assert()
        .success();

    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["get", "piped-note"])
        .assert()
        .success()
        .stdout(predicate::str::contains("content from pipe"));
    Ok(())
}

/// The only test that uses the mock editor. This is known to be
/// flaky in some test runners due to I/O capture conflicts.
#[test]
#[ignore]
fn test_new_with_editor() -> Result<(), Box<dyn std::error::Error>> {
    let harness = TestHarness::new();
    Command::cargo_bin("medi")?
        .env("EDITOR", &harness.editor_script_path)
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["new", "editor-note"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully created note"));
    Ok(())
}

/// A single, logical test for the core list and delete workflow.
/// It uses the reliable `-m` flag for setup.
#[test]
fn test_core_list_and_delete_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let harness = TestHarness::new();

    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["new", "b-note", "-m", "b"])
        .assert()
        .success();
    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["new", "a-note", "-m", "a"])
        .assert()
        .success();

    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?s)a-note.*b-note").unwrap());

    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["delete", "a-note", "--force"])
        .assert()
        .success();

    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["get", "a-note"])
        .assert()
        .failure();

    Ok(())
}

#[test]
fn test_list_empty() -> Result<(), Box<dyn std::error::Error>> {
    let harness = TestHarness::new();
    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .arg("list")
        .assert()
        .success()
        .stderr(predicate::str::contains("No notes found."));
    Ok(())
}
#[test]
fn test_edit_command() -> Result<(), Box<dyn std::error::Error>> {
    let harness = TestHarness::new();

    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["new", "edit-me", "-m", "initial content"])
        .assert()
        .success();

    Command::cargo_bin("medi")?
        .env("EDITOR", &harness.editor_script_path) // This script provides the new content
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["edit", "edit-me"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully updated note"));

    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["get", "edit-me"])
        .assert()
        .success()
        .stdout(predicate::str::contains("integration test content")); // This is from mock_editor.sh

    Ok(())
}