<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  let { currentUiKind, projectDetailId, projectPrintId } = $props();
</script>

<div
  data-testid="projects-view"
  data-kind={currentUiKind}
  data-detail-id={projectDetailId ?? ""}
  data-print-id={projectPrintId ?? ""}
>
  Projects View
</div>
