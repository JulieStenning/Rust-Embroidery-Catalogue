// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

fn main() {
    // sqlx::migrate! embeds migrations at compile time.
    // Track the migrations directory so new files/folders trigger recompilation.
    println!("cargo:rerun-if-changed=migrations");
    tauri_build::build()
}
