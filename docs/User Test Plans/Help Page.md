## 📋 User Test Suite: Help Documentation View

This issue tracks the user-facing functionality and content validation for the **Help** view.

### ⚙️ Test Setup
- Ensure the application is connected to a standard testing database state with default configuration values.
- No specific design files are required to populate this view, as it contains static documentation.

### 🔄 Core Workflows to Test

- [x] **Initial Load & Layout Check**
  - [x] Screen opens without lag or visual stutter when clicking "Help" in the main navigation bar 
  - [x] Main header `Help` and sub-header `Quick guidance for using the Embroidery Catalogue.` render correctly 
  - [x] Quick navigation jump buttons render with correct icons and text matching the sections:
    - [x] `🔍 Search`
    - [x] `📭 Importing`
    - [x] `🤖 AI Tagging`
    - [x] `🏷️ Tagging Actions`
    - [x] `📁 Projects`
    - [x] `🛠️ Maintenance`
    - [x] `🔧 Troubleshooting`

- [ ] **Content Integrity & Verbatim Text Validation**
  - [x] **Search Section**:
    - [x] Header text `🔍 Search` matches perfectly.
    - [x] Body block correctly details `Quoted phrases: "cross stitch"`, `Exclude terms: -rose -applique`, `OR searches: rose OR tulip`, `Filename wildcards: rose*.jef`, `Unverified only`, and `Quick search vs. filters`.
  - [ ] **Importing Section**:
    - [ ] Content blocks for `Choosing folders`, `Review and metadata`, `Tag check before import`, `AI tagging notice`, and `Error files and large scans` match verbatim.
  - [ ] **AI Tagging Section**:
    - [ ] Content blocks for `Get an API key`, `Enable tiers`, `Batch size and delay`, `In-app actions`, `Potential costs`, and `Full guide` match verbatim.
  - [ ] **Tagging Actions Section**:
    - [ ] Content blocks for `Tag only untagged designs`, `Tag untagged and unverified designs`, `Re-tag ALL designs`, and `Local stitching backfill` match verbatim.
  - [ ] **Projects Section**:
    - [ ] Content blocks for `What projects are for`, `Adding designs`, `Bulk add`, and `Printing` match verbatim.
  - [ ] **Maintenance Section**:
    - [ ] Content blocks for `What orphaned records are`, `Deleting orphans`, and `Use carefully` match verbatim.
  - [ ] **Troubleshooting Section**:
    - [ ] Content blocks for `Missing folder / changed drive letter`, `Import scan finds nothing`, `Files missing after import`, `Managed storage location`, and `Still stuck` match verbatim.

- [ ] **Link & Navigation Target Verification**
  - [ ] Clicking jump buttons correctly scrolls the viewport to the corresponding header block.
  - [ ] Verify inline hyperlinks within text content redirect to the correct functional pages or external guides:
    - [ ] `Browse` link in Search body text navigates to `/browse/`
    - [ ] `Import` link in Importing body text navigates to `/import/`
    - [ ] `Google AI Studio` link opens the external API key portal
    - [ ] `Settings` link in AI Tagging body navigates to setup page
    - `Admin → Tagging Actions` text breadcrumb references match current view options
    - [ ] `current pricing` link opens the external Google AI billing overview
    - [ ] `AI Tagging Guide` link opens the deeper markdown documentation resource
    - [ ] `Projects` link in Projects body navigates to `/projects/`
    - [ ] `Orphans` link in Maintenance body navigates to `/orphans/`
    - [ ] Footer hyperlinks `About` and `Licence` prompt correct diagnostic models or sub-views

- [ ] **Navigation Boundaries**
  - [ ] User can safely exit or go back using top menu tabs (`Browse`, `Import`, `Projects`) without freezing the application.

### ❌ Failed Tests / Discovered Friction
*Hover over a failed subtest above and click "Convert to issue", or track them below:*