# Data Storage & External Drives

You can store your catalogue on your computer's internal drive, an SD card, or an external USB drive.

Your original embroidery files are never moved, renamed, or modified. The catalogue creates and stores its own database and thumbnail previews in your chosen data folder.

---

## Choose an external storage device

Because the catalogue reads many small stitch files to extract details and generate thumbnails, fast random-read speed matters.

- **Recommended: SD cards marked "A2"**: Look for the **A2** symbol on the card or packaging (Application Performance Class 2). A2 cards are designed for rapid access to thousands of small files.
- **External Solid State Drives (SSDs)**: Fast and reliable over USB 3.0 or USB-C.
- **USB flash drives (thumb drives)**: Work well if rated for high-speed USB 3.0 transfers.
- **Spinning hard drives (HDDs)**: Not recommended. Browsing thumbnails will feel noticeably slower.

---

## Format your drive (exFAT or NTFS)

Before storing a large collection on a new drive or SD card, make sure it is formatted as **exFAT** or **NTFS**:

1. In Windows File Explorer, right-click the drive and choose **Format…**.
2. Select **exFAT** (ideal for moving between Windows and Mac) or **NTFS** (Windows-only).
3. Click **Start**.

> **Avoid FAT32:** Many USB sticks and SD cards come pre-formatted as FAT32. FAT32 cannot store any single file larger than 4 GB. As your catalogue grows, a FAT32 drive will cause errors when the database reaches 4 GB.

---

## Move your catalogue to an external drive

1. Make a backup in [Admin → System Maintenance → Backup](#/admin/system/backup).
2. Go to [Admin → Settings](#/admin/system/settings).
3. Under **Catalogue Storage**, click **Browse…** and select the folder on your external drive.
4. Wait for files to copy, then click **Restart now**.

---

## If something goes wrong

| Problem | What to do |
|---|---|
| The app cannot find the catalogue after reconnecting the drive | Plug the drive into your PC, open the app, and click **Re-connect to this location** on the Startup Recovery screen. |
| The database gives disk full or 4 GB limit errors | Your drive is formatted as FAT32. Back up your catalogue in [Backup](#/admin/system/backup), reformat the drive to exFAT or NTFS, and restore the backup. |
| Thumbnail scrolling feels slow | Move your catalogue to an internal SSD or an A2-rated SD card. |
| Drive removed accidentally | Close the app, reconnect the drive, and reopen the app. If a database file was damaged, the app preserves it and prompts you to restore from backup. |

---

## Related guides

- [App Installer](App%20Installer.md)
- [Settings](SETTINGS.md)
- [Backup and Restore](BACKUP_RESTORE.md)
