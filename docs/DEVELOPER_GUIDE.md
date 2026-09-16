# Developer Guide & How-To Recipes

This guide contains step-by-step developer recipes for the most common extension workflows in the **Rust Embroidery Catalogue**.

---

## Table of Contents

1. [Recipe 1: How to Add a New Embroidery Format Reader](#recipe-1-how-to-add-a-new-embroidery-format-reader)
2. [Recipe 2: How to Add a New Tauri IPC Command](#recipe-2-how-to-add-a-new-tauri-ipc-command)
3. [Recipe 3: How to Add a Database Migration](#recipe-3-how-to-add-a-database-migration)
4. [General Debugging & Tips](#general-debugging--tips)

---

## Recipe 1: How to Add a New Embroidery Format Reader

Embroidery formats are parsed into the unified `EmbPattern` struct via implementations of the `EmbroideryReader` trait.

### Step 1: Create Reader Implementation

Create `src/readers/<format>_reader.rs` (e.g. `src/readers/xxx_reader.rs`):

```rust
use crate::models::{EmbPattern, StitchType, Stitch};
use crate::readers::EmbroideryReader;
use std::io::{Cursor, Read, Seek};

pub struct XxxReader;

impl EmbroideryReader for XxxReader {
    fn read<R: Read + Seek>(&self, reader: &mut R) -> Result<EmbPattern, String> {
        let mut pattern = EmbPattern::new();
        // 1. Read binary header (dimensions, thread count, stitch count)
        // 2. Read thread color palette into pattern.threadlist
        // 3. Read stitch coordinate records into pattern.stitches
        // 4. Coordinates must be converted to 0.1 mm integer units
        Ok(pattern)
    }

    fn supported_extensions(&self) -> &'static [&'static str] {
        &["xxx"]
    }
}
```

### Step 2: Register Reader in Readers Module

Update `src/readers/mod.rs`:

```rust
pub mod xxx_reader;
pub use crate::readers::xxx_reader::XxxReader;
```

### Step 3: Register in Stitch Identifier Dispatcher

Update `src/services/stitch_identifier.rs` to register the new format reader in the extension matcher and header sniffer.

### Step 4: Add Unit Tests & Sample Test Fixture

1. Place a minimal, public-domain `.xxx` test file in `tests/Test Assets/xxx/sample.xxx`.
2. Create `src/readers/xxx_reader_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use crate::readers::{EmbroideryReader, XxxReader};
    use std::fs::File;

    #[test]
    fn test_reads_sample_xxx() {
        let mut file = File::open("tests/Test Assets/xxx/sample.xxx").expect("test file missing");
        let reader = XxxReader;
        let pattern = reader.read(&mut file).expect("parsing failed");
        assert!(!pattern.stitches.is_empty());
        assert!(!pattern.threadlist.is_empty());
    }
}
```

---

## Recipe 2: How to Add a New Tauri IPC Command

Tauri commands bridge the frontend Svelte application with backend Rust services.

### Step 1: Implement Core Logic in Services Layer

Create or update domain logic in `src/services/<domain>.rs` (e.g., `src/services/projects.rs`):

```rust
use crate::error::AppError;
use sqlx::SqlitePool;

pub async fn duplicate_project(pool: &SqlitePool, project_id: i64) -> Result<i64, AppError> {
    // Perform database operations / business logic
    Ok(new_project_id)
}
```

### Step 2: Create IPC Command Handler

Create or update the route in `src/routes/<domain>.rs`:

```rust
use crate::error::AppError;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn duplicate_project(
    state: State<'_, AppState>,
    project_id: i64,
) -> Result<i64, AppError> {
    let pool = state.db_pool()?;
    crate::services::projects::duplicate_project(&pool, project_id).await
}
```

### Step 3: Register Command in `src/main.rs`

Add the command to the `tauri::generate_handler![...]` macro invocation in `src/main.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    // existing handlers...
    routes::projects::duplicate_project,
])
```

### Step 4: Add Typed Frontend Adapter

In `frontend/src/lib/api/projectsAdapter.ts`:

```typescript
import { invokeLoose } from "./ipcClient";

export async function duplicateProject(projectId: number): Promise<number> {
  return invokeLoose<number>("duplicate_project", { projectId });
}
```

---

## Recipe 3: How to Add a Database Migration

The catalogue uses `sqlx` migrations embedded into the binary.

### Step 1: Create Migration SQL File

Create `migrations/<timestamp>_<migration_name>.sql` (e.g. `migrations/20260917000000_add_project_favorites.sql`):

```sql
-- Add column or new table
ALTER TABLE projects ADD COLUMN is_favorite INTEGER NOT NULL DEFAULT 0;
```

### Step 2: Update Rust Database Models

Update `src/database/models.rs` and `src/models/mod.rs` to reflect the schema modification.

### Step 3: Update and Run Migration Tests

Run `cargo test database::migrations` to verify that migrations execute cleanly on empty databases and upgrade existing schemas without data loss.

---

## General Debugging & Tips

- **Live Frontend Dev:** Run `cargo tauri dev` for simultaneous backend compilation and Vite hot-reloading.
- **Inspect SQLite Database:** Connect to your test database at `%APPDATA%/Rust-Embroidery-Catalogue/catalogue.db` using the VS Code SQLite extension or DB Browser for SQLite.
- **Log Files:** Check application logs generated in `%LOCALAPPDATA%/Rust-Embroidery-Catalogue/logs/`.
- **E2E Mock Mode:** Test frontend screens without launching Tauri by supplying stubs via `window.__E2E_IPC_STUBS__`.
