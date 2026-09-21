<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  let { detailDesignId, onDesignDeleted } = $props();
</script>

<button data-testid="design-deleted" onclick={() => onDesignDeleted?.()}> Delete Design </button>
<div data-testid="design-detail-view" data-design-id={detailDesignId ?? ""}>Design Detail View</div>
