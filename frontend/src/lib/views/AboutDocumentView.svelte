<script>
  import { onMount, untrack } from "svelte";
  import { getAboutDocument } from "../api/commandAdapter.js";

  let { slug } = $props();

  let documentItem = $state(null);
  let loading = $state(false);
  let error = $state("");

  function shouldRenderAsHtml(item) {
    if (!item || typeof item !== "object") return false;
    const slugName = String(item.slug || "").toLowerCase();
    const filename = String(item.filename || "").toLowerCase();
    return slugName === "disclaimer" || filename.endsWith(".html");
  }

  async function loadAboutDocumentView(slugName, force = false) {
    const normalizedSlug = String(slugName || "").trim().toLowerCase();
    if (!normalizedSlug) {
      documentItem = null;
      error = "Document not found.";
      return;
    }

    loading = true;
    error = "";

    try {
      const result = await getAboutDocument(normalizedSlug);
      if (normalizedSlug !== String(slug || "").trim().toLowerCase()) return;

      documentItem = result?.item || null;
      if (!documentItem) {
        error = String(result?.error || "Document not found.");
      }
    } catch (e) {
      documentItem = null;
      error = `Could not load document: ${e}`;
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (slug) {
      untrack(() => {
        loadAboutDocumentView(slug);
      });
    }
  });

  onMount(() => {
    if (slug) {
      loadAboutDocumentView(slug);
    }
  });
</script>

<div class="max-w-5xl mx-auto space-y-4 font-sans">
  <div class="bg-white rounded-xl shadow p-6">
    {#if loading}
      <p class="text-sm text-gray-500">Loading document...</p>
    {:else if error}
      <p class="text-sm text-red-650 bg-red-50 border border-red-200 rounded p-3">{error}</p>
    {:else if documentItem?.document_text}
      {#if shouldRenderAsHtml(documentItem)}
        <div class="text-sm text-gray-700 bg-gray-50 border rounded-lg p-4 space-y-4 shadow-inner">
          {@html documentItem.document_text}
        </div>
      {:else}
        <pre class="whitespace-pre-wrap text-sm text-gray-700 bg-gray-50 border rounded-lg p-4 overflow-x-auto font-mono shadow-inner">{documentItem.document_text}</pre>
      {/if}
    {:else}
      <p class="text-sm text-gray-500 italic">Document content is unavailable.</p>
    {/if}
  </div>
</div>
