# Release Checklist

Use this checklist for every release. Complete all items before tagging.
Record evidence in the [release evidence template](./release-evidence-template.md).

---

## Pre-release quality gates

### Version sync
- [ ] `Cargo.toml` `[package] version` updated to target version.
- [ ] `tauri.conf.json` and `src-tauri/tauri.conf.json` `version` match `Cargo.toml`.
- [ ] `frontend/package.json` `version` matches target version.
- [ ] All version files committed on release branch.

### Test gate
- [ ] Backend tests: `cargo test` run — all unit and integration test suites pass.
- [ ] Frontend tests: `npm test` (or `npx vitest run`) from repo root — all Vitest suites pass.
- [ ] Any new failures investigated and either fixed or exception-documented.
- [ ] Evidence (test output log or CI run URL) captured in release evidence doc.

### Lint / format / type-check gate
- [ ] `cargo check` — compiles with zero errors.
- [ ] `cargo clippy --all-targets` — no errors or critical warnings introduced by this release.
- [ ] `cargo fmt --check` (or `rustfmt --edition 2021`) — Rust code formatted cleanly.
- [ ] `cmd /c "cd frontend && npx svelte-check --tsconfig jsconfig.json"` — zero TypeScript/Svelte errors.
- [ ] `cmd /c "cd frontend && npm run lint"` — ESLint checks pass.
- [ ] `cmd /c "cd frontend && npm run format:check"` — Prettier format checks pass.

### Migration gate
- [ ] Release type classified per [release-types-and-migration-scope](../../policies/releases/release-types-and-migration-scope.md).
- [ ] All migration files in `migrations/` verified and valid.
- [ ] Clean database startup smoke test passed (`sqlx` migrations run and `_sqlx_migrations` initialized).
- [ ] Existing database upgrade smoke test passed (existing DB updated cleanly without data loss).
- [ ] Custom data-root persistence verified (`config.json` bootstrap location intact).
- [ ] For destructive migrations: backup-first guidance confirmed; rollback/restore path confirmed.
- [ ] Latest migration revision/timestamp recorded in release evidence doc.

### CI gate
- [ ] CI workflow passes on release branch (all jobs green).
- [ ] CI run URL captured in release evidence doc.

### Dependency / lock-file gate
- [ ] `Cargo.lock` reflects intended dependencies and is committed.
- [ ] `package-lock.json` reflects intended frontend dependencies and is committed.
- [ ] Rust dependency audit: `cargo audit` (no unhandled high or critical vulnerabilities).
- [ ] Frontend dependency audit: `npm audit` (no unhandled high or critical vulnerabilities).

---

## Evidence capture checklist

- [ ] Release evidence document created from [release-evidence-template.md](./release-evidence-template.md).
- [ ] All gate evidence fields completed.
- [ ] Evidence document committed or linked in release PR.

---

## Rollback readiness checks

- [ ] Rollback strategy confirmed (backup restore + known-good installer).
- [ ] Backup-before-update guidance included in release notes.
- [ ] Hotfix/rollback trigger criteria documented (e.g. critical startup issue or data loss triggers rollback).

---

## Artifact checks

- [ ] Release build created via `build-rust-release.bat` or `cargo tauri build`.
- [ ] Installer artifacts built:
  - MSI installer: `target/release/bundle/msi/`
  - NSIS installer: `target/release/bundle/nsis/`
- [ ] Artifact filenames match version (e.g., `Embroidery Catalogue_<version>_x64_en-US.msi`, `*-setup.exe`).
- [ ] SHA-256 checksum computed and noted in release evidence.
- [ ] Artifacts verified against checksums.
- [ ] Test install / upgrade executed on a clean or test Windows environment.
- [ ] Code signing completed if required (see [CODE_SIGNING.md](../../CODE_SIGNING.md)).

---

## Release notes completion checks

- [ ] `CHANGELOG.md` `[Unreleased]` section promoted to versioned entry.
- [ ] Release notes include:
  - [ ] Summary of changes.
  - [ ] Any migration notes (if schema changed).
  - [ ] Backup-before-update reminder (for minor and major releases).
  - [ ] Known issues (if any).
- [ ] GitHub Release draft created with release notes attached.
- [ ] Artifacts uploaded to GitHub Release.
- [ ] Checksum published in release body or as attached file.

---

## Post-publish checks (24–48 h monitoring)

- [ ] Monitoring owner assigned.
- [ ] No critical issues reported within monitoring window.
- [ ] Monitoring sign-off recorded.

---

## Final sign-off

- [ ] Release owner sign-off (name / date).
- [ ] Verifier sign-off (name / date).
- [ ] Final decision: **GO / NO-GO**.

---

## Dependency lock-file maintenance reminder

After a release, if dependencies were updated, ensure both backend and frontend lock files are refreshed and committed:

```bash
# Update Cargo dependencies if planned
cargo update

# Update frontend dependencies if planned
npm install
```

Commit `Cargo.lock` and `package-lock.json` on the next development branch.
