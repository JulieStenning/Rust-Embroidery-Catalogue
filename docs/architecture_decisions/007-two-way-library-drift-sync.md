# Architecture Decision Record 007: Two-Way Library Drift Synchronization

* **Status:** Accepted
* **Deciders:** Julie Stenning
* **Date:** 2026-10-07

---

## Context and Problem Statement

A local embroidery catalogue must coexist with standard file manager operations. Users routinely drop new folders into the design library (`MachineEmbroideryDesigns`) or move/delete obsolete folders using Windows Explorer.

This creates two distinct types of state drift between the physical disk and the SQLite catalogue:
1. **Disk $\rightarrow$ Database Drift (Unmatched Files):** Files exist on disk in the library root but have no corresponding record in SQLite.
2. **Database $\rightarrow$ Disk Drift (Orphaned Records):** Database records point to canonical file paths that no longer exist on disk.

Originally, scanning for unmatched disk files was placed inside the Database Restore view, which was unintuitive and inaccessible for routine cataloguing workflows.

---

## Decision Drivers

* **Intuitive Workflow Placement:** Maintenance tools must be co-located with the views where users perform cataloguing and library cleanup.
* **Separation of Concerns:** Keep disaster recovery (restoring database backups) cleanly decoupled from routine library drift synchronization.
* **Fast Selective Ingestion:** Allow users to scan and selectively import unmatched files into the catalogue without forcing a slow, full-library re-index.
* **Non-Destructive Cleanup:** Offer clear, safe metadata removal for orphaned database records without altering files on disk.

---

## Considered Options

1. **Continuous Real-Time Filesystem Watcher:** Run background filesystem notifications (`notify` crate). (Prone to file-lock issues on external drives, high background CPU/battery drain, and noisy notifications during large copy operations).
2. **Centralized in Database Restore:** Force all drift reconciliation into the backup/restore flow.
3. **Decoupled Two-Way Drift Workflows (Chosen):** Expose on-demand reconciliation in **Bulk Import** (for new disk assets) and **Orphaned Files** (for cataloguing cleanup and missing file reconciliation).

---

## Decision Outcome

**Option 3 (Decoupled Two-Way Drift Workflows)** was chosen.

### Architectural Invariants:
1. **Unmatched Files Reconciler (`UnmatchedFilesReconciler.svelte`):**
   - Available within **Bulk Import** (Tab 2) to onboard newly discovered disk files.
   - Available within **Orphaned Files** to reconcile discrepancies discovered during file maintenance.
   - Computes diffs between the `MachineEmbroideryDesigns` library root on disk and indexed database paths.
2. **Orphaned Records Management (`OrphansView.svelte`):**
   - Detects catalogue records whose physical file is missing from disk.
   - Provides safe, batch metadata deletion (`delete_orphaned_designs`) with explicit confirmation.
3. **Recovery Isolation:** Database Recovery / Restore views remain dedicated to backup archives and corrupt database recovery, free of everyday drift management.
