# Architecture Decision Record 010: In-App Svelte Modal System for User Confirmations and Alerts

* **Status:** Accepted
* **Deciders:** Julie Stenning
* **Date:** 2026-10-07

---

## Context and Problem Statement

Desktop applications commonly require user confirmation before executing destructive or significant operations (such as deleting orphaned records, deactivating a license, discarding unsaved edits, or restoring a backup).

Previously, some flows used native browser dialogs (`window.confirm()`, `window.alert()`) or native OS message boxes. This approach caused severe issues:
1. **Thread Blocking:** Native browser dialogs block the WebView2 UI execution thread, halting animations, background updates, and IPC progress events.
2. **Theme Inconsistency:** Native OS popups bypass the application's design system and look-and-feel specification, ignoring light/dark theme custom properties (`--surface-*`, `--text-*`).
3. **Flaky E2E Automation:** Headless Playwright / WebView2 CDP test runners frequently hang or fail silently when encountering unexpected synchronous OS/browser dialog prompts.

---

## Decision Drivers

* **Design System Consistency:** All modals, alerts, and confirmations must strictly follow the canonical UI Look and Feel & Design Token specification in both light and dark themes.
* **Non-Blocking UI:** Dialogs must be asynchronous Svelte components that do not freeze the WebView event loop or background worker listeners.
* **Deterministic Testability:** Every modal dialog must expose semantic `data-testid` and accessible role attributes for automated unit and Playwright e2e testing.
* **Explicit Action Guardrails:** Destructive operations must require intentional confirmation with clear contextual warning copy.

---

## Considered Options

1. **Native OS Message Boxes (`rfd::MessageDialog` / Tauri Dialog Plugin):** Synchronous, platform-dependent, un-themed.
2. **Native Webview Popups (`window.confirm` / `window.alert`):** Thread-blocking, un-themed, problematic in test automation.
3. **In-App Accessible Svelte Modal System (Chosen):** Dedicated Svelte 5 modal components and `@Notice.svelte` alerts styled with design tokens.

---

## Decision Outcome

**Option 3 (In-App Accessible Svelte Modal System)** was chosen.

### Architectural Invariants:
1. **Zero Native Webview Dialogs:** Complete ban on `window.confirm()`, `window.alert()`, and `window.prompt()` across the frontend codebase.
2. **Dedicated Modal Components:**
   - General notices & alerts use `@Notice.svelte`.
   - Destructive or complex confirmations use dedicated accessible modals (e.g. `@ConfirmDeleteProjectModal.svelte`, `@ConfirmRestoreModal.svelte`, `@DeleteDesignsModal.svelte`, `@CancelBackupModal.svelte`).
3. **Theme & Accessibility Compliance:**
   - Modals use `--surface-overlay`, `--surface-card`, and `--text-primary` custom CSS variables.
   - Backdrop clicks and `Escape` key handlers close non-critical modals safely while trapping focus during active prompts.
4. **Test Selector Contracts:** All interactive modal buttons and dialog shells expose unambiguous `data-testid` attributes (e.g. `data-testid="confirm-delete-btn"`).
