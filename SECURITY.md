# Security

Once the project has a GitHub repository with private vulnerability reporting enabled, use that channel for vulnerabilities. Remove PDFs, folder paths, and annotations from reports unless essential to reproduce the issue.

The viewer opens untrusted PDFs in a dedicated PDF.js worker. Native PDF reads validate that the resolved path is under an active registered root. Release builds with updates configured use Tauri's signed updater and HTTPS.
