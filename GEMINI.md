# Gemini Rules - Embroidery Catalogue Development

You are helping build **Embroidery Catalogue**, a local, offline desktop tool for cataloguing and browsing digital embroidery designs.

---

## 🤖 Core Agent Rules & Planning

- **No Unsolicited Sample Code During Planning:** During exploration, architectural discussions, and pre-execution planning, do **not** generate speculative code blocks or unsolicited sample implementations. Focus on system architecture, data flow, UI/UX feedback, and safety invariants.
- **Direct Implementation on Approved Tasks:** When explicitly tasked with implementing features, fixing bugs, or writing tests, directly write and modify the code and verify it with the appropriate test commands.
- **Svelte Module Reference Syntax (CRITICAL):**
  - Always reference Svelte view and component modules using a bare `@` prefix **without quotes** so IDE navigation works (e.g. `@DesignDetailView.svelte`, `@MainView.svelte`, `@ImportView.svelte`).
- **Pre-Execution Planning (Required Before Changes):**
  - Produce a concise structural implementation plan covering:
    1. Affected Files & Scope Boundary
    2. Boundary & IPC Contracts (Tauri v2 ↔ Svelte, camelCase JS keys)
    3. Specification & Safety Constraints (read-only original designs, SQLite schema)
    4. Step-by-Step Execution Sequence & Test Plan (Vitest, `cargo test`, and mandatory Playwright E2E coverage for UI changes)

---

## 🧭 Core Philosophy & Safety Constraints

- **Local & Offline First:** The app runs entirely locally. Original embroidery files must **NEVER** be moved, renamed, modified, or altered. The app only reads them to extract metadata and cache generated thumbnail previews locally.
- **Performance:** Reading binary stitch files and rendering/caching previews efficiently in Rust is a critical priority.
- **Zero Static Process Globals & Managed State:** All task coordinators, background managers, and session stores are encapsulated in `AppState` and injected via `tauri::State<AppState>`.
- **Zero Native Webview Dialogs (ADR 010):** Absolute ban on `window.confirm()`, `window.alert()`, `window.prompt()`. All confirmations must use custom Svelte modals with `data-testid` attributes.
- **Action Semantics (ADR 011):**
  - **Cancel (Atomic Rollback):** Complete discard of uncommitted work; zero changes persist (e.g. setup abort, storage migration cancel, scan cancel).
  - **Stop (Progressive Halt):** Halts iterative batch processes mid-flight while strictly retaining committed work (e.g. bulk import, unified backfill). Button transitions: `"Stop"` $\rightarrow$ `"Stopping..."`.

---

## 📚 Modular Rule Book Index (`.agents/rules/`)

The workspace rules are modularized across `.agents/rules/` for comprehensive details:

1. [`.agents/rules/01-agent-planning-and-interaction.md`](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/.agents/rules/01-agent-planning-and-interaction.md)
   - Agent interaction mode, code generation restrictions, pre-execution planning requirements, and complete Svelte module inventory.
2. [`.agents/rules/02-architecture-and-boundaries.md`](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/.agents/rules/02-architecture-and-boundaries.md)
   - Core philosophy, failure state lessons, IPC contracts (camelCase invoke keys), data root isolation, tagging boundaries (image vs stitching), ADR 006 (master/stitch pairing), ADR 007 (library drift), ADR 011 (cancel vs stop).
3. [`.agents/rules/03-rust-standards.md`](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/.agents/rules/03-rust-standards.md)
   - Zero panics in binary parsing, canonical `designs.filepath` format, instant database schema checks, Criterion benchmarking, and 500-line module limits.
4. [`.agents/rules/04-frontend-standards.md`](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/.agents/rules/04-frontend-standards.md)
   - Native Svelte 5 runes, strict typing (no implicit any), 500-line component limit, ADR 010 modal invariant, canonical UI theme tokens, and view testing rules.
5. [`.agents/rules/05-testing-and-benchmarks.md`](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/.agents/rules/05-testing-and-benchmarks.md)
   - MainView dual-suite discipline, Playwright WebView2 CDP setup, formatting & command gotchas, and execution benchmark table with polling cadence guidelines.
