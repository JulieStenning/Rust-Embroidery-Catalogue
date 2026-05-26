# Application Settings Runbook

The **Settings** page controls defaults used across import, tagging, backup, and desktop storage behavior.

Use this runbook when you need to:
- configure AI behavior for imports,
- tune performance for large runs,
- set up backup destinations,
- move desktop catalogue storage to another drive,
- troubleshoot settings-related behavior.

---

## What this page controls

Open **Admin -> Settings** to manage:
- default image preview mode for imports (2D vs 3D),
- Google Gemini API key,
- automatic AI tier defaults for imports,
- AI/import batch tuning values,
- desktop catalogue data location,
- read-only storage location display.

Some settings are directly used during import and tagging workflows. Others provide defaults that can be overridden in specific flows.

---

## Before changing settings

1. Keep the app open while saving so redirects and status messages are visible.
2. Back up first before major storage-location changes.
3. If enabling AI tiers, confirm you understand API costs and quotas.
4. For large catalogues, test with a small run before increasing throughput settings.

For AI key setup and tier details, see [AI_TAGGING.md](AI_TAGGING.md).

---

## Quick save workflow

1. Open **Admin -> Settings**.
2. Update one or more fields.
3. Click **Save settings**.
4. Confirm the green success message.
5. If you changed desktop **Catalogue data location**, restart the app.

If you see an error banner, review the troubleshooting section below.

---

## Field-by-field operations guide

### Image preview preference (2D vs 3D)

Options:
- **2D**: faster flat previews, best for bulk throughput.
- **3D**: slower stitch-simulated previews, better visual realism.

Use cases:
- Choose **2D** for very large imports or slower machines.
- Choose **3D** when preview fidelity is more important than speed.

Notes:
- This is a default for new imports.
- Existing designs keep their current image type.
- Some import flows can override this value per session.

Related:
- [FIRST_IMPORT_ACTIONS.md](FIRST_IMPORT_ACTIONS.md)

### Google Gemini API key

Purpose:
- Enables Tier 2 and Tier 3 AI tagging features.

Behavior:
- Key is stored in the project `.env` file.
- You can leave this blank to run keyword-only tagging.

Operational guidance:
- Paste the key, save settings, then verify AI tier checkboxes as needed.
- Use the show/hide button to verify pasted key format before saving.

Security guidance:
- Treat this as a secret.
- Avoid sharing screenshots that expose the key.
- If compromised, rotate the key in Google AI Studio and update Settings.

Related:
- [AI_TAGGING.md](AI_TAGGING.md)

### AI tagging during import (Tier 2/Tier 3 auto-run)

Options:
- **Run Tier 2 automatically** (Gemini text)
- **Run Tier 3 automatically** (Gemini vision)

Behavior:
- Tier 1 keyword tagging is independent and does not require a key.
- Tier 2/3 settings are ignored when no API key is available.

When to enable:
- Enable both for maximum tag quality on newly imported designs.
- Disable on constrained budgets or if you only want keyword-based tagging.

Related:
- [AI_TAGGING.md](AI_TAGGING.md)
- [TAGGING_ACTIONS_BACKFILL.md](TAGGING_ACTIONS_BACKFILL.md)

### AI tagging batch size (optional)

Purpose:
- Limits how many newly imported designs receive AI tier processing in one run.

When to lower:
- You are hitting quota/rate limits.
- You want shorter, incremental import sessions.

When to increase:
- You have stable throughput and want fewer sessions.

Practical pattern:
- Start small, observe failures/logs, then scale up.

### Delay between Gemini calls (seconds) (optional)

Purpose:
- Adds spacing between API calls to reduce rate-limit failures.

When to raise delay:
- Frequent `429 Too Many Requests` responses.
- API quota pressure during larger runs.

When to keep low/default:
- Small imports with no rate-limit errors.

Notes:
- If troubleshooting AI rate limits, tune delay and batch size together.
- Keep expectations realistic: higher delay improves resilience but increases run time.

Related:
- [AI_TAGGING.md](AI_TAGGING.md)

### Import database commit batch size (optional)

Purpose:
- Controls how many design operations are written before each DB commit during import.

Tradeoff:
- Lower values: smaller rollback scope on failure, more commit overhead.
- Higher values: better commit efficiency, larger rollback scope on interruption.

Suggested starting point:
- Use default behavior unless import performance or failure-recovery behavior requires tuning.

### Catalogue data location (desktop installs)

Purpose:
- Moves catalogue database and managed design storage to a different drive/location.

Typical reasons:
- Running out of space on system drive.
- Consolidating catalogue data on a dedicated data disk.

Procedure:
1. Back up first.
2. Open **Admin -> Settings**.
3. Set **Catalogue data location** using Browse or manual path.
4. Save settings.
5. Restart application.
6. Verify expected data appears in Browse and imports still work.

What to expect:
- Required subfolders are created automatically.
- Existing managed data may be copied forward when needed.
- Logs remain in their configured logs area.

Recovery if move goes wrong:
1. Do not continue importing until paths are confirmed.
2. Re-open Settings and set location back to the prior known-good folder.
3. Restart app and validate browse/import behavior.
4. If needed, restore from backup.

Related:
- [BACKUP_RESTORE.md](BACKUP_RESTORE.md)
- [GETTING_STARTED.md](GETTING_STARTED.md)

### Storage locations panel (read-only)

This panel displays:
- current catalogue data location,
- current log folder.

Use it for:
- support/troubleshooting handoff,
- confirming that a data-root move took effect,
- locating logs for diagnostics.

---

## Recommended operational workflows

### Workflow A: Conservative defaults for reliability

1. Keep image preference at 2D.
2. Keep AI tiers disabled unless needed.
3. Add API key only when preparing to use AI tagging.
4. Leave batch and commit fields blank/default.

### Workflow B: AI quality-first imports

1. Configure API key.
2. Enable Tier 2 and Tier 3 auto-run.
3. Start with moderate batch size.
4. Increase delay if rate-limit errors appear.
5. Review tagging outcomes and costs after first run.

### Workflow C: Large-library performance tuning

1. Use 2D image preference.
2. Set conservative AI batch size initially.
3. Tune import commit batch size based on throughput and recovery needs.
4. Process in staged runs, validating after each stage.

### Workflow D: Data-root relocation

1. Run backup first.
2. Move data location in Settings.
3. Restart app.
4. Validate browse, detail pages, and import operations.
5. Keep old storage untouched until validation is complete.

---

## Troubleshooting matrix

| Symptom | Likely cause | What to do |
|---|---|---|
| Settings save shows error banner | `.env` update failed or invalid desktop path write | Retry save; verify write permissions; for data-root path confirm drive exists and is writable. |
| Tier 2/3 not running | Missing API key or tiers not enabled | Confirm key saved, then re-open Settings and verify checkboxes; see [AI_TAGGING.md](AI_TAGGING.md). |
| Frequent `429 Too Many Requests` | API call pressure too high | Increase delay value and/or lower AI batch size; retry smaller runs. |
| Imports feel slow after enabling AI | High delay or larger AI scope | Reduce tier usage for routine runs or lower batch size; reserve full AI runs for targeted updates. |
| Backup actions fail due missing destination | Backup folders not configured | Configure backup destinations in maintenance backup UI; see [BACKUP_RESTORE.md](BACKUP_RESTORE.md). |
| Data-root move appears not applied | App not restarted or wrong target path | Restart app, check Storage locations panel, then verify browse/import behavior. |
| Designs not where expected after move | Pointing to unexpected data folder | Revert to prior known-good data location, restart, validate, then migrate again carefully. |

---

## Security and operational notes

- Keep API keys out of shared screenshots and logs.
- For team or multi-machine usage, configure keys and storage paths on each target machine.
- Revisit settings after major upgrades, especially import/AI defaults.

---

## Related guides

- [AI_TAGGING.md](AI_TAGGING.md) - AI tiers, pricing considerations, and API setup
- [FIRST_IMPORT_ACTIONS.md](FIRST_IMPORT_ACTIONS.md) - first-import defaults and action flow
- [BACKUP_RESTORE.md](BACKUP_RESTORE.md) - backup and restore operations
- [GETTING_STARTED.md](GETTING_STARTED.md) - installation and basic setup
- [TAGGING_ACTIONS_BACKFILL.md](TAGGING_ACTIONS_BACKFILL.md) - bulk maintenance workflows
