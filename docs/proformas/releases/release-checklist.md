# Integrated Release Checklist

## 1. Code Attributions & License Registrations

* [ ]  **Rust License Check:** If new Rust crates were added, confirm their SPDX identifiers are included in the `accepted` array in `about.toml`. If they are not, then the build will fail and the output will explicitly name the unapproved license by its standard SPDX identifier.
* [ ]  **NPM License Check:** Confirm new frontend dependencies in `frontend/package.json` are ready for scanning. If running
  cmd /c "cd frontend && npx license-checker-rseidelsohn --excludePackages `"embroidery-catalogue-frontend@0.1.0`" --onlyAllow `"MIT;Apache-2.0;BSD-2-Clause;BSD-3-Clause;ISC;CC0-1.0;Zlib;MPL-2.0;Python-2.0`""
  runs without errors, then this requirement is satisfied.
* [ ]  **Root `LICENSE` Attributions:** If non-dependency code or ported algorithms were added, add entries to **ACKNOWLEDGEMENTS & SPECIAL ATTRIBUTIONS** in the root `LICENCE`and frontend/src/LICENSE files.
* [ ]  **UI Attributions:** Update **Acknowledgements & Code Porting Attributions** in @AboutView.svelte.

---

## 2. Version Synchronization & Configuration

* [ ]  Update `[package] version` in `Cargo.toml`.
* [ ]  Update `version` in `frontend/package.json`.
* [ ]  Update `version` in `src-tauri/tauri.conf.json`.
* [ ]  Verify `tauri.conf.json` matches `Cargo.toml`.
* [ ]  Commit all updated version files on the release branch.
* [ ]  Confirm root `package.json` contains the `generate:licences` and `build` scripts:

```json
"scripts": {
  "generate:licences": "cargo about generate about.hbs -o ./src/assets/licences.html && npx license-checker-rseidelsohn --start ./frontend --json --out ./src/assets/npm-licences.json",
  "build": "npm run generate:licences && tauri build"
}

```

---

## 3. Dependency Security, Audits & Maintenance

* [ ]  **Cargo Deny Configuration:** Run `cargo deny check` to run all configured checks. (You may need to install cargo-deny with
  `cargo install --locked cargo-deny` )
* [ ]  **Security Vulnerabilities:** Run `cargo audit > audit-results.txt` (you may need to install cargo-audit with
  `cargo install cargo-audit --locked` )
  Inspect audit-results.txt for any errors and warnings. The original developer drops the file into Gemini and asks advice about the results. Remove the file audit-results.txt.
* [ ]  **Dependency Tree & Duplicates:**
* [ ]  Run `cargo deny check bans`.
* [ ]  Run `cargo tree --duplicates >duplicates.txt` to inspect duplicate crate versions.
  Inspect duplicates.txt for any errors and warnings. The original developer drops the file into Gemini and asks advice about the results. Remove the file duplicates.txt.
* [ ]  **License & Source Validation:**
* [ ]  Run `cargo deny check licenses`.
* [ ]  Run `cargo deny check sources` to ensure crates originate from allowed registries.
* [ ]  **Frontend Security:** Run `npm audit` for frontend dependencies.The original developer asks Gemini for help resolving issues.
* [ ]  **Update Previews:**
* [ ]  Run `cargo update`
* [ ]  Run `cargo test`
* [ ]  Run `cargo check`
* [ ]  Run `cargo outdated > outdated.txt` to review available major/minor updates. (You may need to install cargo-outdated with
  `cargo install cargo-outdated`remove
  remove the file outdate.txt
* [ ]  **Lockfile Commit:** Confirm updated `Cargo.lock` and `package-lock.json` are committed.

---

## 4. Quality Gates

**Preparation**

* [ ]  Commit any changes resulting from the above actions.

**Test Gate**

* [ ]  Run backend tests (`cargo test`) — all suites pass.
* [ ]  Run frontend tests (`npx vitest run` from root) — all suites pass.
* [ ]  Capture test evidence in the release evidence document.

**Lint / Format / Type-Check Gate**

* [ ]  Run `cargo check` (zero errors).
* [ ]  Run `cargo clippy --all-targets` (no critical warnings). Run `cargo clippy --fix --bin "embroidery-catalogue" -p Rust-Embroidery-Catalogue --tests --` to fix any errors. Fix any remaining errors.
* [ ]  Run `cargo fmt --check`. Fix any errors with `cargo fmt` and confirm with `cargo fmt --check`
* [ ]  Run `cmd /c "cd frontend && npx svelte-check --tsconfig jsconfig.json"` (zero errors).
* [ ]  Run `cmd /c "cd frontend && npm run lint"`. Try to fix any errors with `cmd /c "cd frontend && npx eslint . --fix"`. Fix any remaining errors.
* [ ]  Run `cmd /c "cd frontend && npm run format:check"`.

**Migration Gate**

* [ ]  Classify release type per migration policies.
* [ ]  Compile and run the release to validate all SQL files in `migrations/`and Pass clean DB startup smoke test .
* [ ]  Pass existing DB upgrade smoke test by checking pages that use the updates in the database.
* [ ]  Verify custom data-root persistence via `config.json` bootstrap location. Do this with a fresh installation.
* [ ]  Confirm rollback/backup guidance for destructive migrations. -- Do this for the first upgrade release

**CI Gate**

* [ ]  All CI jobs pass green on release branch.
* [ ]  Capture CI run URL in release evidence doc.

---

## 5. Rollback Readiness

* [ ]  Confirm rollback strategy (backup restore + known-good installer).
* [ ]  Include backup-before-update guidance in release notes.
* [ ]  Document trigger criteria for hotfixes or rollbacks.

---

## 6. License Asset Generation & Release Build Execution

* [ ]  **Manual License Asset Verification:** Run `npm run generate:licences` in PowerShell.
* [ ]  Confirm generation of `src/assets/licences.html`.
* [ ]  Confirm generation of `src/assets/npm-licences.json`.
* [ ]  **Execute Release Build:** Run `npm run build` (or `build-rust-release.bat` / `cargo tauri build`).
* [ ]  Verify Vite bundles license assets for rendering in @AboutView.svelte and @AboutDocumentView.svelte.

---

## 7. Artifact Verification & Testing

* [ ]  Locate MSI installer in `target/release/bundle/msi/`.
* [ ]  Locate NSIS installer in `target/release/bundle/nsis/`.
* [ ]  Verify installer filenames contain correct version string.
* [ ]  Compute SHA-256 checksums and verify artifacts.
* [ ]  Complete code signing if required.
* [ ]  Execute test installation and upgrade in a clean Windows environment.

---

## 8. Release Documentation & Publishing

* [ ]  Create release evidence document from `release-evidence-template.md`.
* [ ]  Promote `CHANGELOG.md` `[Unreleased]` section to versioned entry.
* [ ]  Compile Release Notes (Summary, Migration Notes, Backup Reminder, Known Issues).
* [ ]  Draft GitHub Release with notes, installer artifacts, and checksums attached.

---

## 9. Final Sign-Off & Post-Publish

* [ ]  Release Owner Sign-off (Name / Date): ____________________
* [ ]  Verifier Sign-off (Name / Date): ____________________
* [ ]  Final Decision: **GO / NO-GO**
* [ ]  **24–48h Post-Publish Monitoring:** Assign owner and record sign-off.
* [ ]  **Post-Release Dependency Maintenance:** Run `cargo update` and `npm install`, then commit refreshed `Cargo.lock` and `package-lock.json` on the next dev branch.
