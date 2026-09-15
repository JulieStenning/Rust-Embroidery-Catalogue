# Exploratory & Hardware Edge Cases Test Plan

This document outlines manual and exploratory testing scenarios for hardware boundaries, removable storage, display factors, and system stress limits.

---

## 1. Removable Storage & External Drives (USB)

* [ ] **1.1 Import Directly from USB Flash Drive**
  * Insert a USB flash drive containing a folder of embroidery designs.
  * Start an import targeting the USB drive.
  * Confirm files are copied cleanly to local managed storage and previews render properly.
* [ ] **1.2 Abrupt Disconnect Recovery Mid-Import**
  * Start importing a large folder (500+ files) directly from the USB flash drive.
  * Unplug the USB drive while the import is in progress.
  * **Expected Result:** The application catches the I/O error gracefully, displays an error notification, halts the batch operation cleanly, and leaves the SQLite database in a consistent, uncorrupted state.
* [ ] **1.3 Backup to Removable / Slow Storage**
  * Configure backup target to an external USB stick or network share (NAS).
  * Run a full backup; confirm progress indicators update steadily and file operations complete without timeout errors.

---

## 2. Display Scaling & Multi-Monitor Transitions

* [ ] **2.1 High-DPI Scaling (125%, 150%, 200%)**
  * Under Windows Settings → Display → Scale & Layout, set scale to **125%** and **150%**.
  * Launch the application and verify:
    * Font sizes, button icons, and table headers render crisp without blurriness.
    * Grid cards maintain proper proportions without overlapping or clipping text.
* [ ] **2.2 Multi-Monitor Window Dragging**
  * Move the application window between monitors with different display resolutions/scaling (e.g. 4K 150% → 1080p 100%).
  * Confirm the WebView2 window and responsive grid dynamically reflow without visual artefacts or lockups.

---

## 3. High-Volume Library & Performance Stress

* [ ] **3.1 Large Collection Browsing (10,000+ Designs)**
  * Load a database populated with 10,000+ embroidery design records.
  * Open **Browse Designs** and rapidly scroll through multiple pages.
  * Verify virtual scrolling / pagination maintains responsive 60 FPS scrolling without memory leaks or UI freeze.
* [ ] **3.2 Rapid Filter & Search Stress**
  * Type rapid keystrokes into the Search box with multiple active tag filters and sort options.
  * Verify search debounce works smoothly and no unhandled IPC race conditions or visual glitches occur.

---

## 4. Visual Rendering Quality & Stitch Preview Accuracy

* [ ] **4.1 Vector & 3D Stitch Simulation Check**
  * Open several intricate embroidery designs with dense stitch patterns, multiple thread color changes, and appliqué layers.
  * In **Design Details**, inspect both 2D flat preview and detailed stitch simulation.
  * Verify thread paths, jump stitches, and color blocks render accurately according to the design specification.
