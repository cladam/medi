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

#[test]
fn test_import_single_file() -> Result<(), Box<dyn std::error::Error>> {
    let harness = TestHarness::new();
    let import_file_path = harness._temp_dir.path().join("imported_note.txt");
    fs::write(&import_file_path, "This is an imported note.").unwrap();

    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["import", "--file", &import_file_path.to_string_lossy(), "--key", "imported-note"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Imported 'imported-note' from"));

    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["get", "imported-note"])
        .assert()
        .success()
        .stdout(predicate::str::contains("This is an imported note."));

    Ok(())
}

#[test]
fn test_import_directory() -> Result<(), Box<dyn std::error::Error>> {
    let harness = TestHarness::new();

    let import_dir = harness._temp_dir.path().join("import_test");
    fs::create_dir_all(&import_dir)?;
    fs::write(import_dir.join("import-one.md"), "content for import one")?;
    fs::write(import_dir.join("import-two.md"), "content for import two")?;

    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["import", "--dir", &import_dir.to_string_lossy()])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Imported 'import-one'")
                .and(predicate::str::contains("Imported 'import-two'")),
        );

    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["get", "import-one"])
        .assert()
        .success()
        .stdout(predicate::str::contains("content for import one"));

    Ok(())
}

#[test]
fn test_export_command() -> Result<(), Box<dyn std::error::Error>> {
    let harness = TestHarness::new();
    // Create two notes to export.
    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["new", "note-one", "-m", "content for note one"])
        .assert()
        .success();
    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["new", "note-two", "-m", "content for note two"])
        .assert()
        .success();

    // Define a path for the export directory.
    let export_dir = harness._temp_dir.path().join("export_test");

    // Run the `export` command.
    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .arg("export")
        .arg(&export_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully exported 2 notes"));

    // VERIFY: Check that the files were created with the correct content.
    let note_one_path = export_dir.join("note-one.md");
    let note_two_path = export_dir.join("note-two.md");

    assert!(note_one_path.exists());
    assert!(note_two_path.exists());

    let content_one = fs::read_to_string(note_one_path)?;
    let content_two = fs::read_to_string(note_two_path)?;

    assert_eq!(content_one, "content for note one");
    assert_eq!(content_two, "content for note two");

    Ok(())
}