# Release Checklist

Use this checklist for every release.  Complete all items before tagging.
Record evidence in the [release evidence template](./release-evidence-template.md).

---

## Pre-release quality gates

### Version sync
- [ ] `pyproject.toml` version updated to target version.
- [ ] `installer/EmbroideryCatalogue.iss` version matches (installer channel only).
- [ ] Both files committed on release branch.

### Test gate
- [ ] `pytest -q` run — all required suites pass.
- [ ] Any new failures investigated and either fixed or exception-documented.
- [ ] Coverage baseline not regressed below previous release baseline.
- [ ] Evidence (test output log or CI run URL) captured in release evidence doc.

### Lint / format gate
- [ ] `ruff check src/` — no new errors introduced by this release.
- [ ] `black --check src/` — no unformatted files introduced by this release.

### Migration gate
- [ ] Release type classified per [release-types-and-migration-scope](../../policies/releases/release-types-and-migration-scope.md).
- [ ] `alembic upgrade head` smoke test passed on clean database.
- [ ] `alembic upgrade head` smoke test passed on stamped existing database.
- [ ] Unstamped existing database scenario tested.
- [ ] For destructive migrations: backup-first guidance confirmed; rollback path confirmed.
- [ ] Alembic head revision recorded in release evidence doc.

### CI gate
- [ ] CI workflow passes on release branch (all jobs green).
- [ ] CI run URL captured in release evidence doc.

### Dependency / lock-file gate
- [ ] `requirements-ci.txt` reflects current pinned versions.
- [ ] No known high or critical vulnerabilities in dependencies (`pip audit`).

---

## Evidence capture checklist

- [ ] Release evidence document created from [release-evidence-template.md](./release-evidence-template.md).
- [ ] All gate evidence fields completed.
- [ ] Evidence document committed or linked in release PR.

---

## Rollback readiness checks

- [ ] Rollback strategy confirmed (backup restore + known-good installer, or alembic downgrade if supported).
- [ ] Backup-before-update guidance included in release notes.
- [ ] Hotfix/rollback trigger criteria documented (e.g. critical issue within 48 h triggers rollback).

---

## Artifact checks

- [ ] Installer artifact built and filename matches version (`EmbroideryCatalogue-vX.Y.Z.exe` or similar).
- [ ] Portable artifact built if required.
- [ ] SHA-256 checksum computed and noted in release evidence.
- [ ] Artifacts verified against checksums.
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

## Lock-file refresh reminder

After a release, if dependencies were updated, refresh the CI lock file:

```bash
pip install -e ".[dev]"
pip freeze > requirements-ci.txt
```

Commit the updated file on the next development branch.
