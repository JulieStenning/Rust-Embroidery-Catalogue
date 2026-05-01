## Release Evidence Template

Use this template to capture objective evidence for a specific release.

**Release Identity**
- Release channel (Installer/Portable):
- Release version:
- Release type (hotfix/minor/major):
- Target commit/tag:
- Date:
- Release owner:
- Verifier:

**Version Sync Evidence**
- pyproject.toml version:
- installer/EmbroideryCatalogue.iss version (installer channel only):
- Match confirmed by:
- Evidence link/snippet:

**Test Gate Evidence**
- Test commands executed:
- Required suites passed:
- Failures/exceptions (if any):
- Exception sign-off (if used):
- Evidence link/snippet:

**Migration Gate Evidence**
- Alembic head revision:
- Prior release revision used for validation:
- Stamped DB scenario result:
- Unstamped DB scenario result:
- Evidence link/snippet:

**Artifact Evidence**
- Artifact filename(s):
- Artifact output path:
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
