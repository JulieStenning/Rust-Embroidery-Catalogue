# ---------------------------------------------------------------------------
# add-spdx-headers.ps1
#
# Prepends the project's SPDX licence header to every tracked source file, so
# the GPL-3.0-or-later licence of Embroidery Catalogue is machine-readable
# (REUSE-style) in every Rust, Svelte and TypeScript/JavaScript file.
#
#   // SPDX-FileCopyrightText: 2026 Julie Stenning
#   // SPDX-License-Identifier: GPL-3.0-or-later
#
# The script is IDEMPOTENT: files that already contain an
# `SPDX-License-Identifier` tag are left untouched, so it is safe to re-run
# after adding new source files.
#
# It preserves each file's existing line-ending style (CRLF or LF) and BOM
# state, and inserts the header *after* any `#!` shebang line.
#
# Usage (from the repo root):
#   pwsh ./scripts/add-spdx-headers.ps1
# ---------------------------------------------------------------------------
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
Push-Location $repoRoot

try {
    $copyrightLine = "SPDX-FileCopyrightText: 2026 Julie Stenning"
    $identifierLine = "SPDX-License-Identifier: GPL-3.0-or-later"

    # Scope: Rust, Svelte and TypeScript/JavaScript sources (including tests,
    # config files and helper scripts).
    $patterns = @("*.rs", "*.svelte", "*.ts", "*.js", "*.mts", "*.mjs")

    $files = @(git ls-files -- $patterns)

    $added = 0
    $alreadyTagged = 0
    $missing = 0

    foreach ($rel in $files) {
        $path = Join-Path $repoRoot $rel
        if (-not (Test-Path -LiteralPath $path)) {
            Write-Warning "[spdx] tracked but missing on disk: $rel"
            $missing += 1
            continue
        }

        $bytes = [System.IO.File]::ReadAllBytes($path)
        $hasBom = ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and
            $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF)
        $text = [System.IO.File]::ReadAllText($path)

        if ($text.Contains("SPDX-License-Identifier")) {
            $alreadyTagged += 1
            continue
        }

        # Preserve the file's dominant line-ending style.
        $nl = if ($text.Contains("`r`n")) { "`r`n" } else { "`n" }

        if ($rel -like "*.svelte") {
            $line1 = "<!-- $copyrightLine -->"
            $line2 = "<!-- $identifierLine -->"
        }
        else {
            $line1 = "// $copyrightLine"
            $line2 = "// $identifierLine"
        }

        # Keep a leading shebang as the very first line. A real shebang is
        # `#!<interpreter>`; a Rust inner attribute is `#![...]` and must NOT be
        # treated as one (otherwise the header lands after the attribute).
        $prefix = ""
        $body = $text
        $firstNl = $text.IndexOf("`n")
        if ($firstNl -ge 0) {
            $firstLine = $text.Substring(0, $firstNl).TrimEnd("`r")
            if ($firstLine.StartsWith("#!") -and -not $firstLine.StartsWith("#![")) {
                $prefix = $text.Substring(0, $firstNl + 1)
                $body = $text.Substring($firstNl + 1)
            }
        }

        $header = $line1 + $nl + $line2 + $nl
        # Avoid a doubled blank line when the file already starts with one.
        if (-not ($body.StartsWith("`r`n") -or $body.StartsWith("`n"))) {
            $header += $nl
        }

        $enc = New-Object System.Text.UTF8Encoding($hasBom)
        [System.IO.File]::WriteAllText($path, ($prefix + $header + $body), $enc)
        $added += 1
    }

    Write-Host "[spdx] headers added: $added"
    Write-Host "[spdx] already tagged (skipped): $alreadyTagged"
    if ($missing -gt 0) {
        Write-Host "[spdx] tracked-but-missing (skipped): $missing"
    }
}
finally {
    Pop-Location
}
