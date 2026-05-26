# Portable Deployment Refactor Checklist

Use this checklist when changing portable deployment tooling, script behavior, or portable runtime startup semantics.

Companion spec: [docs/Specs/portable-deployment-backend-spec.md](docs/Specs/portable-deployment-backend-spec.md)

## 1. Contract Safety
- [ ] Deployment tool command and invocation path remain compatible, or migration impact is documented.
- [ ] Batch command arguments remain backward compatible (`target_root`, optional designs source, `--no-designs`).
- [ ] User-facing deployment labels remain location-neutral and consistent.
- [ ] Portable runtime default port/mode behavior remains compatible (`APP_ENV=portable`, default `APP_PORT=8002`).

## 2. Copy Semantics and Data Safety
- [ ] Deployment still validates source and target prerequisites before copy.
- [ ] Write probe behavior remains in place for destination-root permission checks.
- [ ] `.env` handling remains safe: `.env.example` to target `.env`, never copy live secrets directly.
- [ ] `data/tags.csv` delivery behavior remains intact.
- [ ] Managed design copy remains explicitly controllable (`--no-designs`).

## 3. Runtime Bootstrap Integrity
- [ ] `start.bat` still triggers first-run setup when portable venv is missing.
- [ ] `setup.bat` still bootstraps pip and installs wheels offline.
- [ ] Startup error logging to `logs/startup-error.log` remains intact.
- [ ] `stop.bat` still stops portable listeners/processes safely.

## 4. Path Portability and Persistence
- [ ] Deployment root normalization and validation still accept drive-root and UNC targets.
- [ ] Last-used target persistence still reads canonical key and preserves legacy fallback compatibility.
- [ ] Portable mode data and design paths remain self-contained under app `data/`.

## 5. Logging and Operability
- [ ] Deployment output remains visible to operators (streamed output panel / script output).
- [ ] Failure paths retain actionable messages and non-success exit codes.
- [ ] Launcher failure logging path remains available for diagnosis.

## 6. Packaging and Artifact Integrity
- [ ] Portable helper build still produces `EmbroideryPortableDeploy` artifact via current spec.
- [ ] `prepare_portable_target.bat` is still copied beside built helper artifact for operator use.
- [ ] Packaged icon/resource references remain valid.

## 7. Test Coverage Gate
- [ ] Updated/added tests in:
  - [tests/test_portable_launcher.py](tests/test_portable_launcher.py)
  - [tests/test_portable_scripts.py](tests/test_portable_scripts.py)
  - [tests/test_root_scripts.py](tests/test_root_scripts.py)
- [ ] Any changed behavior has at least one launcher-flow assertion and one deployment-script assertion.
- [ ] Backward compatibility behavior (legacy root fallback) remains covered when touched.

## 8. Documentation Gate
- [ ] [docs/Specs/portable-deployment-backend-spec.md](docs/Specs/portable-deployment-backend-spec.md) is updated for externally visible behavior changes.
- [ ] [docs/USB_DEPLOYMENT.md](docs/USB_DEPLOYMENT.md) wording matches current operator behavior and artifact names.
- [ ] macOS deployment procedures remain documented separately and are not mixed into Windows portable procedures.
- [ ] Any stale links to retired portable plan docs are removed or redirected.
