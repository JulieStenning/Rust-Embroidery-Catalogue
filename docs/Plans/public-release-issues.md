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
We need a reusable workflow for improving test coverage safely before making larger public-release changes. This should define how we expand tests, where to focus first, and what “good enough” coverage means for this app.

Focus first on the highest-risk areas such as `src/services/auto_tagging.py`, `src/services/bulk_import.py`, `src/services/search.py`, `src/routes/bulk_import.py`, and portable launcher smoke coverage. This issue should also capture the current `pytest` baseline and set expectations for future test-backed changes.

---

## Title
Refresh planning documents and create a complete feature inventory

## Body
The planning docs need to reflect the app as it actually exists today, and contributors should be prompted to keep them updated when features change.

Review and update the existing files in `docs/Plans/`, fix stale references in `README.md`, and add a feature inventory covering all current user and admin functionality. Include a short note that future app changes should be reflected in the relevant planning documents.

---

## Title
Add public-facing licensing, acknowledgements, and no-liability information

## Body
Before the repository is made public, we should clearly document third-party licensing, acknowledgements, and “use at your own risk” information.

Audit dependencies referenced by `pyproject.toml`, `requirements.txt`, `wheels/`, and the bundled `python/` runtime. Add public docs such as `LICENSE`, `THIRD_PARTY_NOTICES.md`, `DISCLAIMER.html`, `PRIVACY.md`, and `SECURITY.md`, including guidance for handling `.env` secrets and optional Gemini API usage.

---

## Title
Add end-user documentation for local use and USB-stick deployment

## Body
The repo needs practical user documentation for both technical and non-technical users, especially if it will be shared publicly or run from removable media.

Add getting-started docs for local setup and startup, USB-stick deployment, backup and restore of the SQLite data, troubleshooting, and optional AI tagging setup. The goal is that a new user can get the app running using only the repository documentation.

---

## Title
Add in-app Help/About support and improve contextual guidance

## Body
The app already contains scattered inline hints, but it would benefit from a dedicated Help/About area plus better tooltips on important controls.

Add a `/help` or `/about` page, link it from `templates/base.html`, and consolidate guidance for search syntax, importing, projects, and maintenance. Keep concise tooltips in the UI for important actions and link to troubleshooting where appropriate.

---

## Title
Remove the standalone advanced search page and keep search on the main browse flow

## Body
We are no longer using the separate advanced search page as a distinct destination, and its functionality has mostly moved into the main browsing experience.

Confirm that the required search capabilities are available on the main browse page, then remove or retire the dedicated advanced search route and template. Update navigation, docs, and tests so the simplified search UX is clear and protected.

---

## Title
Create a safe-refactor skill and refactor the app in small, test-backed slices

## Body
We want better maintainability, but the refactor should be controlled, test-backed, and split into small pieces rather than a broad rewrite.

Create a Copilot skill for safe refactoring and identify high-value targets such as `src/routes/designs.py`, duplicated search and filter logic, and any configuration/documentation drift related to portable deployment. Follow-up refactor work should be broken into smaller issues.

---

## Title
Validate portable USB-stick delivery on a clean Windows sandbox or VM

## Body
The repo already supports portable-style delivery, but we should verify that it works cleanly on a fresh machine with no developer setup.

Test the app using `populate_sdcard.bat`, `setup.bat`, `start.bat`, and `portable_launcher.py` in a clean Windows VM or Windows Sandbox. Capture any setup assumptions, limitations, or warnings and feed those results back into the documentation.

---

## Title
Public-repo polish: contributing guide, release checklist, screenshots, and CI

## Body
Once the core release-readiness work is underway, the repo would benefit from a final round of public-facing polish.

Add `CONTRIBUTING.md`, `CHANGELOG.md`, a release checklist, screenshots for `README.md`, and GitHub issue or PR templates. Also consider adding CI checks and pinning dependency versions to make releases more repeatable.
