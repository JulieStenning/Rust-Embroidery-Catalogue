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