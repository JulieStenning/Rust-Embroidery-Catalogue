/**
 * Browse fixture checker.
 *
 * Validates `tests/Test Assets/EmbroideryCatalogue.db` against the Browse
 * fixture contract (see `docs/Help for Developers/e2e testing guide.md`) and confirms
 * that every design row's `filepath` resolves to a real file under
 * `tests/Test Designs`.
 *
 * Run from the repo root:
 *   node scripts/check-browse-fixtures.cjs
 *
 * Exits non-zero when a hard contract rule fails.
 */
const fs = require("node:fs");
const path = require("node:path");
const { DatabaseSync } = require("node:sqlite");

const repoRoot = path.resolve(__dirname, "..");
const dbPath = path.join(
  repoRoot,
  "tests",
  "Test Assets",
  "EmbroideryCatalogue.db",
);
const designsRoot = path.join(repoRoot, "tests", "Test Designs");

if (!fs.existsSync(dbPath)) {
  console.error(`Seed database not found: ${dbPath}`);
  process.exit(2);
}

const db = new DatabaseSync(dbPath);
const rows = db
  .prepare(
    `SELECT d.id, d.filename, d.filepath, d.rating, d.is_stitched,
            d.image_tags_verified, d.stitching_tags_verified,
            (d.image_data IS NULL) AS no_preview,
            des.name AS designer, src.name AS source, h.name AS hoop
       FROM designs d
       LEFT JOIN designers des ON des.id = d.designer_id
       LEFT JOIN sources   src ON src.id = d.source_id
       LEFT JOIN hoops     h   ON h.id = d.hoop_id
      ORDER BY d.filename COLLATE NOCASE`,
  )
  .all();

const tagRows = db
  .prepare(
    `SELECT t.description AS tag, COUNT(*) AS n
       FROM design_tags dt JOIN tags t ON t.id = dt.tag_id
      GROUP BY t.description`,
  )
  .all();

const tagByDesign = new Map();
for (const row of db
  .prepare(
    `SELECT dt.design_id AS id, t.description AS tag
       FROM design_tags dt JOIN tags t ON t.id = dt.tag_id`,
  )
  .all()) {
  if (!tagByDesign.has(row.id)) tagByDesign.set(row.id, []);
  tagByDesign.get(row.id).push(row.tag);
}

db.close();

const failures = [];
const check = (ok, message) => {
  if (!ok) failures.push(message);
  return ok;
};

console.log(`Seed database: ${dbPath}`);
console.log(`Designs: ${rows.length}\n`);

// --- Files on disk ----------------------------------------------------------
const missingFiles = [];
for (const row of rows) {
  const relative = String(row.filepath || row.filename)
    .split("/")
    .join(path.sep);
  const full = path.join(designsRoot, relative);
  if (!fs.existsSync(full)) {
    missingFiles.push(`tests/Test Designs/${row.filepath}`);
  }
}
if (missingFiles.length > 0) {
  console.log(
    `MISSING design files under tests/Test Designs (${missingFiles.length}):`,
  );
  for (const line of missingFiles) console.log(`  - ${line}`);
  console.log("");
} else {
  console.log("All design files resolve under tests/Test Designs.\n");
}

// --- Contract checks --------------------------------------------------------
const noPreview = rows.filter((r) => Number(r.no_preview) === 1);
check(
  noPreview.length === 2,
  `Needs attention: expected exactly 2 rows with NULL image_data, found ${noPreview.length}`,
);
for (const r of noPreview) console.log(`  needs attention: ${r.filename}`);

const unverified = rows.filter(
  (r) =>
    Number(r.image_tags_verified) === 0 ||
    Number(r.stitching_tags_verified) === 0,
);
check(
  unverified.length === 4,
  `Unverified only: expected exactly 4 rows, found ${unverified.length}`,
);
for (const r of unverified) {
  console.log(
    `  unverified: ${r.filename} (${r.image_tags_verified}/${r.stitching_tags_verified})`,
  );
}

const highRating = rows.filter(
  (r) => r.rating !== null && Number(r.rating) >= 4,
);
check(
  highRating.length === 2,
  `Minimum rating >= 4: expected exactly 2 rows, found ${highRating.length}`,
);

const stitched = rows.filter((r) => Number(r.is_stitched) === 1);
check(
  stitched.length === 2,
  `Stitched = yes: expected exactly 2 rows, found ${stitched.length}`,
);

const fillers = rows.filter((r) => /^Filler \d+\.jef$/i.test(r.filename));
check(
  rows.length > 50,
  `Pagination: catalogue needs more than 50 designs, found ${rows.length} (fillers: ${fillers.length})`,
);

const cake3 = rows.find((r) => r.filename === "Cake 3.jef");
check(Boolean(cake3), "Fixture contract: Cake 3.jef is missing");
if (cake3) {
  check(
    String(cake3.hoop || "") === "Hoop B",
    `Cake 3.jef hoop must be "Hoop B", found "${cake3.hoop ?? "NULL"}"`,
  );
  const tags = tagByDesign.get(cake3.id) || [];
  check(
    tags.includes("Food") && tags.includes("Cross Stitch"),
    `Cake 3.jef tags must include Food + Cross Stitch, found ${JSON.stringify(tags)}`,
  );
}

// The hoop filter test asserts membership in two hoops, so these two
// assignments are part of the contract even though the rest are derived.
const cakeApplique = rows.find((r) => r.filename === "Cake Applique.jef");
check(
  String(cakeApplique?.hoop || "") === "Hoop B",
  `Cake Applique.jef hoop must be "Hoop B", found "${cakeApplique?.hoop ?? "missing"}"`,
);
const cakeApplique2 = rows.find((r) => r.filename === "Cake Applique 2.jef");
check(
  String(cakeApplique2?.hoop || "") === "Hoop A",
  `Cake Applique 2.jef hoop must be "Hoop A", found "${cakeApplique2?.hoop ?? "missing"}"`,
);

const designerCounts = {};
const sourceCounts = {};
for (const r of rows) {
  if (r.designer)
    designerCounts[r.designer] = (designerCounts[r.designer] || 0) + 1;
  if (r.source) sourceCounts[r.source] = (sourceCounts[r.source] || 0) + 1;
}
check(
  designerCounts["Thistlebury Stitch"] === 2,
  `Designer "Thistlebury Stitch": expected 2 rows, found ${designerCounts["Thistlebury Stitch"] || 0}`,
);
check(
  designerCounts["Wrenwood Studio"] === 1,
  `Designer "Wrenwood Studio": expected 1 row, found ${designerCounts["Wrenwood Studio"] || 0}`,
);
check(
  designerCounts["Me"] === 2,
  `Designer "Me": expected 2 rows, found ${designerCounts["Me"] || 0}`,
);
check(
  sourceCounts["Threadwise Guild"] === 1,
  `Source "Threadwise Guild": expected 1 row, found ${sourceCounts["Threadwise Guild"] || 0}`,
);

const EXPECTED_TAG_COUNTS = {
  "Cross Stitch": 1,
  Food: 1,
  Filled: 1,
  Applique: 1,
  Flowers: 1,
  Footwear: 1,
};
for (const [tag, expected] of Object.entries(EXPECTED_TAG_COUNTS)) {
  const found = Number(tagRows.find((t) => t.tag === tag)?.n ?? 0);
  check(
    found === expected,
    `Tag "${tag}": expected ${expected} design(s), found ${found}`,
  );
}

const unverifiedFillers = fillers.filter(
  (r) =>
    Number(r.image_tags_verified) === 0 ||
    Number(r.stitching_tags_verified) === 0,
);
check(
  unverifiedFillers.length === 0,
  `${unverifiedFillers.length} filler(s) are not fully verified`,
);

// --- Summary ----------------------------------------------------------------
console.log("");
console.log(
  `Fillers: ${fillers.length}   Unverified: ${unverified.length}   NULL previews: ${noPreview.length}`,
);
console.log(`Rating >= 4: ${highRating.length}   Stitched: ${stitched.length}`);
console.log(`Designers: ${JSON.stringify(designerCounts)}`);
console.log(`Sources:   ${JSON.stringify(sourceCounts)}`);
console.log("");

if (missingFiles.length > 0) {
  console.log(
    `WARNING: ${missingFiles.length} design file(s) missing from tests/Test Designs.\n`,
  );
}

if (failures.length === 0) {
  console.log("Browse fixture contract: OK");
  process.exit(0);
}

console.error(`Browse fixture contract: ${failures.length} FAILURE(S)`);
for (const f of failures) console.error(`  x ${f}`);
process.exit(1);
