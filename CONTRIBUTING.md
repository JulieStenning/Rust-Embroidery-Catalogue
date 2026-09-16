# Contributing to Rust Embroidery Catalogue

Thank you for your interest in contributing to the **Embroidery Catalogue** project! We welcome bug reports, documentation enhancements, feature proposals, and pull requests.

---

## Code of Conduct

Please help us maintain a friendly, welcoming, and collaborative environment. Be respectful, constructive, and supportive in discussions and code reviews.

---

## Development Workflow

### 1. Issue First (for Non-Trivial Changes)

Before undertaking large architectural changes or new feature developments, please open an Issue to discuss the design with the maintainers. For bug fixes and documentation improvements, feel free to proceed directly to a Pull Request.

### 2. Branching Strategy

- Branch from `main`.
- Use descriptive branch names with a category prefix:
  - `feat/feature-name` (e.g. `feat/xxx-reader`)
  - `fix/bug-description` (e.g. `fix/dst-trim-detection`)
  - `docs/doc-update` (e.g. `docs/developer-guide`)
  - `refactor/subsystem-name` (e.g. `refactor/import-state`)

### 3. Commit Guidelines

- Write clear, concise commit messages in imperative mood:
  - Good: `Add Bernina ART reader stub and format detector`
  - Bad: `updated stuff`
- Reference relevant issue numbers when applicable (e.g. `Fixes #42`).

---

## Coding Standards

### Rust Backend

- **Formatting:** Code must be formatted using `cargo fmt`.
- **Linting:** Code must pass `cargo clippy --all-targets -- -D warnings` without warnings.
- **Documentation:** Public structs, enums, traits, functions, and modules must include descriptive doc comments (`///` and `//!`).
- **Error Handling:** Use [`AppError`](src/error.rs) for fallible operations rather than unhandled panics (`unwrap()`, `expect()`).
- **Tests:** All new features and readers must include unit tests. Place binary reader fixtures in `tests/Test Assets/`.

### Frontend (Svelte 5 & TypeScript)

- **Formatting:** Code must be formatted with `prettier` (`npx prettier --write .`).
- **Type Safety:** Ensure all adapter functions and store mutations have explicit TypeScript types.
- **Store Reset:** Every new store must export a `reset()` or `clear()` function for test isolation.
- **Mock Support:** Any new IPC command adapter in `frontend/src/lib/api/` should support E2E mock interception (`ipcClient.ts`).

---

## Pre-Submission Quality Gate Checklist

Before opening a Pull Request, ensure that all local checks pass:

```bash
# 1. Rust formatting & linter
cargo fmt --check
cargo clippy --all-targets -- -D warnings

# 2. Rust documentation
cargo doc --no-deps

# 3. Rust unit tests
cargo test

# 4. Frontend unit & store tests
npm test

# 5. Frontend formatting
npx prettier --check .
```

---

## Submitting a Pull Request

1. Push your branch to your fork.
2. Open a Pull Request targeting `main`.
3. Fill out the PR description with:
   - What changed and why.
   - Any breaking changes or database migration requirements.
   - Verification steps performed.
4. Maintainers will review your PR and provide feedback.
