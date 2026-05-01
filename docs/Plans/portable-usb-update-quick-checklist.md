## Portable/USB Update Quick Checklist

Use this checklist for portable/USB updates only.

**Before First Update (One-Time Setup)**
- [ ] Release policy defined for portable channel
- [ ] Portable artifact naming and checksum location defined
- [ ] Release evidence template prepared
- [ ] Rollback incident template prepared
- [ ] One full rehearsal completed (existing-data target + clean target)
- [ ] First-update readiness sign-off completed

**Release Metadata**
- [ ] Release owner assigned
- [ ] Verifier assigned
- [ ] Target version confirmed
- [ ] Target commit/tag confirmed

**A. Portable Payload Gate**
- [ ] Portable package generated using approved tooling
- [ ] Required runtime/app files included
- [ ] Evidence captured (package location + contents check)
- [ ] GO / NO-GO: GO only if all checks pass

**B. Integrity Gate**
- [ ] SHA256 checksum generated and recorded
- [ ] Checksum matches published artifact
- [ ] GO / NO-GO: GO only if all checks pass

**C. Existing-Data Update Gate**
- [ ] Existing-data portable target update passed
- [ ] Startup/migration/data visibility verified
- [ ] GO / NO-GO: GO only if all checks pass

**D. Clean-Target Gate**
- [ ] Clean media deployment passed
- [ ] First launch/initialization succeeded
- [ ] GO / NO-GO: GO only if all checks pass

**E. Portability/Path Gate**
- [ ] Drive/path portability checks passed
- [ ] Design/database paths resolve after update
- [ ] GO / NO-GO: GO only if all checks pass

**F. Publish Completeness Gate**
- [ ] Portable artifact(s) published
- [ ] SHA256 published
- [ ] Release notes include backup-before-update guidance
- [ ] GO / NO-GO: GO only if all checks pass

**G. Rollback Readiness**
- [ ] Known-good portable package available
- [ ] Restore instructions verified
- [ ] Support response template ready
- [ ] GO / NO-GO: GO only if all checks pass

**Sign-Off**
- [ ] Release owner sign-off
- [ ] Verifier sign-off
