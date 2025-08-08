# ADR 02: Database-Centric Storage with Import/Export for Versioning

* **Status**: Accepted
* **Date**: 2025-08-08

---

## Context

With `sled` as the storage engine (see ADR-01), we need to decide where the canonical source of truth for note content will live.

The core purpose of `medi` is to be the primary tool for managing Markdown notes. This suggests that the application, not the filesystem, should “own” the data. At the same time, content must be easy to version with Git and portable across machines.

Two models were considered:

1. **Filesystem-first**
   Markdown files are the source of truth, and `sled` acts as a performance cache.
   *Drawback*: Splits ownership between the filesystem and `medi`, leading to potential drift or sync issues if notes are edited outside the tool.

2. **Database-first**
   The `sled` database is the single source of truth, with explicit import/export for interoperability.

The filesystem-first model undermines the goal of `medi` being the central interaction point. The database-first model offers more control and consistency.


## Decision ✅

`medi` will use **database-first storage**, with `sled` as the canonical source of truth.

To support versioning, backup, and portability, `medi` will provide explicit **`export`** and **`import`** commands:

* **`export`**: Writes all notes from the database to a specified directory as `.md` files, using the note key as the filename.
* **`import`**: Reads `.md` files from a directory and inserts them into the database, with conflict-handling flags (e.g. `--overwrite`).

This creates a clear “snapshot” workflow for pushing database contents to a version-controlled location.


## Consequences

### ✅ Pros

* **Single Ownership**
  All commands (`new`, `edit`, `get`) operate solely on the database, no file sync complexity.

* **Clear Workflow**
  Users interact with `medi` for content creation and editing. Versioning is an intentional step (`medi export` → `git commit`).

* **Consistency and Safety**
  `sled` transactions protect against partial writes or corruption better than raw filesystem edits.

* **Performance**
  Avoids constant filesystem I/O during normal usage.

### ⚠️ Cons

* **Manual Versioning**
  Users must remember to export before committing to Git, no automatic sync.

* **Hidden Data by Default**
  Notes aren’t directly searchable with standard tools (`grep`, `cat`) unless exported. The user must use `medi get`.

* **Merge Complexity**
  Merging two different exported directories back into a database requires careful conflict-resolution rules in `import`.

