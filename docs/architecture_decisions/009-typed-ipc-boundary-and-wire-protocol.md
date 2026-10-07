# Architecture Decision Record 009: Typed IPC Boundary, CamelCase Wire Protocol, and Structured Error Model

* **Status:** Accepted
* **Deciders:** Julie Stenning
* **Date:** 2026-10-07

---

## Context and Problem Statement

Tauri v2 bridges JavaScript/TypeScript frontend calls to Rust backend commands via inter-process communication (IPC). Two critical issues arose in initial implementations:

1. **CamelCase Argument Dropping:** Tauri v2 automatically transforms camelCase JavaScript parameter keys into snake_case Rust arguments. If the frontend passes snake_case keys directly (e.g. `{ clear_stitching_mode: "all" }`), Tauri silently drops the parameters, setting `Option<T>` arguments to `None` with **no runtime error or warning**.
2. **Unstructured String Errors:** Commands returning `Result<T, String>` converted rich backend errors into arbitrary strings via `.map_err(|e| e.to_string())`. This prevented the frontend from providing contextual error recovery actions (e.g., distinguishing a missing database vs a busy database lock).

---

## Decision Drivers

* **Type Safety Across Boundaries:** Maintain 100% type parity between Rust domain structs/enums and TypeScript interfaces.
* **Predictable Error Handling:** The frontend must receive strongly typed error codes rather than free-form error text.
* **Deterministic IPC Serialization:** Enforce camelCase JSON keys across all `invoke()` calls and assert this via unit tests.
* **Component Isolation:** UI components must never invoke IPC directly, keeping UI logic testable with standard component mocks.

---

## Considered Options

1. **Raw String IPC:** Continue passing ad-hoc objects and raw string error messages.
2. **Generic Error Codes:** Return standard HTTP-like status codes (200, 400, 500) over IPC.
3. **Structured `IpcError` Enum + Service Abstraction Layer (Chosen):** Centralize backend errors in a typed `IpcError` enum mirrored in TypeScript, encapsulate `invoke()` in service modules, and enforce camelCase wire payload contracts.

---

## Decision Outcome

**Option 3 (Structured `IpcError` Enum + Service Abstraction Layer)** was chosen.

### Architectural Invariants:
1. **Centralized Error Enum (`src/error.rs`):**
   - All Tauri commands return `Result<T, IpcError>`.
   - `IpcError` derives `thiserror::Error` and `serde::Serialize`.
   - TypeScript mirrors `IpcError` in `frontend/src/lib/types/errors.ts`.
2. **Strict CamelCase Invoke Contract:**
   - All `invoke()` calls use camelCase keys matching the snake_case Rust parameters.
   - Command adapter test suites must explicitly assert camelCase payload keys.
3. **Frontend Service Isolation:**
   - Svelte components import dedicated service modules (`src/lib/services/db.ts`, `src/lib/services/designs.ts`, etc.) and never call `invoke()` directly.
