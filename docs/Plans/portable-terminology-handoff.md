# Portable Terminology Handoff Plan

## Goal

Standardize terminology for the portable copy workflow so destination can be any location (not only SD cards), while keeping Windows installer artifacts out of scope.

> Planning artifact only. No code changes are included here.

---

## Scope

### Included

1. Portable copy script and deployment-tool wording
2. Portable documentation wording
3. Tests directly tied to renamed portable script, executable, and labels
4. Backward-compatible handling for existing registry value names in deployment-tool logic

### Excluded

1. Windows installer behavior or text updates
2. Installer script changes in installer/EmbroideryCatalogue.iss
3. Commercial packaging and Add/Remove Programs wording
4. Broad app/executable rebranding beyond portable-copy surfaces

---

## Canonical terminology dictionary

1. Distribution channels
- Installed Edition: Windows installer path (customer/commercial path)
- Portable Edition: manual copy path from GitHub (developer/advanced-user path)

2. Destination terminology
- Preferred term: Deployment target location
- Alternate UI-friendly term: Destination location
- Explanatory phrase: Portable media (SD card, USB drive, external drive, or folder path)

3. Action terminology
- Preferred verb: Prepare
- Acceptable technical verb: Deploy
- Avoid in new wording: Populate

4. Tool terminology
- Preferred term: Portable deployment tool
- Alternate plain-English term: Portable copy tool
- Avoid in new wording: Launcher

5. Naming recommendation
- Rename populate_sdcard.bat -> prepare_portable_target.bat
- Rename EmbroiderySdLauncher.exe -> EmbroideryPortableDeploy.exe
- Rename build_launcher.bat -> build_portable_deployment.bat
- Rename user-facing title from SD Card Launcher -> Portable Deployment Tool

---

## Rename map

1. File and identifier map
- populate_sdcard.bat -> prepare_portable_target.bat
- BAT_NAME = "populate_sdcard.bat" -> BAT_NAME = "prepare_portable_target.bat"
- EmbroiderySdLauncher.exe -> EmbroideryPortableDeploy.exe
- EmbroiderySdLauncher.spec -> EmbroideryPortableDeploy.spec
- build_launcher.bat -> build_portable_deployment.bat
- LastSdRoot (registry value) -> LastDeploymentRoot (with fallback read of LastSdRoot)

2. UI string map
- SD Card Launcher -> Portable Deployment Tool
- Target SD card root -> Deployment target location
- Select SD card root directory -> Select deployment target location

3. Docs wording map
- Populating an SD Card -> Preparing a Portable Target
- SD card root -> Deployment target location
- SD card deployment -> Portable deployment

---

## Files to update (portable-only)

1. Runtime and deployment tool
- populate_sdcard.bat (rename to prepare_portable_target.bat)
- portable_launcher.py (or future rename to reflect deployment role)
- build_launcher.bat -> build_portable_deployment.bat
- EmbroiderySdLauncher.spec -> EmbroideryPortableDeploy.spec

2. Documentation
- README.md
- docs/USB_DEPLOYMENT.md
- docs/GETTING_STARTED.md (if SD card-specific language appears)
- docs/TROUBLESHOOTING.md (if SD card-specific language appears)

3. Tests
- tests/test_portable_launcher.py
- tests/test_root_scripts.py

---

## Implementation order

1. Rename portable batch file and update all direct code references
2. Rename the executable/spec/build-script references so SD and launcher terminology are removed from the packaged helper tooling
3. Add registry compatibility logic: read LastDeploymentRoot first, fallback to LastSdRoot
4. Update docs to new naming and location-neutral phrasing
5. Update deployment-tool UI labels and picker text to destination-neutral wording
6. Update tests for renamed file, executable, and revised labels
7. Run targeted deployment-tool/root-script tests and verify manual copy flow

---

## Verification checklist

- Deployment tool resolves and executes prepare_portable_target.bat
- Portable build script is named build_portable_deployment.bat
- No broken references to populate_sdcard.bat outside explicit migration note
- Existing users with LastSdRoot still get location prefill
- Deployment-tool labels clearly indicate any location is valid
- Packaged helper is named EmbroideryPortableDeploy.exe, not EmbroiderySdLauncher.exe
- Portable docs are consistent with new terminology

---

## Migration note text (for release notes/docs)

The portable script populate_sdcard.bat has been renamed to prepare_portable_target.bat, and the packaged helper EmbroiderySdLauncher.exe has been renamed to EmbroideryPortableDeploy.exe. Behavior is unchanged; terminology now reflects that the destination can be any location (for example SD card, USB drive, external drive, or folder path) and that the helper is a deployment tool rather than a launcher.
