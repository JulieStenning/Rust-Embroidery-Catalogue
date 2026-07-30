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