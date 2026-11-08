<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script lang="ts">
  const navItems = [
    { href: "#/help?section=search", label: "🔍 Browse & Search" },
    { href: "#/help?section=design-detail", label: "🎨 Design Detail" },
    { href: "#/help?section=designers-sources", label: "🏷️ Designers, Sources & Hoops" },
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
      Explore your collection on <a
        href="#/designs"
        class="text-[var(--text-brand)] hover:underline">Browse</a
      >. Search with keywords, apply filters, or select multiple cards to update tags and verify
      designs in bulk.
    </p>
    <div class="route-card rounded shadow p-4 space-y-3 text-sm">
      <ul class="list-disc list-outside pl-5 space-y-2 text-[var(--text-secondary)]">
        <li>
          <strong>Search operators:</strong> Use
          <code
            class="bg-[var(--surface-card-subtle)] border border-[var(--border-default)] px-1 rounded font-mono text-xs"
            >"cross stitch"</code
          >
          for exact phrases,
          <code
            class="bg-[var(--surface-card-subtle)] border border-[var(--border-default)] px-1 rounded font-mono text-xs"
            >-rose -applique</code
          >
          to exclude terms,
          <code
            class="bg-[var(--surface-card-subtle)] border border-[var(--border-default)] px-1 rounded font-mono text-xs"
            >rose OR tulip</code
          >
          for either, or
          <code
            class="bg-[var(--surface-card-subtle)] border border-[var(--border-default)] px-1 rounded font-mono text-xs"
            >rose*.jef</code
          > for wildcards.
        </li>
        <li>
          <strong>Verification status:</strong> Filter designs by verification state (<em>All</em>,
          <em>Unverified only</em>, <em>Verified only</em>, or <em>Partially verified</em>) directly
          in the search bar.
        </li>
        <li>
          <strong>Quick search vs. filters:</strong> Click <em>Additional Filters</em> to filter by designer,
          tags, source, hoop size, dimensions (mm), rating, or stitched status.
        </li>
        <li>
          <strong>Active filter chips:</strong> Selected filters appear as removable chips below the
          search bar. Click the <strong>✕</strong> on any chip to remove it, or click
          <em>Clear all</em>.
        </li>
        <li>
          <strong>Sorting:</strong> Sort by Date Added, Date Updated, Name, Rating, Stitched, or
          <em>Folder</em> (folder order with filenames sorted A&ndash;Z).
        </li>
        <li>
          <strong>Fast tag verification:</strong> Search with <em>Unverified only</em> ticked to
          cluster similar designs, select a batch, and use <strong>Choose tags...</strong> to verify tags
          in bulk.
        </li>
      </ul>
      <div
        class="pt-2 border-t border-[var(--border-default)] flex flex-wrap gap-x-6 gap-y-1 text-xs"
      >
        <a
          href="#/about/document/search-and-filtering"
          class="text-[var(--text-brand)] hover:underline font-medium"
        >
          Read guide → Search &amp; Filtering Guide
        </a>
        <a
          href="#/about/document/browse-bulk-actions"
          class="text-[var(--text-brand)] hover:underline font-medium"
        >
          Read guide → Bulk Tagging Guide
        </a>
      </div>
    </div>
  </section>

  <section id="design-detail">
    <h2 class={sectionHeadingClass}>🎨 Design Detail</h2>
    <p class="text-sm text-[var(--text-secondary)] mb-3">
      Inspect stitch patterns, view thread colour sequences, launch external software, and edit
      details for individual designs.
    </p>
    <div class="route-card rounded shadow p-4 space-y-3 text-sm">
      <ul class="list-disc list-outside pl-5 space-y-2 text-[var(--text-secondary)]">
        <li>
          <strong>Open a design:</strong> Click any design card on
          <a href="#/designs" class="text-[var(--text-brand)] hover:underline">Browse</a> to open its
          detail page.
        </li>
        <li>
          <strong>Actual size (1:1) &amp; zoom:</strong> Click <strong>Actual Size (1:1)</strong> to inspect
          stitches at real-world millimetre scale, or pan and zoom to examine density and jump stitches.
        </li>
        <li>
          <strong>Thread colours &amp; stops:</strong> Review the thread colour sequence, brand colour
          codes, and total stitch count before setting up your machine.
        </li>
        <li>
          <strong>Print worksheet:</strong> Click <strong>Print View</strong> to preview and print a clean
          technical summary sheet with design dimensions, colour list, and personal notes.
        </li>
        <li>
          <strong>Open in embroidery software:</strong> Click <strong>Open in Editor</strong> (or the
          format badges when multiple formats exist) to launch the stitch or master file directly in your
          default embroidery software.
        </li>
        <li>
          <strong>Find on disk:</strong> Click <strong>Show in Explorer</strong> to locate the original
          embroidery file in Windows File Explorer.
        </li>
        <li>
          <strong>Edit metadata:</strong> Add or remove tags, assign star ratings, mark as stitched, or
          write personal project notes.
        </li>
      </ul>
      <p class="pt-2 border-t border-[var(--border-default)] text-xs">
        <a
          href="#/about/document/design-detail"
          class="text-[var(--text-brand)] hover:underline font-medium"
        >
          Read guide → Design Detail Guide
        </a>
      </p>
    </div>
  </section>

  <section id="designers-sources">
    <h2 class={sectionHeadingClass}>🏷️ Designers, Sources & Hoops</h2>
    <p class="text-sm text-[var(--text-secondary)] mb-3">
      Organise your catalogue by creator, supplier, and machine hoop compatibility.
    </p>
    <div class="route-card rounded shadow p-4 space-y-3 text-sm">
      <ul class="list-disc list-outside pl-5 space-y-2 text-[var(--text-secondary)]">
        <li>
          <strong>First import setup:</strong> Set up your machine hoops and initial designers before
          importing so designs are automatically categorised.
        </li>
        <li>
          <strong>Assign per folder:</strong> During import, assign Designers and Sources to entire scanned
          folders to keep your catalogue structured.
        </li>
        <li>
          <strong>Manage creators &amp; suppliers:</strong> Add, rename, or delete designers in
          <a href="#/admin/data/designers" class="text-[var(--text-brand)] hover:underline"
            >Designers</a
          >
          and acquisition sources in
          <a href="#/admin/data/sources" class="text-[var(--text-brand)] hover:underline">Sources</a
          >.
        </li>
        <li>
          <strong>Machine hoop compatibility:</strong> Enter your machine's hoop sizes in
          <a href="#/admin/data/hoops" class="text-[var(--text-brand)] hover:underline">Hoops</a> to easily
          filter designs that fit your machine.
        </li>
      </ul>
      <div
        class="pt-2 border-t border-[var(--border-default)] flex flex-wrap gap-x-6 gap-y-1 text-xs"
      >
        <a
          href="#/about/document/first-import-actions"
          class="text-[var(--text-brand)] hover:underline font-medium"
        >
          Read guide → First Import Actions Guide
        </a>
        <a
          href="#/about/document/import-folder-assignment"
          class="text-[var(--text-brand)] hover:underline font-medium"
        >
          Read guide → Import Folder Assignment Guide
        </a>
      </div>
    </div>
  </section>

  <section id="importing">
    <h2 class={sectionHeadingClass}>📥 Importing</h2>
    <p class="text-sm text-[var(--text-secondary)] mb-3">
      Add digital embroidery designs to your catalogue using <a
        href="#/import"
        class="text-[var(--text-brand)] hover:underline">Import</a
      >. Original files are never modified.
    </p>
    <div class="route-card rounded shadow p-4 space-y-3 text-sm">
      <ul class="list-disc list-outside pl-5 space-y-2 text-[var(--text-secondary)]">
        <li>
          <strong>Choosing folders:</strong> Add one or more source folders and click
          <em>Scan folder(s)</em>. Subfolders are scanned automatically.
        </li>
        <li>
          <strong>Live scan progress:</strong> Live counters show folders and files found, and existing
          catalogue duplicates are skipped.
        </li>
        <li>
          <strong>Review and metadata:</strong> Review scanned files grouped by folder, toggle designs,
          and assign Designer or Source per folder or globally.
        </li>
        <li>
          <strong>Tag check before import:</strong> The <em>Before You Import</em> step lets you review
          hoop setup and tags before confirming.
        </li>
        <li>
          <strong>Tagging during import:</strong> Imports run fast, local File &amp; Folder Rules. Optional
          Gemini Vision tagging can be run afterwards in Batch Operations.
        </li>
        <li>
          <strong>Alphabet &amp; font compression:</strong> Scanned alphabet collections are automatically
          compressed into companion zip archives, cataloguing representative sample glyphs only.
        </li>
      </ul>
      <p class="pt-2 border-t border-[var(--border-default)] text-xs">
        <a
          href="#/about/document/import-workflow"
          class="text-[var(--text-brand)] hover:underline font-medium"
        >
          Read full guide → Import Workflow Guide
        </a>
      </p>
    </div>
  </section>

  <section id="word-matches">
    <h2 class={sectionHeadingClass}>🔤 Tag Word Matches</h2>
    <p class="text-sm text-[var(--text-secondary)] mb-3">
      Map words that appear in filenames and folder paths to catalogue tags for automatic offline
      tagging.
    </p>
    <div class="route-card rounded shadow p-4 space-y-3 text-sm">
      <ul class="list-disc list-outside pl-5 space-y-2 text-[var(--text-secondary)]">
        <li>
          <strong>Offline &amp; automatic:</strong> Runs locally during import and batch tagging without
          an internet connection or API key.
        </li>
        <li>
          <strong>Smart matching:</strong> Whole word boundaries, case-insensitivity, and English singular/plural
          forms are handled automatically.
        </li>
        <li>
          <strong>Folder hierarchies &amp; supplier masking:</strong> Supplier and designer names in folder
          paths are automatically excluded from tag matching.
        </li>
        <li>
          <strong>Manage matches:</strong> Add and remove keyword matches in
          <a href="#/admin/data/tag-matches" class="text-[var(--text-brand)] hover:underline"
            >Word Matches</a
          >
          or via the <em>Matches</em> button in
          <a href="#/admin/data/tags" class="text-[var(--text-brand)] hover:underline">Tags</a>.
        </li>
      </ul>
      <p class="pt-2 border-t border-[var(--border-default)] text-xs">
        <a
          href="#/about/document/tag-word-matches"
          class="text-[var(--text-brand)] hover:underline font-medium"
        >
          Read full guide → Tag Word Matches Guide
        </a>
      </p>
    </div>
  </section>

  <section id="storage">
    <h2 class={sectionHeadingClass}>💾 Data Storage & External Drives</h2>
    <p class="text-sm text-[var(--text-secondary)] mb-3">
      Host your catalogue database and thumbnail cache on your internal drive, an SD card, or an
      external USB drive.
    </p>
    <div class="route-card rounded shadow p-4 space-y-3 text-sm">
      <ul class="list-disc list-outside pl-5 space-y-2 text-[var(--text-secondary)]">
        <li>
          <strong>Original files untouched:</strong> The application never modifies, moves, or renames
          your original embroidery files.
        </li>
        <li>
          <strong>SD card recommendation:</strong> Use an <strong>A2</strong> (Application Performance
          Class 2) SD card for fast thumbnail access.
        </li>
        <li>
          <strong>Filesystem formatting:</strong> Use <strong>exFAT</strong> or
          <strong>NTFS</strong>. Avoid FAT32 formatting because FAT32 cannot store single files of 4
          GB or larger.
        </li>
        <li>
          <strong>Keep drives connected:</strong> Reconnect external drives before launching the app;
          the Startup Recovery screen lets you re-link with one click if a drive letter changes.
        </li>
      </ul>
      <p class="pt-2 border-t border-[var(--border-default)] text-xs">
        <a
          href="#/about/document/data-storage"
          class="text-[var(--text-brand)] hover:underline font-medium"
        >
          Read full guide → Data Storage Guide
        </a>
      </p>
    </div>
  </section>

  <section id="ai-tagging">
    <h2 class={sectionHeadingClass}>🤖 AI Tagging</h2>
    <p class="text-sm text-[var(--text-secondary)] mb-3">
      Optionally use Google Gemini Vision to analyse preview images and suggest subject matter tags.
    </p>
    <div class="route-card rounded shadow p-4 space-y-3 text-sm">
      <ul class="list-disc list-outside pl-5 space-y-2 text-[var(--text-secondary)]">
        <li>
          <strong>Get an API key:</strong> Create an API key in
          <a
            href="https://aistudio.google.com/"
            class="text-[var(--text-brand)] hover:underline"
            target="_blank"
            rel="noopener">Google AI Studio</a
          >, check
          <a
            href="https://ai.google.dev/pricing"
            class="text-[var(--text-brand)] hover:underline"
            target="_blank"
            rel="noopener">pricing</a
          >, and save it in
          <a href="#/admin/system/settings" class="text-[var(--text-brand)] hover:underline"
            >Settings</a
          >.
        </li>
        <li>
          <strong>Free tier protection:</strong> Tick <em>My Google API key is on the free tier</em> in
          Settings to protect against rate limits.
        </li>
        <li>
          <strong>In-app actions:</strong> AI tagging runs on demand in
          <a href="#/admin/batch-operations" class="text-[var(--text-brand)] hover:underline"
            >Batch Operations</a
          > rather than during import.
        </li>
      </ul>
      <p class="pt-2 border-t border-[var(--border-default)] text-xs">
        <a
          href="#/about/document/ai-tagging"
          class="text-[var(--text-brand)] hover:underline font-medium"
        >
          Read full guide → AI Tagging &amp; Batch Operations Guide
        </a>
      </p>
    </div>
  </section>

  <section id="batch-operations">
    <h2 class={sectionHeadingClass}>🏷 Batch Operations</h2>
    <p class="text-sm text-[var(--text-secondary)] mb-3">
      Run bulk tagging and file maintenance across your collection in <a
        href="#/admin/batch-operations"
        class="text-[var(--text-brand)] hover:underline">Admin → Batch Operations</a
      >.
    </p>
    <div class="route-card rounded shadow p-4 space-y-3 text-sm">
      <ul class="list-disc list-outside pl-5 space-y-2 text-[var(--text-secondary)]">
        <li>
          <strong>Tagging scopes:</strong> Target untagged designs only, specific folders, or your entire
          collection without overwriting verified tags.
        </li>
        <li>
          <strong>Local stitching backfill:</strong> Detect stitch types (e.g. Satin Stitch, Applique,
          Lace) from pattern geometry.
        </li>
        <li>
          <strong>File maintenance:</strong> Regenerate preview thumbnails, refresh thread colour and
          stitch totals, or update hoop matching.
        </li>
        <li>
          <strong>Tag verification workflow:</strong> Automated tagging clusters similar designs together
          so you can verify whole pages in Browse.
        </li>
      </ul>
      <p class="pt-2 border-t border-[var(--border-default)] text-xs">
        <a
          href="#/about/document/ai-tagging"
          class="text-[var(--text-brand)] hover:underline font-medium"
        >
          Read full guide → Batch Operations Guide
        </a>
      </p>
    </div>
  </section>

  <section id="projects">
    <h2 class={sectionHeadingClass}>📁 Projects</h2>
    <p class="text-sm text-[var(--text-secondary)] mb-3">
      <a href="#/projects" class="text-[var(--text-brand)] hover:underline">Projects</a> let you group
      designs together for planned embroidery tasks or customer orders.
    </p>
    <div class="route-card rounded shadow p-4 space-y-3 text-sm">
      <ul class="list-disc list-outside pl-5 space-y-2 text-[var(--text-secondary)]">
        <li>
          <strong>What projects are for:</strong> Collect and organise designs for a specific sewing session
          or gift set.
        </li>
        <li>
          <strong>Adding designs:</strong> Add single designs from their detail page or add multiple designs
          in bulk from Browse.
        </li>
        <li>
          <strong>Printing:</strong> Print a clean project worksheet with thumbnails, dimensions,
          and colour counts using the <strong>Print Sheet</strong> button.
        </li>
        <li>
          <strong>Safe removal:</strong> Removing designs or deleting a project never deletes the underlying
          design records or files.
        </li>
      </ul>
      <p class="pt-2 border-t border-[var(--border-default)] text-xs">
        <a
          href="#/about/document/projects-guide"
          class="text-[var(--text-brand)] hover:underline font-medium"
        >
          Read full guide → Projects Guide
        </a>
      </p>
    </div>
  </section>

  <section id="maintenance">
    <h2 class={sectionHeadingClass}>🛠 Maintenance</h2>
    <p class="text-sm text-[var(--text-secondary)] mb-3">
      Keep your catalogue synchronized and reconcile files in <a
        href="#/admin/system/discrepancies"
        class="text-[var(--text-brand)] hover:underline">Library Discrepancies</a
      >.
    </p>
    <div class="route-card rounded shadow p-4 space-y-3 text-sm">
      <ul class="list-disc list-outside pl-5 space-y-2 text-[var(--text-secondary)]">
        <li>
          <strong>Find Unmatched Design Files:</strong> Scan for physical files on disk that have no catalogue
          record, with options to import them.
        </li>
        <li>
          <strong>Missing Design Files (Orphans):</strong> Scan the database for records whose files were
          deleted or moved outside the app.
        </li>
        <li>
          <strong>Deleting orphans:</strong> Removes orphaned database records only; physical files on
          disk are never touched.
        </li>
        <li>
          <strong>Backups &amp; Restore:</strong> Create online database backups and incremental
          design mirrors in
          <a href="#/admin/system/backup" class="text-[var(--text-brand)] hover:underline">Backup</a
          >.
        </li>
      </ul>
      <p class="pt-2 border-t border-[var(--border-default)] text-xs">
        <a
          href="#/about/document/backup-restore"
          class="text-[var(--text-brand)] hover:underline font-medium"
        >
          Read full guide → Backup &amp; Restore Guide
        </a>
      </p>
    </div>
  </section>

  <section id="settings">
    <h2 class={sectionHeadingClass}>⚙️ Settings & Configuration</h2>
    <p class="text-sm text-[var(--text-secondary)] mb-3">
      Manage preferences, appearance, database maintenance, and format rules in <a
        href="#/admin/system/settings"
        class="text-[var(--text-brand)] hover:underline">Admin → Settings</a
      >.
    </p>
    <div class="route-card rounded shadow p-4 space-y-3 text-sm">
      <ul class="list-disc list-outside pl-5 space-y-2 text-[var(--text-secondary)]">
        <li>
          <strong>Appearance:</strong> Switch between System Default, Light, and Dark themes instantly.
        </li>
        <li>
          <strong>Google Gemini &amp; AI Tagging:</strong> Store API keys, select models, and tune worker
          concurrency.
        </li>
        <li>
          <strong>Database Maintenance:</strong> Reclaim disk space using
          <em>Optimise &amp; Compact Database</em>.
        </li>
        <li>
          <strong>Digitising &amp; Format Rules:</strong> Set your preferred machine stitch format, enable
          software master outline formats, and configure alphabet font compression.
        </li>
        <li>
          <strong>Catalogue Storage:</strong> Move your catalogue database and designs folder with safe
          live migration.
        </li>
        <li>
          <strong>Product licence:</strong> Manage licence activation in
          <a href="#/about" class="text-[var(--text-brand)] hover:underline">About</a>, or view
          open-source licences in
          <a href="#/about/licence" class="text-[var(--text-brand)] hover:underline"
            >Licence &amp; Dependencies</a
          >.
        </li>
      </ul>
      <p class="pt-2 border-t border-[var(--border-default)] text-xs">
        <a
          href="#/about/document/settings-guide"
          class="text-[var(--text-brand)] hover:underline font-medium"
        >
          Read full guide → Settings Guide
        </a>
      </p>
    </div>
  </section>

  <section id="troubleshooting">
    <h2 class={sectionHeadingClass}>🔧 Troubleshooting</h2>
    <div class="route-card rounded shadow p-4 space-y-3 text-sm">
      <ul class="list-disc list-outside pl-5 space-y-2 text-[var(--text-secondary)]">
        <li>
          <strong>Missing folder / changed drive letter:</strong> Reconnect your drive; the Startup Recovery
          screen will help you re-link to the new location in one click.
        </li>
        <li>
          <strong>Database preparation on startup:</strong> If database indexes are being upgraded, leave
          the window open until the main view appears.
        </li>
        <li>
          <strong>Database full / 4 GB limit:</strong> If you see disk full errors, your drive may be
          formatted as FAT32. Back up and reformat the drive to exFAT or NTFS.
        </li>
        <li>
          <strong>AI tagging errors (429):</strong> If you see "429 Too Many Requests", tick
          <em>My Google API key is on the free tier</em> in Settings.
        </li>
        <li>
          <strong>Import scan finds nothing:</strong> Verify that the selected folder contains supported
          embroidery stitch formats.
        </li>
      </ul>
      <p class="pt-2 border-t border-[var(--border-default)] text-xs">
        <a
          href="#/about/document/getting-started"
          class="text-[var(--text-brand)] hover:underline font-medium"
        >
          Read full guide → Getting Started &amp; Setup Guide
        </a>
      </p>
    </div>
  </section>
</div>
