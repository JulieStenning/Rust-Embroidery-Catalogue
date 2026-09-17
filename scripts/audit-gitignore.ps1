# =====================================================================
# audit-gitignore.ps1 — Audit .gitignore for stale, duplicate, or tracked entries
# =====================================================================

param (
    [string]$GitignorePath = ".gitignore",
    [string]$OutputFile = "./audit-logs/gitignore-audit.txt"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $GitignorePath)) {
    Write-Error ".gitignore file not found at: $GitignorePath"
    exit 1
}

# Ensure parent directory for output file exists
$outputDir = Split-Path -Parent $OutputFile
if ($outputDir -and -not (Test-Path $outputDir)) {
    New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
}

$lines = Get-Content $GitignorePath
$issues = [System.Collections.Generic.List[PSCustomObject]]::new()
$seenPatterns = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)

# Known secret/credential or OS-generated filenames that may legitimately not exist on all machines
$allowedMissingFiles = @(
    '.env',
    '.env.*',
    'Google Api Key.txt',
    'Deepseek Key.txt',
    '.DS_Store',
    'Thumbs.db'
)

# Get all currently tracked files in git for fast lookup
$trackedFiles = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)
try {
    $gitTracked = git ls-files
    foreach ($tf in $gitTracked) {
        if ($tf) {
            [void]$trackedFiles.Add($tf.Replace('\', '/').Trim())
        }
    }
} catch {
    Write-Warning "Could not execute 'git ls-files'. Tracked file check may be limited."
}

$lineNumber = 0
foreach ($rawLine in $lines) {
    $lineNumber++
    $pattern = $rawLine.Trim()

    # Skip empty lines and comments
    if (-not $pattern -or $pattern.StartsWith('#')) {
        continue
    }

    # 1. Duplicate Rule Check
    if ($seenPatterns.Contains($pattern)) {
        $issues.Add([PSCustomObject]@{
            LineNumber = $lineNumber
            Type       = "DUPLICATE"
            Pattern    = $pattern
            Message    = "Rule '$pattern' is defined more than once in .gitignore."
        })
    } else {
        [void]$seenPatterns.Add($pattern)
    }

    # Normalized path for checking
    $cleanPath = $pattern.TrimStart('/').Replace('\', '/')

    # 2. Tracked File Conflict Check
    # Check if this exact pattern is a tracked file or if git ls-files matches it
    if ($trackedFiles.Contains($cleanPath)) {
        $issues.Add([PSCustomObject]@{
            LineNumber = $lineNumber
            Type       = "TRACKED_FILE"
            Pattern    = $pattern
            Message    = "File '$cleanPath' is currently tracked by Git but also ignored in .gitignore."
        })
    }

    # 3. Stale / Non-Existent Explicit File Check
    # Only evaluate explicit paths (no wildcards *, ?, !, and not directory patterns ending with /)
    $hasWildcard = $pattern.Contains('*') -or $pattern.Contains('?') -or $pattern.StartsWith('!')
    $isDirectory = $pattern.EndsWith('/')
    $isAllowedMissing = $allowedMissingFiles -contains $pattern

    if (-not $hasWildcard -and -not $isDirectory -and -not $isAllowedMissing) {
        $exists = Test-Path $cleanPath
        if (-not $exists) {
            $issues.Add([PSCustomObject]@{
                LineNumber = $lineNumber
                Type       = "STALE_PATH"
                Pattern    = $pattern
                Message    = "Explicit file path '$pattern' does not exist on disk."
            })
        }
    }
}

# Build Report
$report = [System.Collections.Generic.List[string]]::new()
$report.Add("=======================================================")
$report.Add("             GITIGNORE PRE-RELEASE AUDIT               ")
$report.Add("=======================================================")
$report.Add("Audit Timestamp: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')")
$report.Add("Audited File: $GitignorePath")
$report.Add("Total Rules Audited: $($seenPatterns.Count)")
$report.Add("")

if ($issues.Count -eq 0) {
    $report.Add("STATUS: PASS")
    $report.Add("No stale entries, duplicates, or tracked-file conflicts found in $GitignorePath.")
} else {
    $report.Add("STATUS: ISSUES DETECTED")
    $report.Add("Total Issues Found: $($issues.Count)")
    $report.Add("")
    $report.Add("Detailed Findings:")
    $report.Add("-------------------------------------------------------")
    foreach ($issue in $issues) {
        $report.Add("ISSUE: [$($issue.Type)] Line $($issue.LineNumber): $($issue.Message)")
    }
    $report.Add("")
    $report.Add("Recommended Action:")
    $report.Add("Review and remove or fix the reported lines in $GitignorePath before releasing.")
}

$reportContent = $report -join "`r`n"
$reportContent | Out-File -FilePath $OutputFile -Encoding utf8

# Console Summary
if ($issues.Count -eq 0) {
    Write-Host "[OK] .gitignore audit passed cleanly ($($seenPatterns.Count) rules checked)." -ForegroundColor Green
} else {
    Write-Host "[WARN] .gitignore audit found $($issues.Count) issue(s):" -ForegroundColor Yellow
    foreach ($issue in $issues) {
        Write-Host "  - Line $($issue.LineNumber) [$($issue.Type)]: $($issue.Message)" -ForegroundColor Red
    }
}
