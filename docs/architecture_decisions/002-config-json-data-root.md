# Architecture Decision Record 002: Bootstrap Config Persistence for Data Root

* **Status:** Accepted
* **Deciders:** Julie Stenning
* **Date:** 2026-10-06

---

## Context and Problem Statement

The application allows users to configure a custom catalogue data root path (e.g., `D:\EmbroideryData` or removable media). The SQLite database itself resides *under* the configured data root directory (`<data_root>/Database/catalogue.db`).

If the active data root path were stored in the SQLite `settings` table, a bootstrap circular dependency would occur: the application could not know where to find the database file without opening the database file first.

Furthermore, developer test harnesses (`ExecutionMode::Dev`) must never read or overwrite the live user installation configuration in `%APPDATA%\EmbroideryCatalogue\config.json`.

---

## Decision Drivers

* **Bootstrap Independence:** The application must know where its data files and database reside before opening any database connection.
* **Environment Segregation:** Development, end-to-end tests, and installed production environments must never cross-pollute configuration files.
* **Clean Relocation:** Changing the data root must persist to a bootstrap config file, followed by an intentional application restart.

---

## Decision Outcome

1. **Bootstrap File Persistence:** The data root configuration is persisted exclusively to `config.json` via `paths::write_bootstrap_data_root` / `set_configured_data_root`, never to SQLite.
2. **SQLite Invariant:** Settings view model handlers (`save_settings_view_model_inner`) deliberately ignore `request.data_root` when saving application preferences to SQLite.
3. **Execution Mode Isolation:**
   - In `ExecutionMode::Dev`, the data root is local to the debug workspace (e.g. `dev_data/`) and debug env override `EMBROIDERY_DATA_ROOT`.
   - In `ExecutionMode::Installed`, the configuration resides in `%APPDATA%\EmbroideryCatalogue\config.json`.
