# Application Settings Runbook

The **Settings** page controls operational defaults and system parameters used across appearance, AI tagging, database maintenance, catalogue storage, and digitising format filters.

Use this runbook when you need to:
- switch between system, light, and dark appearance themes,
- set up your Google Gemini API key and tune AI throughput for Batch Operations,
- test and select specific Google Gemini models,
- monitor database storage usage and optimise/compact the SQLite database,
- adjust automatic database fragmentation health checks,
- relocate desktop catalogue storage to a different location with live migration progress,
- configure enabled digitising software master outline formats and custom extensions,
- manage digital alphabet and font companion packaging,
- troubleshoot settings and performance issues.

> **Note:** **Product Licence & Activation** is managed on the [About](#/about) page ([Admin → About](#/about)), where you can review your registered email, check validity, or deactivate/switch your licence key. You can also inspect open-source dependencies and review terms in [Licence & Dependencies](#/about/licence).

---

## What this page controls

Open [Admin → Settings](#/admin/system/settings) to manage:
1. **Appearance**: toggle between System Default (OS preference), Light, or Dark theme modes.
2. **Google Gemini & AI Tagging**:
   - Google Gemini API key (with show/hide password toggle).
   - Free tier API rate-limit protection mode.
   - Batch chunk size, commit cadence, parallel worker concurrency, and call delay.
   - Live Gemini model selection, model list refreshing, and connection testing.
3. **Database Maintenance & Diagnostics**:
   - Live database storage size and estimated recoverable space metrics.
   - FAT32 4 GB single-file limit boundary warnings.
   - Manual database compaction (`VACUUM` + `PRAGMA optimize`) with optional pre-compaction backup.
   - Automatic database fragmentation check interval.
4. **Catalogue Storage**:
   - Desktop catalogue data relocation with safe interactive migration and restart.
   - Active storage paths (data root, log folder, database path).
5. **Digitising & Format Rules**:
   - Software-native working formats (Embird, Wilcom/Hatch, Bernina, Embrilliance, Janome, mySewnet/Premier+).
   - Additional custom file extensions.
   - Alphabet & font compression into companion `.zip` archives.

---

## Before changing settings

1. Keep the app open while saving so redirects and status messages remain visible.
2. Back up your catalogue before major storage relocations or manual database compaction.
3. If entering a Google Gemini API key, confirm your tier (free vs paid) and enable the Free Tier checkbox if applicable.
4. For large catalogues, test AI tagging with a small batch before increasing parallel workers.
5. Ensure your target drive has sufficient free space when migrating catalogue storage or compacting the database.

For AI key setup and vision tagging details, see [BATCH_OPERATIONS_BACKFILL.md](BATCH_OPERATIONS_BACKFILL.md).

---

## Quick save workflow

The Settings page tracks unsaved changes:
1. Open [Admin → Settings](#/admin/system/settings).
2. Update one or more form fields.
3. When editable fields differ from the current saved database state, an **Unsaved changes** indicator (orange accent dot) appears in the header and the **Save settings** button activates.
4. Click **Save settings**. The button shows **Saving...** while writing to the database.
5. Confirm the green success toast notification (**"Settings saved successfully."**).

> **Note:** Some controls take effect immediately or use dedicated modals rather than the Save button:
> - **Appearance (Theme)** changes take effect instantly.
> - **Catalogue Data Location** opens an interactive migration modal upon folder selection and prompts for an application restart.
> - **Database Compaction** opens a confirmation modal to execute immediately.

---

## Field-by-field operations guide

### 1. Appearance

Select the interface theme for the desktop application:
- **🌓 System Default**: dynamically mirrors your operating system's light or dark mode setting.
- **☀️ Light**: forces high-contrast light mode backgrounds and controls.
- **🌙 Dark**: forces dark mode styling for comfortable low-light browsing.

Theme changes apply immediately across all views without requiring a page reload or restart.

---

### 2. Google Gemini & AI Tagging

#### Google Gemini API key
- **Purpose**: Enables Gemini Vision image-based tagging features in Admin → Batch Operations.
- **Behaviour**: Stored securely in your local application database.
- **Visibility**: Toggle using the eye icon (👁 / 🙈) to verify your entered key format.
- **Blank State**: Leave this field blank for local-only operation. The application will continue to run free keyword and stitching rule tags locally with no external network calls.

#### Google API key free tier mode
- **Purpose**: Configures conservative concurrency and delay values when using Google's free Gemini API key tier.
- **Behaviour**: Google's free tier imposes strict rate limits (~15 requests per minute).
- **Automatic Tuning**: Checking **"My Google API key is on the free tier"** automatically adjusts default parameters:
  - **Workers**: defaults to **2** (instead of 4).
  - **Delay between calls**: defaults to **10 seconds** (instead of 0 seconds).

#### AI performance & throughput tuning
- **AI tagging batch size (optional, default: 100)**: Number of designs fetched and processed together per chunk during a Batch Operations run.
- **Commit every (optional, default: 100)**: Progress and database commit cadence during an AI run. Determines how often tag updates are committed to SQLite and reflected in the live UI counters.
- **Workers (optional, min: 1, max: 32, default: 4, or 2 on free tier)**: Number of designs tagged concurrently in parallel worker threads. Lower to **1 or 2** if encountering API rate limits (`429 Too Many Requests`).
- **Delay between Gemini calls (seconds) (optional, step: 0.5, default: 0 s, or 10 s on free tier)**: Pause duration inserted between consecutive API calls. Increase this if rate-limit errors persist during large runs.

#### Gemini model selection, refresh & testing
- **Model Dropdown**: Defaults to **Auto-select (recommended)**. When left on Auto-select, the application automatically chooses an optimal Gemini Flash model.
- **Flash First Ordering**: Models retrieved from Google's API are sorted Flash models first. Flash models are strongly recommended: they are significantly faster and cheaper than Pro or thinking models while excelling at embroidery image vision tagging.
- **Refresh Button**: Queries Google's API using your saved key and populates the dropdown with all currently supported models.
- **Test Model Button**: Sends a validation ping with the selected model to verify that your API key and chosen model are valid and responding. Displays a green success or red error message below the input.
- **Fallback Resilience**: If a previously selected model is retired by Google, the app automatically falls back to an available Flash model at run time.

---

### 3. Database Maintenance & Diagnostics

#### Database size & recoverable space
- Displays total on-disk SQLite file size and estimated reclaimable space in real time.

#### FAT32 4 GB limit warning
- If your catalogue is stored on a FAT32-formatted drive (common on USB flash drives and SD cards) and the database file reaches between **3.5 GB and 4.0 GB**, an alert banner warns that FAT32 drives enforce a strict 4.0 GB single-file ceiling.
- **Recommended Action**: Back up your catalogue immediately and reformat the storage drive to **exFAT** or **NTFS**.

#### Optimize & Compact Database button
- Opens the `ConfirmCompactDatabaseModal` with two options:
  - **Backup & Compact (Recommended)**: creates a full database backup in your configured backup directory first, then runs compaction.
  - **Compact Only**: proceeds directly with compaction.
- Compaction runs `VACUUM` and `PRAGMA optimize`, defragmenting database pages and returning free disk space to the operating system.
- **Disk Space Requirement**: Compaction requires free space on the database drive equal to at least the current database file size to assemble the defragmented file safely.

#### Database health check interval (seconds)
- Sets how often the application checks the SQLite database for page fragmentation while idle (default: **1800 seconds / 30 minutes**; minimum: 5 seconds).
- When the idle check detects that reclaimable free space exceeds **20%** of database size and is at least **20 MB**, a lightweight background compaction (`PRAGMA incremental_vacuum`) reclaims disk space without pausing or freezing the user interface.

---

### 4. Catalogue Storage

#### Catalogue data location & migration
- **Purpose**: Relocates your catalogue database and design library folder (`MachineEmbroideryDesigns`) to a different location.
- **Procedure**:
  1. Under **Catalogue Storage**, click **Browse…**.
  2. Select an existing destination folder for the new location. (The app normalises the path so the internal layout `Database/`, `MachineEmbroideryDesigns/`, and `logs/` is preserved without double-nesting).
  3. In the interactive **Moving your catalogue…** modal, monitor real-time copy counts, byte transfers, and percentage progress. Click **Cancel** at any time to safely halt the process and roll back.
  4. Once copying completes, the modal displays **Restart required**.
  5. Click **Restart now** to relaunch the application pointing to the new catalogue location.

#### Storage locations panel
- **Data Root**: Parent directory containing the library and database folders.
- **Database File**: Absolute path to the active SQLite `.db` file.
- **Log Folder**: Directory where system logs are kept (preserved across storage moves).

---

### 5. Digitising & Format Rules

#### Digitising & master file formats
- **Purpose**: Allows the import scanner to detect and index software-native master outline design files alongside standard machine stitch formats.
- **Software Presets**:
  - **Embird Studio & Cross Stitch**: `.eof`, `.ecf`
  - **Wilcom / Hatch**: `.emb`
  - **Bernina**: `.art`, `.art60`, `.art70`, `.art80`
  - **Embrilliance**: `.be`
  - **Janome Digitizer**: `.jan`
  - **mySewnet / Premier+**: `.edo`, `.vp4`
- **Custom Extensions**: Type comma-separated custom extensions (e.g. `pxf, pat, can`) into the **Additional / Custom extensions** field.
- **Behaviour**:
  - **Paired with stitch file**: If a master file is found alongside a matching machine stitch file in the same folder (e.g. `flower.eof` and `flower.pes`), they are paired together into one card with a `[🎨 Master]` badge.
  - **Master-only**: If only a master file exists without a corresponding machine stitch file, it is catalogued with a placeholder and marked for export (*"Export to a machine stitch format to generate preview and stitch data."*).
  - By default, master formats are disabled. Enable only the software you actively use to avoid unwanted files being scanned.

#### Alphabet & font compression (default: enabled)
- **Purpose**: Automatically detects embroidery font and alphabet packages during import.
- **Behaviour**:
  - **Automatic packaging**: Scanned folders recognised as an embroidery alphabet or font collection have their complete contents packaged into a companion `.zip` archive in the library folder (`MachineEmbroideryDesigns/<AlphabetFolder>/<AlphabetName>.zip`) if one does not already exist, or copies an existing companion `.zip` from the source folder.
  - **Representative glyphs catalogued**: For **each size** found in the alphabet package, only sample designs are imported into the database: **Upper A** (`Upper A`, `Uppercase A`, `Cap A`), **Lower a** (`Lower a`, `Lowercase a`), **Digit 9** (`9`, `Number 9`), and all punctuation and symbols.
  - **Redundant letters omitted**: Letters `B`–`Z`, `b`–`z`, and digits `0`–`8` are skipped during database insertion, preventing hundreds of redundant design cards from cluttering your browse view.
  - **Instant extraction**: The full package (including all letters, sizes, and Embrilliance `.bx` files) is preserved in the companion `.zip` in the folder, ready to extract whenever needed for stitching.
  - **Safety**: Original source files are strictly read-only and never altered or deleted.

---

## Recommended operational workflows

### Workflow A: Everyday setup & appearance
1. Select your preferred Appearance theme (System, Light, or Dark).
2. Review your Google Gemini API key or leave blank for local-only tagging.
3. Check only the digitising master formats you use.
4. Click **Save settings**.

### Workflow B: AI tagging with Google free-tier key
1. Enter your Google Gemini API key.
2. Tick **My Google API key is on the free tier**.
3. Leave AI Workers at **2** and Delay at **10 seconds**.
4. Leave Gemini Model on **Auto-select (recommended)** and click **Test model** to verify.
5. Click **Save settings**.
6. Run targeted Gemini Vision jobs from [Admin → Batch Operations](#/admin/batch-operations).

### Workflow C: Routine database health & compaction
1. Review the **Database size** and **Recoverable** metrics in the Database Maintenance section.
2. If recoverable space is substantial, click **Optimize & Compact Database**.
3. Choose **Backup & Compact** to ensure a restorable copy is preserved before compacting.
4. Confirm completion toast reporting reclaimed pages and file size reduction.

### Workflow D: Relocating catalogue to a different location
1. Run a database backup from [Admin → System Maintenance → Backup](#/admin/system/backup).
2. Go to [Admin → Settings](#/admin/system/settings) → **Catalogue Storage** and click **Browse…**.
3. Select your destination folder.
4. Monitor the live migration progress bar.
5. When finished, click **Restart now** to relaunch Embroidery Catalogue.

---

## Troubleshooting matrix

| Symptom | Likely cause | What to do |
|---|---|---|
| Settings save shows error banner | Write permission issue or invalid path | Retry save; verify write permissions on the catalogue directory. |
| Frequent `429 Too Many Requests` during AI tagging | Rate limits exceeded on Google API key | Tick the **My Google API key is on the free tier** checkbox; raise Delay to 10s and lower Workers to 1 or 2. |
| Gemini model test fails | Invalid API key or model name unavailable | Re-check key in Google AI Studio; click **Refresh** to fetch valid model names; verify network connectivity. |
| Test model button is disabled | No API key entered or task busy | Enter an API key and wait for any ongoing operations to complete. |
| Compaction fails | Insufficient free disk space on database drive | Ensure free disk space exceeds current database file size before compacting. |
| Database approaching 4 GB warning | Database on FAT32 drive reaching file limit | Back up your catalogue and reformat drive to exFAT or NTFS before database reaches 4 GB. |
| Storage migration fails or stalls | Destination path unwritable or disk full | Verify destination location is accessible and has enough space; click Cancel to roll back cleanly. |
| App does not restart after migration | System permissions prevented relaunch | Close the application and open it manually; verify new paths in the Storage Locations panel. |
| Looking for licence key activation | Licence moved to About | Open [Admin → About](#/about) to review registration or deactivate/switch keys. |
| Looking for third-party licences & terms | Reviewing open-source crates and packages | Open [Licence & Dependencies](#/about/licence) to search, filter, and view licence terms. |

---

## Security and operational notes

- Google API keys are stored locally in the SQLite database and are never transmitted to third parties other than Google's Gemini API endpoints.
- Keep API keys out of shared screenshots and export logs.
- Original embroidery design files are **strictly read-only**: the application never modifies, renames, or deletes original files.
- The companion `.zip` font compression preserves all original glyph files, Embrilliance `.bx` files, and notes for full fidelity.

---

## Related guides

- [BATCH_OPERATIONS_BACKFILL.md](BATCH_OPERATIONS_BACKFILL.md) - Gemini Vision tagging, bulk operations, and Library Discrepancies reconciliation
- [FIRST_IMPORT_ACTIONS.md](FIRST_IMPORT_ACTIONS.md) - first-import defaults and action flow
- [BACKUP_RESTORE.md](BACKUP_RESTORE.md) - backup and restore operations
- [SUPPORTED_FORMATS.md](SUPPORTED_FORMATS.md) - supported stitch and master design formats
- [GETTING_STARTED.md](GETTING_STARTED.md) - installation and basic setup
