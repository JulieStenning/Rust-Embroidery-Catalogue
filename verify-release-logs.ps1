# =====================================================================
# verify-release-logs.ps1 — Automated Release Audit Parser
# =====================================================================

$logDir = "./audit-logs"

function Test-LogCondition {
    param (
        [string]$Name,
        [string]$FilePath,
        [scriptblock]$Condition,
        [string]$FixHint
    )

    if (-not (Test-Path $FilePath)) {
        return [PSCustomObject]@{
            Gate   = $Name
            Status = "MISSING"
            Detail = "Log file not found at $FilePath"
            Fix    = "Run the release check script to generate this log."
        }
    }

    $content = Get-Content $FilePath -Raw
    $passed = &$Condition $content

    if ($passed) {
        [PSCustomObject]@{
            Gate   = $Name
            Status = "OK"
            Detail = "Passed cleanly."
            Fix    = "-"
        }
    } else {
        [PSCustomObject]@{
            Gate   = $Name
            Status = "NEEDS WORK"
            Detail = "Failures or diffs detected in log."
            Fix    = $FixHint
        }
    }
}

$results = @(
    Test-LogCondition "Rust Check" "$logDir/cargo-check-results.txt" `
        { param($c) $c -match 'Finished `dev`' -and $c -notmatch 'error\[E' } `
        "Fix backend type check or borrow errors."

    Test-LogCondition "Rust Clippy" "$logDir/cargo-clippy-results.txt" `
        { param($c) $c -match 'Finished `dev`' -and $c -notmatch 'error:' } `
        "Fix compiler warnings or clippy lints."

    Test-LogCondition "Rust Tests" "$logDir/cargo-test-results.txt" `
        { param($c) $c -match 'test result: ok\.' -and $c -notmatch 'failed;' } `
        "Fix failing backend unit tests."

    Test-LogCondition "Rust Formatting" "$logDir/rustfmt-results.txt" `
        { param($c) $c -notmatch 'Diff in' } `
        "Run 'cargo fmt' to format backend code."

    Test-LogCondition "Frontend Vitest" "$logDir/vitest-results.txt" `
        { param($c) $c -match 'Test Files\s+\d+ passed' -and $c -notmatch 'FAIL' } `
        "Fix failing frontend unit tests."

    Test-LogCondition "Prettier Check" "$logDir/format-check-results.txt" `
        { param($c) $c -notmatch 'Code style issues found' } `
        "Run 'npm --prefix frontend run format' to auto-fix."

    Test-LogCondition "Svelte Type Check" "$logDir/svelte-check.txt" `
        { param($c) $c -match 'found 0 errors' -and $c -notmatch 'Error:' } `
        "Fix TypeScript/Svelte errors in @DesignDetailView.svelte."

    Test-LogCondition "Tauri Packaging" "$logDir/build-results.txt" `
        { param($c) $c -match 'Release build successful!' } `
        "Review compilation/bundling error trace in build log."
)

Write-Host "`n=======================================================" -ForegroundColor Cyan
Write-Host "             EMBROIDERY CATALOGUE RELEASE AUDIT        " -ForegroundColor Cyan
Write-Host "=======================================================`n" -ForegroundColor Cyan

$results | Format-Table -Property @(
    @{ Name = "Gate Name"; Expression = { $_.Gate }; Width = 22 },
    @{ Name = "Status"; Expression = { $_.Status }; Width = 14 },
    @{ Name = "Recommended Action / Fix"; Expression = { $_.Fix }; Width = 45 }
)

$hasIssues = $results | Where-Object { $_.Status -ne "OK" }
if ($hasIssues) {
    Write-Host "`n[FAIL] Release audit completed with issues. Fix items listed under 'NEEDS WORK'.`n" -ForegroundColor Red
} else {
    Write-Host "`n[SUCCESS] All release gates passed! Ready to publish release tags.`n" -ForegroundColor Green
}
