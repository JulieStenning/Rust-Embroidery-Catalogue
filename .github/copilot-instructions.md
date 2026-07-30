# Rust Refactoring Rules for Embroidery Catalogue

When suggesting or applying Rust code refactorings, strictly adhere to the following principles:

## 1. Error Handling & Safety
- **No `unwrap()` or `expect()` in Production Code:** Replace all explicit panics with proper error propagation using the `?` operator.
- **Strongly Typed Errors:** Use `thiserror` for library/domain-specific error enums (e.g., binary parsing failures, file I/O) and `anyhow` only at top-level command boundaries.
- **Fail Gracefully on Corrupt Files:** When parsing binary embroidery designs (`.pes`, `.jef`, etc.), return structured results/warnings rather than halting full directory scans.

## 2. Performance & Memory Management
- **Zero-Copy Parsing:** Use `zerocopy` or `nom` slice references where possible to avoid unnecessary allocations when reading binary headers.
- **Minimize Clone:** Prefer passing references (`&str`, `&[u8]`, `&Path`) over owned types unless ownership transfer is strictly required.
- **Streaming & Chunking:** Process file scans and stitch batch operations using iterators or chunked streams to keep memory overhead predictable during large library imports.

## 3. Architecture & Separation of Concerns
- **Isolate Binary Readers:** Keep binary format decoders strictly isolated from database logic and Tauri IPC commands (`#[tauri::command]`).
- **Domain vs. DB Models:** Maintain clear separation between raw binary parser output, SQLite database models, and IPC transport DTOs.
- **Tauri Commands as Thin Adapters:** Ensure `#[tauri::command]` functions contain minimal logic—they should simply delegate to core domain services and handle response serialization.

## 4. Code Quality & Idiomatic Rust
- **Immutability First:** Keep variables immutable (`let`) by default; restrict `mut` to localized scopes.
- **Leverage Combinators:** Prefer iterator combinators (`map`, `filter_map`, `flat_map`) over manual `for` loops with mutable accumulators.
- **Type Safety:** Use newtypes or explicit enums rather than raw primitives for domain concepts (e.g., `StitchIndex(usize)`, `FormatType`).