# Installing Embroidery Catalogue

You can install Embroidery Catalogue on your PC or run it portably from an SD card or USB stick.

---

## Install on your PC

1. Download and run the installer (`.msi` or `-setup.exe`).
2. Follow the setup wizard on screen.
3. Open **Embroidery Catalogue** from your Start menu or desktop shortcut.
4. Complete the initial setup wizard to choose where your catalogue data is stored.

---

## Run portably from an SD card or USB stick

In portable mode, your database and catalogue stay entirely on the removable drive. No files are written to the host PC, and you can plug the drive into any Windows computer.

1. Create a folder on your SD card or USB stick, for example `E:\EmbroideryCatalogue\`.
2. Copy `embroidery-catalogue.exe` into that folder.
3. Create an empty subfolder named `data` in the same folder (`E:\EmbroideryCatalogue\data\`).
4. Double-click `embroidery-catalogue.exe`.

The application detects the `data` folder, sets up your catalogue on the drive, and launches.

---

## If something goes wrong

| Problem | What to do |
|---|---|
| The installer is blocked by Windows SmartScreen | Click **More info**, then click **Run anyway**. |
| Portable mode writes data to your PC drive instead | Check that the `data` subfolder is created directly next to `embroidery-catalogue.exe`. |
| Removable drive letter changes after reconnecting | The app automatically displays the **Startup Recovery** screen; click **Re-connect to this location**. |

---

## Related guides

- [Getting Started](GETTING_STARTED.md)
- [Data Storage Guide](DATA_STORAGE_GUIDE.md)
- [Settings](SETTINGS.md)
