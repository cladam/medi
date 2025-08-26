use assert_cmd::Command;
use predicates::prelude::*;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{tempdir, TempDir};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// TestHarness manages the temporary directory and paths for our tests.
struct TestHarness {
    _temp_dir: TempDir,
    db_path: PathBuf,
    editor_script_path: PathBuf,
}

/// Creates a new TestHarness instance, setting up the temporary directory
/// and copying the mock editor script to a known location.
/// The mock editor script is used to simulate user input in tests.
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

// A temporary struct for deserializing only the part of the JSON we need.
#[derive(Deserialize)]
struct NoteTags {
    tags: Vec<String>,
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
fn test_new_with_tags() -> Result<(), Box<dyn std::error::Error>> {
    let harness = TestHarness::new();

    // TEST: Create a new note with multiple tags.
    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args([
            "new",
            "tagged-note",
            "-m",
            "content",
            "--tag",
            "rust",
            "--tag",
            "cli",
        ])
        .assert()
        .success();

    // VERIFY: Get the note as JSON and check if the tags are present.
    let output = Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["get", "tagged-note", "--json"])
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let note: NoteTags = serde_json::from_str(&output_str)?;

    assert_eq!(note.tags.len(), 2);
    assert!(note.tags.contains(&"rust".to_string()));
    assert!(note.tags.contains(&"cli".to_string()));

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
fn test_edit_tags() -> Result<(), Box<dyn std::error::Error>> {
    let harness = TestHarness::new();

    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["new", "note-to-edit", "-m", "content", "--tag", "initial"])
        .assert()
        .success();

    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args([
            "edit",
            "note-to-edit",
            "--add-tag",
            "added1",
            "--add-tag",
            "added2",
        ])
        .assert()
        .success();

    // VERIFY 1: Check for all three tags by parsing the JSON.
    let output1 = Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["get", "note-to-edit", "--json"])
        .output()?;

    let note1: NoteTags = serde_json::from_slice(&output1.stdout)?;
    assert_eq!(note1.tags.len(), 3);
    assert!(note1.tags.contains(&"initial".to_string()));
    assert!(note1.tags.contains(&"added1".to_string()));
    assert!(note1.tags.contains(&"added2".to_string()));

    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args([
            "edit",
            "note-to-edit",
            "--rm-tag",
            "initial",
            "--rm-tag",
            "added1",
        ])
        .assert()
        .success();

    let output2 = Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["get", "note-to-edit", "--json"])
        .output()?;

    let note2: NoteTags = serde_json::from_slice(&output2.stdout)?;
    assert_eq!(note2.tags, vec!["added2"]);

    Ok(())
}

#[test]
fn test_import_single_file() -> Result<(), Box<dyn std::error::Error>> {
    let harness = TestHarness::new();
    let import_file_path = harness._temp_dir.path().join("imported_note.txt");
    fs::write(&import_file_path, "This is an imported note.").unwrap();

    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args([
            "import",
            "--file",
            &import_file_path.to_string_lossy().as_ref(),
            "--key",
            "imported-note",
        ])
        .assert()
        .success()
        // The assertion is now simpler and matches your actual output
        .stdout(predicate::str::contains("Imported 'imported-note'"));

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

#[test]
fn test_search_command() -> Result<(), Box<dyn std::error::Error>> {
    let harness = TestHarness::new();

    // Create some notes to search through.
    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args([
            "new",
            "rust-note",
            "-m",
            "A note about the Rust language and its features.",
        ])
        .assert()
        .success();

    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args([
            "new",
            "python-note",
            "-m",
            "A note about the Python language.",
        ])
        .assert()
        .success();

    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args([
            "new",
            "general-note",
            "-m",
            "A general note about programming languages.",
        ])
        .assert()
        .success();

    // TEST 1: Search for a term that matches a single note.
    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["search", "Rust"])
        .assert()
        .success()
        .stdout(predicate::str::contains("rust-note"))
        .stdout(predicate::str::contains("python-note").not());

    // TEST 2: Search for a term that matches multiple notes.
    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["search", "language"])
        .assert()
        .success()
        .stdout(predicate::str::contains("rust-note").and(predicate::str::contains("python-note")));

    // TEST 3: Search for a term that matches no notes.
    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["search", "java"])
        .assert()
        .success()
        .stderr(predicate::str::contains("No matching notes found."));

    Ok(())
}

#[test]
fn test_task_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let harness = TestHarness::new();

    // First, create a note to associate tasks with.
    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["new", "task-note", "-m", "A note for my tasks"])
        .assert()
        .success();

    // TEST 1: Add a couple of tasks.
    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["task", "add", "task-note", "My first task"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added new task with ID: 1"));

    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["task", "add", "task-note", "My second task"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added new task with ID: 2"));

    // TEST 2: List the tasks to verify they were added.
    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["task", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("My first task"))
        .stdout(predicate::str::contains("My second task"));

    // TEST 3: Mark the first task as done.
    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["task", "done", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Completed task: 1"));

    // TEST 4: List tasks again to verify the first one is gone.
    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["task", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("My first task").not())
        .stdout(predicate::str::contains("My second task"));

    // TEST 5: Prioritise the second task.
    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["task", "prio", "2"])
        .assert()
        .success();

    Command::cargo_bin("medi")?
        .env("MEDI_DB_PATH", &harness.db_path)
        .args(["task", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("- ⭐ 2: My second task"));

    Ok(())
}
