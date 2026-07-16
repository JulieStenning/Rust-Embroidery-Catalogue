export function splitTagsByGroup(tags) {
  const allTags = Array.isArray(tags) ? tags : [];
  return {
    image: allTags.filter((tag) => {
      const group = String(tag?.tag_group || tag?.tagGroup || "");
      return group === "image";
    }),
    stitching: allTags.filter((tag) => {
      const group = String(tag?.tag_group || tag?.tagGroup || "");
      return group === "stitching";
    }),
    unclassified: allTags.filter((tag) => {
      const group = String(tag?.tag_group || tag?.tagGroup || "");
      return group !== "image" && group !== "stitching";
    }),
  };
}
