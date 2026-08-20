<script>
  const APP_NAME = "Embroidery Catalogue";
  const APP_VERSION = "0.1.0";
</script>

<div class="max-w-4xl mx-auto space-y-6 font-sans">
  <div class="bg-white rounded-xl shadow p-6 space-y-4">
    <div class="flex items-center gap-4">
      <span class="text-4xl" aria-hidden="true">🧵</span>
      <div>
        <h1 class="text-2xl font-bold text-gray-800 font-sans">{APP_NAME}</h1>
        <p class="text-sm text-gray-500">Version v{APP_VERSION}</p>
      </div>
    </div>

    <p class="text-sm text-gray-700">
      Embroidery Catalogue is a local, offline desktop catalogue for browsing, tagging, and managing a
      collection of digital embroidery designs. It reads a broad range of embroidery formats directly
      via built-in readers, including <code>.jef</code>, <code>.pes</code>, <code>.hus</code>,
      <code>.vp3</code>, <code>.dst</code>, <code>.exp</code>, <code>.sew</code>, and many more, and
      stores all catalogue data in a local database — no internet connection is required for normal use.
    </p>

    <div class="bg-gray-50 border border-gray-200 rounded-lg p-4 text-sm space-y-1">
      <h2 class="ui-section-label font-semibold text-gray-850">Primary Copyright</h2>
      <p class="text-gray-700">Copyright (C) 2026 Julie Stenning</p>
    </div>
  </div>

  <div class="bg-white rounded-xl shadow p-6 space-y-4">
    <h2 class="text-lg font-bold text-gray-800 font-sans">Acknowledgements & Code Porting Attributions</h2>
    <div class="space-y-4 text-sm">
      <div class="bg-gray-50 border border-gray-200 rounded-lg p-4">
        <h3 class="ui-section-label font-semibold text-gray-850 mb-1">pyembroidery (MIT License)</h3>
        <p class="text-gray-700">
          Binary parsing routines for <code>.pes</code>, <code>.jef</code>, <code>.vp3</code>,
          <code>.hus</code>, <code>.dst</code> and <code>.exp</code> and png_writerwere ported/derived into native Rust modules
          from
          <a
            href="https://github.com/EmbroidePy/pyembroidery"
            target="_blank"
            rel="noopener noreferrer"
            class="text-indigo-600 hover:underline font-medium break-all"
          >https://github.com/EmbroidePy/pyembroidery</a>.
        </p>
      </div>
     
    </div>
  </div>

  <div class="bg-white rounded-xl shadow p-6 flex items-center justify-between gap-4">
    <p class="text-sm text-gray-600">
      The full application licence, Rust crate credits, and frontend package credits are available here.
    </p>
    <a
      href="#/about/licence"
      class="shrink-0 bg-indigo-600 text-white px-4 py-2.5 rounded text-sm hover:bg-indigo-700 font-medium shadow-sm transition"
    >
      View Full Licences
    </a>
  </div>
</div>