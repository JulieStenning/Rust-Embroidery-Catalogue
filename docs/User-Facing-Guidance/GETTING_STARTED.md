# Getting Started with Embroidery Catalogue

Embroidery Catalogue is a fast, offline desktop tool for organising and browsing digital embroidery designs. Your original embroidery files are never moved or modified.

---

## 1. Install and launch the application

1. Download the Windows installer (`.msi` or `-setup.exe`) or the portable release.
2. If using the installer, follow the setup wizard and launch **Embroidery Catalogue**.
3. If running portably, copy `embroidery-catalogue.exe` into a folder on your drive alongside an empty `data` subfolder, then double-click the executable.

See [Installing Embroidery Catalogue](App%20Installer.md) for full installation details.

---

## 2. Complete initial setup

On first launch, the setup wizard guides you through your basic preferences:

1. **Data location**: Choose where your catalogue database and thumbnail cache will live (e.g. `C:\Users\<Name>\Documents\EmbroideryCatalogue` or an external drive).
2. **Designers and Sources (optional)**: Add designers or shops you frequently purchase from, or click **Continue** to add them later.
3. **Hoops (optional)**: Add your embroidery machine hoops to enable automatic hoop matching during import.
4. **Digitising Formats & Alphabets**: Select any digitising software you use (such as Embird, Hatch, Bernina, or Embrilliance), and leave alphabet compression enabled (recommended).
5. Click **Finish**.

---

## 3. Import your first designs

1. Click **Import** in the top navigation bar.
2. Click **Browse…** and select the folder containing your embroidery designs.
3. Review the discovered designs in the review table.
4. Click **Import Designs**.
5. Once importing completes, click **Go to Browse** to explore your catalogue.

---

## 4. Browse and search your designs

1. Click **Browse** in the top navigation bar.
2. Use the search bar to find designs by title, filename, designer, or tag.
3. Use the filter panel on the left to filter by hoop size, format, stitch count, or stitch type.
4. Click any design card to open its detail page, view colours, technical stitch data, or open the design in your default embroidery software.

---

## 5. Print worksheets and project plans

1. Open a design in [Browse](#/browse) and click **Print View**, or select a project in [Projects](#/projects) and click **Print Sheet**.
2. A clean, formatted summary opens without window navigation bars.
3. Click **Print** (or press <kbd>Ctrl+P</kbd>) to print to your printer or save as a PDF.

---

## If something goes wrong

| Problem | What to do |
|---|---|
| No designs appear after scanning a folder | Ensure the folder contains supported stitch files (such as `.pes`, `.jef`, `.dst`, `.vp3`, `.hus`). See [Supported Formats](SUPPORTED_FORMATS.md). |
| App prompts to re-connect on startup | If your catalogue is stored on a removable drive, plug the drive in and click **Re-connect to this location**. |
| You want to move your catalogue to another drive | Go to [Admin → Settings](#/admin/system/settings) and select a new folder under **Catalogue Storage**. |
| Right-clicking a page does not show browser options | Embroidery Catalogue runs as an offline desktop application. Generic browser options such as Save Page As or Refresh are disabled to prevent state desynchronisation. Right-clicking inside text input fields still provides standard Cut, Copy, and Paste. |

---

## Related guides

- [Installing Embroidery Catalogue](App%20Installer.md)
- [Import Workflow](IMPORT_WORKFLOW.md)
- [Settings](SETTINGS.md)
- [Supported Formats](SUPPORTED_FORMATS.md)
- [Batch Operations](BATCH_OPERATIONS_BACKFILL.md)
