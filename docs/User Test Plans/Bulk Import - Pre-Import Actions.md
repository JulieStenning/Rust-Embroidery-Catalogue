## 📋 User Test Suite: Bulk Import - Pre-Import Actions & Preferences

This issue tracks the user-facing functionality for the **Bulk Import - Pre-Import Actions** decision step view based on `image_d1d240.jpg` which follows the review selection screen.

### ⚙️ Test Setup & Prerequisites
Before executing these tests, prepare the following local environment states:
- **API Key Configurations**: 
  - **State A (Enabled)**: Ensure a valid Google Gemini API key is configured in settings to trigger the active auto-run parameters.
  - **State B (Disabled)**: Temporarily clear the API key to test the Tier 1 fallback banner state.
- **Default Preview Preference**: Configure the base global preference to "3D" in the main settings view to verify session initialization inheritance.

---

### 🔄 Core Workflows to Test

- [ ] **Initial Load & Layout Verification**
  - [ ] Screen opens smoothly immediately following the "Continue with [X] designs" review handshake.
  - [ ] Default layout elements and informational blocks render exactly matching `image_d1d240.jpg`:
    - [ ] Title header text: "Bulk Import"
    - [ ] Sub-section label header: "Before You Import"
    - [ ] Instruction reminder caption text: "Consider reviewing your hoops, tags, sources, or designers before importing. Hoops usually only need special attention on the first import."
  - [ ] Action buttons render clearly in a horizontal control layout:
    - [ ] Reference Review Data actions: "Review Hoops", "Review Tags", "Review Sources", "Review Designers"
    - [ ] Execution actions: "Import Designs", "Cancel"

- [ ] **AI Tagging Banner State & Dynamic Settings Sync**
  - [ ] **Active API Key Information Rendering**: When an API key is present, the dynamic banner text displays: "Google AI tagging is enabled for this installation".
    - [ ] Subtext clearly highlights rate limits ("15 requests per minute and 1,500 requests per day") and current tier status.
    - [ ] Current live tracking details display accurately based on system configuration: `Tier 2 auto: on - Tier 3 auto: off - AI batch limit: 10 designs - DB commit batch: 10 designs`.
  - [ ] **Settings Link Interactivity**: Clicking the inline text link "Change in Settings" successfully jumps focus to the global application configuration page.
  - [ ] **Dynamic Sync Reflectivity**: Changing the Tier 2/Tier 3 toggle or batch limit values within the main settings tab dynamically updates the text parameters on this pre-import view when navigating back without forcing an app reload.
  - [ ] **Missing API Key State**: Clearing out the API key transforms the banner layout to display the "Tier 1 keyword tagging only" structural warning block.

- [ ] **Image Preview Preference Selection & Overrides**
  - [ ] **Saved Setting Inheritance**: The radio group correctly identifies the global profile configuration and displays an inline notation label next to it (e.g., `(Saved setting: 3D)`).
  - [ ] **Session Override Interactivity**:
    - [ ] Changing the selection to the **"2D - Fast flat preview"** radio option switches active item focus smoothly.
    - [ ] Changing the selection back to **"3D - Detailed stitch simulation"** functions flawlessly.
  - [ ] **Persistence Scope Safeguards**: Overriding the preview rendering setting on this screen acts strictly as a temporary session override; confirming or canceling the import does not alter the base global setting configured on the primary application settings page.

- [ ] **Admin Reference Pages Navigation (Import Context Persistence)**
  - [ ] **Review Hoops Execution**: Clicking **Review Hoops** successfully redirects the view layout to the Hoops management screen while maintaining an active underlying import context flag.
  - [ ] **Review Tags Execution**: Clicking **Review Tags** opens the Tags dashboard within the import context.
  - [ ] **Review Sources Execution**: Clicking **Review Sources** opens the Sources control panel within the import context.
  - [ ] **Review Designers Execution**: Clicking **Review Designers** opens the Designers overview database within the import context.
  - [ ] **Import Mode Return State**: Saving data or interacting within any of these four sub-review spaces displays a prominent "Continue with import" option that routes the user cleanly back to this Pre-Import workflow choice step.

- [ ] **Workflow Finalization & Exit Boundaries**
  - [ ] **Cancel Execution**: Clicking **Cancel** safely aborts the current processing transaction and returns the focus framework back to the folder review selection stage.
  - [ ] **Import Designs Commit**: Clicking **Import Designs** triggers the final background batch operation sequence:
    - [ ] Initiates appropriate 2D flat renders or 3D stitch simulation profiles based on the session preview selection.
    - [ ] Generates image previews, applies database entries, duplicates files into managed storage, and successfully routes the user to the "Browse Designs" catalog overview page upon successful finish.
    - [ ] New designs are shown when sort order set to date added/descending

---

### ❌ Failed Tests / Discovered Friction
*Hover over a failed subtest above and click "Convert to issue", or track them below:*