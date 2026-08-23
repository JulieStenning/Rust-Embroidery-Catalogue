# Integrated Release Checklist

## Setup

Most of these actions can be run using the powershell script `run-release-checks.ps1`. To run the script, open powershell. Cd to the root location of the repo. Run `.\run-release-checks.ps1` in the powershell window. Links to the individual output files are shown in this document. The files are in the folder audit-logs.

The script `verify-release-logs.ps1` identifies successful and failing scripts.  To run the script, open powershell. Cd to the root location of the repo. Run `.\verify-release-logs.ps1` in the powershell window.

## 1. Code Attributions & License Registrations

* [ ]  **Rust License Check.** If new Rust crates were added, confirm their SPDX identifiers are included in the accepted array in about.toml. If an unapproved license is detected, the command will fail immediately and display the standard SPDX identifier needing review. See [audit-logs/licences-preview.html](../../../audit-logs/licences-preview.html)
* [ ]  [ ] **NPM License Check.** Confirms all active frontend dependencies use approved open-source licenses. If an unapproved license is present, the scan will stop immediately and display the non-compliant package name. A successful check produces a large text list mapping every installed frontend package to its license details<sup></sup>. See [audit-logs/npm-license-check.txt](../../../audit-logs/npm-license-check.txt)
* [ ]  **Root `LICENSE` Attributions.** If non-dependency code or ported algorithms were added, add entries to **ACKNOWLEDGEMENTS & SPECIAL ATTRIBUTIONS** in the root `LICENCE`and frontend/src/LICENSE files. See the sections in these files about pyembroidery for examples.
* [ ]  **UI Attributions.** Update **Acknowledgements & Code Porting Attributions** in @AboutView.svelte. See section on pyembroidery for an example.

---

## 2. Version Synchronization & Configuration

* [ ]  Update `[package] version` in `Cargo.toml`.
* [ ]  Update `version` in `frontend/package.json`.
* [ ]  Update `version` in `src-tauri/tauri.conf.json`.
* [ ]  Verify `tauri.conf.json` matches `Cargo.toml`.
* [ ]  Commit all updated version files on the release branch.
* [ ]  Confirm root `package.json` contains the `generate:licences` and `build` scripts:

## 3. Dependency Security, Audits & Maintenance

* [ ]  **Cargo Deny Configuration.** You may need to install cargo-deny with`cargo install --locked cargo-deny`. See [cargo-deny-results](../../../audit-logs/cargo-deny-results.txt)
* [ ]  **Security Vulnerabilities.** You may need to install cargo-audit with`cargo install cargo-audit --locked` .
  Inspect [cargo-audit-results](../../../audit-logs/cargo-audit-results.txt) for any errors and warnings. The original developer drops the file into Gemini and asks advice about the results.
* [ ]  **Dependency Tree & Duplicates:**
* [ ]  Check bans. See [cargo-deny-bans](../../../audit-logs/cargo-deny-bans.txt)
* [ ]  Inspect duplicate crate versions. Inspect [duplicates](../../../audit-logs/duplicates.txt) for any errors and warnings. The original developer drops the file into Gemini and asks advice about the results.
* [ ]  **License & Source Validation:**
* [ ]  Check Licenses. See [cargo-deny-licenses](../../../audit-logs/cargo-deny-licenses.txt)
* [ ]  Ensure crates originate from allowed registries. See  [cargo-deny-sources](../../../audit-logs/cargo-deny-sources.txt)
* [ ]  **Frontend Security:**
* [ ]  Check frontend dependencies. See [npm-audit-results](../../../audit-logs/npm-audit-results.txt). Run `npm audit fix` if there are issues and run `npm audit --prefix frontend 2>&1 | Out-File ./audit-logs/npm-audit-results.txt` again. This does not always fix the vulnerabilities. The original developer asks Gemini for help resolving issues.
* [ ]  **Update Previews:**
* [ ]  Look for newer compatabile releases of dependencies. See [cargo-update-results](../../../audit-logs/cargo-update-results.txt)
* [ ]  Test the backend. See [cargo-test-results2](../../../audit-logs/cargo-test-results2.txt) Covered by verification script.
* [ ]  Compile the project and dependencies. See [cargo-check-results2](../../../audit-logs/cargo-check-results2) Covered by verification script.
* [ ]  Review available major/minor updates. You may need to install cargo-outdated with
  `cargo install cargo-outdated`
  See [outdated](../../../audit-logs/outdated.txt)
* [ ]  **Lockfile Commit:** Confirm updated `Cargo.lock` and `package-lock.json` are committed.

---

## 4. Quality Gates

**Preparation**

* [ ]  Commit any changes resulting from the above actions.

**Test Gate**

* [ ]  Check backend tests pass. See [cargo-test-results2](../../../audit-logs/cargo-test-results2.txt) Covered by verification script.
* [ ]  Check frontend tests pass. See [vitest-results](../../../audit-logs/vitest-results.txt) Covered by verification script.
* [ ]  Capture test evidence in the release evidence document.

**Lint / Format / Type-Check Gate**

* [ ]  Compile the project and dependencies. See [cargo-check-results2](../../../audit-logs/cargo-check-results2.txt). Covered by verification script.
* [ ]  Analyse code with clippy and check there are no critical warnings. Run`cargo clippy --fix --bin "embroidery-catalogue" -p Rust-Embroidery-Catalogue --tests --` to fix any errors. You may need to manually fix any remaining errors. See [cargo-clippy-results](../../../audit-logs/cargo-clippy-results.txt) Covered by verification script.
* [ ]  Check Rust formatting. Fix any errors with `cargo fmt`and confirm with`$env:CARGO_TERM_COLOR="never"; cargo fmt --check -- -v 2>&1 | Out-File ./audit-logs/rustfmt-results.txt` See [rust-fmt-results](../../../audit-logs/rustfmt-results.txt) Covered by verification script
* [ ]  Check Svelte types. There should be no errors. After fixing errors run `npx svelte-check --tsconfig frontend/jsconfig.json 2>&1 | Out-File ./audit-logs/svelte-check.txt` See [svelte-check](../../../audit-logs/svelte-check.txt) Covered by verification script.
* [ ]  Run Linting checks. Try to fix any errors with `cmd /c "cd frontend && npx eslint . --fix"`. Fix any remaining errors. Ru `Set-Location frontend; $env:FORCE_COLOR=0; npm run lint 2>&1 | Out-File ../audit-logs/eslint-results.txt; Set-Location` when errors are fixed. See [eslint-results](../../../audit-logs/eslint-results.txt)
* [ ]  Run 'npx prettier --check frontend/src. Fix any errors with `npx prettier --write frontend/src`. See [format-prettier-results](../../../audit-logs/format-prettier-results.txt) Covered by verification script.

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

* [ ]  **Manual License Asset Verification:** See [license-assets](../../../audit-logs/license-assets.txt)
* [ ]  Confirm generation of `src/assets/licences.html`.
* [ ]  Confirm generation of `src/assets/npm-licences.json`.
* [ ]  **Execute Release Build.** See [build-results](../../../audit-logs/build-results.txt). Covered by Verification script.
* [ ]  Verify Vite bundles license assets for rendering in @AboutView.svelte and @AboutDocumentView.svelte.

---

## 7. Artifact Verification & Testing

* [ ]  Locate MSI installer in `target/release/bundle/msi/`.
* [ ]  Locate NSIS installer in `target/release/bundle/nsis/`.
* [ ]  Verify installer filenames contain correct version string.
* [ ]  Compute SHA-256 checksums and verify artifacts. See [checksums](../../../audit-logs/checksums.txt)
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
