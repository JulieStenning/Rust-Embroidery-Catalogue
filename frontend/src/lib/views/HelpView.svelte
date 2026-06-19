<div class="space-y-10 max-w-3xl">
   <nav class="flex flex-wrap gap-3 text-sm no-print">
    <a href="#search" class="bg-indigo-50 text-indigo-700 px-3 py-1 rounded hover:bg-indigo-100">🔍 Search</a>
    <a href="#importing" class="bg-indigo-50 text-indigo-700 px-3 py-1 rounded hover:bg-indigo-100">📥 Importing</a>
    <a href="#ai-tagging" class="bg-indigo-50 text-indigo-700 px-3 py-1 rounded hover:bg-indigo-100">🤖 AI Tagging</a>
    <a href="#tagging-actions" class="bg-indigo-50 text-indigo-700 px-3 py-1 rounded hover:bg-indigo-100">🏷 Tagging Actions</a>
    <a href="#projects" class="bg-indigo-50 text-indigo-700 px-3 py-1 rounded hover:bg-indigo-100">📁 Projects</a>
    <a href="#maintenance" class="bg-indigo-50 text-indigo-700 px-3 py-1 rounded hover:bg-indigo-100">🛠 Maintenance</a>
    <a href="#troubleshooting" class="bg-indigo-50 text-indigo-700 px-3 py-1 rounded hover:bg-indigo-100">🔧 Troubleshooting</a>
  </nav>

  <section id="search">
    <h2 class="text-xl font-semibold mb-3 border-b pb-1">🔍 Search</h2>
    <p class="text-sm text-gray-700 mb-3">
      Use the general search box on the <a href="#/designs" class="text-indigo-600 hover:underline">Browse</a> page to find designs by keyword.
      The search checks the filename, notes, designer, source, and tags.
    </p>
    <div class="bg-white rounded shadow p-4 space-y-3 text-sm">
      <p><strong>Quoted phrases:</strong> <code class="bg-gray-100 px-1 rounded">"cross stitch"</code></p>
      <p><strong>Exclude terms:</strong> <code class="bg-gray-100 px-1 rounded">rose -applique</code></p>
      <p><strong>OR searches:</strong> <code class="bg-gray-100 px-1 rounded">rose OR tulip</code></p>
      <p><strong>Filename wildcards:</strong> <code class="bg-gray-100 px-1 rounded">rose*.jef</code></p>
      <p><strong>Unverified only:</strong> show designs that still need metadata review.</p>
      <p><strong>Quick search vs. filters:</strong> use Additional filters for exact phrase, any words, none words, designer, hoop, and more.</p>
    </div>
  </section>

  <section id="importing">
    <h2 class="text-xl font-semibold mb-3 border-b pb-1">📥 Importing</h2>
    <p class="text-sm text-gray-700 mb-3">
      Use <a href="#/import" class="text-indigo-600 hover:underline">Import</a> to scan one or more folders and their sub-folders.
    </p>
    <div class="bg-white rounded shadow p-4 space-y-3 text-sm">
      <p><strong>Choosing folders:</strong> enter paths, use Browse, or add multiple folders; sub-folders are scanned automatically.</p>
      <p><strong>Review and metadata:</strong> set Designer and Source per folder before import.</p>
      <p><strong>Tag check before import:</strong> first import asks for tag review, later imports can continue directly.</p>
      <p><strong>AI tagging notice:</strong> blue notice for Tier 1 only; amber notice when API key is configured.</p>
      <p><strong>Error files and large scans:</strong> unreadable files are skipped; large scans may summarise rows.</p>
    </div>
  </section>

  <section id="ai-tagging">
    <h2 class="text-xl font-semibold mb-3 border-b pb-1">🤖 AI Tagging</h2>
    <p class="text-sm text-gray-700 mb-3">
      Optional Google AI tagging can suggest design-type tags for imported embroidery files.
    </p>
    <div class="bg-white rounded shadow p-4 space-y-3 text-sm">
      <p><strong>Get an API key:</strong> <a href="https://aistudio.google.com/" class="text-indigo-600 hover:underline">Google AI Studio</a>, then save it in <a href="#/admin/settings" class="text-indigo-600 hover:underline">Settings</a>.</p>
      <p><strong>Enable tiers:</strong> in Settings, choose Tier 2 and Tier 3 auto-run options for import.</p>
      <p><strong>Batch size and delay:</strong> set limits and delay to reduce 429 rate-limit errors.</p>
      <p><strong>In-app actions:</strong> use <a href="#/admin/tagging-actions" class="text-indigo-600 hover:underline">Admin → Tagging Actions</a> for existing designs.</p>
      <p><strong>Potential costs:</strong> review <a href="https://ai.google.dev/pricing" class="text-indigo-600 hover:underline">current pricing</a>.</p>
      <p><strong>Full guide:</strong> open the <a href="#/about/document/ai-tagging" class="text-indigo-600 hover:underline">AI Tagging Guide</a>.</p>
    </div>
  </section>

  <section id="tagging-actions">
    <h2 class="text-xl font-semibold mb-3 border-b pb-1">🏷 Tagging Actions</h2>
    <p class="text-sm text-gray-700 mb-3">
      Run AI tagging on existing designs from <a href="#/admin/tagging-actions" class="text-indigo-600 hover:underline">Admin → Tagging Actions</a>.
    </p>
    <div class="bg-white rounded shadow p-4 space-y-3 text-sm">
      <p><strong>Tag only untagged designs:</strong> safe; does not overwrite verified tags.</p>
      <p><strong>Tag untagged and unverified designs:</strong> includes unverified suggestions.</p>
      <p><strong>Re-tag ALL designs:</strong> overwrites everything and requires confirmation.</p>
      <p><strong>Local stitching backfill:</strong> updates stitching group tags using local pattern geometry.</p>
    </div>
  </section>

  <section id="projects">
    <h2 class="text-xl font-semibold mb-3 border-b pb-1">📁 Projects</h2>
    <p class="text-sm text-gray-700 mb-3">
      <a href="#/projects" class="text-indigo-600 hover:underline">Projects</a> let you group designs for planned embroidery tasks.
    </p>
    <div class="bg-white rounded shadow p-4 space-y-3 text-sm">
      <p><strong>What projects are for:</strong> collect designs for a specific plan or session.</p>
      <p><strong>Adding designs:</strong> use Add to Project from design detail.</p>
      <p><strong>Bulk add:</strong> on Browse, select designs and use the bottom action bar project selector.</p>
      <p><strong>Printing:</strong> each project includes a print view.</p>
    </div>
  </section>

  <section id="maintenance">
    <h2 class="text-xl font-semibold mb-3 border-b pb-1">🛠 Maintenance</h2>
    <p class="text-sm text-gray-700 mb-3">
      Use <a href="#/admin/orphans" class="text-indigo-600 hover:underline">Orphans</a> to find records whose files are missing.
    </p>
    <div class="bg-white rounded shadow p-4 space-y-3 text-sm">
      <p><strong>What orphaned records are:</strong> database records where design files are no longer present.</p>
      <p><strong>Deleting orphans:</strong> removes database records only, not physical files.</p>
      <p><strong>Use carefully:</strong> verify files are truly gone before deleting records.</p>
    </div>
  </section>

  <section id="troubleshooting">
    <h2 class="text-xl font-semibold mb-3 border-b pb-1">🔧 Troubleshooting</h2>
    <div class="bg-white rounded shadow p-4 space-y-3 text-sm">
      <p><strong>Missing folder / changed drive letter:</strong> ensure the full data folder moved with the app.</p>
      <p><strong>Import scan finds nothing:</strong> confirm folder path and supported formats.</p>
      <p><strong>Files missing after import:</strong> run orphan cleanup, then re-import from new location.</p>
      <p><strong>Managed storage location:</strong> imported files live under data/MachineEmbroideryDesigns.</p>
      <p><strong>Still stuck:</strong> check permissions and review application logs.</p>
    </div>
  </section>
</div>
