\## 📋 User Test Suite: Browse



This issue tracks the user-facing functionality for the \*\*Browse view\*\*.

\> \*\*Automated coverage:\*\* The \*\*Needs attention\*\* filter is also asserted by unit tests — `BrowseView.test.ts` (the invoke payload `additional_filters.needs_attention`), `BatchOperationsView.backfill.test.ts` (the "Needs attention:" summary and \*\*Review these in Browse\*\* link), and the Rust `designs_tests` (\*\*`d.image_data IS NULL`\*\* predicate). The manual checks below cover the end-to-end user experience.



\### 🔄 Core Workflows to Test



\- \[x] \*\*Setup\*\*

&#x20; - \[x] Import files from `data\\machine embroidery designs\\user tests`

&#x20; - \[x] Ensure specific fixture files exist:

&#x20;   - \[x] `Cake 3 - to be verified.jef` (Status: Unverified)

&#x20;   - \[x] `Cake 3 Cross Stitch Fred.jef` (No `Cross Stitch` tag, contains "Cross Stitch" in name)

&#x20;   - \[x] `Cake 3.jef` (Has `Cross Stitch` and `Food` tags)

&#x20;   - \[X] `Bean X.jef` 

&#x20;   - \[X] `Bean Z.jef` 

&#x20;   - \[x] `Cake 3 - Food.jef` (No `Food` tag, contains "Food" in name)

&#x20;   - \[x] \*\*Additional Filter Target Fixtures:\*\*

&#x20;     - \[x] `Cake Applique.jef` (Designer: `Me`, Source: `Me`, Hoop: `Hoop B`, Rating: `4 Stars`, Image Tags: `\[Flowers]`, Stitching Tags: `\[Filled]`, Stitched Status: `Yes`)

&#x20;     - \[x] `Cake Applique 2.jef` (Designer: `Brother`, Source: `Previous Owner`, Hoop: `Hoop A`, Rating: `2 Stars`, Image Tags: `\[Footwear]`, Stitching Tags: `\[Applique]`, Stitched Status: `No`)

&#x20;   - \[ ] \*\*Needs Attention Fixture:\*\*

&#x20;     - \[ ] Create a deliberately unreadable file with a supported extension in the import source (e.g. `data\\machine embroidery designs\\user tests\\ZZ-broken.pes` containing the text `not an embroidery file`).

&#x20;     - \[ ] Import it — expect the flagged-import behaviour: the design row is created with \*\*no preview\*\* and a warning toast reads \*"1 need attention (preview could not be generated) — regenerate in Batch Operations."\*

&#x20;     - \[ ] Confirm at least one healthy fixture (e.g. `Cake 3.jef`) \*\*does\*\* have a preview, so the filter's negative case can be proven.



\- \[x] \*\*Initial Load\*\*

&#x20; - \[x] Screen opens without lag or visual stutter

&#x20; - \[x] Default layout elements and text render correctly based on UI schema:

&#x20;   - \[x] Title banner shows "Browse Designs"

&#x20;   - \[x] Top global navigation ribbon is active (`Browse`, `Import`, `Projects`, `Help`)

&#x20;   - \[x] Admin sub-menu options are accessible (`Designers`, `Tags`, `Sources`, `Hoops`, `Settings`, `Backup`, `Batch Operations`, `Orphans`)

&#x20;   - \[x] General Search input displays placeholder example text: `e.g. rose "cross stitch" -applique or \*.hus`

&#x20;   - \[x] "SEARCH IN:" controls display checkboxes: `File name`, `Tags`, `Folder name` (all default to \*\*Ticked\*\*)

&#x20;   - \[x] Helper syntax text reads: `Supports Google-like syntax: "exact phrase" · -exclude · word1 OR word2 · \*.hus · Search help`

&#x20;   - \[x] "Unverified only" checkbox defaults to \*\*Unticked\*\*

&#x20;   - \[x] "Sort by" dropdown defaults to \*\*Name\*\*

&#x20;   - \[x] "Direction" dropdown defaults to \*\*Ascending\*\*

&#x20;   - \[x] "Reset filters" action button is visible next to Direction dropdown. It is disabled.

&#x20;   - \[ ] "Needs attention" checkbox is present in the \*\*Additional Filters\*\* drawer (hidden until expanded) and defaults to \*\*Unticked\*\*

&#x20;   - \[ ] Helper text under the "Needs attention" checkbox reads: `Designs with no preview image — the file may be corrupt or unreadable.`

&#x20; - \[x] Initial data or records populate as expected:

&#x20;   - \[x] Main grid populates with design cards in a responsive grid layout

&#x20;   - \[x] Status line correctly calculates and renders: `X designs found · Y of Z selected` (e.g., `61 designs found · 0 of 50 selected`)

&#x20;   - \[x] Master "Select all on page" checkbox displays next to selection count (defaults to \*\*Unticked\*\*)

&#x20;   - \[x] Grid includes row-level selection checkboxes on the far left margin for multi-card row selection

&#x20;   - \[x] Each design card cleanly renders its individual components: item thumbnail frame, checkbox in top-left, filename label, hoop size, sources/notes, tag list, rating stars (if set), green checkmark verification status icon, and `▶ + Add to project` collapse bar; a design with no preview instead renders the placeholder `Preview could not be generated — the file may be corrupt or unreadable` in the thumbnail frame

&#x20;   - \[x] Batch action bottom floating toolbar is \*\*Hidden\*\* when 0 items are selected



\- \[x] \*\*Primary Action: Additional Filters Drawer\*\*

&#x20; - \[x] \*\*Toggle Drawer Visibility\*\*

&#x20;   - \[x] Click `▶ ADDITIONAL FILTERS` accordion header to expand panel; verify layout matches fields: `Designer` multi-select list, `Tag` multi-select list, `Source` multi-select list, `Hoop size` dropdown, `Rating` dropdown, `Stitched` dropdown.

&#x20;   - \[x] Click `▼ ADDITIONAL FILTERS` accordion header to collapse panel; fields hidden from screen.

&#x20; - \[x] \*\*Multi-Select Filter Lists\*\*

&#x20;   - \[x] \*\*Designer Selection\*\*: Check multi-select options (e.g., `Alice Scott Morris`, `Bernina`, `Brother`) -> Filter grid updates to matching designs.

&#x20;   - \[x] \*\*Tag Selection\*\*: Check multi-select options (e.g., `Words and Letters`, `Angels`, `Applique`, `Badges and Crests`) -> Filter grid updates to matching tags.

&#x20;   - \[x] \*\*Source Selection\*\*: Check multi-select options (e.g., `Brother Embroidery Software`, `Craftsy...`) -> Filter grid updates accordingly.

&#x20; - \[x] \*\*Dropdown Filters\*\*

&#x20;   - \[x] \*\*Hoop size\*\*: Select specific hoop size (e.g., `Hoop B`, `Hoop A`) -> Filters cards matching hoop size parameter.

&#x20;   - \[x] \*\*Rating\*\*: Select minimum rating requirement -> Filters cards by star rating.

&#x20;   - \[x] \*\*Stitched\*\*: Toggle between `Any`, `Yes`, and `No` -> Filters designs based on stitched verification state.

&#x20; - \[ ] \*\*Needs Attention Filter\*\*

&#x20;   - \[ ] Expand \*\*ADDITIONAL FILTERS\*\*: verify the `Needs attention` checkbox sits below `Minimum rating` / `Stitched`, with the helper text `Designs with no preview image — the file may be corrupt or unreadable.`

&#x20;   - \[ ] Tick `Needs attention` -> the grid restricts to the no-preview design(s); the status line count drops to the flagged total.

&#x20;   - \[ ] Untick `Needs attention` -> the grid returns to the full library.

&#x20;   - \[ ] With only `Needs attention` ticked, `Reset filters` becomes enabled; clicking it unticks the box, disables the button again, and refreshes the grid to the full library.

&#x20; - \[x] \*\*Reset Filters Action\*\*

&#x20;   - \[x] Click `Reset filters` button: Purges active search inputs, unchecks all selected Designers, Tags, and Sources list items, restores dropdowns (`Hoop size`, `Rating`, `Stitched`) back to default `Any`, and refreshes grid results.

\### 🧪 Filter \& Search Verification Tests



\- \[x] \*\*Category Combinations (`AND` logic):\*\* Verify that selecting options across different filter categories (e.g., Designer: \*Bernina\* `AND` Tag: \*Animals\*) restricts results to designs matching \*\*all\*\* selected categories.

\- \[x] \*\*Multi-Select Within Category (`OR` logic):\*\* Verify that selecting multiple items within the same category list (e.g., Tags: \*Animals\* `OR` \*Birds\*) returns designs containing \*\*any\*\* of the selected options.

\- \[x] \*\*Additional Filter Controls:\*\* Verify that filtering by \*\*Hoop Size\*\*, \*\*Minimum Rating\*\*, and \*\*Stitched Status\*\* properly constrains the result set.

\- \[x] \*\*General Search Query Rules:\*\* Test exact phrases (`"cross stitch"`), exclusions (`-applique`), wildcards (`\*.hus`), and inline `OR` statements in the search bar to ensure proper parsing.

\- \[x] \*\*Combined Search \& Filters:\*\* Ensure active General Search queries and Additional Filters operate simultaneously using `AND` logic without resetting or overriding each other.

\- \[x] \*\*Unverified Only Toggle:\*\* Confirm that checking \*\*Unverified only\*\* restricts results to unverified items while preserving any active search terms or additional filters.

\- \[x] \*\*Empty State \& Reset:\*\* Confirm that clearing search inputs or clicking \*\*Reset filters\*\* reverts the results grid back to the full library without SQL errors.

\- \[ ] \*\*Needs Attention Filter (standalone):\*\* Confirm that ticking \*\*Needs attention\*\* restricts results to designs with no stored preview (`image_data IS NULL`), matched against the corrupt fixture, and that unticking restores the full library.

\- \[ ] \*\*Needs Attention combined with other filters (`AND` logic):\*\* Confirm \*\*Needs attention\*\* narrows the intersection when combined with a Designer, Tag, Source, Hoop/Rating/Stitched value, a General Search term, and \*\*Unverified only\*\* (a design that has a preview is excluded even when it matches the other filters).

\- \[ ] \*\*Needs Attention + Sorting/Pagination:\*\* Confirm that sorting (e.g. Name/Rating) and pagination operate only on the flagged result set.

\- \[ ] \*\*Needs Attention empty state:\*\* Confirm that ticking the filter when no flagged designs exist shows `0 designs found` without SQL errors.



\- \[x] \*\*Primary Action: Selection \& Batch Action Toolbar\*\*

&#x20; - \[x] \*\*Selection Mechanics\*\*

&#x20;   - \[x] \*\*Individual Card Selection\*\*: Toggle individual checkbox on `01dstPeacock.dst`: Top bar status updates to `1 of 50 selected`.

&#x20;   - \[x] \*\*Row-Level Selection\*\*: Click left margin row selection checkbox: Instantly toggles or clears all 5 cards within that specific row array synchronously. Status updates (e.g., `5 of 50 selected`).

&#x20;   - \[x] \*\*Page-Level Selection\*\*: Click master `Select all on page` checkbox -> Every visible checkbox on current page updates uniformly. Status updates to `50 of 50 selected`.

&#x20;   - \[x] \*\*Toolbar Display\*\*: Verify that when 1 or more designs are checked, the fixed batch action toolbar smoothly slides/mounts at the bottom right of the viewport with options: `Choose tags`, `Verify tags`, `Add to project...`, `Delete selected`, `Clear selection`.

&#x20; - \[x] \*\*Batch Action Execution\*\*

&#x20;   - \[x] \*\*Choose tags Button\*\*:

&#x20;     - \[x] Click `Choose tags` button.

&#x20;     - \[x] Verify Choose Tags Modal opens with active selection context.

&#x20;   - \[x] \*\*Verify tags Button\*\*:

&#x20;     - \[x] Press `Verify tags` button.

&#x20;     - \[x] Verify selected unverified targets update state to Verified (green checkmark icon).

&#x20;   - \[x] \*\*Add to project... Button\*\*:

&#x20;     - \[x] Press `Add to project...` button.

&#x20;     - \[x] Confirm project target menu opens to append selected items into workspace projects.

&#x20;   - \[x] \*\*Delete selected Button\*\*: - See issue 30 for tests

&#x20;   - \[ ] \*\*Clear selection Button\*\*:

&#x20;     - \[x] Press `Clear selection` button.

&#x20;     - \[x] Verify all individual card, row-level, and master checkboxes drop active states, bottom toolbar unmounts/hides.



\- \[x] \*\*Primary Action: Choose Tags Modal Workflow\*\*

&#x20; - \[x] \*\*Modal Launch \& Layout\*\*

&#x20;   - \[x] Select items and press `Choose tags` on bottom toolbar.

&#x20;   - \[x] Verify modal backdrop dims main background browse window.

&#x20;   - \[x] Title header shows `Choose tags for selected designs`.

&#x20;   - \[x] Sub-header displays correct selected item count (e.g., `5 designs selected.`).

&#x20;   - \[x] Verify presence of global `Untagged (clear all tags)` checkbox.

&#x20;   - \[x] Verify taxonomy sections are cleanly split into distinct list columns with vertical scrollbars.

&#x20; - \[x] \*\*Tag Mutation \& Commit Operations\*\*

&#x20;   - \[x] Toggle tag checkboxes for selected items.

&#x20;   - \[x] Press `Cancel` button: Closes modal without committing metadata updates.

&#x20;   - \[x] Press `Apply tags` button: Modal closes, updates tag badges across selected card views instantly, auto-promotes unverified designs to verified status.



\- \[x] \*\*Primary Action: Sorting \& Filtering\*\*

&#x20; - \[x] Select "Sort by" dropdown: `Name`, `Folder`, `Date added`, 'Rating', 'Stitched'.

&#x20; - \[x] Select "Direction" dropdown: `Ascending`, `Descending`.

&#x20; - \[x] Verify grid layout reorganizes immediately upon changing sort parameter or direction.

&#x20; - \[ ] Verify sorting and pagination operate only on the \*\*Needs attention\*\* result set when that filter is ticked.



\- \[ ] \*\*Primary Action: General Search \& "Search In" Scoping\*\*

&#x20; - \[x] \*\*Unverified Only Filter\*\*

&#x20;   - \[x] Tick `Unverified only` checkbox -> Grid filters to display only unverified items (red 'x' status).

&#x20;   - \[x] Untick `Unverified only` checkbox -> Grid returns both verified and unverified items.

&#x20; - \[x] \*\*Search Target Scope ("SEARCH IN:" Combinations)\*\*

&#x20;   - \[x] \*\*All Ticked (`File name`, `Tags`, `Folder name`)\*\*:

&#x20;     - \[x] Search "Cake 3 Cross" -> Note the double quotes. Only designs containing "Cake 3 Cross" return.

&#x20;     - \[x] Search `Cake 3` -> Returns all designs matching "Cake 3" across filename, tag, or parent folder name.

&#x20;     - \[x] Search `"Cross Stitch"` -> Matches exact phrase across filenames, tags, and folder names.

&#x20;     - \[x] Search `Cake 3 -cross` -> Returns "Cake 3" matches excluding any item matching "cross" in filename, tags, or folder.

&#x20;     - \[x] Search `Cake 3 OR Bean` -> Note upper case OR. Returns entries matching "Cake 3" OR "Bean" across all indexed metadata fields.

&#x20;     - \[x] Search `\*.jef` -> Wildcard query returns all `.jef` file format extensions.

&#x20;   - \[x] \*\*File name Ticked, Tags \& Folder name Unticked\*\*:

&#x20;     - \[x] Search `Cross` -> Only returns designs with "Cross" in the file name (e.g., `Cake 3 Cross Stitch Fred.jef`).

&#x20;     - \[x] Search `Cake 3 Cross` -> Returns `Cake 3 Cross Stitch Fred.jef`; excludes `Bean Z.jef` located in folder `Cake 3 Cross`.

&#x20;   - \[x] \*\*Tags Ticked, File name \& Folder name Unticked\*\*:

&#x20;     - \[x] Search `Cross` -> Returns `Cake 3.jef` (which has the `Cross Stitch` tag); excludes `Cake 3 Cross Stitch Fred.jef` (which lacks the tag).

&#x20;     - \[x] Search `Food` -> Returns `Cake 3.jef` (has `Food` tag); excludes `Cake 3 - Food.jef` (lacks `Food` tag).

&#x20;   - \[x] \*\*Folder name Ticked, File name \& Tags Unticked\*\*:

&#x20;     - \[x] Search `Cross` -> Returns `Bean Z.jef` (located inside `Cross` folder).

&#x20;   - \[ ] \*\*Combination Scenarios\*\*:

&#x20;     - \[x] Tick `File name` + `Tags`, Untick `Folder name`:

&#x20;       - \[x] Search `Cross` -> Returns both `Cake 3 Cross Stitch Fred.jef` (filename match) and `Cake 3.jef` (tag match); excludes `Bean Z.jef` (folder match only).

&#x20;     - \[x] Tick `File name` + `Folder name`, Untick `Tags`:

&#x20;       - \[x] Search `Cross` -> Returns `Cake 3 Cross Stitch Fred.jef` and `Bean Z.jef`; excludes `Cake 3.jef` (tag-only match).

&#x20;     - \[x] Untick All (`File name`, `Tags`, `Folder name`):

&#x20;       - \[x] Verify entering search terms yields no matching results

&#x20; - \[ ] \*\*Needs Attention + Search combination:\*\* With \*\*Needs attention\*\* ticked, entering a search term keeps the no-preview constraint while applying the query (`AND`); clearing the query returns the full flagged set.



\- \[ ] \*\*Navigation\*\*

&#x20; - \[x] User can safely exit or navigate away without application stutter or freezing.

&#x20; - \[?] Leaving the page midway prompts a "Save changes?" warning if pending tag or metadata modifications are unsaved. Note: Updates are too quick to test.

&#x20; - \[x] Verify pagination controls operate smoothly at bottom of grid results when total count exceeds page limits.

&#x20; - \[ ] \*\*"Needs attention" filter persistence:\*\* With \*\*Needs attention\*\* ticked, open a flagged design in \*\*Design Detail\*\* and return to \*\*Browse\*\* -> the filter, current page, sorting and results are preserved.

&#x20; - \[ ] Clicking a flagged (no-preview) card opens \*\*Design Detail\*\* for triage (see the \*\*Design Details\*\* test plan for the no-preview banner and \*\*Regenerate\*\* action).

&#x20; - \[ ] Clicking \*\*Reset filters\*\* while \*\*Needs attention\*\* is ticked clears the flag, disables the Reset button, and restores the full library.



\### 🔗 Cross-View Entry Point: "Review these in Browse"

&#x20; - \[ ] In \*\*Admin -> Batch Operations\*\*, open the \*\*Maintenance & File Processing\*\* tab and run an \*\*images\*\* task while designs are still missing a preview.

&#x20; - \[ ] The run summary shows `Needs attention: N before -> M after`.

&#x20; - \[ ] When the "after" count is greater than 0, a \*\*Review these in Browse\*\* link is shown with the note \*(opens Browse with the "Needs attention" filter — designs without a preview)\*.

&#x20; - \[ ] Clicking it navigates to \*\*Browse Designs\*\* with \*\*Needs attention\*\* ticked, all other filters cleared, and page 1 selected.

\### ❌ Failed Tests / Discovered Friction

\*Hover over a failed subtest above and click "Convert to issue", or track them below:\*



