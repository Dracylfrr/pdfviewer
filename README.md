# PDF Viewer

PDF Viewer is a local desktop reader for PDFs already organized in folders. Version **0.1.0** is a development preview for Windows x64.

Choose a library folder, browse it in Columns, Tree, or Graph, then select **Open PDF**. The app keeps highlights, underlines, bookmarks, notes, tags, and reading settings in SQLite. **Pastel Blue** is the default application theme. The reader's paper theme has its own selector.

## Run on Windows

Run the NSIS installer produced by the build, or launch the standalone `pdfviewer.exe` on a machine with Microsoft Edge WebView2. The standalone executable uses the same per-user metadata location as the installer.

## Development

Install Node.js 24, Rust stable with the MSVC target, Visual Studio C++ Build Tools, and WebView2. From this folder run:

```powershell
npm ci
npm run check
npm run tauri -- dev
cargo test --locked --manifest-path src-tauri/Cargo.toml
npm run tauri -- build
```

The last command creates the x64 executable and an NSIS installer under `src-tauri/target/release`. Use an existing writable directory in the `PDFVIEWER_DATA_DIR` environment variable to isolate a development run.

## Reader keys

| Key | Action |
| --- | --- |
| Ctrl+F | Find within the PDF, or focus library search outside the reader |
| Ctrl+plus, Ctrl+minus, Ctrl+0 | Change or reset zoom |
| Ctrl+B | Bookmark the current page |
| H | Highlight the current text selection |
| Left, Right, Page Up, Page Down | Change pages |
| Esc | Dismiss the selection toolbar |

For text search across an entire library, choose **Index PDF text** in Settings. Existing completed entries are reused when their size and modification time match. PDFs that require a password and unreadable PDFs are skipped. Search within an open PDF runs on demand.

See [privacy](PRIVACY.md), [architecture](docs/ARCHITECTURE.md), [performance](docs/PERFORMANCE.md), and [releasing](docs/RELEASING.md) for storage details, measurements, and release requirements.
