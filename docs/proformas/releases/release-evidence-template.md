## Release Evidence Template

Use this template to capture objective evidence for a specific release.

**Release Identity**
- Release channel (MSI / NSIS Installer):
- Release version:
- Release type (hotfix / minor / major):
- Target commit/tag:
- Date:
- Release owner:
- Verifier:

**Version Sync Evidence**
- Cargo.toml version:
- tauri.conf.json & src-tauri/tauri.conf.json version:
- frontend/package.json version:
- Match confirmed by:
- Evidence link/snippet:

**Test Gate Evidence**
- Backend tests (`cargo test`) passed: (Yes/No)
- Frontend tests (`npm test` / Vitest) passed: (Yes/No)
- Failures/exceptions (if any):
- Exception sign-off (if used):
- Evidence link/snippet:

**Lint & Type-Check Gate Evidence**
- `cargo check` passed: (Yes/No)
- `cargo clippy` passed: (Yes/No)
- `cargo fmt --check` / `rustfmt` passed: (Yes/No)
- `svelte-check` passed: (Yes/No)
- Frontend `eslint` & `prettier` checks passed: (Yes/No)
- Evidence link/snippet:

**Migration Gate Evidence**
- Latest migration revision / timestamp:
- Prior release revision used for validation:
- Clean database migration result:
- Existing database upgrade result:
- Custom data-root persistence result:
- Evidence link/snippet:

**Artifact Evidence**
- Artifact filename(s) (MSI / NSIS):
- Artifact output path (`target/release/bundle/`):
- Build timestamp:
- Evidence link/snippet:

**Checksum Evidence**
- Algorithm: SHA256
- Checksum value:
- File verified against checksum:
- Verified by:
- Evidence link/snippet:

**Signing Evidence**
- Signing required for this release: (Yes/No)
- Signature verification result:
- Waiver reference (if signing not required):
- Evidence link/snippet:

**Upgrade Validation Evidence**
- Existing-data upgrade scenario result:
- Clean-machine scenario result:
- Custom data-root persistence result:
- Evidence link/snippet:

**Publish Evidence**
- Release URL:
- Artifact(s) attached: (Yes/No)
- Checksum published: (Yes/No)
- Release notes published: (Yes/No)
- Backup-before-update guidance included: (Yes/No)
- Evidence link/snippet:

**Monitoring Evidence (24-48h)**
- Monitoring owner:
- Critical issue trend observed: (Yes/No)
- Triage summary:
- Evidence link/snippet:

**Final Sign-Off**
- Release owner sign-off (name/date):
- Verifier sign-off (name/date):
- Final decision: GO / NO-GO
