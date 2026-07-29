# Installing & Deploying the Embroidery Catalogue

## Installed Mode (local HDD)

Run the installer from your local hard drive:

```
target/release/bundle/msi/Embroidery Catalogue_0.1.0_x64_en-US.msi
```

Or the NSIS installer:

```
target/release/bundle/nsis/Embroidery Catalogue_0.1.0_x64-setup.exe
```

The installer places the application and a seed database in the correct locations.
On first run the app seeds `<data_root>/Database/EmbroideryCatalogue.db` automatically.

---

## Portable Mode (SD card / USB stick)

No installer is needed. The portable executable includes the seed database embedded
inside the binary.

### Setup

1. Create a folder on your SD card, e.g. `E:\EmbroideryCatalogue\`

2. Copy the release executable into it:
   ```
   Copy from:  target/release/embroidery-catalogue.exe
   Copy to:    E:\EmbroideryCatalogue\embroidery-catalogue.exe
   ```

3. Create an empty `data\` subfolder:
   ```
   E:\EmbroideryCatalogue\data\
   ```

4. Double-click `embroidery-catalogue.exe`

That's it — no additional files are needed; the database is embedded in the `.exe`.

### First-run behaviour

- The app detects `data\` next to the executable → runs in **Portable Mode**
- No `Database\EmbroideryCatalogue.db` exists yet → the app writes the embedded
  seed database to `data\Database\EmbroideryCatalogue.db` automatically
- Everything stays on the SD card — no files written to the host PC

### Cross-computer use

Drive letters don't matter (E:, F:, G:, etc.). All paths are resolved relative
to the executable at runtime. The same SD card works on any Windows PC.