Run `cargo llvm-cov --summary-only` and update COVERAGE_EXCEPTIONS.md to reflect the current status of the codebase.

Instructions for updating COVERAGE_EXCEPTIONS.md:
1. Identify all Rust modules with Line, Function, or Region coverage below 80%.
2. Update the Backend table structure to include distinct columns for Line %, Function %, and Region %.
3. For any newly appearing low-coverage module NOT currently listed, add a row with:
   - Module / File Path
   - Line Coverage %
   - Function Coverage %
   - Region Coverage %
   - Current Date (YYYY-MM-DD)
   - "[PENDING REVIEW]" in the Reason / Rationale column.
4. For existing modules already in the table:
   - Do NOT overwrite the old entry.
   - Insert a new row directly below the existing entry with the updated coverage metrics (Line %, Function %, Region %), current date (YYYY-MM-DD), and "[PENDING REVIEW]" in Reason / Rationale so I can compare historical progress.
5. Order the table by module including file path
6. Do NOT attempt to write, modify, or generate unit tests for any files. Do not write or edit source code files—only update COVERAGE_EXCEPTIONS.md.
-------
Run `npx vitest run --coverage` and update COVERAGE_EXCEPTIONS.md to reflect the current status of the Svelte frontend codebase.

Instructions for updating COVERAGE_EXCEPTIONS.md:
1. Identify all Svelte view modules with Statement/Line, Branch/Region, or Function coverage below 80%.
2. Update the Frontend table structure to include distinct columns for Line %, Function %, and Branch/Region %.
4. For any newly appearing low-coverage Svelte file NOT currently listed, add a row with:
   - Module / File Path
   - Line Coverage %
   - Function Coverage %
   - Branch / Region Coverage %
   - Current Date (YYYY-MM-DD)
   - "[PENDING REVIEW]" in the Reason / Rationale column.
5. For existing Svelte modules already in the table:
   - Do NOT overwrite the old entry.
   - Insert a new row directly below the existing entry with the updated coverage metrics, current date (YYYY-MM-DD), and "[PENDING REVIEW]" in Reason / Rationale for historical comparison.
6. Order the table by module including file path
7. Do NOT attempt to write, modify, or generate unit tests for any files. Do not write or edit source code files—only update COVERAGE_EXCEPTIONS.md.

------
Prompt to update tests to increase coverage
@ModuleName has a NumberHere% function/line/region coverage. Can it be improved? Use information in @/.clinerules for information on how to write the tests. Explain your reasons if the coverage should be under 100%. Update @/docs\policies\testing\COVERAGE_EXCEPTIONS.md with the new coverage at the end of the task. This update should be in a new row in the table so it can be compared to the previous coverage run.