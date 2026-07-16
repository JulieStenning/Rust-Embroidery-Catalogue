## Stage 1: Functional bug fixes (MainView.svelte only)

1. **Implement advertised general search syntax**
   - Parse `browseFilters.q` for:
     - `"exact phrase"` (quoted phrases)
     - `-exclude` (exclusion terms)
     - `word1 OR word2` (any-word matching)
     - `*.hus` / `rose*` (wildcard patterns)
     - plain terms (all-words matching)
   - Apply against filename, tags, and folder based on the existing "Search in" checkboxes.
   - Keep the separate Filename field as-is for explicit filename-only wildcard search.

2. **Fix "Folder" and "Date added" sorting**
   - Extend `normalizeCardItem` to retain `folder` (from `source` or extracted from `filepath`) and `dateAdded` (from `date_added` or inferred from `id`).
   - Update `compareBrowseItems` to sort by these fields.

3. **Stabilize browse selection across pages**
   - Remove or restrict `syncBrowseSelectionFromDom()` so it does not overwrite `browseSelectedIds` on every DOM change.
   - Keep rune state as the single source of truth.

4. **Add request tokens to route-driven loaders**
   - `loadDesignDetail`, `loadProjectDetailView`, `loadProjectPrint`, `loadAboutDocumentView` should use request tokens or check the current route after `await` before committing results.

5. **Canonicalize help/route fragments**
   - Change help links from `#search`, `#projects`, `#/help#importing` to `#/help?section=search`, `#/help?section=projects`, `#/help?section=importing`.
   - Update `normalizeHash` to parse `?section=` query parameters instead of splitting on extra `#` characters.
   - Remove the global reservation of bare fragment IDs like `#search` → Help.

6. **Fix notice clearing timing**
   - Move notice clearing from the route-change effect to the page-load effects (e.g. clear `orphanActionMessage` when orphans page loads, not when navigating away).

## Stage 2: Conservative extraction of shared primitives

7. **Extract small shared helpers/components**
   - Create `frontend/src/lib/utils/tagHelpers.js` with `splitTagsByGroup` (remove duplicate `splitDetailTagsByGroup`).
   - Create `frontend/src/lib/components/Notice.svelte` for status/action messages.
   - Create `frontend/src/lib/components/Pagination.svelte` for browse and orphan pagination.
   - Create `frontend/src/lib/utils/asyncData.js` with a standard `{ loading, loaded, error, source }` pattern (optional, may just document the pattern).

8. **Remove dead code**
   - Delete empty `$effect(() => { return undefined; })` blocks.
   - Remove unused-looking state like `browseSource`, `browseProjectsSource`, `browseTagsSource`, `importPreviewSource` if they are not meaningfully consumed.

9. **Rename misleading functions**
   - `runBackupActionUiOnly` → `runBackupAction`
   - `saveBackupDestinationsUiOnly` → `saveBackupDestinations`
   - `browseBackupDestinationUiOnly` → `browseBackupDestination`

## Stage 3: Extract feature page components

10. **Extract Settings page**
    - Move settings state and logic into `frontend/src/lib/views/SettingsView.svelte`.
    - Keep `loadSettingsFromBackend`, `saveSettingsFromBackend`, `browseDataRootFromBackend` in the component.

11. **Extract Backup page**
    - Move backup state and logic into `frontend/src/lib/views/BackupView.svelte`.

12. **Extract Tagging Actions page**
    - Move tagging actions state and logic into `frontend/src/lib/views/TaggingActionsView.svelte`.

13. **Extract Orphans page**
    - Move orphans state and logic into `frontend/src/lib/views/OrphansView.svelte`.

14. **Extract Projects pages**
    - Move projects list/new/detail/print into `frontend/src/lib/views/ProjectsView.svelte` (or split into `ProjectsListView`, `ProjectNewView`, `ProjectDetailView`, `ProjectPrintView`).

15. **Extract Design detail/print pages**
    - Move design detail/print into `frontend/src/lib/views/DesignDetailView.svelte` and `frontend/src/lib/views/DesignPrintView.svelte`.

16. **Extract Import wizard**
    - Move import state and logic into `frontend/src/lib/views/ImportView.svelte`.

17. **Keep MainView as a thin router/shell**
    - MainView retains routing logic, nav, and route-switching, but delegates page rendering to extracted components.

## Stage 4: Add tooling scripts

18. **Install dev dependencies**
    - Add `svelte-check`, `eslint`, `eslint-plugin-svelte`, `prettier`, `prettier-plugin-svelte`, `typescript` to `frontend/package.json`.

19. **Add scripts to `frontend/package.json`**
    - `"check": "svelte-check"`
    - `"lint": "eslint src"`
    - `"format": "prettier --write src"`
    - `"format:check": "prettier --check src"`

20. **Add minimal config files**
    - `frontend/svelte.config.js` (already exists, may need tweaks)
    - `frontend/.eslintrc.cjs` or `eslint.config.js`
    - `frontend/.prettierrc` with Svelte plugin

## What I will not do in this pass

- I will not generalize admin CRUD (designers/sources/hoops/tags) into a generic system yet. That can be a follow-up after the conservative extraction is stable.
- I will not change the backend API contracts; I will only normalize frontend handling of existing snake_case/camelCase variations.
