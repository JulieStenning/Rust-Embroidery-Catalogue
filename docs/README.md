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

### System Architecture & Specifications

- [**Desktop Architecture Guardrails**](Specs/desktop-only-architecture-guardrails.md): Core platform scope, local-first principles, and Rust IPC boundaries.
- [**AI Model Evaluation & Selection**](Specs/ai-model-evaluation.md): Comparative evaluation (September 2026) of Gemini Vision, Claude Sonnet, and Hugging Face.
- [**Stitch Identifier Architecture**](<Help for Developers/stitch-identifier-architecture.md>): Format detection, magic byte analysis, and reader routing.
- [**State Synchronization Architecture**](<Specs/State Synchronization Architecture.md>): Reactive frontend stores, cross-view synchronisation, and mutation patching.
- [**Import & Folder Assignment Spec**](Specs/import-folder-assignment-backend-spec.md): Import scanner, multi-folder library assignment, and duplicate resolution.
- [**Import Refactor Checklist**](Specs/import-folder-assignment-refactor-checklist.md): Invariant safety checklist for import parsing and assignment.

### UI & Styling Standards

- [**UI Global Standards**](Specs/ui-global-standards.md): Cross-page layout rules, card contract, responsive breakpoints, and accessibility.
- [**Look & Feel Implementation Spec**](Specs/look-and-feel-implementation-spec.md): Design tokens, typography, form styling, and component density.

### Frontend Guides

- [**Frontend API & IPC Adapter Guide**](../frontend/src/lib/api/README.md): Architecture of the IPC client, domain adapters, and E2E mock stub injection.
- [**Frontend Stores Guide**](../frontend/src/lib/stores/README.md): State management design, session stores, toast queue, and test reset patterns.

---

## 🧪 Testing & Quality Assurance

- [**E2E Testing Guide**](<Help for Developers/e2e testing guide.md>): Playwright test suite setup, fixtures, running headed/headless, and mock modes.
- [**Coverage Report Updates**](<Help for Developers/Coverage Report Update Prompts.md>): Measuring and reporting code coverage across Rust and frontend suites.
- [**Testing Strategy & Verification**](<User Test Plans/README.md>): Testing pyramid, Playwright feature matrix, and manual acceptance suites.

---

## 📦 Release Procedures & Runbooks

- [**Windows Installer & Update Delivery SOP**](<Help for Developers/windows-installer-update-delivery-sop.md>): Detailed operator steps for building, verifying, and delivering Windows installer updates.
- [**Windows Installer Update Quick Checklist**](<Help for Developers/windows-installer-update-quick-checklist.md>): One-page release-day verification checklist.
- [**Release Checklist & Quality Gates**](proformas/releases/release-checklist.md): Integrated pre-release auditing, licensing checks, and quality gates.
