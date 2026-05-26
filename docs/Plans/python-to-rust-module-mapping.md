# Python to Rust Module Mapping Plan

This document provides a detailed mapping plan for migrating the Python modules in the Rust-Embroidery-Catalogue project to a Rust module structure. It lists all relevant Python files, their purposes, and proposes a Rust module tree that matches or improves upon the current organization. For each Python file or logical group, the corresponding Rust module/file is specified, along with any recommended changes in structure or naming. This plan is intended to guide a full migration and ensure all major features are covered.

---

## 1. Python Files and Their Purposes

### A. `src/` Directory

| Python File                | Purpose                                                                                  |
|--------------------------- |-----------------------------------------------------------------------------------------|
| `__init__.py`              | Package initializer (may be empty or set up imports)                                    |
| `config.py`                | Application configuration and settings management                                       |
| `database.py`              | Database connection, session management, and helpers                                    |
| `main.py`                  | Main application entry point (likely starts the web server or CLI)                      |
| `models.py`                | ORM models and data structures                                                          |
| `templating.py`            | Template rendering logic                                                                |
| `utils/`                   | Utility functions (helpers, common logic)                                               |
| `readers/`                 | File readers/parsers for embroidery formats                                             |
| `routes/`                  | Web/API route handlers                                                                  |
| `services/`                | Business logic and service layer                                                        |
| `models_db.rs`             | (Rust) Database models (already present, may be partial or experimental)                |
| `models.rs`                | (Rust) Data models (already present, may be partial or experimental)                    |
| `main.rs`                  | (Rust) Main entry point (already present, may be partial or experimental)               |
| `png_writer.rs`            | (Rust) PNG image writing (already present, may be partial or experimental)              |
| `schema.rs`                | (Rust) Database schema (already present, may be partial or experimental)                |

### B. `py_source/` Directory

- Not present in the current structure. If it exists, repeat the process for any files found there.

### C. `tests/` Directory

| Python File                        | Purpose                                                      |
|-------------------------------------|--------------------------------------------------------------|
| `__init__.py`                       | Test package initializer                                     |
| `conftest.py`                       | Pytest fixtures and test configuration                       |
| `test_bulk_import_extra.py`         | Tests for bulk import features                               |
| `test_database.py`                  | Database-related tests                                       |
| `test_desktop_launcher.py`          | Desktop launcher tests                                       |
| `test_folder_picker_and_tagging.py` | Folder picker and tagging tests                              |
| `test_gemini_client.py`             | Gemini client integration tests                              |
| `test_legacy_tagging_actions.py`    | Legacy tagging logic tests                                   |
| `test_portable_launcher.py`         | Portable launcher tests                                      |
| `test_portable_scripts.py`          | Portable script tests                                        |
| `test_regression_e2e.py`            | End-to-end regression tests                                  |
| `test_root_scripts.py`              | Root script tests                                            |
| `test_routes.py`                    | Route handler tests                                          |
| `test_services.py`                  | Service layer tests                                          |
| `test_stitch_identifier.py`         | Stitch identifier logic tests                                |
| `test_unified_backfill.py`          | Unified backfill logic tests                                 |
| `testdata/`                         | Test data files                                              |

---

## 2. Proposed Rust Module Structure

The Rust module structure should be idiomatic, modular, and reflect the logical separation of concerns. The following is a proposed `src/` tree for Rust:

```
src/
    main.rs                // Application entry point
    config.rs              // Configuration and settings
    database/
        mod.rs             // Database module root
        connection.rs      // DB connection/session
        models.rs          // ORM/data models
        schema.rs          // DB schema (Diesel or equivalent)
        migrations.rs      // Migration helpers (if needed)
    models/
        mod.rs             // Data models (domain objects)
    services/
        mod.rs             // Service layer root
        import.rs          // Import logic
        tagging.rs         // Tagging logic
        backfill.rs        // Backfill logic
        portable.rs        // Portable/desktop launcher logic
        gemini_client.rs   // Gemini client integration
        stitch_identifier.rs // Stitch identifier logic
        ...                // Other business logic
    routes/
        mod.rs             // Route handler root
        api.rs             // API endpoints
        admin.rs           // Admin endpoints
        import.rs          // Import endpoints
        designs.rs         // Design endpoints
        ...                // Other route groups
    readers/
        mod.rs             // File readers/parsers
        ...                // Format-specific readers
    templating.rs          // Template rendering
    utils.rs               // Utility functions
    png_writer.rs          // PNG/image writing
    disclaimer.rs          // Disclaimer logic
    settings.rs            // Settings management
    tests/
        mod.rs             // Test module root
        ...                // Test files mirroring main modules
```

---

## 3. Mapping Python Files/Groups to Rust Modules

| Python File/Group                | Rust Module/File                | Notes/Changes                                                                                  |
|----------------------------------|---------------------------------|-----------------------------------------------------------------------------------------------|
| `src/__init__.py`                | _none_                          | Not needed in Rust                                                                            |
| `src/config.py`                  | `src/config.rs`                 | Direct mapping; use Rust structs for config                                                   |
| `src/database.py`                | `src/database/connection.rs`    | Split DB logic into submodules: connection, models, schema                                    |
| `src/models.py`                  | `src/database/models.rs` or `src/models/mod.rs` | If models are DB-specific, keep in `database/`; otherwise, use `models/` for domain objects   |
| `src/templating.py`              | `src/templating.rs`             | Direct mapping; use Tera, Askama, or similar Rust templating                                  |
| `src/utils/`                     | `src/utils.rs` or `src/utils/`  | If many utilities, use a folder; otherwise, a single file                                     |
| `src/readers/`                   | `src/readers/`                  | Direct mapping; each reader/parser as its own file                                            |
| `src/routes/`                    | `src/routes/`                   | Direct mapping; group by endpoint type (api, admin, import, etc.)                             |
| `src/services/`                  | `src/services/`                 | Direct mapping; each business logic area as its own file                                      |
| `src/main.py`                    | `src/main.rs`                   | Main entry point; may need to refactor for Rust idioms                                        |
| `src/png_writer.rs`              | `src/png_writer.rs`             | Already present; ensure it matches Rust conventions                                           |
| `src/models_db.rs`               | `src/database/models.rs`        | Consolidate with other models if appropriate                                                  |
| `src/models.rs`                  | `src/models/mod.rs`             | Use as domain models if not DB-specific                                                       |
| `src/schema.rs`                  | `src/database/schema.rs`        | Already present; ensure it matches Diesel or chosen ORM                                       |
| `src/disclaimer.py` (if exists)  | `src/disclaimer.rs`             | New file for disclaimer logic                                                                 |
| `src/settings.py` (if exists)    | `src/settings.rs`               | New file for settings management                                                              |
| `tests/`                         | `tests/`                        | Mirror Rust test files to main modules; use Rust's test framework                             |

---

## 4. Grouping and Refactoring Recommendations

- **Business Logic & Services:**  
  Group related business logic into the `services/` module. For example, all tagging logic (`tagging.py`, legacy tagging, etc.) should be in `services/tagging.rs`. Import logic, backfill, and portable/desktop logic should each have their own files.
- **Routes:**  
  Organize route handlers by endpoint type. For example, admin routes in `routes/admin.rs`, import routes in `routes/import.rs`, etc. Use submodules for large route groups.
- **Database:**  
  Separate database connection/session management, models, and schema into their own files under `database/`.
- **Models:**  
  If there are domain models not tied to the database, place them in `models/`.
- **Utilities:**  
  Consolidate small utility functions into `utils.rs`; if there are many, use a `utils/` folder with multiple files.
- **Templating:**  
  Use a single `templating.rs` unless there is a need for multiple template engines or complex logic.
- **Testing:**  
  Mirror the main module structure in the `tests/` directory. Use Rust's built-in test framework and place integration tests in `tests/`, unit tests alongside modules.

---

## 5. Major App Features and Their Mapping

| Feature Area         | Python Location(s)         | Rust Module(s)                |
|--------------------- |---------------------------|-------------------------------|
| Settings            | `config.py`, `settings.py` | `config.rs`, `settings.rs`    |
| Disclaimer          | `disclaimer.py`            | `disclaimer.rs`               |
| Import              | `services/import.py`, `routes/import.py`, `readers/` | `services/import.rs`, `routes/import.rs`, `readers/` |
| Tagging             | `services/tagging.py`, `test_legacy_tagging_actions.py` | `services/tagging.rs`         |
| Database            | `database.py`, `models.py`, `schema.py` | `database/connection.rs`, `database/models.rs`, `database/schema.rs` |
| Services            | `services/`                | `services/`                   |
| Routes              | `routes/`                  | `routes/`                     |
| Utilities           | `utils/`                   | `utils.rs` or `utils/`        |
| Templating          | `templating.py`            | `templating.rs`               |
| PNG/Image Writing   | `png_writer.rs`            | `png_writer.rs`               |
| Portable/Desktop    | `test_portable_launcher.py`, `test_portable_scripts.py` | `services/portable.rs`        |
| Gemini Client       | `test_gemini_client.py`    | `services/gemini_client.rs`   |
| Stitch Identifier   | `test_stitch_identifier.py`| `services/stitch_identifier.rs`|
| Backfill            | `test_unified_backfill.py` | `services/backfill.rs`        |

---

## 6. Example Rust Module Tree (Summary)

```
src/
    main.rs
    config.rs
    settings.rs
    disclaimer.rs
    templating.rs
    png_writer.rs
    utils.rs
    database/
        mod.rs
        connection.rs
        models.rs
        schema.rs
        migrations.rs
    models/
        mod.rs
    services/
        mod.rs
        import.rs
        tagging.rs
        backfill.rs
        portable.rs
        gemini_client.rs
        stitch_identifier.rs
    routes/
        mod.rs
        api.rs
        admin.rs
        import.rs
        designs.rs
    readers/
        mod.rs
        ... (format-specific readers)
tests/
    ... (mirroring main modules)
```

---

## 7. Additional Notes

- **Naming:** Use snake_case for Rust files and modules.
- **Refactoring:** Where Python logic is split across multiple files but tightly coupled, consider consolidating in Rust for clarity and maintainability.
- **Documentation:** Each Rust module should include doc comments summarizing its purpose and usage.
- **Testing:** Place unit tests in the same file as the module (using `#[cfg(test)]`), and integration tests in the `tests/` directory.

---

## 8. Migration Steps

1. **Inventory** all Python files and confirm their current responsibilities.
2. **Create** the Rust module tree as outlined above.
3. **Map** each Python file/group to its Rust equivalent, refactoring as needed.
4. **Document** any changes in logic, grouping, or naming.
5. **Implement** modules incrementally, starting with core infrastructure (config, database, models).
6. **Test** each module as it is ported, using Rust's test framework.
7. **Update** this mapping plan as the migration progresses.

---

This plan should serve as a comprehensive guide for migrating the Python codebase to a well-structured, idiomatic Rust project.
