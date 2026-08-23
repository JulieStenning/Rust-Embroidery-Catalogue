# Create an output directory for release audit logs
New-Item -ItemType Directory -Force -Path "./audit-logs" | Out-Null
Write-Host "Starting Pre-Release Automated Checks..." -ForegroundColor Cyan

# 1. License & Security Audits
Write-Host "--> Checking Rust SPDX License Compliance..." -ForegroundColor Yellow
cargo about generate about.hbs -o ./audit-logs/licences-preview.html

Write-Host "--> Checking Frontend NPM License Compliance..." -ForegroundColor Yellow
npx license-checker-rseidelsohn --start ./frontend --excludePackages "embroidery-catalogue-frontend" --onlyAllow "MIT;Apache-2.0;BSD-2-Clause;BSD-3-Clause;ISC;CC0-1.0;Zlib;MPL-2.0;Python-2.0;BlueOak-1.0.0" 2>&1 | Out-File ./audit-logs/npm-license-check.txt

Write-Host "--> Checking Cargo Deny Configuration..." -ForegroundColor Yellow
cargo deny check > ./audit-logs/cargo-deny-results.txt

Write-Host "--> Running Cargo Audit..." -ForegroundColor Yellow
cargo audit > ./audit-logs/cargo-audit-results.txt

Write-Host "--> Checking Cargo bans..." -ForegroundColor Yellow
cargo deny check bans > ./audit-logs/cargo-deny-bans.txt

Write-Host "--> Checking Duplicate Dependencies..." -ForegroundColor Yellow
cargo tree --duplicates > ./audit-logs/duplicates.txt

Write-Host "--> Checking Cargo Deny Licenses..." -ForegroundColor Yellow
cargo deny check licenses > ./audit-logs/cargo-deny-licenses.txt

Write-Host "--> Checking Cargo Deny Sources..." -ForegroundColor Yellow
cargo deny check sources > ./audit-logs/cargo-deny-sources.txt

Write-Host "--> Running NPM Audit..." -ForegroundColor Yellow
npm audit --prefix frontend 2>&1 | Out-File ./audit-logs/npm-audit-results.txt

Write-Host "--> Updating Cargo Dependencies..." -ForegroundColor Yellow
cargo update > ./audit-logs/cargo-update-results.txt

Write-Host "--> Checking back end tests pass" -ForegroundColor Yellow
cargo test  > ./audit-logs/cargo-test-results.txt

Write-Host "--> Running Cargo Check..." -ForegroundColor Yellow
cargo check 2>&1 | Out-File ./audit-logs/cargo-check-results.txt

Write-Host "--> Checking Outdated Crates..." -ForegroundColor Yellow
cargo outdated > ./audit-logs/outdated.txt

Write-Host "--> Checking back end tests pass" -ForegroundColor Yellow
cargo test  > ./audit-logs/cargo-test-results2.txt

Write-Host "--> Checking frontend tests pass" -ForegroundColor Yellow
npx vitest run --silent  2>&1 | Out-File ./audit-logs/vitest-results.txt

# 2. Rust Quality Gates
Write-Host "--> Running Rust Checks, Formatting & Clippy..." -ForegroundColor Yellow
$env:CARGO_TERM_COLOR="never"; cargo check 2>&1 | Out-File ./audit-logs/cargo-check-results2.txt
$env:CARGO_TERM_COLOR="never"; cargo clippy --all-targets -- -D warnings 2>&1 | Out-File ./audit-logs/cargo-clippy-results.txt
$env:CARGO_TERM_COLOR="never"; cargo fmt --check -- -v 2>&1 | Out-File ./audit-logs/rustfmt-results.txt


# 3. Frontend Quality Gates
Write-Host "--> Running Frontend Lint, Format & Type Checks..." -ForegroundColor Yellow
npx svelte-check --tsconfig frontend/jsconfig.json 2>&1 | Out-File ./audit-logs/svelte-check.txt
Set-Location frontend; $env:FORCE_COLOR=0; npm run lint 2>&1 | Out-File ../audit-logs/eslint-results.txt; Set-Location ..
Write-Host "--> Running Prettier Format Check and updating ugly files..." -ForegroundColor Yellow
npx prettier --write frontend/src | Out-Null
npx prettier --check frontend/src 2>&1 | Out-File ./audit-logs/format-prettier-results.txt

# 4. License Asset Generation & Build
Write-Host "--> Generating License Assets..." -ForegroundColor Yellow
Set-Location frontend; $env:FORCE_COLOR=0; npm run generate:licences 2>&1 | Out-File ../audit-logs/license-assets.txt; Set-Location ..
Write-Host "--> Executing Release Build. This will take time ..." -ForegroundColor Yellow
$env:RUST_APP_NO_PAUSE="1"
$ErrorActionPreference = "SilentlyContinue"
./build-rust-release.bat 2>&1 | Tee-Object -FilePath ./audit-logs/build-results.txt
$ErrorActionPreference = "Continue"

# 5. Checksum Generation
Write-Host "--> Computing Installer SHA-256 Checksums..." -ForegroundColor Yellow
Get-ChildItem -Path "target/release/bundle" -Recurse -File -Include *.exe, *.msi | Get-FileHash -Algorithm SHA256 | Out-File ./audit-logs/checksums.txt

Write-Host "`nAll checks complete! Audit files saved to ./audit-logs/" -ForegroundColor Green
Get-Content ./audit-logs/checksums.txt

# 6. Execute Release Audit Verification
Write-Host "--> Verifying Release Audit Logs..." -ForegroundColor Yellow
./verify-release-logs.ps1


