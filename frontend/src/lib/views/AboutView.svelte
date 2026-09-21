<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  const APP_NAME = "Embroidery Catalogue";
  const APP_VERSION = "0.1.0";

  /**
   * The verbatim MIT notice required by pyembroidery's licence. It is reproduced
   * in full here (and in the repo-root `NOTICE` file) because the MIT License
   * requires its copyright and permission notices to be retained in derived
   * works. Note the upstream notice's copyright line carries no name — that is
   * how it appears in pyembroidery's own LICENSE file, so it is kept verbatim.
   */
  const PYEMBROIDERY_MIT_NOTICE = `MIT License

Copyright (c) 2018

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.`;
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
      Embroidery Catalogue is a local, desktop catalogue for browsing, tagging, and managing a
      collection of digital embroidery designs. It reads a broad range of embroidery formats
      directly via built-in readers (<code>.jef</code>, <code>.pes</code>, <code>.hus</code>,
      <code>.vp3</code>, <code>.dst</code>, <code>.exp</code>) and stores all catalogue data in a
      local database. An Internet connection is required for advanced tagging.
    </p>

    <div class="bg-gray-50 border border-gray-200 rounded-lg p-4 text-sm space-y-1">
      <h2 class="ui-section-label font-semibold text-gray-850">Primary Copyright</h2>
      <p class="text-gray-700">Copyright (C) 2026 Julie Stenning</p>
    </div>
  </div>

  <div class="bg-white rounded-xl shadow p-6 space-y-4">
    <h2 class="text-lg font-bold text-gray-800 font-sans">
      Acknowledgements & Code Porting Attributions
    </h2>
    <div class="space-y-4 text-sm">
      <div class="bg-gray-50 border border-gray-200 rounded-lg p-4">
        <h3 class="ui-section-label font-semibold text-gray-850 mb-1">
          pyembroidery (MIT License)
        </h3>
        <p class="text-gray-700">
          Binary parsing routines for <code>.pes</code>, <code>.jef</code>, <code>.vp3</code>,
          <code>.hus</code>, <code>.dst</code> and <code>.exp</code>, together with the PNG preview
          rendering logic derived from pyembroidery's PngWriter module, were ported/derived into
          native Rust modules from
          <a
            href="https://github.com/EmbroidePy/pyembroidery"
            target="_blank"
            rel="noopener noreferrer"
            class="text-indigo-600 hover:underline font-medium break-all"
            >https://github.com/EmbroidePy/pyembroidery</a
          >.
        </p>
        <p class="text-gray-700 mt-2">
          Copyright (c) 2018 Tatarize and the EmbroidePy pyembroidery contributors.
        </p>
        <p class="text-gray-700 mt-2">
          Our thanks go to Tatarize and the EmbroidePy pyembroidery contributors for publishing
          their work under a permissive licence, and for providing the reference implementations of
          the embroidery file formats that made this application possible.
        </p>
        <details class="mt-3" data-testid="pyembroidery-mit-notice">
          <summary
            class="cursor-pointer ui-section-label font-semibold text-indigo-600 hover:underline"
          >
            View the MIT License notice required by pyembroidery
          </summary>
          <pre
            class="whitespace-pre-wrap text-xs text-gray-700 bg-white border border-gray-200 rounded-lg p-3 mt-2 overflow-x-auto font-mono">{PYEMBROIDERY_MIT_NOTICE}</pre>
        </details>
      </div>
    </div>
  </div>

  <div class="bg-white rounded-xl shadow p-6 flex items-center justify-between gap-4">
    <p class="text-sm text-gray-600">
      The full application licence, Rust crate credits, and frontend package credits are available
      here.
    </p>
    <a
      href="#/about/licence"
      class="shrink-0 bg-indigo-600 text-white px-4 py-2.5 rounded text-sm hover:bg-indigo-700 font-medium shadow-sm transition"
    >
      View Full Licences
    </a>
  </div>
</div>
