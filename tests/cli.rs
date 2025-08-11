use std::fs;
use std::path::{Path, PathBuf};
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::{tempdir, TempDir};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

// A test harness to manage setup and teardown
struct TestHarness {
    _temp_dir: TempDir, // The underscore silences a warning, but TempDir cleans up on drop
    db_path: PathBuf,
    editor_script_path: PathBuf,
}

impl TestHarness {
    fn new() -> Self {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let editor_script_path = temp_dir.path().join("mock_editor.sh");

        // Copy the mock editor script
        let source_script_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/resources/mock_editor.sh");
        fs::copy(source_script_path, &editor_script_path).unwrap();

        // Make it executable
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

    // A helper to get a pre-configured Command for running `medi`
    fn medi(&self) -> Command {
        let mut cmd = Command::cargo_bin("medi").unwrap();
        cmd.env("EDITOR", &self.editor_script_path);
        cmd.env("MEDI_DB_PATH", &self.db_path);
        cmd
    }
}

#[test]
fn test_new_command_new() -> Result<(), Box<dyn std::error::Error>> {
    let harness = TestHarness::new();

    // Run `medi new` with a note key
    harness
        .medi()
        .args(["new", "test-note"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully created note"));

    // Check if the note was created
    harness
        .medi()
        .args(["get", "test-note"])
        .assert()
        .success()
        .stdout(predicate::str::contains("integration test content"));

    Ok(())
}

#[test]
fn test_list_command() -> Result<(), Box<dyn std::error::Error>> {
    let harness = TestHarness::new();

    // Create note "b"
    harness
        .medi()
        .args(["new", "b-note"])
        .assert()
        .success();
    // Create note "a"
    harness
        .medi()
        .args(["new", "a-note"])
        .assert()
        .success();
    // Run `medi list` and check the output
    harness
        .medi()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?s)a-note.*b-note").unwrap());
    // Create a note to list
    harness
        .medi()
        .args(["new", "test-list"])
        .assert()
        .success();

    // Run `medi list` and check the output
    harness
        .medi()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-list"));

    Ok(())
}

#[test]
fn test_list_command_empty() -> Result<(), Box<dyn std::error::Error>> {
    let harness = TestHarness::new();
    // Run `medi list` on an empty database
    harness
        .medi()
        .arg("list")
        .assert()
        .success()
        .stderr(predicate::str::contains("No notes found"));
    Ok(())
}

#[test]
fn test_delete_command() -> Result<(), Box<dyn std::error::Error>> {
    let harness = TestHarness::new();
    // Create a note to delete
    harness
        .medi() // Get a pre-configured command
        .args(["new", "delete-me"])
        .assert()
        .success();

    // Delete the note
    harness
        .medi()
        .args(["delete", "delete-me"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully deleted note"));

    // Try to get the deleted note
    harness
        .medi()
        .arg("get")
        .arg("delete-me")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Key 'delete-me' not found"));

    Ok(())
}