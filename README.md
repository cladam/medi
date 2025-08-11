# medi ✒️

**A fast, editor-centric, command-line notes manager.**

`medi` is a simple and powerful tool for creating and managing your notes, articles, and documentation directly from the terminal. It's built for developers, writers, and anyone who loves the speed and focus of a command-line workflow.

## Core Philosophy

medi is built on a few guiding principles:

- **CLI-first**: Everything is done through the terminal. No GUIs or TUIs, no distractions.
- **Editor-centric**: Your text editor (`$EDITOR`) is where you write. `medi` gets you there quickly and saves your work securely.
- **Local & private**: All content is stored on your local machine in a high-performance embedded database. No cloud services, no network access.
- **Zero-config start**: Install it and start writing immediately.

## Why Use medi?

You can absolutely manage notes with a directory of Markdown files. However, `medis offers a layer of structure and tooling designed to make that process faster, more organized, and more powerful.

1. Speed: Instant access to any note ⚡

   A simple `ls` and `cat` works for a dozen files, but it breaks down with hundreds or thousands. `medi` uses an embedded database (`sled`) as a high-performance index.
   `medi get my-old-note` is instantaneous, no matter how many notes you have. It doesn't need to scan a directory; it performs a direct key lookup.
   `medi list` is equally fast, giving you a clean overview without the noise of filesystem details.

2. Workflow: An editor-centric hub ✍️

   `medi` isn't just a file store; it's a workflow tool. It standardises the way you create, find, and edit notes, acting as a central hub for your writing.

   - **No `cd` required**: You can create or edit a note from any directory in your terminal. You don't need to navigate to ~/Documents/notes first.
   - **Atomic operations**: When you save a note, the database update is atomic. You avoid issues like partially written files or saving temporary files by mistake.
   - **Future-proof**: This centralised workflow allows for powerful future features like **full-text search**, **tagging**, and **task management** that would be complex and slow to implement on a plain directory of files.

3. Abstraction: key vs. filename

   With `medi`, you think in terms of a `key`, which is a clean, abstract identifier. You don't have to worry about filesystem limitations, illegal filename characters, or file extensions. This clean separation of `key` (the identifier) from `value` (the Markdown content) is a simple but powerful concept that keeps your collection of notes tidy.

In short, `medi` provides the **simplicity of Markdown** with the **speed and structure of a database**, creating a focused workflow for command-line writing.

## How It Works, DB as Source-of-Truth

`medi` uses a database-first approach. All your notes are stored in a `sled` key-value database, making access fast and reliable. This database is the single source of truth.

To version control your work with Git, `medi` provides a simple and deliberate workflow:

- **Write**: Use `medi new` and `medi edit` to manage your notes.
- **Export**: Run `medi export ./my-notes` to write all your notes to a local directory as `.md` files.
- **Commit**: Use Git to commit the exported directory, giving you a complete, version-controlled snapshot of your work.

```
+-----------+    medi new/edit    +----------------+     medi export      +-------------------+
|           | ------------------> |                | -------------------> |                   |
|   You     |                     |  Sled Database |                      |  /notes Directory |
|           | <------------------ |                | <------------------- |                   |
+-----------+      medi get       +----------------+      medi import     +-------------------+
                                                                                   |
                                                                                   | git commit
                                                                                   V
                                                                             +-----------+
                                                                             |           |
                                                                             | Git Repo  |
                                                                             |           |
                                                                             +-----------+
```

## Project Status & Roadmap 🗺️

This section tracks the implementation status of `medi`'s features. Contributions are welcome!

### Core Functionality

- [x] `new`: Create a new note.
- [x] `get`: Read a note's content.
- [x] `list`: List all note keys.
- [ ] `edit`: Open an existing note in the editor.
- [x] `delete`: Remove a note from the database.
- [ ] `import`: Restore notes from a directory of Markdown files.
- [ ] `export`: Save all notes to a directory as Markdown files.

### Future Ideas

- [ ] Configuration file for settings (e.g., database path).
- [ ] Support for note metadata (tags, creation dates).
- [ ] Full-text search over note content.
- [ ] Implement a `task` command, for tracking the status of notes.

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
