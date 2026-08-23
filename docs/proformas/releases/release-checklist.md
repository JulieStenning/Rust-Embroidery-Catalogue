# Integrated Release Checklist

## Setup

Most of these actions can be run using the powershell script `run-release-checks.ps1`. To run the script, open powershell. Cd to the root location of the repo. Run `.\run-release-checks.ps1` in the powershell window. Links to the individual output files are shown in this document. The files are in the folder audit-logs.

## 1. Code Attributions & License Registrations

* [ ]  Rust License Check: Run `cargo about generate about.hbs`. If new Rust crates were added, confirm their SPDX identifiers are included in the accepted array in about.toml. If an unapproved license is detected, the command will fail immediately and display the standard SPDX identifier needing review. See [audit-logs/licences-preview.html](../../../audit-logs/licences-preview.html)
* [ ]  [ ] **NPM License Check:** Run `npx license-checker-rseidelsohn --start ./frontend --onlyAllow "MIT;Apache-2.0;BSD-2-Clause;BSD-3-Clause;ISC;CC0-1.0;Zlib;MPL-2.0;Python-2.0"`. Confirms all active frontend dependencies use approved open-source licenses. If an unapproved license is present, the scan will stop immediately and display the non-compliant package name. A successful check produces a large text list mapping every installed frontend package to its license details<sup></sup>. See [audit-logs/npm-license-check.txt](../../../audit-logs/npm-license-check.txt)
* [ ]  **Root `LICENSE` Attributions:** If non-dependency code or ported algorithms were added, add entries to **ACKNOWLEDGEMENTS & SPECIAL ATTRIBUTIONS** in the root `LICENCE`and frontend/src/LICENSE files. See the sections in these files about pyembroidery for examples.
* [ ]  **UI Attributions:** Update **Acknowledgements & Code Porting Attributions** in @AboutView.svelte. See section on pyembroidery for an example.

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
  `cargo install --locked cargo-deny` ). See [cargo-deny-results](../../../audit-logs/cargo-deny-results.txt)
* [ ]  **Security Vulnerabilities:** Run `cargo audit > audit-results.txt` (you may need to install cargo-audit with
  `cargo install cargo-audit --locked` ).
  Inspect [cargo-audit-results](../../../audit-logs/cargo-audit-results.txt) for any errors and warnings. The original developer drops the file into Gemini and asks advice about the results.
* [ ]  **Dependency Tree & Duplicates:**
* [ ]  Run `cargo deny check bans`. See [cargo-deny-bans](../../../audit-logs/cargo-deny-bans.txt)
* [ ]  Run `cargo tree --duplicates >duplicates.txt` to inspect duplicate crate versions. See
  Inspect [duplicates](../../../audit-logs/duplicates.txt) for any errors and warnings. The original developer drops the file into Gemini and asks advice about the results.
* [ ]  **License & Source Validation:**
* [ ]  Run `cargo deny check licenses`. See [cargo-deny-licenses](../../../audit-logs/cargo-deny-licenses.txt)
* [ ]  Run `cargo deny check sources` to ensure crates originate from allowed registries. See  [cargo-deny-sources](../../../audit-logs/cargo-deny-sources.txt)
* [ ]  **Frontend Security:**
* [ ]  Run `npm audit` for frontend dependencies. See [npm-audit-results](../../../audit-logs/npm-audit-results.txt). Run `npm audit fix` if there are issues and run 'npm audit' again. This does not always fix the vulnerabilities. The original developer asks Gemini for help resolving issues.
* [ ]  **Update Previews:**
* [ ]  Run `cargo update` This isn't included in the script
* [ ]  Run `cargo test` See [cargo-test-results](../../../audit-logs/cargo-test-results.txt)
* [ ]  Run `cargo check` This isn't included in the script
* [ ]  Run `cargo outdated > outdated.txt` to review available major/minor updates. (You may need to install cargo-outdated with
  `cargo install cargo-outdated`
  see [outdated.txt](../../../audit-logs/outdated.txt)
* [ ]  **Lockfile Commit:** Confirm updated `Cargo.lock` and `package-lock.json` are committed.

---

## 4. Quality Gates

**Preparation**

* [ ]  Commit any changes resulting from the above actions.

**Test Gate**

* [ ]  Run backend tests (`cargo test`) — all suites pass. See [cargo-test-results2](../../../audit-logs/cargo-test-results2.txt)
* [ ]  Run frontend tests (`npx vitest run` from root) — all suites pass. See [vitest-results](../../../audit-logs/vitest-results.txt)
* [ ]  Capture test evidence in the release evidence document.

**Lint / Format / Type-Check Gate**

* [ ]  Run `cargo check` (zero errors). See [cargo-check-results](../../../audit-logs/cargo-check-results.txt)
* [ ]  Run `cargo clippy --all-targets` (no critical warnings). Run `cargo clippy --fix --bin "embroidery-catalogue" -p Rust-Embroidery-Catalogue --tests --` to fix any errors. Fix any remaining errors. See [cargo-clippy-results](../../../audit-logs/cargo-clippy-results.txt)
* [ ]  Run `cargo fmt --check`. Fix any errors with `cargo fmt` and confirm with `cargo fmt --check` See [rust-fmt-results](../../../audit-logs/rustfmt-results.txt)
* [ ]  Run `cmd /c "cd frontend && npx svelte-check --tsconfig jsconfig.json"` (zero errors). See [svelte-check](../../../audit-logs/svelte-check.txt)
* [ ]  Run `cmd /c "cd frontend && npm run lint"`. Try to fix any errors with `cmd /c "cd frontend && npx eslint . --fix"`. Fix any remaining errors. See [eslint-results](../../../audit-logs/eslint-results.txt)
* [ ]  Run `cmd /c "cd frontend && npm run format:check"`. See [format-check-results](../../../audit-logs/format-check-results.txt)

**Migration Gate**

* [ ]  Classify release type per migration policies.
* [ ]  Compile and run the release to validate all SQL files in `migrations/`and Pass clean DB startup smoke test .
* [ ]  Pass existing DB upgrade smoke test by checking pages that use the updates in the database.
* [ ]  Verify custom data-root persistence via `config.json` bootstrap location. Do this with a fresh installation.
* [ ]  Confirm rollback/backup guidance for destructive migrations. -- Do this for the first upgrade release

**CI Gate**

* [ ]  All CI jobs pass green on release branch - run `release-check.yaml` in actions in github. Don't worry about the warning about Node.js 20 being depracated. This is to do with actions rather than the app and will disppear once the github team have updated action versions.
* [ ]  Capture CI run URL in release evidence doc. Copy the URL from github.

---

## 5. Rollback Readiness - Not until first upgrade

* [ ]  Confirm rollback strategy (backup restore + known-good installer).
* [ ]  Include backup-before-update guidance in release notes.
* [ ]  Document trigger criteria for hotfixes or rollbacks.

---

## 6. License Asset Generation & Release Build Execution

* [ ]  **Manual License Asset Verification:** Run `npm run generate:licences` in PowerShell. See [license-assets](../../../audit-logs/license-assets.txt)
* [ ]  Confirm generation of `src/assets/licences.html`.
* [ ]  Confirm generation of `src/assets/npm-licences.json`.
* [ ]  **Execute Release Build:** Run `npm run build` (or `build-rust-release.bat` / `cargo tauri build`). See [build-results](../../../audit-logs/build-results.txt)
* [ ]  Verify Vite bundles license assets for rendering in @AboutView.svelte and @AboutDocumentView.svelte.

---

## 7. Artifact Verification & Testing

* [ ]  Locate MSI installer in `target/release/bundle/msi/`.
* [ ]  Locate NSIS installer in `target/release/bundle/nsis/`.
* [ ]  Verify installer filenames contain correct version string.
* [ ]  Compute SHA-256 checksums and verify artifacts.
  `Get-ChildItem -Path "target/release/bundle" -Recurse -File -Include *.exe, *.msi | Get-FileHash -Algorithm SHA256 | Format-Table -AutoSize`
  This passes if 2 SHA256 Hash keys are returned, 1 for the msi and the other for the nsis. See [checksums](../../../audit-logs/checksums.txt)
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
