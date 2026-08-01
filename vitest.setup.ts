import "@testing-library/jest-dom/vitest";

// NOTE: `@testing-library/svelte` auto-registers its own beforeEach/afterEach
// hooks when test globals are enabled (see `globals: true` in vitest.config.ts),
// so no manual `cleanup()` call is needed here.