# Gemini Rules - Embroidery Catalogue Development

You are helping build **Embroidery Catalogue**, a local, offline desktop tool for cataloguing and browsing digital embroidery designs.

---

## 🤖 Agent Interaction Mode & Code Generation Rules

- **No Unsolicited Sample Code:** Do **not** generate code blocks or sample implementations unless explicitly asked. The user works with coding agents in the IDE to write the code. Focus on:
  - High-level system architecture and data flow.
  - UI/UX feedback, layout specifications, and component boundaries.
  - Verification of safety invariants and edge cases.
- **Svelte Module Reference Syntax (CRITICAL):**
  - Always reference Svelte view and component modules using a bare `@` prefix **without quotes** so IDE navigation works:
    - ✅ `@DesignDetailView.svelte`, `@MainView.svelte`
    - ❌ `'@DesignDetailView.svelte'`, `"@MainView.svelte"`, `DesignDetailView.svelte`

### Svelte View Inventory (Current as of July 2026)

@AboutDocumentView.svelte
@AboutView.svelte
@App.svelte
@BackupView.svelte
@DeleteDesignsModal.svelte
@DesignDetailView.svelte
@DesignPrintView.svelte
@DisclaimerView.svelte
@HelpView.svelte
@ImportTestHarness.svelte
@ImportView.svelte
@Inspector.svelte
@MainView.svelte
@Notice.svelte
@OrphansView.svelte
@Pagination.svelte
@ProjectsView.svelte
@SelectionHeader.svelte
@SettingsView.svelte
@TaggingActionsView.svelte
@TagSelectionModal.svelte
@TagView.svelte
@TagTable.svelte
@TechnicalDataGrid.svelte
@ToastContainer.svelte

---

## 📋 Pre-Execution Planning Rules (Required Before Code Changes)

When tasked with feature implementation, refactoring, or cross-boundary changes, the agent must produce a **concise structural implementation plan** and await approval before touching files.

The plan must be structural and high-level—**no code snippets or pseudocode**—and must strictly cover:

1. **Affected Files & Scope Boundary:**
   - Specific files to create or modify (using `@Component.svelte` format for Svelte files).
   - Reusable/shared components that are intentionally **not** to be touched (e.g., @Pagination.svelte, @TechnicalDataGrid.svelte).
2. **Boundary & IPC Contracts (Tauri v2 ↔ Svelte):**
   - Exact `#[tauri::command]` names and argument signatures.
   - Enforce camelCase keys for JS `invoke()` payloads.
   - Rust return types and their matching TypeScript interfaces.
3. **Specification & Safety Constraints Check:**
   - Affirm that source embroidery files remain strictly read-only and unmutated.
   - SQLite queries / schema adjustments involved.
   - File cache / thumbnail storage locations.
4. **Step-by-Step Execution Sequence:**
   - A short, numbered list of the order of execution and targeted test commands.

---

## 🛠️ Application Architecture & Tech Stack

- **Backend:** Rust (Idiomatic, clean, explicitly typed)
- **Desktop Framework:** Tauri (v2)
- **Frontend:** Svelte (v5) / TypeScript
- **Database:** Local SQLite database for metadata, tags, and file references
- **Core Logic:** Interfacing with binary embroidery file formats (reading metadata, stitches, and properties from formats like `.jef`, `.pes`, `.hus`, `.vp3`, etc., migrating logic inspired by `pyembroidery`).
- **AI Integration (Optional):** Google Gemini API for Gemini Vision (vision analysis on rendered thumbnails) to handle automated metadata/tag suggestions (tagging relies on offline File & Folder rules and online Gemini Vision tagging).

---

## 🧭 Core Philosophy & Constraints

- **Local & Offline First:** The app must run entirely locally. Original embroidery files must **NEVER** be moved, renamed, modified, or altered. The app only reads them to extract metadata and cache generated thumbnail previews locally.
- **Performance:** Reading binary stitch files and rendering/caching previews efficiently in Rust is a critical priority. Keep the UI responsive.
- **Separation of Concerns:** Maintain a clean architectural boundary between the Rust backend and the Svelte frontend.

---

## ⚠️ Core System & Failure State Lessons

### "Missing/Invalid file" is a state with side-effects, not just a file check

**Treat "the file is missing or invalid" as a system state that disables or redirects EVERY side-effect touching that path — not merely the main file operation.**

When a configured resource (database, data root, seed asset) is absent or invalid:

1. **Audit every side effect** that touches the stale/invalid path — folder creation, log writes, cache/thumbnail generation, temp files, background tasks — and gate ALL of them on the same single recovery/invalid-state decision.
2. **Write the "must NOT happen" test explicitly.** Add negative assertions (e.g. "recovery mode must NOT create dirs at the stale root") alongside positive ones.
3. **Enumerate the full state space before designing the flow.** _exists-valid_, _missing_, and _exists-but-invalid/corrupt_ are distinct states with different user paths.
4. **Order writes so nothing is persisted until a valid decision is made.** For replacement of an invalid file, prefer **rename-aside** (`<file>.corrupt-<timestamp>`) over delete — it is never safe to destroy the only copy of user data.
5. **Single source of truth for the invalid state.** Compute the recovery/invalid condition once (e.g. `paths::database_recovery_mode(&AppPaths)`) and reference it everywhere — startup, logging, background init, UI gating.

---

## 🧱 Core Architectural Boundaries

### 1. Database & Queries

- Keep all SQLite queries **strictly inside the Rust backend**.
- Never expose raw SQL or database connections to the frontend. Expose data to Svelte only via high-level, intentional Tauri commands.
- Manage the SQLite connection via Tauri's native managed state (`tauri::State`). Do not spin up or open separate database instances per command.

### 2. Tauri IPC Bridge

- All backend functions exposed to the frontend must use Tauri commands returning a `Result<T, E>`.
- The error type `E` must be a custom, descriptive, and serializable enum (e.g., using `thiserror` and `serde::Serialize`) so the frontend receives explicit error strings instead of generic panics.
- **CamelCase Invoke Keys (CRITICAL):** Tauri v2 automatically maps camelCase JavaScript invoke keys to snake_case Rust command parameters. **All `invoke()` payload keys MUST use camelCase** matching the Rust parameter names. Passing snake_case keys causes Tauri to silently drop them (parameters arrive as `None` with **NO error**).
  ```rust
  // Rust command signature
  fn run_stitching_backfill(
      clear_stitching_mode: Option<String>,  // snake_case in Rust
      batch_size: Option<i64>,
  )
  ```
