# Removable Media Rename Plan

## Goal

Rename `populate_sdcard.bat` to `populate_removablemedia.bat` and standardise the user-facing terminology on **removable media** instead of mixing **SD card** and **USB drive**.

> This file records the recommended changes only. No code changes are included here.

---

## Recommended minimal change set

1. Rename `populate_sdcard.bat` to `populate_removablemedia.bat`.
2. Update code that directly references the batch filename.
3. Update documentation to use **removable media** as the generic term.
4. Keep existing launcher artifact names such as `portable_launcher.py` and `EmbroiderySdLauncher.exe` for now, unless a broader rename is wanted later.

---

## Required code changes

These files should be updated so the rename works correctly:

| File | Required change |
|---|---|
| `populate_sdcard.bat` | Rename file to `populate_removablemedia.bat` |
| `portable_launcher.py` | Change `BAT_NAME = "populate_sdcard.bat"`; update user-facing text that says "SD card" where appropriate |
| `tests/test_portable_launcher.py` | Update expected batch-file paths and related test strings |
| `build_launcher.bat` | Change `set BAT_NAME=populate_sdcard.bat` to the new filename |

---

## Documentation updates

These documents contain wording that should be updated to use **removable media** and the new batch filename:

- `README.md`
- `docs/USB_DEPLOYMENT.md`
- `docs/BACKUP_RESTORE.md`
- `docs/GETTING_STARTED.md`
- `docs/TROUBLESHOOTING.md`
- `docs/AI_TAGGING.md`
- `docs/feature-inventory.md`
- `docs/Plans/public-release-issues.md`
- `PRIVACY.md`
- `SECURITY.md`

There are also a few related wording/comment references in:

- `setup.bat`
- `start.bat`
- `src/config.py`

---

## Terminology recommendation

Use **removable media** as the standard generic term.

Recommended first-reference phrasing:

> Copy the portable app to **removable media**, such as a USB drive or SD card.

This keeps the docs consistent while still giving concrete examples.

---

## Optional follow-up changes

These are optional and can be handled separately if desired:

- Rename launcher UI text from **SD Card Launcher** to **Removable Media Launcher**
- Rename labels such as **Target SD card root** to **Target removable media root**
- Consider whether to rename:
  - `EmbroiderySdLauncher.exe`
  - `EmbroiderySdLauncher.spec`

These are not strictly required for the batch-file rename itself.

---

## Suggested implementation order

1. Rename the batch file.
2. Update `portable_launcher.py`, `tests/test_portable_launcher.py`, and `build_launcher.bat`.
3. Update the user-facing documentation.
4. Run the relevant tests and manually verify the launcher workflow on Windows.

---

## Verification checklist

After making the rename in the future branch, verify:

- [ ] The launcher still finds and runs the renamed batch file
- [ ] Tests in `tests/test_portable_launcher.py` pass
- [ ] No docs still refer to `populate_sdcard.bat` unless intentionally noted for compatibility
- [ ] User-facing docs consistently use **removable media**
