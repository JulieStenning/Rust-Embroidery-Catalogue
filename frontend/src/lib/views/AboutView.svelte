<script>
  let { documents = [], loading = false, error = "" } = $props();
</script>

<div class="max-w-4xl mx-auto space-y-6">
  <div class="bg-white rounded-xl shadow p-6 space-y-4">
    <h1 class="text-2xl font-bold text-gray-800">About Embroidery Catalogue</h1>

    <div class="space-y-3 text-sm text-gray-600">
      <p>
        Embroidery Catalogue is a local, offline tool for cataloguing and browsing an embroidery design collection.
      </p>
      <p>
        Optional AI features use Google Gemini and require an API key configured in
        <a href="#/admin/settings" class="text-indigo-600 hover:underline">Settings</a>.
      </p>
      <p>
        Generated tags are suggestions and should be verified.
      </p>
    </div>

    <div class="rounded-lg border border-indigo-200 bg-indigo-50 p-4 text-sm text-indigo-900">
      <p class="font-medium">Licence</p>
      <p class="mt-1">Open the project licence document for full terms.</p>
      <p class="mt-2">
        <a href="#/about/document/licence" class="underline hover:no-underline">Open licence text</a>
      </p>
    </div>
  </div>

  {#if error}
    <div class="bg-red-50 border border-red-300 text-red-700 rounded px-4 py-3 text-sm">
      {error}
    </div>
  {/if}

  <div class="bg-white rounded-xl shadow divide-y divide-gray-100">
    {#if loading}
      <div class="p-5 text-sm text-gray-500">Loading documents...</div>
    {:else if !Array.isArray(documents) || documents.length === 0}
      <div class="p-5 text-sm text-gray-500">No about documents are configured.</div>
    {:else}
      {#each documents as doc}
        <div class="p-5 flex items-start justify-between gap-4">
          <div>
            <h2 class="text-lg font-semibold text-gray-800">{doc.title}</h2>
            <p class="text-sm text-gray-600">{doc.description}</p>
          </div>
          {#if doc.available}
            <a href={`#/about/document/${doc.slug}`} class="shrink-0 bg-indigo-600 text-white px-3 py-2 rounded text-sm hover:bg-indigo-700">
              Open
            </a>
          {:else}
            <span class="shrink-0 text-sm text-red-600">Not found</span>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>
