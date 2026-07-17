<script>
  import { onMount } from "svelte";
  import { getAboutDocuments } from "../api/commandAdapter.js";

  /** @type {any[]} */
  let aboutDocuments = $state([]);
  let aboutDocumentsLoading = $state(false);
  let aboutDocumentsError = $state("");

  async function loadAboutDocuments(force = false) {
    if (aboutDocumentsLoading && !force) return;

    aboutDocumentsLoading = true;
    aboutDocumentsError = "";

    try {
      const result = await getAboutDocuments();
      aboutDocuments = Array.isArray(result?.items) ? result.items : [];
    } catch (error) {
      aboutDocuments = [];
      aboutDocumentsError = `Could not load about documents: ${error}`;
    } finally {
      aboutDocumentsLoading = false;
    }
  }

  onMount(() => {
    loadAboutDocuments();
  });
</script>

<div class="max-w-4xl mx-auto space-y-6 font-sans">
  <div class="bg-white rounded-xl shadow p-6 space-y-4">
    <h1 class="text-2xl font-bold text-gray-800 font-sans">About Embroidery Catalogue</h1>

    <div class="space-y-3 text-sm">
      <div>
        <p class="ui-section-label font-semibold text-gray-850">What this app is</p>
        <p class="text-gray-600">
          Embroidery Catalogue is a local, offline tool for cataloguing and browsing an embroidery
          design collection. It supports a broad range of pyembroidery-readable embroidery formats,
          including <code>.jef</code>, <code>.pes</code>, <code>.hus</code>, <code>.vp3</code>,
          <code>.dst</code>, <code>.exp</code>, <code>.sew</code>, <code>.u01</code>, and many more,
          with limited support for <code>.art</code>. It stores all data in a local database file — no
          internet connection is required for normal use.
        </p>
      </div>
      <div>
        <p class="ui-section-label font-semibold text-gray-850">Where data is stored</p>
        <p class="text-gray-600">
          The catalogue database and any generated preview images are stored locally on your machine.
          Your embroidery files are not moved or modified — the catalogue only reads them to extract
          metadata and generate thumbnail previews.
        </p>
      </div>
      <div>
        <p class="ui-section-label font-semibold text-gray-850">AI / Gemini features</p>
        <p class="text-gray-600">
          Some optional features use the Google Gemini API for automatic tagging and description
          generation. These features require an internet connection and a valid API key configured
          in <a href="#/admin/settings" class="text-indigo-600 hover:underline font-medium">Settings</a>.
          Tier 2 (text AI) and Tier 3 (vision AI) tagging during import can be enabled or
          disabled independently in Settings. They are entirely optional and the catalogue works
          fully without them using Tier 1 keyword tagging.
        </p>
      </div>
      <div>
        <p class="ui-section-label font-semibold text-gray-850">A note on accuracy</p>
        <p class="text-gray-600">
          Automatically generated tags and metadata should be treated as suggestions. Always verify
          results before relying on them, especially for important cataloguing decisions.
        </p>
      </div>
    </div>

    <p class="text-sm text-gray-600">
      This page also provides quick access to the app's legal, privacy, security, and licensing documents.
    </p>

    <div class="rounded-lg border border-indigo-200 bg-indigo-50 p-4 text-sm text-indigo-900 shadow-sm">
      <p class="ui-section-label font-semibold">Licence</p>
      <p class="mt-1">
        This repository is licensed under <strong>AGPL-3.0-or-later</strong>.
      </p>
      <p class="mt-2">
        <a href="#/about/document/licence" class="underline hover:no-underline font-medium">Open licence text </a>
      </p>
    </div>
  </div>

  {#if aboutDocumentsError}
    <div class="bg-red-50 border border-red-300 text-red-700 rounded px-4 py-3 text-sm">
      {aboutDocumentsError}
    </div>
  {/if}

  <div class="bg-white rounded-xl shadow divide-y divide-gray-100">
    {#if aboutDocumentsLoading}
      <div class="p-5 text-sm text-gray-500">Loading documents...</div>
    {:else if aboutDocuments.length === 0}
      <div class="p-5 text-sm text-gray-500">No about documents are configured.</div>
    {:else}
      {#each aboutDocuments as doc}
        <div class="p-5 flex items-start justify-between gap-4">
          <div>
            <h2 class="text-lg font-bold text-gray-800 font-sans">{doc.title}</h2>
            <p class="text-sm text-gray-600 mt-1">{doc.description}</p>
          </div>
          {#if doc.available}
            <a href={`#/about/document/${doc.slug}`} class="shrink-0 bg-indigo-600 text-white px-3.5 py-2 rounded text-sm hover:bg-indigo-700 font-medium shadow-sm transition">
              Open
            </a>
          {:else}
            <span class="shrink-0 text-sm text-red-650 font-semibold">Not found</span>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>
