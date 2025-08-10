use std::fs;
use std::os::unix::fs::PermissionsExt;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_new_command() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_db");
    let editor_script_path = temp_dir.path().join("mock_editor.sh");

    let script_content = r#"
#!/bin/sh
# This mock editor script writes content to the file provided by `medi`.
# The file path is passed as the first argument ($1).
echo "integration test content" > "$1"
    "#;
    fs::write(&editor_script_path, script_content)?;

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
/*    let mut get_cmd = Command::cargo_bin("medi")?;
    get_cmd.env("MEDI_DB_PATH", &db_path);
    get_cmd.arg("get").arg("test-note");

    get_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("integration test content"));
*/
    Ok(())
}