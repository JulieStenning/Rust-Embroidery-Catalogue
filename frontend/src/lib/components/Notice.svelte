<script>
  let { message = "", type = "info", error = false } = $props();
  let resolvedType = $derived(error ? "error" : type);
</script>

{#if message}
  <div
    class="border rounded px-3 py-2 text-sm"
    class:bg-green-50={resolvedType === "success"}
    class:border-green-300={resolvedType === "success"}
    class:text-green-800={resolvedType === "success"}
    class:bg-red-50={resolvedType === "error" || resolvedType === "danger"}
    class:border-red-300={resolvedType === "error" || resolvedType === "danger"}
    class:text-red-800={resolvedType === "error" || resolvedType === "danger"}
    class:bg-blue-50={resolvedType === "info" || resolvedType === "default"}
    class:border-blue-200={resolvedType === "info" || resolvedType === "default"}
    class:text-blue-800={resolvedType === "info" || resolvedType === "default"}
  >
    {message}
  </div>
{/if}
