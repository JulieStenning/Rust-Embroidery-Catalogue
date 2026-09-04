/**
 * Subscribes to the `database-backup-completed` event emitted by the Rust
 * backend the moment the database phase of a combined ("both") backup finishes.
 *
 * In `run_both_backups` the database copy runs first and the designs copy runs
 * afterwards, so this signal lets `BackupView` switch its cancel-confirmation
 * wording from "the database copy is currently running" to "the database copy
 * has completed" while the confirmation modal is still open.
 */

import type { UnlistenFn } from "@tauri-apps/api/event";

export const DATABASE_BACKUP_COMPLETED_EVENT = "database-backup-completed";

/**
 * Install a Tauri event listener for database-backup completion. The payload is
 * empty; the callback is invoked whenever the backend reports the database
 * phase of a combined backup has finished.
 *
 * Returns an async cleanup function (call it in `onDestroy` of the owning
 * view). `listen` is imported dynamically so the module can be code-split.
 */
export async function initDatabaseBackupCompletedEvent(
  onDatabaseBackupCompleted: () => void
): Promise<UnlistenFn> {
  const { listen } = await import("@tauri-apps/api/event");
  return listen(DATABASE_BACKUP_COMPLETED_EVENT, () => onDatabaseBackupCompleted());
}
