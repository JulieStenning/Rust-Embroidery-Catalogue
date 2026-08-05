## Plan: Detailed Portable/USB Update Delivery SOP

This runbook covers update delivery for portable/USB deployments only. It is separate from the Windows installer SOP.

**Scope**
- Included: portable/USB updates prepared with deployment scripts or launcher tools.
- Excluded: Windows installer packaging and installer upgrade flow.

**Before First Update (One-Time Preparation)**
- Confirm update policy for portable channel using [update-policy-decision-record.md](../proformas/update-policy-decision-record.md).
- Confirm release type and migration scope baseline using [release-types-and-migration-scope.md](../policies/releases/release-types-and-migration-scope.md).
- Define portable artifact naming and checksum storage rules.
- Confirm source locations for runtime bundle, wheels, scripts, and app content.
- Prepare release evidence and rollback templates:
- [release-evidence-template.md](../proformas/releases/release-evidence-template.md)
- [rollback-incident-template.md](../proformas/releases/rollback-incident-template.md)
- Run one full rehearsal on both:
- Existing-data portable target.
- Clean target media.
- Record rehearsal timings and friction points.

**Steps**
- Item P1 - Assign release roles.
- Select Release Owner and Verifier.
- Confirm access to deployment tooling and release publishing location.
- Output: named owner/verifier.

- Item P2 - Confirm release baseline.
- Confirm clean source baseline and target version.
- Confirm release notes draft includes portable-specific user guidance.
- Output: locked baseline commit and scope.

- Item P3 - Prepare portable artifact.
- Build or generate portable payload using the approved process (script/launcher path used by the project).
- Confirm payload includes required runtime and app content.
- Output: candidate portable package location and build timestamp.

- Item P4 - Portable payload integrity checks.
- Verify core folders/files are present on target package:
- runtime (embedded Python), wheels, app source, templates/static assets, docs/start scripts.
- Generate SHA256 for distributed portable package (zip/folder artifact as applicable).
- Output: integrity checklist and checksum.

- Item P5 - Existing-data portable update test.
- Update an existing portable target that contains representative catalogue data.
- Preserve existing data folders and apply update procedure.
- Launch and verify startup, migration completion, and data visibility.
- Output: existing-data scenario result.

- Item P6 - Clean-target portable test.
- Deploy to clean removable media.
- Run first-launch flow and verify initialization succeeds.
- Confirm no missing dependency/resource errors.
- Output: clean-target scenario result.

- Item P7 - Path and portability checks.
- Verify app runs after drive-letter changes (or equivalent path changes) where expected.
- Verify design paths and database paths resolve correctly after update.
- Output: portability/path validation result.

- Item P8 - Publish portable release package.
- Publish package artifact(s), checksum, and release notes with portable update instructions.
- Include explicit backup guidance before updating portable media.
- Output: published release URL and contents checklist.

- Item P9 - Early monitoring and triage (24-48h).
- Monitor startup errors, migration issues, and media/path-related failures.
- Record triage decisions and escalation status.
- Output: monitoring summary.

- Item P10 - Rollback readiness.
- Keep known-good portable package available.
- Validate restore guidance and support response template.
- Output: rollback readiness confirmation.

**Portable Gates (Pass/Fail)**
- Gate P-A: portable payload completeness.
- Gate P-B: checksum published and verified.
- Gate P-C: existing-data update scenario passes.
- Gate P-D: clean-target scenario passes.
- Gate P-E: path/portability checks pass.
- Gate P-F: publish completeness passes.
- Gate P-G: rollback readiness confirmed.

If any gate fails, pause release for remediation and re-validation.

**Related documents**
- [windows-installer-update-delivery-sop.md](windows-installer-update-delivery-sop.md)
- [portable-usb-update-quick-checklist.md](portable-usb-update-quick-checklist.md)
- [release-evidence-template.md](../proformas/releases/release-evidence-template.md)
- [rollback-incident-template.md](../proformas/releases/rollback-incident-template.md)
- [update-policy-decision-record.md](../proformas/update-policy-decision-record.md)
