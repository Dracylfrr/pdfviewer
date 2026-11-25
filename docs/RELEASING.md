# Releasing

The canonical version is in `src-tauri/Cargo.toml`. Tauri uses it for the app and installer. The local preview is 0.3.3.

1. Run the test suite, package Windows builds, and finish the release checks in the implementation specification.
2. Create a Tauri signing key with `npm run tauri -- signer generate -- -w <private-key-path>`. Keep the private key outside the repository.
3. In the GitHub repository, set Actions variable `PDFVIEWER_UPDATER_PUBLIC_KEY` to the public key contents. Add Actions secrets `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
4. Set the Cargo version to `1.0.0`, regenerate the Cargo lockfile, then commit. Push tag `v1.0.0` to that repository.
5. The tag workflow builds the signed x64 NSIS installer and `latest.json`, adds a SHA-256 checksum, and prepares a draft. Verify the draft assets and release notes before publishing.
6. Verify a signed upgrade on a clean Windows profile with a future higher version and confirm that the release feed fails gracefully offline.

Local builds check GitHub for newer versions. In-app installation requires a configured signed feed. Signed updates are downloaded in the background and installed silently on normal app close for the per-user NSIS installation. The app starts the updated version on the next launch. The preview installer produced locally is unsigned. Tauri updater signing differs from Windows Authenticode signing.

For a standalone archive, zip the optimized `pdfviewer.exe` with the license notices. It still requires WebView2 and uses the per-user metadata folder by default. Set `PDFVIEWER_DATA_DIR` to an existing directory to select a separate metadata location.
