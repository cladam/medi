<div align="center">

<p align="center">
  <img src="assets/medi-logo.png" alt="medi logo" width="200"/>
</p>

<p align="center">
  <b>⚡ A speedy CLI Markdown Manager ⚡</b><br/>
  Lightweight • Local-first • Editor-friendly
</p>

[![Crates.io](https://img.shields.io/crates/v/medi.svg)](https://crates.io/crates/medi)
[![Downloads](https://img.shields.io/crates/d/medi.svg)](https://crates.io/crates/medi)

</div>

## About

**[[medi]] is a fast, editor-centric, command-line notes manager.**

`medi` is a simple and powerful tool for creating and managing your notes, articles, and documentation directly from the
terminal. It's built for developers, writers, and anyone who loves the speed and focus of a command-line workflow.

![A terminal running medi commands](docs/medi.gif "A demo of medi")

## Philosophy

`medi` is built on a few guiding principles:

- **CLI-first**: Everything is done through the terminal. No GUIs or TUIs, no distractions.
- **Editor-centric**: Your text editor (`$EDITOR`) is where you write. `medi` gets you there quickly and saves your work
  securely.
- **Local & private**: All content is stored on your local machine in a high-performance embedded database. No cloud
  services, no network access.
- **Zero-config start**: Install it and start writing immediately.

## Features ✨

- **Create & Manage Notes**: Quickly create new notes using a simple command (`medi new <note-key>`). You can add
  content directly with a flag, pipe it from other commands, or open your favorite text editor for more detailed
  entries.
- **Powerful Organisation**: Keep your notes tidy with tags. Add multiple tags when creating a note (`--tag`) and easily
  add or remove them later (`medi edit --add-tag ...`).
- **Integrated Task Management**: Turn your notes into actionable to-do lists. Add tasks to any note (
  `medi task add ...`), list all your pending items, mark them as complete (`medi task done ...`), and set priorities to
  focus on what's important.
- **Full-text Search**: Instantly find what you're looking for with a powerful search command (`medi search <term>`)
  that scans the content of all your notes.
- **Import & Export**: `medi` is not a data silo. You can easily import entire directories of Markdown files to get
  started, and export all your notes back to Markdown or JSON at any time.
- **List & Review**: Get a clean, sorted list of all your notes (`medi list`) or view the content of any specific note (
  `medi get <note-key>`).
- **Finding Connections & Notes**: Use `[[wiki-like]]` syntax to link back to a note, and use a fuzzy finder to quickly
  find and edit a note by its key.
- **Colourful & Clear Output**: Uses coloured output to clearly distinguish between success messages, information,
  warnings, and errors.

## How It Works, DB as Source-of-Truth

`medi` uses a database-first approach. All your notes are stored in a `sled` key-value database, making access fast and
reliable. This database is the single source of truth.

To version control your work with Git, `medi` provides a simple and deliberate workflow:

- **Write**: Use `medi new` and `medi edit` to manage your notes.
- **Export**: Run `medi export ./my-notes` to write all your notes to a local directory as `.md` files.
- **Commit**: Use Git to commit the exported directory, giving you a complete, version-controlled snapshot of your work.

### Installation

You need [Rust and Cargo](https://www.rust-lang.org/tools/install) installed.

#### Installing from crates.io

The easiest way to install `medi` is to download it from [crates.io](https://crates.io/crates/medi). You can do it using
the following command:

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

## Configuration

On the first run, `medi` will automatically create a default configuration file at the standard location for your
operating system:

* **macOS**: `~/Library/Application Support/medi/config.toml`
* **Linux**: `~/.config/medi/config.toml`
* **Windows**: `C:\Users\<user>\AppData\Roaming\medi\config.toml`

You can edit this file to customize `medi`'s behaviour.

### Example `config.toml`

```toml
# Path to the database file.
# You can change this to store your notes in a different location,
# for example, inside a cloud-synced folder like Dropbox.
db_path = "/Users/cladam/Library/Application Support/medi/medi_db"

# Default directory for the `medi export` command.
# If this is set, you can run `medi export` without specifying a path.
# Leave it as an empty string ("") if you don't want a default.
default_export_dir = "/Users/cladam/Documents/medi_backups"
```

## Usage

### Creating and Editing Notes

* **Create a new note**

  `medi` provides three ways to create a new note:

    1. **Interactively (default)**: Opens your default editor for long-form content.
       ```bash
       medi new "my-long-article"
       # With tags: Add tags to your note for better organisation.
       medi new "my-long-article" --tag tag1 --tag tag2
       # With a title: Specify a title for your note.
       medi new "my-long-article" --title "My Long Article"
       ```

    2. **With a direct message**: Perfect for quick, one-line notes.
       ```bash
       medi new quick-idea -m "Remember to buy milk"
       ```

    3. **From a pipe**: Use the output of other commands as your note content.
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

### Using Templates 📝

`medi` can create new notes from predefined templates to speed up your workflow. On its first run, `medi` creates an
example `meeting.md` template in the configuration directory.

- **Location of templates:**

    * **macOS**: `~/Library/Application Support/medi/templates/`
    * **Linux**: `~/.config/medi/templates/`
    * **Windows**: `%APPDATA%\medi\templates\`

- **Create a note from a template:**

  ```bash
  medi new my-project-meeting --template meeting
  ```

- **Create your own templates:**
  Simply add any `.md` file to your templates directory. If you create `blog.md`, you can use it with
  `medi new ... --template blog`.

### Finding, Viewing & Listing Notes

⚠️ This feature is not supported on Windows.

- **Interactively find a note**
  Open a fuzzy finder to quickly search for and edit a note by its key.

  ```bash
  medi find
  ```

  This will open an interactive TUI to help you find the note you want to edit.


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

- **List all notes**
  The `list` command provides a rich overview of your notes, including their keys and tags.

  ```bash
  medi list
  ```

  *Output:*

  ```
  - cladam_github_io_readme [#blog #project]
  - medi-blogpost [#rust]
  ```

- **Sort your notes**
  You can sort the list by creation or last modification date using the `--sort-by` flag. The default is to sort
  alphabetically by key.

  ```bash
  # Sort by the most recently modified notes
  medi list --sort-by modified

  # Sort by when the notes were created
  medi list --sort-by created
  ```

### Searching & Indexing

`medi` includes a full-text search engine (`tantivy`) that lets you find notes by their content, title, or tags.

**Search for a note**

```bash
# Find all notes containing the word "rust"
medi search rust

# Search for a phrase
medi search "database design"
```

_Output_:

```
Search Results:
- medi-blogpost
- rust-cli-ideas
```

**Rebuild the search index**

If your search index ever gets out of sync or you're setting up `medi` for the first time with an existing database, you
can rebuild the entire index.

```bash
medi reindex
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
  Restores notes from a directory of `.md` files.

  ```bash
  # By default, skips any notes that already exist
  medi import --dir /path/to/notes

  # Import a single
  medi import --file /path/to/note.md --key my-note

  # Overwrite existing notes with the imported versions
  medi import --file /path/to/note.md --key my-note --overwrite
  ```

### Task Management

`medi` includes a simple task manager to help you turn notes into actionable to-do lists.

- **Add a task to a note**

  ```bash
  medi task add my-blog-post "Finish the conclusion section"
  ```

- **List all tasks**

  The list is sorted by priority and status.

  ```bash
  medi task list
  ```

  _Output:_

  ```
  Tasks:
  [42] [Prio] ⭐ : Review final draft (for note 'medi-readme')
  [43] [Open] : Add usage examples (for note 'medi-readme')
  [44] [Done] : Write introduction (for note 'my-blog-post')
  ```

- **Prioritise a task**

  ```bash
  medi task prio 43
  ```

- **Complete a task**

  ```bash
  medi task done 43
  ```

- **Delete a task**

  ```bash
  medi task delete 43
  ```

- **Clear all tasks**

  This is a destructive action

  ```bash
  medi task reset
  ```

### Checking Status

The `status` command provides a high-level overview of your database or detailed statistics for a single note.

- **Get a global overview:**

  ```bash
  medi status
  ```

  _Output:_

  ```
  medi status
    Notes: 42
    Tasks: 8 open (3 priority)
  ```

- **Get stats for a specific note:**

  ```bash
  medi status medi-blogpost
  ```

  _Output:_

  ```
  medi-blogpost
    Key: medi-blogpost
    Tags: rust, cli
    Words: 584
    Reading Time: ~3 minute(s)
    Created: Mon, 12 Sep 2025 10:30:00 +0200
    Modified: Tue, 13 Sep 2025 15:00:00 +0200
  ```

### Discover Connections

- **Find backlinks for a note**
  Discover all the notes that link to a specific topic using `[[wiki-link]]` syntax.

  ```bash
  medi backlinks rust
  ```

  _Output:_

  ```
  Found 1 backlinks for 'rust':
  - medi-project
  ```

### Markdown Linting

`medi` can check your notes for common Markdown issues using [`rumdl`](https://github.com/rvben/rumdl).

- **Lint a specific note**

    ```bash
    medi lint my-blog-post
    ```

- **Lint all notes**

    ```bash
    medi lint
    ```

Formatting and fixing the issues is left to your editor or other tools.

### Markdown Preview

`medi` includes a simple preview mode using `egui` to render your Markdown notes in a window.

- **Preview a note**

  ```bash
  medi preview my-blog-post
  ```

### Shell Completion

To make `medi` even faster to use, you can enable shell completion. Add one of the following lines to your shell's
configuration file.

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
- [x] Configuration file for settings (e.g. database path).
- [x] Support for note metadata (tags, creation dates).
- [x] `export` notes by tag.
- [x] Full-text search over note content.
- [x] Implement a `task` command for tracking the status of notes.
- [x] Implement a simple status command
- [x] Add a Fuzzy Finder for notes
- [x] Add templates for different documents
- [x] Wiki-Style Linking
- [x] Linting of notes
- [x] A preview mode using egui
- [ ] ... your idea here?


