# Frontend State Management (Svelte Stores)

This directory houses the reactive state management layer for the Embroidery Catalogue frontend, built with Svelte's `writable`, `readable`, and `derived` stores.

## Store Categories

The application categorizes state into three primary types:

### 1. Persistent Session Stores

Stores that retain user navigation context, filter selections, and active entity state across view transitions:

- **`browseSessionStore.ts`**: Manages search filters, active tags, sorting order, pagination, and view modes (grid vs list) in the main catalogue browser.
- **`designSessionStore.ts`**: Tracks the currently selected design, detail drawer state, zoom level, and active color block highlights.
- **`importSessionStore.ts`**: Preserves scanned folder paths, file conflict selections, and staged files during an import wizard session.

### 2. Global UI & Feedback Stores

Transient state shared across components:

- **`busyStore.ts`**: Global loading spinner and activity counter (`incrementBusy()`, `decrementBusy()`, `isBusy`). Prevents concurrent conflicting UI actions.
- **`toastStore.ts`**: Toast notification queue (`addToast()`, `removeToast()`) supporting `info`, `success`, `warning`, and `error` alerts with auto-dismiss timers.
- **`tagChangeStore.ts`**: Emits events when tags are modified, created, or deleted to prompt cache invalidation in dependent views.
- **`unmatchedFilesStore.ts`**: Tracks files encountered during scanning that could not be parsed by any known embroidery format reader.

### 3. Long-Running Progress Stores

Live streaming state updated by backend event listeners:

- **`backfillProgressStore.ts`**: Live progress metrics (processed counts, error counts, current stage) during bulk AI auto-tagging or thumbnail generation.
- **`restoreProgressStore.ts`**: Progress tracking for database archive restoration and schema rebuilding.

## Testing & Store Reset Pattern

To prevent test cross-contamination in Vitest and component tests, all stores expose a dedicated `reset()` or `clear()` helper. Test suites should call these helpers in `beforeEach()` or `afterEach()` hooks:

```typescript
import { resetBrowseSession } from "$lib/stores/browseSessionStore";
import { clearToasts } from "$lib/stores/toastStore";

beforeEach(() => {
  resetBrowseSession();
  clearToasts();
});
```
