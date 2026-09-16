# Developer Documentation Portal

Welcome to the documentation portal for the **Rust Embroidery Catalogue**. This index organizes technical specifications, architecture designs, developer recipes, testing guides, and historical planning documents.

---

## 🚀 Getting Started & Contributing

- [**Developer Guide & How-To Recipes**](DEVELOPER_GUIDE.md): Step-by-step recipes for adding new embroidery format readers, creating Tauri IPC commands, and adding database migrations.
- [**Build & Run Guide**](<Help for Developers/Build and Run Guide.md>): Comprehensive instructions on toolchains, prerequisites, and build commands.
- [**Troubleshooting Guide**](TROUBLESHOOTING.md): Solutions for common installation, build, and runtime issues.
- [**Useful Terminal Commands**](<Help for Developers/Useful Terminal Commands.md>): Quick cheat sheet for Cargo, npm, and test scripts.
- [**Contributing Guidelines**](../CONTRIBUTING.md): Branching, commit standards, quality gates, and PR process.

---

## 🏛️ Architecture & Specifications

### Core Subsystems

- [**Stitch Identifier Architecture**](<Help for Developers/stitch-identifier-architecture.md>): Format detection, magic byte analysis, and reader routing.
- [**State Synchronization Architecture**](<Specs/State Synchronization Architecture.md>): Reactive frontend stores, cross-view synchronisation, and backend event channels.
- [**Import & Folder Assignment Spec**](Specs/import-folder-assignment-backend-spec.md): Import scanner, multi-folder library assignment, and duplicate resolution.
- [**UI Management Overview**](<Specs/Embroidery Design Management UI Overview.md>): High-level UI/UX layouts and interaction patterns.

### Frontend Guides

- [**Frontend API & IPC Adapter Guide**](../frontend/src/lib/api/README.md): Architecture of the IPC client, domain adapters, and E2E mock stub injection.
- [**Frontend Stores Guide**](../frontend/src/lib/stores/README.md): State management design, session stores, toast queue, and test reset patterns.

---

## 🧪 Testing & Quality Assurance

- [**E2E Testing Guide**](<Help for Developers/e2e testing guide.md>): Playwright test suite setup, fixtures, running headed/headless, and mock modes.
- [**Coverage Report Updates**](<Help for Developers/Coverage Report Update Prompts.md>): Measuring and reporting code coverage across Rust and frontend suites.
- [**Database Restore Unit Tests**](<Plans/Unit tests for restore database functionality.md>): Testing backup and recovery pipelines.

---

## 📋 Feature Specifications & Plans

- [**Bulk Deletion Plan**](<Plans/Bulk Deletion.md>): Design specification for bulk deletion with transaction safety.
- [**Long-Running Action Guards**](<Plans/Disabling menu and buttons during long running actions.md>): UI lockout and busy indicators for background operations.
- [**Database Compaction & Vacuuming**](Plans/chunk.md): Freelisting, page size tuning, and vacuum scheduling.
- [**Design Details Layout**](<Plans/design details layout refactor.md>): Full-screen and drawer design details view specifications.
- [**Star Rating System**](<Plans/Star Rating on Browse Designs.md>): User rating mechanics and database schema.
- [**Windows Installer & Update Delivery SOP**](Plans/windows-installer-update-delivery-sop.md): Release packaging and auto-update checklists.
