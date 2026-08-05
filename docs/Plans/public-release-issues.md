# Public Release GitHub Issues

Use the sections below as a quick copy/paste source for GitHub issues.

## Numbered Priority Order

1. **Create a test coverage skill and establish a release-quality baseline**
2. **Refresh planning documents and create a complete feature inventory**
3. **Add public-facing licensing, acknowledgements, and no-liability information**
4. **Add end-user documentation for local use and USB-stick deployment**
5. **Add in-app Help/About support and improve contextual guidance**
6. **Remove the standalone advanced search page and keep search on the main browse flow**
7. **Create a safe-refactor skill and refactor the app in small, test-backed slices**
8. **Validate portable USB-stick delivery on a clean Windows sandbox or VM**
9. **Public-repo polish: contributing guide, release checklist, screenshots, and CI**

## Checklist Version

- [ ] Create a test coverage skill and establish a release-quality baseline
- [ ] Refresh planning documents and create a complete feature inventory
- [ ] Add public-facing licensing, acknowledgements, and no-liability information
- [ ] Add end-user documentation for local use and USB-stick deployment
- [ ] Add in-app Help/About support and improve contextual guidance
- [x] Remove the standalone advanced search page and keep search on the main browse flow
- [ ] Create a safe-refactor skill and refactor the app in small, test-backed slices
- [ ] Validate portable USB-stick delivery on a clean Windows sandbox or VM
- [ ] Public-repo polish: contributing guide, release checklist, screenshots, and CI

---

## Title
Create a test coverage skill and establish a release-quality baseline

## Body
We need a reusable workflow for improving test coverage safely before making larger public-release changes. This should define how we expand tests, where to focus first, and what "good enough" coverage means for this app.

Focus first on the highest-risk areas such as `src/routes/bulk_import.rs`, `src/services/auto_tagging.rs`, `src/routes/designs.rs` (search parsing lives here), and portable launcher smoke coverage (e.g., running the built debug/release EXEs via `start-rust-app-no-build.bat` / `start-rust-debug-exe.bat`). This issue should also capture the current `cargo test` baseline — 891 passing as of `docs/Plans/rust-refactor.md` Phase 0 — and set expectations for future test-backed changes.

---

## Title
Refresh planning documents and create a complete feature inventory

## Body
The planning docs need to reflect the app as it actually exists today, and contributors should be prompted to keep them updated when features change.

Review and update the existing files in `docs/Plans/`, fix stale references in `README.md` (currently minimal — it needs a proper project overview), and add a feature inventory covering all current user and admin functionality. Include a short note that future app changes should be reflected in the relevant planning documents.

---

## Title
Add public-facing licensing, acknowledgements, and no-liability information

## Body
Before the repository is made public, we should clearly document third-party licensing, acknowledgements, and "use at your own risk" information.

Audit dependencies referenced by `Cargo.toml`/`Cargo.lock` and `frontend/package.json`/`frontend/package-lock.json`. Public docs such as `LICENCE`, `third_party_notices.html`, `disclaimer.html`, `templates/info/privacy.html`, and `templates/info/security.html` already exist — this task is primarily an audit to confirm they are complete, accurate, and discoverable from the repo root and README. Also document guidance for handling `.env` secrets and optional Gemini API usage.

---

## Title
Add end-user documentation for local use and USB-stick deployment

## Body
The repo needs practical user documentation for both technical and non-technical users, especially if it will be shared publicly or run from removable media.

`docs/User-Facing-Guidance/` already contains a good set of guides (GETTING_STARTED, App Installer, BACKUP_RESTORE, IMPORT_WORKFLOW, SETTINGS, PROJECTS, SUPPORTED_FORMATS, and more). Remaining work: add or expand documentation specifically for USB-stick/portable deployment (feeding back findings from the portable validation issue), confirm each existing guide is accurate for the current build, and ensure a new user can get the app running using only the repository documentation plus `start-rust-app.bat`.

---

## Title
Add in-app Help/About support and improve contextual guidance

## Body
The app already contains scattered inline hints, but it would benefit from a dedicated Help/About area plus better tooltips on important controls.

Help and About views already exist in the Svelte frontend — `frontend/src/lib/views/HelpView.svelte`, `frontend/src/lib/views/AboutView.svelte`, and `frontend/src/lib/views/AboutDocumentView.svelte` — and are reachable via the `#/help` and `#/about` hash routes wired into `frontend/src/lib/MainView.svelte`. Remaining work: consolidate and expand the guidance content for search syntax, importing, projects, and maintenance, and keep concise tooltips in the UI for important actions, linking to troubleshooting where appropriate.

---

## Title
Remove the standalone advanced search page and keep search on the main browse flow

## Body
We are no longer using the separate advanced search page as a distinct destination, and its functionality has mostly moved into the main browsing experience.

The Rust/Svelte app never reintroduced a standalone advanced search route — search is implemented entirely on the main browse flow via `frontend/src/lib/MainView.svelte`, with query parsing and filtering logic in `src/routes/designs.rs` (e.g., `parse_general_search_groups`, `push_general_search_clause`). Confirm that the required search capabilities are available on the main browse page, update docs and tests so the simplified search UX is clear and protected, and keep any tests covering the search parser in `src/routes/designs.rs` up to date.

---

## Title
Create a safe-refactor skill and refactor the app in small, test-backed slices

## Body
We want better maintainability, but the refactor should be controlled, test-backed, and split into small pieces rather than a broad rewrite.

A safe-refactor plan already exists in `docs/Plans/rust-refactor.md` and should be the reference point. Create a Copilot skill that codifies its delivery model (freeze module, refactor, run focused tests, review) and identify high-value targets such as `src/routes/designs.rs`, duplicated search and filter logic, and the configuration/documentation drift related to portable deployment. The known hotspot files from `rust-refactor.md` — `src/routes/bulk_import.rs`, `src/routes/designs.rs`, `src/services/backfill.rs`, `src/routes/admin.rs`, `src/routes/maintenance.rs` — are the best starting candidates. Follow-up refactor work should be broken into smaller issues.

---

## Title
Validate portable USB-stick delivery on a clean Windows sandbox or VM

## Body
The repo already supports portable-style delivery, but we should verify that it works cleanly on a fresh machine with no developer setup.

Test the app using the Rust launchers — `build-rust-release.bat`, `start-rust-app.bat`, `start-rust-app-no-build.bat`, `start-rust-app.vbs`, and `start-rust-release.vbs` — in a clean Windows VM or Windows Sandbox. Confirm the portable vs installed execution mode resolution in `src/paths.rs` (`ExecutionMode::Portable` / `ExecutionMode::Installed`) behaves as documented. Capture any setup assumptions, limitations, or warnings and feed those results back into the end-user documentation.

---

## Title
Public-repo polish: contributing guide, release checklist, screenshots, and CI

## Body
Once the core release-readiness work is underway, the repo would benefit from a final round of public-facing polish.

Issue templates already exist under `.github/ISSUE_TEMPLATE/`, and `docs/screenshots/` already contains screenshots. Add `CONTRIBUTING.md`, `CHANGELOG.md`, a release checklist, and GitHub PR templates. Also consider adding CI checks — `cargo test`, `npx svelte-check`, and the frontend Vitest suite — and pinning dependency versions in `Cargo.lock` and `frontend/package-lock.json` to make releases more repeatable.