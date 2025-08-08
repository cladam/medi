# ADR 01: Use `sled` for Key-Value Storage

* **Status**: Accepted
* **Date**: 2025-08-08

---

## Context

`medi` needs a fast, lightweight, on-disk key-value store to manage notes. Each note is stored as a Markdown string, associated with a unique, human-readable key (e.g. `my-first-article`). The storage layer should:

* Be embedded within the application.
* Require zero configuration or setup.
* Perform well for typical usage patterns (frequent reads, occasional writes).
* Be written in Rust to align with the rest of the project.

### Alternatives Considered

* **SQLite**
  A powerful, well-known option with rich tooling. However, it introduces a large dependency (`libsqlite3`) and requires SQL even for simple operations.

* **Plain Filesystem**
  Keys could be mapped to filenames, with content stored as `.md` files. While easy to inspect and version, this approach lacks atomicity and becomes less efficient at scale.

* **Single JSON File**
  Simple to implement, but degrades quickly as the number of notes grows. Any write requires parsing and rewriting the entire file.


## Decision ✅

I will use [`sled`](https://docs.rs/sled), an embedded key-value store written in pure Rust.

It offers:

* A minimal, intuitive API (similar to a `BTreeMap`).
* High performance on modern hardware.
* No external dependencies or runtime configuration.

`sled` is well-suited for `medi`'s needs: it stays out of the way, handles persistence safely, and enables fast access to note content by key.


## Consequences

### ✅ Pros

* **Pure Rust**
  No C dependencies or compilation issues across platforms.

* **Performance**
  Designed for SSDs/NVMe, `sled` performs well for both reads and writes.

* **Low Friction**
  No config files, schema management, or runtime services.

* **Fits the Model**
  A natural match for the note-key abstraction in `medi`.

### ⚠️ Cons

* **Opaque Format**
  The database file is binary and not human-readable, making it unsuitable for version control or manual inspection.

* **Limited Ecosystem**
  Fewer external tools or visualisers exist for `sled` compared to SQLite.

* **Early Maturity**
  While stable, `sled` has a smaller adoption footprint and may have rough edges in some advanced use cases.

