# Architecture Decision Records (ADRs)

This directory documents key architectural decisions, invariants, and trade-offs for **Embroidery Catalogue**.

---

## Index of Records

| ADR | Title | Status | Date |
| :--- | :--- | :--- | :--- |
| [001](001-immutable-original-files.md) | **Immutable Original Files & Non-Destructive Cataloguing** | Accepted | 2026-10-06 |
| [002](002-config-json-data-root.md) | **Bootstrap Config Persistence for Data Root** | Accepted | 2026-10-06 |
| [003](003-tagging-isolation.md) | **Strict Tag Category & Verification Isolation** | Accepted | 2026-10-06 |
| [004](004-recovery-mode-gating.md) | **Recovery Mode Side-Effect Gating** | Accepted | 2026-10-06 |
| [005](005-sqlite-wal-connection-pool.md) | **SQLite WAL Mode and Multi-Connection Pool** | Accepted | 2026-10-06 |
| [006](006-master-design-stitch-pairing.md) | **Master Design & Stitch File Pairing and Lifecycle** | Accepted | 2026-10-07 |
| [007](007-two-way-library-drift-sync.md) | **Two-Way Library Drift Synchronization** | Accepted | 2026-10-07 |
| [008](008-zero-static-process-globals.md) | **Zero Static Process Globals and Managed State Lifecycle** | Accepted | 2026-10-07 |
| [009](009-typed-ipc-boundary-and-wire-protocol.md) | **Typed IPC Boundary, CamelCase Wire Protocol, and Structured Error Model** | Accepted | 2026-10-07 |
| [010](010-in-app-modal-dialog-system.md) | **In-App Svelte Modal System for User Confirmations and Alerts** | Accepted | 2026-10-07 |

---

## ADR Template Structure

Each Architecture Decision Record follows this lightweight format:
* **Context and Problem Statement:** Description of the problem and why an architectural decision was necessary.
* **Decision Drivers:** Constraints, requirements, and quality attributes (performance, safety, user experience).
* **Considered Options:** Alternatives evaluated.
* **Decision Outcome:** Chosen approach and core architectural invariants to uphold.
