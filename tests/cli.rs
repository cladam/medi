use std::fs;
use std::path::Path;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::tempdir;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[test]
fn test_new_command() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_db");
    let editor_script_path = temp_dir.path().join("mock_editor.sh");

    let source_script_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/resources/mock_editor.sh");

    fs::copy(source_script_path, &editor_script_path)?;

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&editor_script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&editor_script_path, perms)?;
    }

    let mut cmd = Command::cargo_bin("medi")?;
    cmd.env("EDITOR", &editor_script_path);
    cmd.env("MEDI_DB_PATH", &db_path);

    cmd.arg("new").arg("test-note");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Successfully created note"));

    // Get is not implemented yet, so we will not run it here.
    let mut get_cmd = Command::cargo_bin("medi")?;
    get_cmd.env("MEDI_DB_PATH", &db_path);
    get_cmd.arg("get").arg("test-note");

    get_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("integration test content"));

    Ok(())
}

#[test]
fn test_list_command() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_db_list");

    let editor_script_path = temp_dir.path().join("mock_editor.sh");

    let source_script_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/resources/mock_editor.sh");

    fs::copy(source_script_path, &editor_script_path)?;

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&editor_script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&editor_script_path, perms)?;
    }

    // Create note "b"
    Command::cargo_bin("medi")?
        .env("EDITOR", &editor_script_path)
        .env("MEDI_DB_PATH", &db_path)
        .args(["new", "b-note"])
        .assert()
        .success();

    // Create note "a"
    Command::cargo_bin("medi")?
        .env("EDITOR", &editor_script_path)
        .env("MEDI_DB_PATH", &db_path)
        .args(["new", "a-note"])
        .assert()
        .success();

    // 3. Run `medi list` and check the output.
    let mut cmd = Command::cargo_bin("medi")?;
    cmd.env("MEDI_DB_PATH", &db_path);
    cmd.arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::is_match("(?s)a-note.*b-note").unwrap(),
        );
    Ok(())
}

#[test]
fn test_list_command_empty() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_db_empty");

    let mut cmd = Command::cargo_bin("medi")?;
    cmd.env("MEDI_DB_PATH", &db_path);
    cmd.arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("No notes found."));

    Ok(())
}

#[test]
fn test_delete_command() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_db_delete");

    let editor_script_path = temp_dir.path().join("mock_editor.sh");
    let source_script_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/resources/mock_editor.sh");

    fs::copy(source_script_path, &editor_script_path)?;

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&editor_script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&editor_script_path, perms)?;
    }

    // Create a note to delete
    Command::cargo_bin("medi")?
        .env("EDITOR", &editor_script_path)
        .env("MEDI_DB_PATH", &db_path)
        .args(["new", "delete-me"])
        .assert()
        .success();
    // Delete the note
    let mut cmd = Command::cargo_bin("medi")?;
    cmd.env("MEDI_DB_PATH", &db_path);
    cmd.arg("delete").arg("delete-me");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Successfully deleted note"));
    // Try to get the deleted note
    let mut get_cmd = Command::cargo_bin("medi")?;
    get_cmd.env("MEDI_DB_PATH", &db_path);
    get_cmd.arg("get").arg("delete-me");
    get_cmd.assert()
        .failure()
        .stderr(predicate::str::contains(" Key 'delete-me' not found in the database"));
    Ok(())
}