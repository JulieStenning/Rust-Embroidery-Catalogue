# Portable macOS Delivery Plan (Portable-First)

Date: 2026-04-14  
Repository: Embroidery-Catalogue  
Target branch: master

## Objective

Deliver a portable macOS version first, with support for both Apple Silicon and Intel Macs, while preserving existing Windows portable behavior.

## Scope Decisions (Locked)

- Architecture target: Universal support (arm64 + x86_64).
- Signing target for v1: Unsigned internal build (with Gatekeeper bypass instructions).
- Included in v1: Portable workflow and launcher support on macOS.
- Excluded in v1: Notarization, signed distribution, DMG/PKG installer, full desktop app bundle rollout.

## Success Criteria

- A mac user can deploy the portable app to a mounted volume and run it end-to-end.
- First-run bootstrap works offline from bundled wheels/runtime assets.
- App starts, serves UI, persists data, and restarts correctly.
- Existing Windows portable behavior remains unchanged.
- CI verifies key portable behavior on macOS and Windows.

## Current Constraints to Address

1. Windows batch scripts drive portable setup/start/stop/deploy and do not run on macOS.
2. Deployment flow relies on Windows-specific tools and path formats.
3. Launcher persistence and removable-media detection are Windows-biased.
4. Runtime/storage path logic contains Windows assumptions in some branches.
5. CI currently lacks macOS validation for the portable pipeline.
6. The final native macOS launcher or app bundle cannot be built reliably on a Windows-only machine; it should be built on a macOS GitHub Actions runner or a physical Mac.

> Practical implication: the Windows development machine can prepare the portable payload, scripts, and most repository changes, but the final native mac launcher artifact should be produced and verified on macOS.

## Phased Execution Plan

## Phase 1: Scope Contract and Delivery Baseline

1. Document the v1 contract (portable-first, universal target, unsigned internal release).
2. Define behavior parity requirements with current Windows portable mode:
   - self-contained portable root
   - first-run setup semantics
   - startup/stop behavior and logs
   - offline dependency install path
3. Record non-goals to prevent scope creep (installer/notarization/desktop bundle work deferred).

Deliverables:
- This plan approved as implementation baseline.
- Acceptance checklist template for release gate.

## Phase 2: Cross-Platform Runtime Entrypoints

1. Replace Windows-only runtime behavior from start/setup/stop batch scripts with cross-platform Python entrypoints under scripts.
2. Keep Windows behavior identical (ports, logs, failure messaging) to reduce regression risk.
3. Keep existing .bat files as Windows compatibility wrappers that call new Python entrypoints.

Dependencies:
- Blocks Phase 3 and Phase 6.

Deliverables:
- New portable runtime entrypoints callable on macOS and Windows.
- Windows wrapper compatibility retained.

## Phase 3: Deployment Pipeline Refactor

1. Move deployment logic from batch into a cross-platform deploy command.
2. Implement platform-aware copy backends:
   - Windows: robocopy
   - macOS: rsync
3. Add mac target validation for mounted volumes and standard absolute paths.
4. Preserve current portable layout and offline asset copy rules.

Dependencies:
- Depends on Phase 2.

Deliverables:
- One deploy flow that supports Windows and macOS targets.
- Equivalent copy semantics and failure reporting across platforms.

## Phase 4: Portable Launcher macOS Enablement

1. Replace registry-only launcher persistence with platform-agnostic storage.
2. Extend root-path validation to support mac-style mount paths while preserving Windows rules.
3. Implement mac-capable removable-media detection with fallback to manual folder selection.
4. Keep tkinter-first approach for v1 to avoid introducing GUI framework risk.

Dependencies:
- Can run in parallel with late Phase 3 tasks once Phase 2 starts.

Deliverables:
- Launcher can remember last deployment root on macOS.
- Launcher can target mounted mac volumes reliably.

## Phase 5: Runtime Path and Storage Hardening

1. Update runtime path resolution in config for platform-safe behavior on macOS.
2. Preserve current desktop state/data separation behavior used by Windows installs.
3. Confirm APP_MODE behavior remains stable across development, portable, and desktop paths.

Dependencies:
- Parallel with Phase 4, must complete before Phase 6 final verification.

Deliverables:
- Platform-safe path handling without breaking existing Windows data behavior.

## Phase 6: Universal Build Output for Portable Launcher

1. Extend build orchestration to produce mac portable launcher artifacts on a macOS runner or a physical Mac, not on the Windows development machine.
2. Validate universal packaging strategy:
   - Preferred: single universal artifact.
   - Fallback: dual-arch artifacts with deterministic packaging and naming.
3. Validate artifact execution on both Apple Silicon and Intel.

Dependencies:
- Depends on Phases 2-5.

Deliverables:
- Reproducible build output for portable mac launcher.
- Architecture validation evidence for arm64 and x86_64.

## Phase 7: Documentation and Operator Runbook

1. Add mac portable deployment runbook adjacent to existing USB deployment docs.
2. Update platform support wording in README to reflect portable mac v1 status.
3. Add clear Gatekeeper bypass steps for unsigned internal distribution.
4. Include troubleshooting for mounted-volume permissions and first-run bootstrap errors.

Dependencies:
- Parallel with Phase 6 validation.

Deliverables:
- Operator-ready docs that do not require tribal knowledge.

## Phase 8: Tests and CI Expansion

1. Extend portable launcher tests with mac path/persistence/removable-media cases.
2. Add tests for cross-platform runtime and deploy entrypoints.
3. Add macOS jobs in CI for portable validation.
4. Establish smoke matrix:
   - Windows portable unchanged
   - macOS arm64 portable works
   - macOS x86_64 portable works

Dependencies:
- Depends on Phases 4-6.

Deliverables:
- Automated validation for core portable behavior on macOS and Windows.

## Phase 9: Release Gate and Handoff Acceptance

1. Execute release checklist:
   - build reproducibility
   - deploy success
   - first-run offline bootstrap
   - start/stop reliability
   - restart and data persistence
2. Maintain rollback option via Windows wrappers until two stable release cycles.
3. Package implementation handoff bundle for web agent:
   - prioritized task list
   - acceptance criteria
   - risk log
   - deferred backlog

Dependencies:
- Depends on Phases 6-8.

Deliverables:
- Handoff-ready execution packet and go/no-go decision.

## Detailed Task Backlog for Web Agent

Priority P0 (must complete first):
1. Build cross-platform Python entrypoints for setup/start/stop.
2. Refactor deploy logic into a platform-aware implementation.
3. Update launcher persistence and path validation for mac volumes.
4. Add mac path/storage handling in config where Windows assumptions exist.

Priority P1:
1. Produce universal (or dual-arch fallback) launcher artifacts.
2. Add mac CI jobs and portable smoke tests.
3. Write mac operator docs and Gatekeeper guidance.

Priority P2:
1. Polish removable media auto-detection UX.
2. Improve diagnostics and error reporting consistency.
3. Prepare future signing/notarization plan for v2.

## Risk Register and Mitigations

1. Risk: Windows regressions while replacing batch logic.
   - Mitigation: Keep wrapper shims; run Windows smoke/regression tests every phase.
2. Risk: Universal artifact instability due to dependency/toolchain differences.
   - Mitigation: Maintain dual-arch fallback path in build pipeline.
3. Risk: Removable-media auto-detection inconsistency on mac systems.
   - Mitigation: Keep manual folder picker fallback as primary-safe path.
4. Risk: Unsigned app friction from Gatekeeper.
   - Mitigation: Provide clear operator bypass instructions and expected prompts.
5. Risk: Assuming Windows can produce the final native mac launcher.
   - Mitigation: Treat GitHub Actions on macOS, or a real Mac, as the release build host for the final artifact.

## Verification Plan

1. Unit tests:
   - launcher path validation and persistence
   - runtime command and environment selection
   - deploy copy include/exclude behavior
2. Integration tests:
   - deploy to mounted volume
   - first-run setup from bundled wheels
   - start, browse, stop, restart
3. Manual architecture checks:
   - Apple Silicon machine
   - Intel machine
4. Regression checks:
   - Windows portable setup/start/stop/deploy behavior unchanged
5. CI checks:
   - Linux existing suite still green
   - Windows suite green
   - mac suite green for portable flow

## Do We Need Both Parts?

Yes. To deliver a usable mac portable release, both parts are required:

1. Make the portable application itself work correctly on macOS.
   - This is the feature and compatibility work.
   - It includes the scripts, path handling, launcher logic, and portable runtime behavior.

2. Build the native mac launcher or app artifact on macOS.
   - This is the packaging and distribution work.
   - It produces the file that can be downloaded, tested, and handed to users.

If only the first part is done, the app may be compatible but there is no proper mac deliverable.
If only the second part is done, you may have a mac artifact that still fails at runtime.

Recommended order:

1. First, complete the cross-platform portable functionality.
2. Then, run the macOS build job to generate the release artifact.

## Building on a Windows Development Machine

You can do most of the work from Windows:

- implement and test the cross-platform Python logic
- prepare the portable folder structure
- maintain the deployment scripts and docs
- stage the repository for release

You should not assume the Windows machine can produce the final native mac launcher artifact.
For that step, the recommended release path is:

1. Develop and commit from Windows.
2. Push the branch to GitHub.
3. Run the macOS build job in GitHub Actions.
4. Download the generated mac artifact and test it on a real Mac.

## How to Create a macOS GitHub Actions Build Job

Recommended approach: keep test validation and release artifact creation as separate jobs. Use a macOS runner for the build so PyInstaller and the platform toolchain run in the correct environment.

### Step-by-step

1. Open the workflow file in .github/workflows.
2. Add a new job after the existing test jobs, for example named build-macos-portable.
3. Use runs-on: macos-latest.
4. Install Python 3.12 and project dependencies.
5. Make the mac build script executable.
6. Run the mac build script for either universal or per-architecture output.
7. Upload the dist output as a workflow artifact.
8. Optionally trigger this only on tags, release branches, or manual workflow dispatch.

### Example workflow job

```yaml
  build-macos-portable:
    name: Build macOS portable artifact
    runs-on: macos-latest
    needs: test-macos

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Set up Python 3.12
        uses: actions/setup-python@v5
        with:
          python-version: "3.12"

      - name: Install dependencies
        run: |
          python -m pip install --upgrade pip
          pip install -r requirements-ci.txt
          pip install -e ".[dev]" --no-deps
          pip install pyinstaller

      - name: Ensure build script is executable
        run: chmod +x ./build_portable_deployment_mac.sh

      - name: Build universal portable launcher
        run: ./build_portable_deployment_mac.sh --arch universal2

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: embroidery-portable-macos
          path: |
            dist/EmbroideryPortableDeploy
            dist/EmbroideryPortableDeploy.app
```

### Recommended enhancements

- Add workflow_dispatch so you can start a build manually from GitHub.
- Add a matrix build if universal output is unstable, using separate arm64 and x86_64 jobs.
- Add release upload on version tags once the artifact is stable.
- Keep signing and notarization out of v1; add them only after the unsigned internal build is proven.

### Operational note for the web agent

If the repository contains the mac build script and spec file, the web agent should prefer a macOS GitHub Actions build to answer the question, “can this ship to a Mac?” That gives a reproducible artifact and removes the dependency on access to a physical Mac during normal Windows-based development.

## Simple Checklist After Downloading the Build Artifact

Use this checklist on the Mac:

1. Unzip the downloaded artifact.
2. Find `EmbroideryPortableDeploy` or `EmbroideryPortableDeploy.app`.
3. Remove the macOS quarantine block once:

```bash
xattr -d com.apple.quarantine EmbroideryPortableDeploy
```

or:

```bash
xattr -rd com.apple.quarantine EmbroideryPortableDeploy.app
```

4. Open the launcher.
5. Choose where you want the portable copy to be created.
6. Let it deploy the `EmbroideryApp` folder.
7. If needed, open Terminal in the deployed `app` folder and run:

```bash
python3 scripts/portable_start.py
```

8. Check that the app opens in the browser and works normally.
9. Stop it, start it again, and confirm it still works.

### Copy-and-paste Terminal commands

If the app does not open by double-clicking, use Terminal:

```bash
cd /path/to/unzipped/artifact
xattr -rd com.apple.quarantine EmbroideryPortableDeploy.app
open EmbroideryPortableDeploy.app
```

Or, after deployment:

```bash
cd /path/to/EmbroideryApp/app
python3 scripts/portable_start.py
```

To stop it:

```bash
python3 scripts/portable_stop.py
```

### Quick success check

The test is successful if:

- the launcher opens
- the portable files copy successfully
- the app opens in the browser
- the data folder is created
- restart works without errors

## Handoff Packet Contents (for Web Agent)

1. This plan file.
2. Current portable behavior map (source-of-truth references).
3. Ordered implementation checklist with dependencies.
4. Acceptance criteria and test matrix.
5. Known deferred scope (notarization/installer).

## Source Files to Prioritize

- portable_launcher.py
- prepare_portable_target.bat
- start.bat
- setup.bat
- stop.bat
- src/config.py
- build_portable_deployment.bat
- EmbroideryPortableDeploy.spec
- tests/test_portable_launcher.py
- .github/workflows/ci.yml
- docs/USB_DEPLOYMENT.md
- README.md

## Timeline Estimate

- Phase 1-2: 1 to 2 weeks
- Phase 3-5: 2 to 4 weeks
- Phase 6-8: 2 to 3 weeks
- Phase 9: 0.5 to 1 week

Total: approximately 6 to 10 weeks for portable mac v1, depending on universal build stability and test environment availability.
