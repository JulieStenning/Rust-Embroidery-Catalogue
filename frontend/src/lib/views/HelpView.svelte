<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script lang="ts">
  const navItems = [
    { href: "#/help?section=search", label: "🔍 Browse & Search" },
    { href: "#/help?section=importing", label: "📥 Importing" },
    { href: "#/help?section=word-matches", label: "🔤 Tag Word Matches" },
    { href: "#/help?section=storage", label: "💾 Data Storage & External Drives" },
    { href: "#/help?section=ai-tagging", label: "🤖 AI Tagging" },
    { href: "#/help?section=batch-operations", label: "🏷 Batch Operations" },
    { href: "#/help?section=projects", label: "📁 Projects" },
    { href: "#/help?section=maintenance", label: "🛠 Maintenance" },
    { href: "#/help?section=settings", label: "⚙️ Settings" },
    { href: "#/help?section=troubleshooting", label: "🔧 Troubleshooting" },
  ];

  const sectionHeadingClass =
    "text-xl font-semibold mb-3 border-b border-[var(--border-default)] text-[var(--text-primary)] pb-1";
</script>

<div class="space-y-10 max-w-3xl">
  <div class="space-y-1 font-sans no-print">
    <h1 class="ui-page-title text-2xl font-bold text-[var(--text-primary)]">Help</h1>
    <p class="text-[var(--text-secondary)] text-sm">
      Quick guidance for using the Embroidery Catalogue.
    </p>
  </div>

  <nav class="flex flex-wrap gap-3 text-sm no-print">
    {#each navItems as item (item.href)}
      <a
        href={item.href}
        class="bg-[var(--surface-card-subtle)] text-[var(--text-brand)] border border-[var(--border-default)] px-3 py-1 rounded hover:bg-[var(--surface-hover)]"
        >{item.label}</a
      >
    {/each}
  </nav>

  <section id="search">
    <h2 class={sectionHeadingClass}>🔍 Browse & Search</h2>
    <p class="text-sm text-[var(--text-secondary)] mb-3">
      Use the general search box on the <a
        href="#/designs"
        class="text-[var(--text-brand)] hover:underline">Browse</a
      > page to find designs by keyword. The search checks the filename, notes, designer, source, and
      tags.
    </p>
    <div class="route-card rounded shadow p-4 space-y-3 text-sm">
      <p>
        <strong>Quoted phrases:</strong>
        <code
          class="bg-[var(--surface-card-subtle)] border border-[var(--border-default)] px-1 rounded font-mono text-xs"
          >"cross stitch"</code
        >
      </p>
      <p>
        <strong>Exclude terms:</strong>
        <code
          class="bg-[var(--surface-card-subtle)] border border-[var(--border-default)] px-1 rounded font-mono text-xs"
          >-rose -applique</code
        >
      </p>
      <p>
        <strong>OR searches:</strong>
        <code
          class="bg-[var(--surface-card-subtle)] border border-[var(--border-default)] px-1 rounded font-mono text-xs"
          >rose OR tulip</code
        >
      </p>
      <p>
        <strong>Filename wildcards:</strong>
        <code
          class="bg-[var(--surface-card-subtle)] border border-[var(--border-default)] px-1 rounded font-mono text-xs"
          >rose*.jef</code
        >
      </p>
      <p>
        <strong>Verification status:</strong> filter designs by verification state using the
        dropdown in the search bar (<em>All</em>, <em>Unverified only</em>, <em>Verified only</em>,
        or <em>Partially verified</em>). You can also filter Image and Stitching tag verification
        independently under Additional Filters.
      </p>
      <p>
        <strong>Quick search vs. filters:</strong> use Additional filters for designer, image and stitching
        tags, sources, hoop size, dimensions (width and height mm ranges), rating, and stitched status.
      </p>
      <p>
        <strong>Active filter chips:</strong> whenever criteria under Additional filters are
        selected, removable chips appear beneath the search controls. Click the <strong>✕</strong>
        on any chip to remove that filter instantly, or click <em>Clear all</em>. A counter badge on
        the
        <em>Additional Filters</em> heading shows how many filters are currently active.
      </p>
      <p>
        <strong>Dimension ranges:</strong> specify minimum and/or maximum width or height in mm. Setting
        only a maximum (e.g. Max H: 80) matches designs from 0mm up to 80mm; setting only a minimum (e.g.
        Min W: 100) matches all designs 100mm and wider.
      </p>
      <p>
        <strong>Sorting:</strong> sort designs by Date Added, Date Updated, Name, Rating, Stitched
        status, or <em>Folder</em>. When sorting by Folder, designs are grouped by folder path, with
        filenames automatically sorted alphabetically (A&ndash;Z) within each folder to streamline
        bulk-tagging related files.
      </p>
      <p>
        <strong>Fast tag verification:</strong> automated tags don't need to be perfect to save
        time. Search for a subject keyword (e.g.
        <code
          class="bg-[var(--surface-card-subtle)] border border-[var(--border-default)] px-1 rounded font-mono text-xs"
          >birds</code
        >) with <em>Unverified only</em> ticked to gather similar designs into rows and pages. You
        can then select whole rows or pages at once and use <strong>Choose tags...</strong> in the bottom
        action bar to apply clean tags and verify the batch (with independent checkboxes to verify image
        or stitching tags as needed).
      </p>
      <p>
        <strong>Missing previews &amp; hoop boundaries:</strong> if a design card displays
        <em>Design Exceeds Hoop Area</em>, the design is larger than the hoop saved in its
        embroidery software, leaving machine stitches uncompiled. You can quickly isolate all such
        designs by selecting
        <em>Exceeds hoop area</em> in the <strong>Needs attention</strong> filter. Open the design in
        your embroidery editor, assign a larger hoop (or scale the design to fit), re-save, and regenerate
        the preview.
      </p>
      <p>
        <strong>Master outline files without stitch files:</strong> if you catalogue digitising
        master outline files (such as Embird <code>.eof</code> or Bernina <code>.art</code>), they
        appear with a placeholder preview until an exported machine stitch file (e.g.
        <code>.jef</code>, <code>.pes</code>) is provided. Filter by
        <em>Needs attention &rarr; Master files without stitch files</em>, open the design detail,
        and use <strong>Check Folder for Stitch Files</strong> to pair newly exported stitch files in
        that folder in one click while preserving all tags and metadata.
      </p>
      <p>
        <strong>Full guide:</strong> open the
        <a
          href="#/about/document/browse-bulk-actions"
          class="text-[var(--text-brand)] hover:underline">Browse &amp; Bulk Actions Guide</a
        >.
      </p>
    </div>
  </section>

  <section id="importing">
    <h2 class={sectionHeadingClass}>📥 Importing</h2>
    <p class="text-sm text-[var(--text-secondary)] mb-3">
      Use <a href="#/import" class="text-[var(--text-brand)] hover:underline">Import</a> to scan one or
      more folders and their sub-folders.
    </p>
    <div class="route-card rounded shadow p-4 space-y-3 text-sm">
      <p>
        <strong>Choosing folders:</strong> enter paths, use Browse, or add multiple folders;
        sub-folders are scanned automatically. You can also tick
        <em>Include uncatalogued files from your library storage folder</em>
        to scan for loose design files already stored on disk that have not yet been added to the catalogue
        database.
      </p>
      <p>
        <strong>Live scan progress:</strong> while scanning, the scan button displays real-time counts
        of folders and files discovered as well as catalogue check progress.
      </p>
      <p>
        <strong>Review and metadata:</strong> set Designer and Source per folder before import, or keep
        the automatically inferred values. Inference evaluates folder paths from the immediate containing
        folder upwards (leaf-first hierarchy), prioritising the folder directly containing your files
        over parent directories while ignoring catalogue library root folders.
      </p>
      <p>
        <strong>Tag check before import:</strong> first import asks for tag review, later imports can
        continue directly.
      </p>
      <p>
        <strong>Tagging during import:</strong> imports apply File & Folder Rules only (matching tag
        names and configured
        <a href="#/about/document/tag-word-matches" class="text-[var(--text-brand)] hover:underline"
          >Word Matches</a
        >) and never call Gemini.
      </p>
      <p>
        <strong>Error files and large scans:</strong> unreadable files are skipped; large scans may summarise
        rows.
      </p>
      <p>
        <strong>Large collections &amp; folder review:</strong> when reviewing large scans, folders are
        paginated (25 per page) with a folder search filter. Folders default to collapsed for speed; click
        Show files to inspect individual designs. Top and bottom action buttons allow continuing or selecting
        all across all pages at once.
      </p>
      <p>
        <strong>Alphabet &amp; font compression:</strong> enabled by default during initial setup
        (and configurable in Settings), full embroidery alphabet and font collections are
        automatically recognised and packaged into a companion
        <code
          class="bg-[var(--surface-card-subtle)] border border-[var(--border-default)] px-1 rounded font-mono text-xs"
          >.zip</code
        >
        archive in the designs library folder. Only representative preview samples (Upper A, Lower a,
        9, and all punctuation &amp; symbols) for each size are catalogued into the database and tagged
        with the protected <strong>Alphabets</strong> system tag, keeping your catalogue clean and responsive
        while preserving full font packages for immediate extraction whenever needed.
      </p>
    </div>
  </section>

  <section id="word-matches">
    <h2 class={sectionHeadingClass}>🔤 Tag Word Matches</h2>
    <p class="text-sm text-[var(--text-secondary)] mb-3">
      Tag Word Matches allow words in filenames and folder paths to automatically assign tags during
      import and batch backfill operations.
    </p>
    <div class="route-card rounded shadow p-4 space-y-3 text-sm">
      <p>
        <strong>Offline &amp; automatic:</strong> runs 100% locally with no API key; matches whole
        words, plurals, singulars, and automatically splits trailing numbers (e.g.
        <em>Butterfly1</em>) and CamelCase names. Text designs are cleanly separated into the
        protected system tag <strong>Alphabets</strong> (fonts, monograms, number sets 0&ndash;9,
        and punctuation) and <strong>Words</strong> (quotes, phrases, sayings, and typography).
      </p>
      <p>
        <strong>Folder hierarchies &amp; supplier masking:</strong> broad category folders (such as
        <em>Animals</em>
        or <em>Floral</em>) higher up in your folder structure are retained and accurately tagged.
        When a folder segment matches an assigned or inferred
        <strong>Source</strong> (e.g. a supplier or store name like <em>Embroidery Super Deal</em>
        or
        <em>Urban Threads</em>), that folder name is automatically masked out so commercial brand
        words do not create spurious tags. Generic medium terms (such as <em>embroidery</em> and
        <em>stitch</em>) and format directories (such as <em>JEF</em> or <em>PES</em>) are also
        ignored.
      </p>
      <p>
        <strong>Manage matches:</strong> open
        <a href="#/admin/data/tag-matches" class="text-[var(--text-brand)] hover:underline"
          >Admin → Manage Data → Word Matches</a
        > to view cards, filter by tag group, and quickly add keywords.
      </p>
      <p>
        <strong>Contextual access:</strong> click the <strong>Matches</strong> button next to any
        tag in
        <a href="#/admin/data/tags" class="text-[var(--text-brand)] hover:underline"
          >Admin → Manage Data → Tags</a
        > or add matches immediately after creating a new tag.
      </p>
      <p>
        <strong>Full guide:</strong> open the
        <a href="#/about/document/tag-word-matches" class="text-[var(--text-brand)] hover:underline"
          >Tag Word Matches Guide</a
        >.
      </p>
    </div>
  </section>

  <section id="storage">
    <h2 class={sectionHeadingClass}>💾 Data Storage & External Drives</h2>
    <p class="text-sm text-[var(--text-secondary)] mb-3">
      The catalogue manages two data components: your design files (copied from your original files)
      and the internal catalogue database containing images and other details about the designs.
    </p>
    <div class="route-card rounded shadow p-4 space-y-3 text-sm">
      <p><strong>Internal drives:</strong> work automatically with no extra setup.</p>
      <p>
        <strong>External storage:</strong> SD cards, portable SSDs, and USB drives are supported for your
        library.
      </p>
      <p>
        <strong>SD card recommendation:</strong> use an Application Performance Class "A2" card (e.g.
        A2 V30/U3) for responsive thumbnail browsing and scanning of the design files.
      </p>
      <p>
        <strong>Filesystem formatting:</strong> format external cards and drives with
        <strong>exFAT</strong> or <strong>NTFS</strong>. Avoid FAT32 formatting because FAT32
        enforces a strict 4 GB maximum single-file size limit.
      </p>
      <p>
        <strong>Keep drives connected:</strong> connect external drives before launching and safely eject
        them before unplugging.
      </p>
      <p>
        <strong>Full guide:</strong> open the
        <a href="#/about/document/data-storage" class="text-[var(--text-brand)] hover:underline"
          >Data Storage & External Drives Guide</a
        >.
      </p>
    </div>
  </section>

  <section id="ai-tagging">
    <h2 class={sectionHeadingClass}>🤖 AI Tagging</h2>
    <p class="text-sm text-[var(--text-secondary)] mb-3">
      Optional Google AI tagging can suggest design-type tags for imported embroidery files.
    </p>
    <div class="route-card rounded shadow p-4 space-y-3 text-sm">
      <p>
        <strong>Get an API key:</strong>
        <a href="https://aistudio.google.com/" class="text-[var(--text-brand)] hover:underline"
          >Google AI Studio</a
        >, then save it in
        <a href="#/admin/system/settings" class="text-[var(--text-brand)] hover:underline"
          >Settings</a
        >.
      </p>
      <p>
        <strong>In-app actions:</strong> run Gemini Vision from
        <a href="#/admin/batch-operations" class="text-[var(--text-brand)] hover:underline"
          >Admin → Batch Operations</a
        >. AI tagging runs on demand in Batch Operations rather than during import.
      </p>
      <p>
        <strong>Potential costs:</strong> review
        <a href="https://ai.google.dev/pricing" class="text-[var(--text-brand)] hover:underline"
          >current pricing</a
        >.
      </p>
      <p>
        <strong>Full guide:</strong> open the
        <a href="#/about/document/ai-tagging" class="text-[var(--text-brand)] hover:underline"
          >AI Tagging &amp; Batch Operations Guide</a
        >.
      </p>
    </div>
  </section>

  <section id="batch-operations">
    <h2 class={sectionHeadingClass}>🏷 Batch Operations</h2>
    <p class="text-sm text-[var(--text-secondary)] mb-3">
      Run AI tagging and file maintenance on existing designs from <a
        href="#/admin/batch-operations"
        class="text-[var(--text-brand)] hover:underline">Admin → Batch Operations</a
      >.
    </p>
    <div class="route-card rounded shadow p-4 space-y-3 text-sm">
      <p><strong>Tag only untagged designs:</strong> safe; does not overwrite verified tags.</p>
      <p>
        <strong>Tag specific folders:</strong> select one or more library folders inline with optional
        subfolder inclusion.
      </p>
      <p><strong>Tag untagged and unverified designs:</strong> includes unverified suggestions.</p>
      <p><strong>Re-tag ALL designs:</strong> overwrites everything and requires confirmation.</p>
      <p>
        <strong>Local stitching backfill:</strong> updates stitching group tags using local pattern geometry.
      </p>
      <p>
        <strong>Date Updated tracking:</strong> whenever automated tagging adds or alters tags on a design,
        its Date Updated timestamp refreshes so recently enriched designs are easily found by sorting
        in Browse.
      </p>
      <p>
        <strong>Tag verification workflow:</strong> automated tagging clusters similar designs together
        so you can review and verify whole rows or pages at once on Browse using search filters and bulk
        actions.
      </p>
    </div>
  </section>

  <section id="projects">
    <h2 class={sectionHeadingClass}>📁 Projects</h2>
    <p class="text-sm text-[var(--text-secondary)] mb-3">
      <a href="#/projects" class="text-[var(--text-brand)] hover:underline">Projects</a> let you group
      designs for planned embroidery tasks.
    </p>
    <div class="route-card rounded shadow p-4 space-y-3 text-sm">
      <p><strong>What projects are for:</strong> collect designs for a specific plan or session.</p>
      <p><strong>Adding designs:</strong> use Add to Project from design detail.</p>
      <p>
        <strong>Bulk add:</strong> on Browse, select designs and use the bottom action bar project selector.
      </p>
      <p><strong>Printing:</strong> each project includes a print view.</p>
    </div>
  </section>

  <section id="maintenance">
    <h2 class={sectionHeadingClass}>🛠 Maintenance</h2>
    <p class="text-sm text-[var(--text-secondary)] mb-3">
      Use <a href="#/admin/system/discrepancies" class="text-[var(--text-brand)] hover:underline"
        >Library Discrepancies</a
      > to find records whose files are missing or uncatalogued disk files.
    </p>
    <div class="route-card rounded shadow p-4 space-y-3 text-sm">
      <p>
        The <strong>Library Discrepancies</strong> page provides two user-initiated tools to synchronise
        your catalogue and disk files without automatic background scanning on page load:
      </p>
      <ul class="list-disc list-inside space-y-1 text-[var(--text-secondary)]">
        <li>
          <strong>Find Unmatched Design Files:</strong> click <em>Scan for unmatched files</em> to detect
          disk files not yet catalogued, with options to import them.
        </li>
        <li>
          <strong>Missing Design Files (Orphans):</strong> click <em>Scan Disk</em> to check for database
          records whose physical files are no longer found on disk; discovered orphans are cached for
          instant pagination and review without repeated disk scanning.
        </li>
      </ul>
      <p>
        <strong>What orphaned records are:</strong> database records where design files are no longer
        present.
      </p>
      <p>
        <strong>Deleting orphans:</strong> removes database records only, not physical files. Deleting
        all orphans clears the list immediately without requiring another scan.
      </p>
      <p><strong>Use carefully:</strong> verify files are truly gone before deleting records.</p>
    </div>
  </section>

  <section id="settings">
    <h2 class={sectionHeadingClass}>⚙️ Settings & Configuration</h2>
    <p class="text-sm text-[var(--text-secondary)] mb-3">
      Manage your application preferences, appearance themes, and digitising format filters in
      <a href="#/admin/system/settings" class="text-[var(--text-brand)] hover:underline"
        >Admin → Settings</a
      >.
    </p>
    <div class="route-card rounded shadow p-4 space-y-3 text-sm">
      <p>
        <strong>Appearance:</strong> choose between System Default, Light, or Dark theme modes.
      </p>
      <p>
        <strong>Google Gemini &amp; AI Tagging:</strong> store optional API keys, select vision models,
        and tune concurrency.
      </p>
      <p>
        <strong>Database Maintenance:</strong> optimise and compact storage and monitor automatic health
        checks.
      </p>
      <p>
        <strong>Catalogue Storage:</strong> relocate your database and design library to a different location.
      </p>
      <p>
        <strong>Digitising &amp; Format Rules:</strong> configure software master outline formats and
        alphabet font compression.
      </p>
      <p>
        <strong>Product licence:</strong> manage your licence activation and registration in
        <a href="#/about" class="text-[var(--text-brand)] hover:underline">About</a>, or explore
        open-source dependencies and terms in
        <a href="#/about/licence" class="text-[var(--text-brand)] hover:underline"
          >Licence &amp; Dependencies</a
        >.
      </p>
      <p>
        <strong>Full guide:</strong> open the
        <a href="#/about/document/settings-guide" class="text-[var(--text-brand)] hover:underline"
          >Settings Guide</a
        >.
      </p>
    </div>
  </section>

  <section id="troubleshooting">
    <h2 class={sectionHeadingClass}>🔧 Troubleshooting</h2>
    <div class="route-card rounded shadow p-4 space-y-3 text-sm">
      <p>
        <strong>Missing folder / changed drive letter:</strong> ensure the full data folder moved with
        the app.
      </p>
      <p>
        <strong>Database preparation on startup:</strong> when the application upgrades or optimises
        indexes for large catalogues, the window opens immediately with a
        <em>Preparing Catalogue Database</em>
        progress screen. Please keep the app open while these one-off optimisations complete.
      </p>
      <p>
        <strong>Database missing or unreadable on startup:</strong> Embroidery Catalogue
        automatically opens in Recovery Mode to help you reconnect a relocated drive, restore from a
        backup, or create a clean catalogue (damaged databases are safely archived as
        <code
          class="bg-[var(--surface-card-subtle)] border border-[var(--border-default)] px-1 rounded font-mono text-xs"
          >.corrupt-&lt;timestamp&gt;</code
        >).
      </p>
      <p><strong>Import scan finds nothing:</strong> confirm folder path and supported formats.</p>
      <p>
        <strong>Files missing after import:</strong> run orphan cleanup, then re-import from new location.
      </p>
      <p>
        <strong>Managed storage location:</strong> imported files live under data/MachineEmbroideryDesigns.
      </p>
      <p>
        <strong>Database full / 4 GB limit:</strong> if you see "database or disk is full", your catalogue
        data root or backup destination may be on a FAT32 drive reaching the 4 GB file limit. Reformat
        the drive to exFAT or NTFS.
      </p>
      <p><strong>Still stuck:</strong> check permissions and review application logs.</p>
    </div>
  </section>
</div>
