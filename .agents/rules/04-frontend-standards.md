# Frontend & Svelte 5 Coding Standards

## 🎨 Frontend & TypeScript Coding Standards

### 1. Native Svelte 5 TypeScript & Type Parity

- All new and refactored Svelte components must use `<script lang="ts">` with typed runes (`$state`, `$derived`, `$props<{ ... }>()`).
- Maintain strict type parity across the IPC bridge. If a Rust `struct` is returned by a Tauri command, create a matching TypeScript `interface` in `src/lib/types/` (and mirror `IpcError` in `src/lib/types/errors.ts`).

### 2. Component Size Limit (< 500 lines)

- Svelte components and views must remain under **500 lines**.
- Decompose complex views into cohesive subcomponents (e.g., `BrowseFilterPanel.svelte`, `BrowseSelectionBar.svelte`, `BrowseCardGrid.svelte` under `frontend/src/lib/components/browse/`).

### 3. Strict Typing & No Implicit Any

- Every single function/method/arrow parameter must be explicitly typed:
  - **No Implicit Any (TS7006):** Never leave arrow functions or inner closure parameters untyped (e.g. `const rank = (/** @type {string} */ name) => { ... }`).
  - **Empty Collection State (TS7005):** Always provide explicit JSDoc annotations when initializing empty array/object state with `$state([])` or `$state({})` (e.g. `/** @type {string[]} */ let items = $state([]);` or `let items = $state(/** @type {string[]} */ ([]));`), otherwise TypeScript infers `any[]`.
- **Guard nullable values before narrowing calls:** Short-circuit `null` first when passing into APIs requiring non-null types:
  ```typescript
  const uiKind = resolveCurrentUiKind(route); // string | null
  // ✅ uiKind !== null && UTILITY_KINDS.has(uiKind)
  ```

### 4. Lint & Type Verification

- Execute `cmd /c "cd frontend && npx svelte-check --tsconfig jsconfig.json"` after modifying frontend files to ensure zero type errors.
- Do **not** run `npm run check` from the repo root (the script lives in `frontend/package.json`).

### 5. Route-Level State Persistence

- `MainView.svelte` conditionally mounts one view per `currentUiKind`. Unmounting destroys local `$state`. Lift multi-step wizard state into module stores (e.g. `src/lib/stores/importSessionStore.ts`).
- Use context-aware back navigation via `previousRoute` in `MainView.svelte`.

### 6. Ambient module declarations (.d.ts)

- A `.d.ts` file with top-level `import`/`export` becomes a module augmentation and stops applying globally. Keep wildcard files pure scripts and use `/// <reference types="..." />`.

### 7. Visual Consistency & Canonical UI Theme Spec

- All UI components, surfaces, typography, forms, tables, and modal dialogs must strictly comply with the canonical [Look and Feel & UI Theme Implementation Specification](docs/Specs/look-and-feel-implementation-spec.md) and [UI Global Standards](docs/Specs/ui-global-standards.md).
- **Theme Parity & Token Rules:** Always use CSS custom property tokens (`--surface-*`, `--text-*`, `--border-*`). Never use un-gated `@media (prefers-color-scheme: dark)` overrides without `:root:not([data-theme="light"])`, and avoid hardcoded raw opacity classes (like `bg-gray-50/50`) on cards.
- **Reference Gold Standards:** Model new pages and components on **Browse Designs**, **Choose Tags**, and **Design Details**.
- Primary action buttons use the app's purple/indigo + white look (`settings-primary-button menu-button-primary` classes). Do not override with ad-hoc colors.

### 8. Zero Native Webview Dialogs & Svelte Modal Invariant (ADR 010)

- **Absolute Ban on Native Webview Dialogs:** Never use `window.confirm()`, `window.alert()`, or `window.prompt()`. Synchronous webview popups freeze the WebView2 UI thread, break theme styling, and stall headless Playwright e2e test runs.
- **Theme-Compliant Svelte Modals:** All user confirmations and alerts must use accessible in-app Svelte components (`@Notice.svelte`, `@ConfirmDeleteProjectModal.svelte`, `@DeleteDesignsModal.svelte`, `@ConfirmRestoreModal.svelte`, etc.) styled with CSS theme custom properties.
- **Deterministic Test Selectors:** Every modal dialog container and confirmation/dismiss button must provide explicit `data-testid` attributes (e.g. `data-testid="confirm-delete-button"`).

---

## 🧪 Svelte 5 View Testing Rules

- **Flush synchronous reactivity with `tick()`, not polling:**
  ```typescript
  import { tick } from "svelte";
  await fireEvent.change(select, { target: { value: "some-value" } });
  await tick();
  const cards = screen.getAllByRole("article");
  ```
- **Scope `getByText` with `within()`** when text appears in multiple DOM regions.
- **Direct mock assertions over `waitFor`** for synchronous side-effects.
- **Choose fixture data that proves state changes:** Default order should differ from expected sorted/filtered order.
- **Mock components should expose `data-testid` and `data-*` attributes.**
- **Route-driven props branching templates must use `$derived`, not `const`.**
- **`vi.mock` hoisting:** Use `vi.hoisted()` for fixture objects passed into `vi.mock()`.
- **Module-scope re-evaluations:** Use `vi.resetModules()` + `vi.doMock()` and dynamically re-import both the test harness (`render`, `tick`) and component together to avoid dual Svelte instance issues.
- **Module Singleton Store Isolation:** Module-level state stores (e.g. `importSessionStore.ts`, `browseStore.ts`) persist across tests. Always export and call a reset helper (e.g. `importSessionStore.reset()`) in `beforeEach()` to avoid cross-test contamination.
- **Prettier & Svelte textContent Whitespace:** Multi-line template text can format with variable indentation. Use regex matchers (e.g. `/Scanning\.\.\./i`) or normalize whitespace `(el.textContent ?? "").replace(/\s+/g, " ").trim()` before asserting.
