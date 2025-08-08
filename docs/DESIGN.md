# medi: Design & Architecture

* **Version**: 1.0
* **Status**: Proposed
* **Date**: 2025-08-08

This document outlines the design and guiding principles behind `medi`, a terminal-based markdown manager.

---

## 1. Vision & Purpose 🎯

**`medi`** is a command-line tool for creating and managing markdown-based notes, articles, and documentation. It's built for developers, writers, and anyone who prefers working in the terminal with an editor-centric workflow.

The goal is to provide a smooth path from idea to finished document, without leaving the command line.

---

## 2. Core Concepts

The system is built around a few simple concepts:

* **Note**
  A markdown file containing your content. Notes can be anything, from a one-liner to a detailed article.

* **Key**
  A unique, human-readable identifier (e.g. `cli-ideas`, `my-first-article`) used to reference a note.

* **Database (`db`)**
  A local `sled` key-value store used to index notes by key. This database is an internal implementation detail, not the primary source of truth. 
  That would be the content itself.

* **CLI Interface**
  The command-line interface is the only way to interact with `medi`. Built with `clap`, it exposes all functionality through clear commands and flags (e.g. `medi new`, `medi edit`).

---

## 3. High-Level Architecture & Workflow

`medi` follows a standard Rust “lib + bin” structure to encourage separation of concerns and maintainability.

### 🧭 Typical Flow: `medi new "my-idea"`

1. **Command execution**
   The user runs a command in their shell.

2. **Argument parsing (`main.rs`)**
   The `clap` parser validates arguments and produces a `Cli` struct.

3. **Delegation (`main.rs` → `lib.rs`)**
   `main.rs` passes control to `run()` in `lib.rs`, handing over the parsed CLI input.

4. **Command dispatch (`lib.rs`)**
   The `run()` function matches the command and calls the appropriate module function.

5. **Data handling (`lib.rs` → `db.rs`)**
   For creation commands, `db.rs` handles storing and retrieving content from `sled`.

6. **Editor launch**
   Using the `edit` crate, `medi` opens the default `$EDITOR` in a new buffer.

7. **Content creation**
   The user writes, saves, and exits.

8. **Persistence**
   The content is captured and stored in the database using the assigned key.

9. **Result**
   Success or error messages are printed to the console.

This structure keeps `main.rs` thin and focused, while keeping application logic modular and testable.

---


## 4. Guiding Principles

These principles shape the design and usage of `medi`.

* **CLI-first**
  Everything is done through the terminal. There's no GUI and no intention to add one. The CLI should be easy to script and compose with other tools.

* **Editor-centric**
  Your text editor is where you write. `medi` isn't an editor, it just launches yours with the right buffer.

* **Local & private by default**
  All content stays on your local machine. No external services, no sync, no network traffic. Your data is yours.

* **Zero-config startup**
  `medi` should be usable immediately after install. Sensible defaults mean you don’t need to configure anything to get started (but you can if you want to).

---


