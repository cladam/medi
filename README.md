# medi ✒️

**A fast, editor-centric, command-line notes manager.**

`medi` is a simple and powerful tool for creating and managing your notes, articles, and documentation directly from the terminal. It's built for developers, writers, and anyone who loves the speed and focus of a command-line workflow.

## Core Philosophy

medi is built on a few guiding principles:

- **CLI-first**: Everything is done through the terminal. No GUIs or TUIs, no distractions.
- **Editor-centric**: Your text editor (`$EDITOR`) is where you write. `medi` gets you there quickly and saves your work securely.
- **Local & private**: All content is stored on your local machine in a high-performance embedded database. No cloud services, no network access.
- **Zero-config start**: Install it and start writing immediately.

## How It Works, DB as Source-of-Truth

`medi` uses a database-first approach. All your notes are stored in a `sled` key-value database, making access fast and reliable. This database is the single source of truth.

To version control your work with Git, `medi` provides a simple and deliberate workflow:

- **Write**: Use `medi new` and `medi edit` to manage your notes.
- **Export**: Run `medi export ./my-notes` to write all your notes to a local directory as `.md` files.
- **Commit**: Use Git to commit the exported directory, giving you a complete, version-controlled snapshot of your work.

```
+-----------+      medi new/edit      +----------------+      medi export        +-------------------+
|           | ----------------------> |                | ----------------------> |                   |
|   You     |                         |  Sled Database |                         |  /notes Directory |
|           | <---------------------- |                | <---------------------- |                   |
+-----------+       medi get          +----------------+       medi import       +-------------------+
                                                                                            |
                                                                                            | git commit
                                                                                            V
                                                                                      +-----------+
                                                                                      |           |
                                                                                      | Git Repo  |
                                                                                      |           |
                                                                                      +-----------+
```

## Installation

You need [Rust and Cargo](https://www.rust-lang.org/tools/install) installed.

### Install directly from GitHub:

It will clone, build, and install the medi binary into your Cargo bin directory (~/.cargo/bin/).

```bash
cargo install --git https://github.com/cladam/medi.git
```

### Build from a local clone:

```bash
git clone https://github.com/cladam/medi.git
cd medi
cargo install --path .
```

## Usage

### Creating and editing notes

- **Create a new note**

This opens your default editor. Save and quit to store the note.

```bash
medi new "my-first-article"
```

- **Edit an existing note**

```bash
medi edit "my-first-article"
```

### Viewing and listing notes

- **Get a note's content**
Prints the note directly to the console. This is perfect for piping to other tools.

```bash
medi get "my-first-article"

# Pipe to a Markdown renderer like mdcat
medi get "my-first-article" | mdcat
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

**Next steps:**
1. Create a repository on your git provider (e.g. GitHub).
2. Run the following command to link it:
   `git remote add origin <your-repository-url>`
3. Then run this command to push your initial commit:
   `git push -u origin main`

- **Import notes from a directory**
Restores notes from a directory of .md files.

```bash

# By default, skips any notes that already exist
medi import ./my_notes_backup

# Overwrite existing notes with the imported versions
medi import ./my_notes_backup --overwrite
```
