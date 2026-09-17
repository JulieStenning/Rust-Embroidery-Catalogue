# Security Policy

The **Rust Embroidery Catalogue** team takes security vulnerabilities seriously. We appreciate the efforts of security researchers and the community in helping us keep our software and users safe.

---

## Supported Versions

We provide security updates for the following versions:

| Version                  | Supported          |
| ------------------------ | ------------------ |
| `0.1.x` (latest release) | :white_check_mark: |
| `< 0.1.0` (pre-release)  | :x:                |

---

## Reporting a Vulnerability

If you discover a potential security vulnerability, please do **NOT** open a public issue on GitHub. Instead, follow responsible disclosure practices:

1. **Email Report:** Send an email to the project maintainers with details of the vulnerability.
2. **Include Details:**
   - Detailed description of the vulnerability.
   - Steps or proof-of-concept to reproduce the issue.
   - Impact assessment (e.g., local arbitrary file read/write, denial of service, remote code execution).
   - Any suggested remediations or mitigations.

### Response Timeline

- **Initial Response:** Within 48 hours acknowledging receipt of the report.
- **Assessment & Fix:** We will investigate and provide regular status updates until a patch is developed and verified.
- **Disclosure:** Once a fix is released, we will publish an advisory and credit the reporter (unless you prefer to remain anonymous).

---

## Security Best Practices in this Codebase

- **Local-First Architecture:** The catalogue operates locally on the user's filesystem and does not expose open network ports.
- **Path Validation:** All file paths passed across Tauri IPC commands are validated to prevent directory traversal attacks.
- **Memory Safety:** Binary parsers in `src/readers/` are implemented in memory-safe Rust with bounds checks on all slice and byte buffer indexing.
- **Database Safety:** SQLite database queries use parameterized SQL via `sqlx` to prevent SQL injection vulnerabilities.
