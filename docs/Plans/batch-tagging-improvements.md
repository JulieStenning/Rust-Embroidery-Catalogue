# Batch Tagging Improvements Plan

_Date:_ 2026-04-07

## Purpose

Give the user more control over AI-assisted tagging during import, especially for large collections, while keeping the current fast keyword tagging in place.

This plan is intended as a **handoff document for a web agent** to implement the UI and wiring changes.

---

## Current confirmed behaviour

The existing code already supports three tagging tiers:

| Tier | Current behaviour | API key required? | `tagging_tier` |
|---|---|---:|---:|
| 1 | Keyword matching from the **filename and folder path** | No | `1` |
| 2 | Gemini **text** tagging from the **filename stem** | Yes | `2` |
| 3 | Gemini **vision** tagging from the stored **preview image** | Yes | `3` |

### Notes

- Tier 1 runs automatically during import.
- Tier 2 and Tier 3 currently run automatically during import **if** `GOOGLE_API_KEY` is present.
- Tier 2 is **not image analysis**. It sends the cleaned filename text to Gemini.
- The repo already has batch-oriented scripts:
  - `auto_tag.py`
- However, **the app UI does not currently expose batch controls**.

---

## Agreed decisions

- **Tier 1 remains automatic**.
- The user should have clear control over Gemini usage from the app.
- **Tier 3 should run if it has been ticked in Settings**, provided an API key is available.
- The user should **not** have to re-confirm this on every import once they have chosen their settings.
- If no API key is present, the app should warn the user and point them to the settings page.
- If an API key is present, the app should warn that Gemini usage may incur cost.
- Users should be able to run AI tagging in **batches** when importing large sets of files.

---

## Cost and quota warning text

Use the guidance already documented in `docs/AI_TAGGING.md`:

- Free tier: **15 requests per minute**
- Free tier: **1,500 requests per day**
- Historical estimate from the original February 2026 tagging work:
  - **Tier 3 on 4,000 images was about $0.33 on the paid tier**
- Pricing may have changed, so the UI should also direct the user to:
  - <https://ai.google.dev/pricing>

This warning should be shown in a friendly but clear way wherever the user enables AI tagging.

---

## Desired user experience

### 1. Settings page

Extend the existing settings page so it becomes the main control point for AI tagging.

#### Keep
- existing Google Gemini API key field

#### Add
- `Run Tier 2 automatically during import` checkbox
- `Run Tier 3 automatically during import` checkbox
- optional batch settings such as:
  - default batch size
  - optional delay between calls

### Required behaviour

- If `Run Tier 3 automatically during import` is checked **and** a valid API key exists, Tier 3 should run automatically for eligible imported designs.
- If only Tier 2 is checked, only Tier 2 should run automatically.
- If neither is checked, the import should remain **Tier 1 only**.

---

### 2. Import wizard

The import flow should make the tagging state visible before the final import step.

#### If no API key is present
Show a banner similar to:

> Google AI tagging is optional. No API key is currently saved, so this import will use Tier 1 keyword tagging only. You can add an API key in Settings if you want Gemini-based tagging.

Include links/buttons to:
- `Admin Settings`
- `AI Tagging Guide`

#### If an API key is present
Show a banner similar to:

> Google AI tagging is enabled for this installation. Depending on your saved settings, Tier 2 and/or Tier 3 may run during import. Gemini usage may incur cost. Free-tier limits and pricing may change over time.

Include the historical estimate and the pricing link.

---

### 3. Batch tagging for large imports

The app should support running AI tagging in manageable batches during import, without forcing the user to use the CLI.

#### Suggested behaviour
- Allow the user to import all selected files as normal.
- Allow AI tagging to be limited to a **batch size** for that import run.
- Good examples:
  - `Tag only the first 100 newly imported designs this run`
  - `Run Tier 2 now, leave Tier 3 for later`
  - `Import now and tag later`

#### Important
The first implementation should stay simple:
- **do not add a background job system** yet
- reuse the existing import flow and the current tier functions
- process the imported subset, commit safely, and finish normally

---

## Recommended implementation approach

The core service layer already has the right shape, so this should be an incremental change rather than a rewrite.

### Existing code to reuse
- `src/services/auto_tagging.py`
  - `suggest_tier1()`
  - `suggest_tier2_batch()`
  - `suggest_tier3_vision()`
- `src/services/bulk_import.py`
  - `confirm_import(..., run_tier2=..., run_tier3=...)`

### Proposed changes

#### `src/services/settings_service.py`
Add saved setting keys for:
- Tier 2 auto-run enabled/disabled
- Tier 3 auto-run enabled/disabled
- optional default batch size / delay

#### `src/routes/settings.py`
- load the saved AI-tagging preferences
- save them from the form

#### `templates/admin/settings.html`
- add the new checkboxes and explanatory text
- include the cost/quota warning near the AI controls

#### `src/routes/bulk_import.py`
- read the saved settings during the precheck / confirm flow
- expose API-key status to the template
- pass `run_tier2` and `run_tier3` into `confirm_import()` based on the saved settings
- optionally pass batch/limit values for the current import run

#### `templates/import/step3_precheck.html`
- show the missing-key or cost-warning banner
- show whether Tier 2 / Tier 3 are currently enabled in settings
- optionally show batch controls for large imports

#### `src/services/bulk_import.py`
- preserve Tier 1 as the default
- only run Tier 2 / Tier 3 when requested by settings / import options
- add small batch/limit handling if needed for the first version

#### `docs/AI_TAGGING.md`
- update the user-facing documentation so it matches the new in-app workflow

### Documentation updates required

The implementation should include a documentation pass so the user-facing guidance stays accurate.

#### Must update
- `docs/AI_TAGGING.md`
  - explain the new settings-driven Tier 2 / Tier 3 behaviour
  - document missing-key warnings, cost/quota warnings, and batch-tagging options
- `docs/feature-inventory.md`
  - update the **Bulk Import** and **Auto-Tagging Pipeline** sections
- `templates/info/help.html`
  - update the **Importing** and **AI Tagging** help text shown inside the app

#### Also review and update if needed
- `docs/GETTING_STARTED.md`
  - first-import guidance and optional AI-tagging setup notes
- `docs/TROUBLESHOOTING.md`
  - missing API key, disabled settings, and batching/rate-limit guidance
- `templates/about.html`
  - high-level wording about optional Gemini features being controlled from Settings
- `docs/USB_DEPLOYMENT.md`
  - portable-device import wording if the settings-driven AI behaviour needs to be mentioned there

> No update is needed to other files in `docs/Plans/` as part of the implementation itself.

---

## Relevant files

Likely files to update:

- `src/services/settings_service.py`
- `src/routes/settings.py`
- `templates/admin/settings.html`
- `src/routes/bulk_import.py`
- `templates/import/step3_precheck.html`
- `src/services/bulk_import.py`
- `docs/AI_TAGGING.md`
- `docs/feature-inventory.md`
- `templates/info/help.html`
- `docs/GETTING_STARTED.md` *(review and update if needed)*
- `docs/TROUBLESHOOTING.md` *(review and update if needed)*
- `templates/about.html` *(review and update if needed)*
- `docs/USB_DEPLOYMENT.md` *(review and update if needed)*

Likely test files:

- `tests/test_routes.py`
- `tests/test_services.py`
- `tests/test_bulk_import_extra.py`

---

## Acceptance criteria

The work is complete when the following are true:

1. **No API key present**
   - the import flow warns the user
   - only Tier 1 runs
   - no Gemini calls are attempted

2. **API key present, Tier 2 and Tier 3 both off**
   - import remains Tier 1 only

3. **API key present, Tier 2 on, Tier 3 off**
   - Tier 2 runs automatically during import
   - Tier 3 does not run

4. **API key present, Tier 3 ticked in Settings**
   - Tier 3 runs automatically during import for still-untagged designs with preview images

5. **Cost warning visible**
   - the user is informed that charges may apply
   - the historical February 2026 estimate is shown with a note that pricing may have changed

6. **Batch control available for large imports**
   - the user can avoid processing all AI tagging in one go

7. **No regression to the current free path**
   - Tier 1 keyword/path tagging still works exactly as before

---

## Suggested rollout

### Phase 1
- add the saved Tier 2 / Tier 3 settings
- add API-key and cost warnings
- wire import behaviour to those saved settings

### Phase 2
- add simple per-import batch controls for large imports
- keep the CLI tools available for advanced / overnight runs

---

## Summary

This change should make AI tagging **safer, clearer, and more user-controlled** without removing the current automation.

The key requirement is:

> **If Tier 3 has been ticked in Settings, it should run automatically during import when an API key is available.**

At the same time, the app should:
- warn users when no API key is present,
- warn that Gemini usage may incur cost when a key is present,
- and make large-import tagging manageable through batching.

---

## Agent implementation checklist

Use this checklist at the end of the work to confirm nothing has been missed.

### Settings
- [ ] Add saved settings for `Run Tier 2 automatically during import`
- [ ] Add saved settings for `Run Tier 3 automatically during import`
- [ ] Ensure the settings page loads the current values correctly
- [ ] Ensure the settings page saves the updated values correctly
- [ ] Keep the existing API key field working as before

### Import flow
- [ ] Show a clear warning when no `GOOGLE_API_KEY` is configured
- [ ] Link the user to `Admin Settings` and the AI tagging guide when the key is missing
- [ ] Show a clear cost/quota warning when an API key is present
- [ ] Reflect the saved Tier 2 / Tier 3 settings during the import precheck step
- [ ] Ensure Tier 1 remains the default path when AI options are not enabled

### Tier behaviour
- [ ] If Tier 2 is enabled in settings, run Tier 2 automatically during import
- [ ] If Tier 3 is enabled in settings, run Tier 3 automatically during import for eligible still-untagged designs
- [ ] If Tier 3 is not enabled, do not run it
- [ ] Do not attempt Gemini calls when no API key exists
- [ ] Preserve `tagging_tier` correctly for each tagging route

### Batch handling
- [ ] Add a simple way to limit AI tagging to a batch size for large imports
- [ ] Keep the first implementation synchronous and simple
- [ ] Do not introduce a background job framework in this pass
- [ ] Keep the CLI tools available for advanced or overnight runs

### Documentation and tests
- [ ] Update `docs/AI_TAGGING.md` so it matches the new in-app behaviour
- [ ] Update `docs/feature-inventory.md` to reflect the changed import and auto-tagging behaviour
- [ ] Update `templates/info/help.html` so the in-app help matches the new workflow
- [ ] Review and update, if needed, `docs/GETTING_STARTED.md`, `docs/TROUBLESHOOTING.md`, `templates/about.html`, and `docs/USB_DEPLOYMENT.md`
- [ ] Add or update route tests for the settings-driven import flow
- [ ] Add or update service tests for Tier 2 / Tier 3 enablement logic
- [ ] Verify there is no regression to existing Tier 1 keyword tagging

---


---

## Next Phase: Advanced CLI Tagging Actions and Controls (Web Agent Handoff)

**Note:** The first tranche of work (settings-driven, batchable, and user-controlled Gemini tagging during import) has been completed and is documented above. The following plan is for the next phase, to be implemented by a web agent.

### Overview
Expose advanced CLI tagging actions (re-tag all, tag untagged, batch/delay controls, verified/unverified options) in the app UI, integrate with settings, and provide user prompts/warnings consistent with the import flow. Make delay configurable in the UI, and add user guidance for new users and rate-limit handling.

---

### Phase 1: CLI Action Integration & Settings

1. **Design CLI Action UI**
  - Add a new "Tagging Actions" section in the admin or maintenance UI.
  - List available actions:
    - Re-tag all designs (overwrite existing tags)
    - Tag all untagged designs
    - Tag only unverified designs (do not overwrite verified)
    - Advanced: Choose between overwrite/add/replace for verified/unverified
  - For each action, provide options for batch size, delay, and tier selection (using current settings as defaults).

2. **User Prompts & Warnings**
  - For "include verified tags," prompt: “This will overwrite your previous verified tagging edits. Continue?”
  - For all actions, show the same warnings as the import flow (API key, cost, batch size, delay, etc.).
  - Add a tip: “If you’re new, try tagging a few designs first. You can set a batch size in Settings.”

3. **Settings Integration**
  - Ensure batch size and delay are settings-driven and can be overridden per action.
  - Add a new setting for delay (if not present) in settings_service.py and expose in the settings UI.

---

### Phase 2: Backend & CLI Wiring

4. **Backend Endpoints**
  - Add FastAPI endpoints to trigger CLI actions from the UI, passing user-selected options (action, batch size, delay, overwrite/add/replace, verified/unverified).
  - Ensure endpoints validate options and return progress/status.

5. **CLI Script Updates**
  - Update auto_tag.py to support new modes (overwrite/add/replace, verified/unverified logic).
  - Ensure scripts accept batch size and delay from both CLI and API calls.
  - Add logic to warn/abort if user chooses to overwrite verified tags.

---

### Phase 3: UI & Documentation

6. **UI Implementation**
  - Implement the "Tagging Actions" UI with all options and prompts.
  - Show warnings, tips, and progress/status.
  - Add delay field to settings and action forms.

7. **Documentation & Help**
  - Update docs/AI_TAGGING.md, docs/feature-inventory.md, and in-app help to document new actions, options, and best practices.
  - Add guidance for new users and rate-limit handling (429 Too Many Requests).

---

### Phase 4: Testing & Verification

8. **Testing**
  - Add/extend tests for new CLI options, backend endpoints, and UI flows.
  - Test all combinations: overwrite/add/replace, verified/unverified, batch/delay, rate-limit handling.

9. **Verification**
  - Confirm all actions work as expected and warnings/prompts are shown.
  - Verify settings are respected and can be overridden per action.
  - Ensure documentation and help are up to date.

---

**Relevant Files**
- auto_tag.py — CLI logic
- src/services/settings_service.py — settings storage
- src/routes/settings.py, src/routes/maintenance.py (or new) — UI endpoints
- templates/admin/settings.html, templates/admin/maintenance.html — UI
- docs/AI_TAGGING.md, docs/feature-inventory.md, templates/info/help.html — docs/help
- tests/test_routes.py, tests/test_services.py, tests/test_bulk_import_extra.py — tests

---

**Verification**
- All new actions are available in the app UI and work as described
- User is prompted/warned appropriately for destructive actions
- Batch size and delay are settings-driven and overridable
- Rate-limit (429) handling is documented and actionable
- Documentation and help are updated
- No regression to existing tagging/import flows

---

**Decisions & Scope**
- Delay setting is added to the UI and settings
- All CLI actions are exposed in the app for non-technical users
- User guidance and warnings are consistent with import flow
- No background job system is added in this pass
- Advanced/overnight CLI use remains available for power users

---

**Further Considerations**
1. Destructive actions (overwrite verified tags) should require extra confirmation.
2. Progress/status can be shown as a summary for now.
3. Failed/partial tagging runs do not need to be resumable in this pass.

When handing the work back, the agent should include:

1. A short summary of what changed
2. The list of files updated
3. Which documentation files were updated and why
4. Which tests were added or updated
5. The exact verification performed
6. Confirmation that this requirement is satisfied:

> If Tier 3 has been ticked in Settings, it runs automatically during import when an API key is available.
