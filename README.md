# Rust-Embroidery-Catalogue

## Licence
This program is free software: you can redistribute it and/or modify it under the terms of the MIT License. See the LICENCE file for full details.

# Developer Environment Setup & Contributing Guide

Welcome to the **Embroidery Catalogue** repository. This document outlines the prerequisites, editor setup, toolchains, and tasks required to set up your local development environment for building and maintaining the application.

---

## Workspace Tech Stack

* **Desktop Shell:** Tauri v2
* **Backend:** Rust (pinned toolchain, SQLite metadata storage, binary embroidery file parsing)
* **Frontend:** Svelte / TypeScript
* **UI Structure:** Core application flows managed via main view modules like @App.svelte, @MainView.svelte, @DesignDetailView.svelte, @ImportView.svelte, and test suites like @ImportTestHarness.svelte.

---

## 1. Prerequisites & System Dependencies

Before opening the workspace, ensure your system has the base language runtimes installed.

### Rust Toolchain

* **Installer:** Install via [rustup.rs](https://rustup.rs/).
* **Version:** Pinned via `rust-toolchain.toml` (`1.80.0-x86_64-pc-windows-msvc`).
* Required system target and compiler components are managed automatically upon running `cargo` or opening the repository.

### Node.js & Package Manager

* **Node.js:** LTS version installed.
* **Package Manager:** `pnpm` (recommended) or `npm`.

### Tauri v2 System Prerequisites

* **Windows:** C++ build tools via Visual Studio Community / Build Tools (MSVC workload).
* **macOS / Linux:** Native C compilers and WebKit libraries (see official Tauri v2 prerequisites documentation if developing cross-platform).

---

## 2. Mandatory Rust Components & Global Tools

Run the following commands in your terminal to ensure your local Rust environment has all required CLI tooling and components attached to the toolchain:

### Rustup Components

```bash
rustup component add rust-analyzer clippy rustfmt

```

* `rust-analyzer`: Powers the language server for inline type checking and diagnostics.
* `clippy`: Runs static code analysis (catches memory inefficiencies when processing `.pes`, `.jef`, or other binary stitch buffers).
* `rustfmt`: Formats all Rust code on save according to standard style guidelines.

### Global Cargo Subcommands

```bash
cargo install tauri-cli --version "^2.0.0"

```

---

## 3. Recommended VS Code Setup

If developing inside Visual Studio Code (or compatible IDEs like Cursor/Antigravity), workspace recommendations are configured in `.vscode/extensions.json`.

### Automatic Setup

Upon opening the project workspace folder in VS Code, accept the prompt to **"Install Recommended Extensions"**, or open the Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`) and select **Extensions: Show Recommended Extensions**.

### Workspace Extension List

| Extension ID | Name / Description | Purpose |
| --- | --- | --- |
| `rust-lang.rust-analyzer` | rust-analyzer | Rust language server & inline CodeLens runner |
| `svelte.svelte-vscode` | Svelte for VS Code | Svelte component support (@App.svelte, @MainView.svelte) |
| `tauri-apps.tauri-vscode` | Tauri | Tauri v2 config validation & command tooling |
| `tamasfe.even-better-toml` | Even Better TOML | Schema validation for `Cargo.toml` and `tauri.conf.json` |
| `vadimcn.vscode-lldb` | CodeLLDB | Native debugging for Rust backend and IPC handlers |
| `esbenp.prettier-vscode` | Prettier - Code formatter | Formatting TypeScript, Svelte templates, and CSS |
| `alexcvzz.vscode-sqlite` | SQLite | Direct inspection of local catalog metadata databases |
| `vitest.explorer` | Vitest Explorer | Discovers and executes Svelte unit test suites in the Beaker panel |
| `swellaby.vscode-rust-test-adapter` | Rust Test Explorer | Integrates Rust `#[test]` suites into the VS Code Testing panel |

---

## 4. Environment Verification & Initial Run

Once all dependencies and extensions are installed, verify your environment setup:

1. **Verify Rust Components:**
```bash
rustup component list --installed

```

*Confirm `clippy`, `rustfmt`, and `rust-analyzer` are listed.*

2. **Verify Tauri CLI Version:**
```bash
cargo tauri --version

```

3. **Install Frontend Dependencies:**
```bash
pnpm install

```

4. **Launch Local Development Environment:**
```bash
cargo tauri dev

```

5. **Run Test Suites:**
* **Rust Unit Tests (Binary Parsers & Database IPC):** `cargo test`
* **Svelte Component Tests:** `pnpm test` (or execute via the VS Code Testing Panel)

## Reader Requirements

All embroidery file readers (DST, PES, JEF, VP3, EXP, etc.) **must** provide enough data for the PNG renderer to generate a preview image. This means:

- The `stitches` vector in `EmbPattern` must contain all stitch positions and commands.
- The `threadlist` must contain at least one thread (with color) for each color block.

Readers are not required to provide metadata beyond what is needed for rendering.

This contract ensures that any supported file can be previewed visually in the catalogue.

---

For more details, see the doc comment on the `EmbroideryReader` trait in `src/readers/embroidery_reader.rs`.








