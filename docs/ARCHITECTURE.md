# Architecture

A Tauri 2 host uses the system WebView2 runtime on Windows. Rust handles indexed file access, SQLite, backups, and filesystem notifications. Svelte and TypeScript render library views and the reader. PDF.js parses and renders in a dedicated worker, loaded when reading or text indexing begins.

SQLite uses WAL and full synchronization for metadata durability. Each folder scan commits a transaction. Migrations run transactionally and keep a prior snapshot when upgrading an existing database. Daily startup snapshots retain three files.

The reader mounts page canvases near the visible range and cancels renders when pages leave that range. Continuous scrolling uses an estimated uniform slot height. Documents with mixed page sizes can show extra space around shorter pages. The host currently reads each opened PDF into memory before transferring it to PDF.js.

Core dependencies and license families are listed in [third-party notices](THIRD_PARTY.md). Committed Cargo and npm lockfiles record resolved versions.
