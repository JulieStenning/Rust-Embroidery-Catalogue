/**
 * Execution mode and path metadata returned by the Rust backend.
 */
export interface AppStatus {
  /** "portable" when running from an SD card / USB stick, "installed" otherwise. */
  execution_mode: "portable" | "installed";
  /** Absolute path to the data root directory. */
  data_root: string;
  /** Absolute path to the MachineEmbroideryDesigns directory. */
  embroidery_dir: string;
  /** Absolute path to the SQLite database file. */
  database_path: string;
}