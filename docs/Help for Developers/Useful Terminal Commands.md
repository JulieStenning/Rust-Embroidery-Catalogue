Checking Coverage

Backend
cargo llvm-cov --manifest-path Cargo.toml

Backend HTML View
cargo llvm-cov --manifest-path Cargo.toml --open

Backend individual file
cargo llvm-cov --manifest-path Cargo.toml -- FILENAME
FILENAME doesn't need the path or filetype

Backend individual file HTML view
cargo llvm-cov test --manifest-path Cargo.toml --open FILENAME

Svelte
npx vitest run --coverage
npx vitest --coverage --ui
npm test
npx vitest run ModuleName.test.ts --coverage

Note (Windows + vitest 4): if the coverage report shows "All files | 0 | 0 | 0 | 0"
with no file rows, set `allowExternal: true` under `test.coverage` in
vitest.config.mts (already done in this repo). Vitest's internal file-inclusion
check compares transform IDs against the project root with a case-sensitive
startsWith(), but on Windows the drive letter casing differs ("D:/..." vs
"d:/..."), which makes every file look "external" and skips instrumentation.
