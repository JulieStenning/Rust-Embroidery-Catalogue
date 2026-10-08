<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script lang="ts">
  interface NoticeProps {
    message?: string;
    type?: "info" | "success" | "warning" | "warn" | "error" | "danger" | "default" | string;
    error?: boolean;
  }

  let { message = "", type = "info", error = false }: NoticeProps = $props();
  let resolvedType = $derived(error ? "error" : type);
</script>

{#if message}
  <div
    class="border rounded px-3 py-2 text-sm"
    class:notice-success={resolvedType === "success"}
    class:notice-error={resolvedType === "error" || resolvedType === "danger"}
    class:notice-warn={resolvedType === "warning" || resolvedType === "warn"}
    class:notice-info={resolvedType === "info" ||
      resolvedType === "default" ||
      !["success", "error", "danger", "warning", "warn"].includes(resolvedType)}
    role="status"
  >
    {message}
  </div>
{/if}

<style>
  .notice-success {
    background-color: var(--notice-success-bg);
    border-color: var(--notice-success-border);
    color: var(--notice-success-text);
  }

  .notice-error {
    background-color: var(--notice-error-bg);
    border-color: var(--notice-error-border);
    color: var(--notice-error-text);
  }

  .notice-warn {
    background-color: var(--notice-warn-bg);
    border-color: var(--notice-warn-border);
    color: var(--notice-warn-text);
  }

  .notice-info {
    background-color: var(--notice-info-bg);
    border-color: var(--notice-info-border);
    color: var(--notice-info-text);
  }
</style>
