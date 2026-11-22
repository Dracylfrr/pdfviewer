# Annotations

Highlights, underlines, and selection notes store a stable document entry ID, a one-based page number, selected text, normalized page rectangles, color, and note content. Bookmarks store page numbers. Document and folder notes belong to their entry without a page number. The reader projects normalized coordinates onto the rendered canvas at its current zoom.

Text edits debounce for 600 ms. Changing selection, blurring the editor, or closing the window flushes pending writes. Changes still waiting in memory can be lost if the process is terminated abruptly.

JSON exports carry a schema version and a document fingerprint. Imports validate the annotation types and coordinates in a transaction and ask before applying a mismatched export. Reimporting the same annotation ID is idempotent. Markdown exports contain human-readable notes and quotations. Source PDFs remain read-only.
