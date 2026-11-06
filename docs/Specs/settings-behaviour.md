# Settings Behaviour Specification

Behaviour of the Settings page (`frontend/src/lib/views/SettingsView.svelte`). The user guide is `docs/User-Facing-Guidance/SETTINGS.md`.

## Saving

- Editable fields are compared with the saved database state; when different, an **Unsaved changes** indicator appears and **Save settings** is enabled. The button shows **Saving...** during the write, followed by a success toast.
- Theme, catalogue data location and database compaction do not use the Save button: theme applies immediately; data location opens the migration modal; compaction opens `ConfirmCompactDatabaseModal`.

## Google Gemini & AI Tagging

| Field | Default | Notes |
|---|---|---|
| API key | blank | Stored locally in the SQLite database; show/hide toggle. Used only by Gemini Vision in Batch Operations; imports never call Gemini. |
| Free tier checkbox | off | When ticked, default Workers becomes 2 (instead of 4) and default delay 10 s (instead of 0 s). |
| AI tagging batch size | 100 | Chunk size per fetch; a run still pages through the full candidate set. |
| Commit every | 100 | Commit and progress-report cadence. |
| Workers | 4 (2 on free tier) | Min 1, max 32. |
| Delay between Gemini calls | 0 s (10 s on free tier) | Step 0.5. |
| Gemini model | Auto-select | Auto-select picks a Flash model. **Refresh** lists models from the API using the saved key, Flash models first. **Test model** sends a validation request. If a saved model is retired, runs fall back to an available Flash model. |

## Database Maintenance

- Displays database file size and estimated recoverable space.
- FAT32 warning (`near_fat32_limit`) shows when the database is between 3.5 GB and 4.0 GB on a FAT32 volume.
- **Optimise & Compact Database** runs `VACUUM` and `PRAGMA optimize`. **Backup & Compact** writes a full backup to the configured backup directory first and aborts compaction if the backup fails. Requires free space at least equal to the database size.
- **Database health check interval**: default 1800 s, minimum 5 s. When idle, if reclaimable space exceeds 20% of database size and is at least 20 MB, `PRAGMA incremental_vacuum` runs in the background.

## Catalogue Storage

- **Browse…** selects a destination; the path is normalised so the `Database/`, `MachineEmbroideryDesigns/` and `logs/` layout is preserved without double nesting.
- The **Moving your catalogue…** modal shows file count, bytes and percentage. **Cancel** rolls back (ADR 011 atomic cancel). On completion, **Restart required** → **Restart now** relaunches against the new data root.
- **Active Storage Locations** shows Data Root, Database File and Log Folder.

## Digitising & Master File Formats

Presets (all disabled by default):

| Preset | Extensions |
|---|---|
| Embird Studio & Cross Stitch | `.eof`, `.ecf` |
| Wilcom / Hatch | `.emb` |
| Bernina | `.art`, `.art60`, `.art70`, `.art80` |
| Embrilliance | `.be` |
| Janome Digitizer | `.jan` |
| mySewnet / Premier+ | `.edo`, `.vp4` |

- Custom extensions are comma-separated.
- A master file with a same-stem stitch file in the same folder is paired into one design with a Master badge (ADR 006). A master-only file is catalogued with a placeholder and flagged for export to a stitch format.

## Alphabet & Font Compression

Default: enabled (also exposed in @InitialSetupView.svelte via @AlphabetCompressionSetting.svelte).

- Detected alphabet/font folders are packaged into `MachineEmbroideryDesigns/<AlphabetFolder>/<AlphabetName>.zip` if no companion zip exists; an existing source companion zip is copied instead.
- Per size, only representative glyphs are catalogued: Upper A (`Upper A`, `Uppercase A`, `Cap A`), Lower a (`Lower a`, `Lowercase a`), Digit 9 (`9`, `Number 9`), and all punctuation and symbols. `B`–`Z`, `b`–`z` and `0`–`8` are skipped.
- The zip keeps the full package, including all sizes and Embrilliance `.bx` files. Source files are never modified.

