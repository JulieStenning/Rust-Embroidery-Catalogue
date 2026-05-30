<script>
  let { documentItem = null, loading = false, error = "" } = $props();
</script>

<div class="max-w-5xl mx-auto space-y-4">
  <div class="flex items-center justify-between gap-4">
    <div>
      <h1 class="ui-page-title">{documentItem?.title || "About Document"}</h1>
      {#if documentItem?.filename}
        <p class="text-sm text-gray-600">Source file: <code>{documentItem.filename}</code></p>
      {/if}
    </div>
    <a href="#/about" class="text-sm text-indigo-600 hover:underline">← Back to About</a>
  </div>

  {#if error}
    <div class="bg-red-50 border border-red-300 text-red-700 rounded px-4 py-3 text-sm">
      {error}
    </div>
  {/if}

  <div class="bg-white rounded-xl shadow p-6">
    {#if loading}
      <p class="text-sm text-gray-500">Loading document...</p>
    {:else if documentItem?.document_text}
      <pre class="whitespace-pre-wrap text-sm text-gray-700 bg-gray-50 border rounded-lg p-4 overflow-x-auto">{documentItem.document_text}</pre>
    {:else}
      <p class="text-sm text-gray-500">Document content is unavailable.</p>
    {/if}
  </div>
</div>
