<script>
  let { documentItem = null, loading = false, error = "" } = $props();

  function shouldRenderAsHtml(item) {
    if (!item || typeof item !== "object") {
      return false;
    }

    const slug = String(item.slug || "").toLowerCase();
    const filename = String(item.filename || "").toLowerCase();
    return slug === "disclaimer" || filename.endsWith(".html");
  }
</script>

<div class="max-w-5xl mx-auto space-y-4">
  <div class="bg-white rounded-xl shadow p-6">
    {#if loading}
      <p class="text-sm text-gray-500">Loading document...</p>
    {:else if documentItem?.document_text}
      {#if shouldRenderAsHtml(documentItem)}
        <div class="text-sm text-gray-700 bg-gray-50 border rounded-lg p-4 space-y-4">
          {@html documentItem.document_text}
        </div>
      {:else}
        <pre class="whitespace-pre-wrap text-sm text-gray-700 bg-gray-50 border rounded-lg p-4 overflow-x-auto">{documentItem.document_text}</pre>
      {/if}
    {:else}
      <p class="text-sm text-gray-500">Document content is unavailable.</p>
    {/if}
  </div>
</div>
