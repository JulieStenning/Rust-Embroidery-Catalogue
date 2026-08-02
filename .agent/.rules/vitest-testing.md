# Svelte Component & Unit Testing Guidelines (Vitest + @testing-library/svelte)

When creating or modifying test files in the Svelte frontend, strictly adhere to the following rules:

## 1. Component Rendering & Props
- **Nested `props` Object:** Always wrap component props in a nested `props` key: `render(Component, { props: { propName: value } })`. Never pass props top-level inside the `render` options parameter.
- **Render Cleanup:** Rely on `@testing-library/svelte` automatic cleanup between tests. Do not add explicit `cleanup()` calls in `afterEach` hooks.

## 2. Tauri IPC & Native Module Mocking
- **Mock Native Invokes:** Tests run in `jsdom` without the Rust runtime. Always mock `@tauri-apps/api/core` (`invoke`) or path modules using `vi.mock(...)` at the top of test files.
- **Typed Mock Payloads:** Ensure mocked `invoke` responses return data matching backend Rust structs/DTOs (e.g., returning valid arrays for design objects, tags, or settings).

## 3. Asynchronous Updates & State Flushing
- **Svelte Reactive Updates:** When asserting DOM changes after triggering an event or updating store state, use `await tick()` from `svelte` or `await screen.findBy*` queries to wait for Svelte DOM reconciliation.
- **User Interactions:** Use `@testing-library/user-event` with `await` for user interactions (clicking, typing, modal triggers) rather than raw `fireEvent`.

## 4. Querying & Accessibility
- **Accessible Queries First:** Prefer semantic queries (`getByRole`, `getByLabelText`, `getByText`) over CSS selectors or `data-testid` attributes.

## 5. Store & State Isolation
- **Prevent Cross-Contamination:** If a test modifies global Svelte stores (e.g., selected design IDs, filter state, modal visibility), explicitly reset store values in `beforeEach` or `afterEach` hooks.