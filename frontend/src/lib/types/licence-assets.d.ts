/// <reference types="vite/client" />

// ---------------------------------------------------------------------------
// licence-assets.d.ts
//
// Ambient (global) type declarations for the statically-imported licence assets
// used by AboutView.svelte. This file must remain a *script* file (no top-level
// import/export) so its `declare module` wildcard entries are treated as global
// ambient declarations rather than scoped module augmentations.
//
// `vite/client` already declares `*?raw` imports; we additionally declare a
// typed shape for `*.json` licence records.
// ---------------------------------------------------------------------------

interface NpmLicenceEntry {
  /** SPDX identifier(s), e.g. "MIT" or "Apache-2.0". */
  licenses?: string;
  /** Package repository URL, when known. */
  repository?: string;
  /** Published-by name, when known. */
  publisher?: string;
  /** Absolute filesystem path of the installed package. */
  path?: string;
  /** Absolute filesystem path of the package's licence file. */
  licenseFile?: string;
}

declare module "*.json" {
  const value: Record<string, NpmLicenceEntry>;
  export default value;
}
