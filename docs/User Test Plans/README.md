# Embroidery Catalogue: Testing Strategy & User Verification

## 1. Overview & Testing Pyramid

The application employs a 3-tier testing strategy to ensure reliability, security, and UI stability:

```
                  ┌───────────────────────────────┐
                  │   Manual Acceptance Suite     │  (OS Dialogs, Real Upgrades,
                  │  (Release & Hardware Charters)│   USB Media, Visual Polish)
                  ├───────────────────────────────┤
                  │     Playwright E2E Tests      │  (Full UI/UX Workflows,
                  │   (tests/e2e/*.spec.ts)       │   Tauri IPC, SQLite DB)
                  ├───────────────────────────────┤
                  │   Unit & Integration Tests    │  (Svelte Stores, Vitest,
                  │  (Vitest + Cargo backend)     │   Rust Handlers, Migrations)
                  └───────────────────────────────┘
```

1. **Unit & Backend Tests:** Fast, isolated testing of business logic, database migrations, state stores, and file parsers.
2. **Playwright E2E Suite:** Automated regression testing covering standard user flows, CRUD operations, batch actions, search/filtering, and modal dialogs across all views.
3. **Manual Acceptance Suite:** High-value manual verifications focusing strictly on areas that cannot be reliably automated (native OS file pickers, Windows installation/uninstallation lifecycles, database schema upgrades over real legacy data, USB drive removal, and DPI scaling).

---

## 2. Automated Feature Index (Playwright E2E)

The following application domains are **100% automated**. Prior to release, these are executed automatically via `run-release-checks.ps1` (or manually with `npm run e2e` / `npx playwright test`):

| Feature / Domain | Automated Test Spec | Coverage Highlights |
| :--- | :--- | :--- |
| **Browse & Search** | `tests/e2e/browse.spec.ts` | Grid layout, responsive columns, Google-like search syntax, tri-state verification, sorting, filter drawers, ratings, and selection counts. |
| **Batch Tagging** | `tests/e2e/batch-tagging.spec.ts` | Multi-select tagging, batch assignment of image & stitching tags, state persistence. |
| **Design Details** | `tests/e2e/design-detail.spec.ts` | Form inputs, dropdown bindings, rating clicks, sequential next/previous navigation, single deletion modal. |
| **Bulk Deletion** | `tests/e2e/browse.spec.ts` | Multi-select batch deletion, "Database Only" vs "Database & Trash" modes, modal safety. |
| **Bulk Import Wizard** | `tests/e2e/import.spec.ts`<br>`tests/e2e/import-folder-selection.spec.ts`<br>`tests/e2e/import-hoop-setup.spec.ts` | 3-step import wizard, directory scanning, duplicate detection, hoop setup gates, rule application, and progress reporting. |
| **Manage Designers** | `tests/e2e/admin-designers.spec.ts` | Add, inline edit, delete, clear, case-insensitive duplicate checks, and relation decoupling. |
| **Manage Hoops** | `tests/e2e/admin-hoops.spec.ts` | Add/edit dimensions (mm/inch), rotation, sorting, delete protection. |
| **Manage Sources** | `tests/e2e/admin-sources.spec.ts` | Source CRUD, duplicate checks, assignment validation. |
| **Manage Tags** | `tests/e2e/admin-tags.spec.ts` | Image vs Stitching tag groups, tag merges, editing, deletion checks. |
| **Projects** | `tests/e2e/projects.spec.ts` | Project creation, adding/removing designs from Browse, renaming, project deletion. |
| **Backup & Restore** | `tests/e2e/backup.spec.ts`<br>`tests/e2e/restore.spec.ts` | Backup creation, restore execution, schema conflict detection, corrupt DB rollback. |
| **Orphans & Maintenance**| `tests/e2e/orphans.spec.ts`<br>`tests/e2e/batch-maintenance.spec.ts` | Orphan detection, backfill actions, database integrity checks. |
| **Help & Settings** | `tests/e2e/help.spec.ts`<br>`tests/e2e/settings.spec.ts` | Help topics, external link attributes, search documentation, settings persistence. |

---

## 3. Manual Testing Suites

Manual testing is reserved for release validation and physical edge cases:

* [**Release-Manual-Acceptance.md**](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/docs/User%20Test%20Plans/Release-Manual-Acceptance.md): Execute for Minor and Major releases per the [Release Types and Migration Scope Policy](../policies/releases/release-types-and-migration-scope.md).
* [**Hardware-and-Edge-Cases.md**](file:///d:/My%20Software%20Development/Rust-Embroidery-Catalogue/docs/User%20Test%20Plans/Hardware-and-Edge-Cases.md): Exploratory testing for external USB drives, high-DPI scaling, and large database volume stress tests.
