# Library indexing

The Rust scanner reads existing folders in a background thread and persists one folder at a time. It ignores non-PDF files and skips symbolic links. Filesystem notifications trigger debounced parent-directory refreshes and traverse newly discovered branches.

A PDF entry keeps its database ID at an unchanged path. On a move inside a registered root, the scanner can reuse the ID if the prior path is absent and its fingerprint is unique. Fingerprints use SHA-256 with the file size and up to three 64 KiB samples. Smaller files are hashed in full. Matching duplicates remain ambiguous. Missing entries retain their metadata and annotations.

Optional PDF text indexing extracts text in the PDF.js worker, writes page batches, and marks a document complete after the last page. A pause cancels the current document. A later run reuses unchanged completed entries and restarts unfinished ones.
