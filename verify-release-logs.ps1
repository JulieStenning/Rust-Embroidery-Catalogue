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

    # Checking the second file from cargo check because that is the most recent.
    Test-LogCondition "Rust Check" "$logDir/cargo-check-results2.txt" `
        { param($c) $c -match 'Finished `dev`' -and $c -notmatch 'error\[E' } `
        "Fix backend type check or borrow errors."

    Test-LogCondition "Rust Clippy" "$logDir/cargo-clippy-results.txt" `
        { param($c) $c -match 'Finished `dev`' -and $c -notmatch 'error:' } `
        "Fix compiler warnings or clippy lints."

    # Checking the second test file because that it is most recent check.
    Test-LogCondition "Rust Tests" "$logDir/cargo-test-results2.txt" `
    { param($c) $c -match 'test result: ok\.' -and $c -match '0 failed;' } `
    "Fix failing backend unit tests."

    Test-LogCondition "Prettier Results" "$logDir/format-prettier-results.txt" `
        { param($c) $c -match 'All matched files use Prettier code style!' } `
        "Run 'npm --prefix frontend run format' to format files."
        
    Test-LogCondition "Rust Formatting" "$logDir/rustfmt-results.txt" `
        { param($c) $c -notmatch 'Diff in' } `
        "Run 'cargo fmt' to format backend code."

    Test-LogCondition "Frontend Unit Tests" "$logDir/vitest-results.txt" `
    { 
        param($c) 
        $clean = $c -replace '\x1b\[[0-9;]*[a-zA-Z]', ''
        $clean -match 'Test Files\s+\d+\s+passed' -and $clean -match 'Tests\s+\d+\s+passed' -and $clean -notmatch 'failed'
    } `
    "Fix failing frontend Vitest unit tests."

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
