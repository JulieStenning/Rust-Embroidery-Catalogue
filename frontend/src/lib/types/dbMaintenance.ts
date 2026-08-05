/**
 * TypeScript type parity for the Rust backend's database maintenance events.
 *
 * These interfaces mirror the serde-serialized payloads in
 * `src/services/db_health.rs` (`DbMaintenanceStartedEvent`,
 * `DbMaintenanceFinishedEvent`). Field names must match exactly.
 */

/** Tauri event name emitted when an incremental-vacuum maintenance run begins. */
export const DB_MAINTENANCE_STARTED = "db-maintenance-started";

/** Tauri event name emitted when an incremental-vacuum maintenance run finishes. */
export const DB_MAINTENANCE_FINISHED = "db-maintenance-finished";

/** Payload for the `db-maintenance-started` event. */
export interface DbMaintenanceStartedEvent {
  page_count: number;
  freelist_pages: number;
  free_ratio: number;
  reclaimable_bytes: number;
}

/** Payload for the `db-maintenance-finished` event. */
export interface DbMaintenanceFinishedEvent {
  reclaimed_pages: number;
  reclaimable_bytes_before: number;
  reclaimable_bytes_after: number;
  duration_ms: number;
}