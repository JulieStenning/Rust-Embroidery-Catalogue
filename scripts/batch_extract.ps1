# TEMPLATE: This script can be adapted to extract #[cfg(test)] modules from
# any Rust source file exceeding 500 lines. To reuse:
#   1. Add entries to the $jobs array below (Src path + Base filename).
#   2. Verify the file's #[cfg(test)] mod tests { marker is at the tail.
#   3. Run the script. It handles the extraction, test-file creation, and
#      production-file rewrite automatically.
#   4. Afterwards, run `cargo check`, `cargo test <module>`, and
#      `rustfmt --edition 2024` on the new test file.
#
# Batch-extract the singleton #[cfg(test)] mod tests block from each of the
# nine remaining large files into a sibling *_tests.rs file, replacing the
# block with a #[path] module declaration.
#
# Each file is verified to contain exactly one #[cfg(test)] mod tests { and
# that marker is at the tail (no production code after it). The body is taken
# from (cfg_line+2) .. (total-2) inclusive on a 1-based basis - i.e. it starts
# at `use super::*;` (line right after `mod tests {`) and ends at the last
# test-fn closing brace (line just before the module outer `}`).

$ErrorActionPreference = "Stop"

# Format: SourcePath | basename for test file
$jobs = @(
    @{ Src = "src/services/stitch_identifier.rs"; Base = "stitch_identifier_tests.rs" }
    @{ Src = "src/main.rs";                 Base = "main_tests.rs" }
    @{ Src = "src/routes/settings.rs";      Base = "settings_route_tests.rs" }
    @{ Src = "src/services/fingerprint.rs"; Base = "fingerprint_tests.rs" }
    @{ Src = "src/readers/vp3_reader.rs";   Base = "vp3_reader_tests.rs" }
    @{ Src = "src/readers/jef_reader.rs";   Base = "jef_reader_tests.rs" }
    @{ Src = "src/png_writer.rs";           Base = "png_writer_tests.rs" }
    @{ Src = "src/services/settings.rs";    Base = "settings_svc_tests.rs" }
    @{ Src = "src/database/migrations.rs";  Base = "migrations_tests.rs" }
)

foreach ($job in $jobs) {
    $file = Resolve-Path $job.Src
    $lines = @(Get-Content -LiteralPath $file)
    $total = $lines.Count

    # Locate the #[cfg(test)] + mod tests marker.
    $cfgIdx = -1
    for ($i = 0; $i -lt $total - 1; $i++) {
        if ($lines[$i] -match '#\[cfg\(test\)\]' -and $lines[$i + 1] -match '^\s*mod tests \{') {
            $cfgIdx = $i
            break
        }
    }
    if ($cfgIdx -lt 0) {
        Write-Host "SKIP $($job.Src): no #[cfg(test)] mod tests marker"
        continue
    }

    # Confirm the outer closing brace is the final non-blank line.
    $bodyEnd = $total - 2   # 0-based index of last test-fn closing brace
    if ($lines[$total - 1] -notmatch '^\}') {
        Write-Host "SKIP $($job.Src): last line is not the module close"
        continue
    }

    # Production head: lines 0 .. cfgIdx-1 (everything before #[cfg(test)]).
    $productionHead = $lines[0..($cfgIdx - 1)]
    # Module body: lines cfgIdx+2 .. bodyEnd (use super::*; ... last test brace).
    $moduleBody = $lines[($cfgIdx + 2)..$bodyEnd]

    $header = @(
        '// Tests for the source module.'
        '//'
        '// This module was split out so the production file can stay focused'
        '// on logic. It is included via a #[path] declaration in a'
        '// #[cfg(test)] mod tests; module, retaining full access to the'
        '// private items in the parent module through use super::*;.'
        ''
    )

    $newTestLines = @()
    $newTestLines += $header
    $newTestLines += $moduleBody

    $testFile = Join-Path (Split-Path $file -Parent) $job.Base
    Set-Content -LiteralPath $testFile -Value $newTestLines -Encoding utf8
    Write-Host "  Wrote $testFile ($($newTestLines.Count) lines)"

    # Rewrite the production file.
    $moduleDecl = @(
        '#[cfg(test)]'
        "#[path = `"$($job.Base)`"]"
        'mod tests;'
        ''
    )
    if ($productionHead.Count -gt 0 -and $productionHead[-1] -eq '') {
        $productionHead = $productionHead[0..($productionHead.Count - 2)]
    }
    $newLines = @()
    $newLines += $productionHead
    $newLines += $moduleDecl
    Set-Content -LiteralPath $file -Value $newLines -Encoding utf8
    Write-Host "  Rewrote $($job.Src) ($($newLines.Count) lines, was $total)"
}

Write-Host "Batch complete."