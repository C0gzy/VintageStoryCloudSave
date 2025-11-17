# Cloud Save Uploader – Detailed Plan

## 1. Product & Technical Goals
- Provide reliable cloud backups for Vintage Story save folders with ~10 GB of free storage per user.
- Support Windows (Win11/Win10) and macOS (Sonoma/Sequoia) with consistent UX.
- Prefer Rust for the core uploader, leaning on API-compatible object storage and a modern GUI toolkit that feels web-like.
- Keep future room for Linux ports, encryption-at-rest, and scheduled syncs.

## 2. Storage Option Assessment
1. **Baseline requirements**
   - ≥10 GB free tier per account.
   - Stable REST or S3-compatible API.
   - Allowing desktop clients (no browser-only auth).
   - Reasonable egress pricing for restore operations.
2. **Candidates**
   - **Cloudflare R2**: 10 GB free, S3-compatible, no egress to Cloudflare network. Rust crates: `aws-sdk-s3`, `serde`, `tokio`.
   - **Backblaze B2**: 10 GB free, native REST API plus S3 compatibility, wider ecosystem tooling. Crates: `b2-sdk-rs` or `reqwest`.
   - **Supabase Storage**: generous free tier (~1 GB by default) but expandable; PostgREST auth; better when pairing with Supabase auth.
3. **Decision matrix**
   - Evaluate on free quota, API ergonomics, latency from target regions, and credentials (API keys vs signed URLs).
   - If global distribution matters, R2 likely best; if simple credentials with official SDK, Backblaze B2.
4. **Action items**
   - Prototype S3 put/list with `aws-sdk-s3` against R2.
   - Measure upload throughput for chunk sizes 8 MB / 32 MB.
   - Confirm credential storage policy (OS keychain on macOS, DPAPI on Windows).

## 3. Upload Pipeline Design
1. **Local discovery**
   - Detect Vintage Story save root: `%APPDATA%/VintagestoryData/Saves` on Windows, `~/Library/Application Support/VintagestoryData/Saves` on macOS.
   - Watch directories for changes (use `notify` crate).
2. **Pre-upload staging**
   - Hash file contents (xxhash64) to detect duplicates.
   - Compress (zstd) when beneficial; skip already-compressed archives.
   - Split into chunks (default 32 MB) and queue jobs.
3. **Upload engine**
   - Async runtime: `tokio`.
   - HTTP/S3 client: `aws-sdk-s3` (or `reqwest` for custom APIs).
   - Implement retries with exponential backoff, resumable multipart uploads.
   - Maintain local metadata DB (`sled` or SQLite via `rusqlite`) storing file → object mapping, etag, last upload time.
4. **Auth & security**
   - Store API keys in OS keychain (`keyring` crate).
   - Optional client-side encryption with `age` or `ring` AES-GCM before upload.
5. **Restore workflow**
   - List backups per world, allow selective download.
   - Validate checksums before writing back to disk.
6. **Telemetry & logging**
   - Structured logs via `tracing`.
   - Optional crash reports (Sentry via `sentry` crate).

## 4. Cross-Platform GUI Strategy
1. **Toolkit options**
   - **Tauri + Dioxus**: HTML/CSS-like layout, Rust backend, system WebView. Good for web dev background.
   - **Slint**: declarative UI, native rendering, lighter footprint.
   - **Iced**: Rust-native but less web-like styling.
2. **Recommended stack**
   - Use **Tauri** shell for Windows/macOS parity.
   - Implement UI with **Dioxus** (React-like components). Styling through Tailwind-like CSS modules.
   - Share Rust core uploader as a library crate consumed by Tauri commands.
3. **Key UI flows**
   - Onboarding: provider selection, API key entry, test connection.
   - Save browser: list local worlds, show sync status, buttons for “Upload now”, “Restore”.
   - Activity pane: show live progress bars, errors, retries.
   - Settings: schedule (interval), bandwidth cap, encryption toggle, log export.
4. **Packaging**
   - macOS: Tauri dmg/notarization steps, embed universal binary.
   - Windows: MSI via `tauri-bundler`, handle code-signing later.

## 5. Implementation Roadmap
1. **Milestone 1 – Prototype CLI**
   - Implement storage client abstraction and upload single save folder on demand.
   - Validate credentials, chunking, retry logic.
2. **Milestone 2 – Background sync service**
   - Add file watching, metadata DB, and configurable schedules.
3. **Milestone 3 – GUI integration**
   - Scaffold Tauri+Dioxus app, expose commands to trigger uploads/restores.
   - Wire progress reporting via channels/events.
4. **Milestone 4 – Polish & release**
   - Add settings persistence, auto-updater, crash reporting.
   - Package for Windows & macOS, document install & troubleshooting.

## 6. Next Steps
- Choose target storage (likely Cloudflare R2) and create API credentials.
- Scaffold Rust workspace: `cloud_core` (library), `cli`, `app-tauri`.
- Build the CLI prototype to validate uploads before investing in UI.
- Draft UI mockups mirroring familiar web dashboards to guide Dioxus components.


