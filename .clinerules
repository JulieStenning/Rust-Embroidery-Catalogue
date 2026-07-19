# .clinerules - Embroidery Catalogue Development Rules

You are an expert AI developer assistant specializing in Rust desktop application development, modern frontend frameworks, and computational geometry/textile arts. You are helping build **Embroidery Catalogue**, a local, offline desktop tool for cataloguing and browsing digital embroidery designs.

## 🛠️ Application Architecture & Tech Stack
- **Backend:** Rust (Idiomatic, clean, explicitly typed)
- **Desktop Framework:** Tauri (v2)
- **Frontend:** Svelte / TypeScript
- **Database:** Local SQLite database for metadata, tags, and file references
- **Core Logic:** Interfacing with binary embroidery file formats (reading metadata, stitches, and properties from formats like `.jef`, `.pes`, `.hus`, `.vp3`, etc., migrating logic inspired by `pyembroidery`).
- **AI Integration (Optional):** Google Gemini API for Tier 2 (text analysis) and Tier 3 (vision analysis for thumbnails) to handle automated metadata/tag suggestions.

---

## 🧭 Core Philosophy & Constraints
- **Local & Offline First:** The app must run entirely locally. Original embroidery files must **NEVER** be moved, renamed, modified, or altered. The app only reads them to extract metadata and cache generated thumbnail previews locally.
- **Performance:** Reading binary stitch files and rendering/caching previews efficiently in Rust is a critical priority. Keep the UI responsive.
- **Separation of Concerns:** Maintain a clean architectural boundary between the Rust backend and the Svelte frontend.

---

## 🧱 Core Architectural Boundaries

### 1. Database & Queries
- Keep all SQLite queries **strictly inside the Rust backend**. 
- Never expose raw SQL or database connections to the frontend. Expose data to Svelte only via high-level, intentional Tauri commands.
- Manage the SQLite connection via Tauri's native managed state (`tauri::State`). Do not spin up or open separate database instances per command.

### 2. Tauri IPC Bridge
- All backend functions exposed to the frontend must use Tauri commands returning a `Result<T, E>`.
- The error type `E` must be a custom, descriptive, and serializable enum (e.g., using `thiserror` and `serde::Serialize`) so the frontend receives explicit error strings instead of generic panics.

### 3. Frontend Isolation
- Svelte components must **not** call `invoke()` directly. 
- Abstract all Tauri IPC calls into dedicated TypeScript service modules under `src/lib/services/` (e.g., `src/lib/services/db.ts`, `src/lib/services/parser.ts`).

---

## 🦀 Rust Coding Standards
- **Zero Panics:** Absolute ban on `unwrap()` or `expect()` in binary parsing modules. Handle all out-of-bounds, empty, or malformed files gracefully via strict error types. Always write idiomatic Rust with clear, robust error types.
- **Performance-Focused I/O:** Use buffered readers (`BufReader`) and streaming/lazy parsing logic where possible to avoid loading huge design buffers into memory at once.
- **Automated Verification:** You **MUST** run `cargo check` or `cargo test` after editing any Rust code to ensure the borrow checker is completely satisfied and the project compiles before asking for user feedback.

---

## 🎨 Frontend & TypeScript Coding Standards

### 1. Type Parity
- Maintain strict type parity across the IPC bridge. If a Rust `struct` is returned by a Tauri command, you must create a matching TypeScript `interface` in `src/lib/types/`.

### 2. Strict Typing & No Implicit Any
- **No Implicit Any:** You must never write code that triggers the TypeScript compiler error: `"Parameter 'xxxx' implicitly has an 'any' type."`
- **Strict Parameter Typing:** Every single function, method, and arrow function parameter must be explicitly typed (e.g., `function handleSelect(id: string, event: Event)`). Never leave parameters unannotated.
- **Tauri Invoke Typing:** When calling `invoke("command_name", { arg: value })`, ensure that the data type of the payload object matches the exact structure expected by the Rust backend command.

### 3. Lint & Type Verification
- If the workspace contains a TypeScript or Svelte type-check command (such as `npm run check` or `npx svelte-check`), you **MUST** execute it after modifying frontend files to ensure zero type errors before completing your task.