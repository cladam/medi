<div align="center">

# medi ✒️

[![Crates.io](https://img.shields.io/crates/v/medi.svg)](https://crates.io/crates/medi)
[![Downloads](https://img.shields.io/crates/d/medi.svg)](https://crates.io/crates/medi)

</div>


## `medi`, a speedy CLI driven Markdown manager

**A fast, editor-centric, command-line notes manager.**

`medi` is a simple and powerful tool for creating and managing your notes, articles, and documentation directly from the terminal. It's built for developers, writers, and anyone who loves the speed and focus of a command-line workflow.

## Core Philosophy

`medi` is built on a few guiding principles:

- **CLI-first**: Everything is done through the terminal. No GUIs or TUIs, no distractions.
- **Editor-centric**: Your text editor (`$EDITOR`) is where you write. `medi` gets you there quickly and saves your work securely.
- **Local & private**: All content is stored on your local machine in a high-performance embedded database. No cloud services, no network access.
- **Zero-config start**: Install it and start writing immediately.

## Features ✨

* **Speed**: Instant access to any note, no matter how large your collection grows, thanks to an embedded database index.
* **Focused Workflow**: A command-line hub for your writing. Create or edit notes from any directory without needing to `cd` first.
* **Flexible Input**: Create notes in the way that suits your workflow:
  * Interactively in your favorite editor for long-form content.
  * Instantly with a one-liner using the `-m` flag for quick thoughts.
  * Powerfully by piping from other commands for scripting.
* **Safe Deletion**: An interactive confirmation prompt on `delete` prevents you from accidentally losing work.
* **Colorful & Clear Output**: Uses colored output to clearly distinguish between success messages, information, warnings, and errors.

## How It Works, DB as Source-of-Truth

`medi` uses a database-first approach. All your notes are stored in a `sled` key-value database, making access fast and reliable. This database is the single source of truth.

To version control your work with Git, `medi` provides a simple and deliberate workflow:

- **Write**: Use `medi new` and `medi edit` to manage your notes.
- **Export**: Run `medi export ./my-notes` to write all your notes to a local directory as `.md` files.
- **Commit**: Use Git to commit the exported directory, giving you a complete, version-controlled snapshot of your work.

### Installation

You need [Rust and Cargo](https://www.rust-lang.org/tools/install) installed.

#### Installing from crates.io

The easiest way to install `medi` is to download it from [crates.io](https://crates.io/crates/medi). You can do it using the following command:

```bash
cargo install medi
```

If you want to update `medi` to the latest version, execute the following command:

```bash
medi update
```

#### Building from source

Alternatively you can build `medi` from source using Cargo:

```bash
git clone https://github.com/cladam/medi.git
cd medi
sudo cargo install --path . --root /usr/local
```


## Usage

### Creating and Editing Notes

* **Create a new note**

  `medi` provides three ways to create a new note:

  1.  **Interactively (default)**: Opens your default editor for long-form content.
      ```bash
      medi new "my-long-article"
      # With tags: Add tags to your note for better organisation.
      medi new "my-long-article" --tag tag1 --tag tag2
      # With a title: Specify a title for your note.
      medi new "my-long-article" --title "My Long Article"
      ```

  2.  **With a direct message**: Perfect for quick, one-line notes.
      ```bash
      medi new quick-idea -m "Remember to buy milk"
      ```

  3.  **From a pipe**: Use the output of other commands as your note content.
      ```bash
      echo "This is a note from a pipe" | medi new piped-note
      ```

* **Edit an existing note**
  ```bash
  medi edit "my-long-article"
  
  # Add tags to a note
  medi edit "my-long-article" --add-tag tag1 --add-tag tag2
  
  # Remove tags from a note
  medi edit "my-long-article" --rm-tag tag1 --rm-tag tag2
  ```

### Viewing and listing notes

- **Get a note's content**
  Prints the note directly to the console. This is perfect for piping to other tools.

  ```bash
  medi get "my-first-article"

  # Pipe to a Markdown renderer like mdcat
  medi get "my-first-article" | mdcat

  # Get a note in Json format
  medi get "my-first-article" --json

  # Get one or several notes via a tag
  medi get --tag my-tag
  ```

- **List all note keys**

  ```bash
  medi list
  ```

  _Output:_

  ```
  Notes:
  - my-first-article
  - another-note
  ```

### Deleting a Note

- **Delete a note**
You will be prompted for confirmation.

```bash
medi delete "my-first-article"

# Skip the confirmation prompt
medi delete "my-first-article" --force
```

### Versioning with Export/Import

- **Export all notes to a directory**
  Creates a version-controllable snapshot of your database.

  ```bash
  medi export ./my_notes_backup
  cd my_notes_backup
  tbdflow init
  ```
- **Export all notes to a Json document**

  ```bash
  medi export medi-export --format json
  ```
- **Export notes via a tag**

  ```bash
  medi export medi-export --tag my-tag
  ```

- **Import notes from a directory**
  Restores notes from a directory of .md files.

  ```bash
  # By default, skips any notes that already exist
  medi import --dir /path/to/notes

  # Import a single
  medi import --file /path/to/note.md --key my-note

  # Overwrite existing notes with the imported versions
  medi import --file /path/to/note.md --key my-note --overwrite
  ```

### Shell Completion

To make `medi` even faster to use, you can enable shell completion. Add one of the following lines to your shell's configuration file.

For Zsh (`~/.zshrc`):

```bash
eval "$(medi generate-completion zsh)"
```

For Bash (`~/.bashrc`):

```bash
eval "$(medi generate-completion bash)"
```

For Fish (`~/.config/fish/config.fish`):

```bash
medi generate-completion fish | source
```

## Project Roadmap 🗺️

This section tracks the implementation status of `medi`'s features. Contributions are welcome!

- [x] All core commands (new, get, list, edit, delete, import, export).
- [ ] Configuration file for settings (e.g., database path).
- [x] Support for note metadata (tags, creation dates).
- [x] `export` notes by tag.
- [ ] Full-text search over note content.
- [ ] Implement a `task` command for tracking the status of notes.

