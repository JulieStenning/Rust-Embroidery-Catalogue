# Settings

Use [Admin → Settings](#/admin/system/settings) to change how the app looks, set up AI tagging, look after your database, and choose which files are imported.

Most changes need **Save settings** at the top of the page. The theme, moving your catalogue, and compacting the database take effect straight away.

---

## Change the theme

1. Under **Appearance**, choose **System Default**, **Light** or **Dark**.

**System Default** follows your Windows light or dark mode.

---

## Set up AI tagging with Google Gemini

AI tagging is optional. Without it, the app still tags designs from their file and folder names.

1. Go to [Google AI Studio](https://aistudio.google.com/), sign in, and choose **Get API key** → **Create API key**. Copy the key.
2. Under **Google Gemini & AI Tagging**, paste it into **API key**.
3. If you have not set up billing with Google, tick **My Google API key is on the free tier**.
4. Leave **Gemini model** on **Auto-select (recommended)** and click **Test model**.
5. Click **Save settings**.

You can now tag designs from their pictures in [Admin → Batch Operations](#/admin/batch-operations).

> Keep your API key private. Don't share it in screenshots.

You don't normally need to change **AI tagging batch size**, **Commit every**, **Workers** or **Delay between Gemini calls**. See *If something goes wrong* below if Google rejects requests.

---

## Reclaim disk space

Over time your catalogue database can hold unused space. Compacting it makes the file smaller.

1. Under **Database Maintenance**, check the **Recoverable** space.
2. If it is large, click **Optimise & Compact Database**.
3. Choose **Backup & Compact** (recommended).

You need free space on the drive at least equal to the current database size.

---

## Move your catalogue to another drive

1. Make a backup in [Admin → System Maintenance → Backup](#/admin/system/backup).
2. Under **Catalogue Storage**, click **Browse…** and choose the new folder.
3. Wait while your catalogue is copied. Click **Cancel** at any point to stop and leave everything where it was.
4. Click **Restart now**.

Your original embroidery files outside the catalogue are never touched.

---

## Include files from your digitising software

By default, only machine stitch files (such as `.pes` or `.dst`) are imported. You can also import the working files from your digitising software.

1. Under **Digitising & Master File Formats**, tick only the software you use.
2. If your software isn't listed, type its extensions in **Additional / Custom extensions**, separated by commas.
3. Click **Save settings**.

This applies to your next import. Where a working file sits next to a stitch file with the same name, they appear together as one design.

---

## Choose how alphabets and fonts are imported

Embroidery alphabets can contain hundreds of letter files.

- **Ticked (recommended):** a few sample letters are added to your catalogue, and the full alphabet is kept together in a zip file in your library.
- **Not ticked:** every letter is added as a separate design.

1. Under **Alphabet & Font Compression**, tick or untick **Compress full alphabet sets and catalogue preview samples only**.
2. Click **Save settings**.

This applies to your next import.

---

## If something goes wrong

| Problem | What to do |
|---|---|
| AI tagging shows "429 Too Many Requests" errors | Tick **My Google API key is on the free tier**, or set **Workers** to 1 and **Delay between Gemini calls** to 10. Save, then try again. |
| **Test model** fails | Check the key in Google AI Studio and your internet connection. Click **Refresh** to reload the list of models. |
| **Test model** can't be clicked | Enter an API key first, and wait for any running task to finish. |
| A warning says the database is close to 4 GB | Your catalogue is on a drive that can't hold files over 4 GB (common on USB sticks). Back up, then move your catalogue to a different drive. |
| Compacting fails | Free up space on the drive, then try again. |
| Moving the catalogue fails | Check the new location has enough space and that you can save files there. Your catalogue stays in its old location. |
| The app doesn't restart after a move | Close it and open it again. |
| Saving settings shows an error | Try again. If it keeps failing, check you can save files in your catalogue folder. |

Looking for your licence key? It's on [Admin → About](#/about).

---

## Related guides

- [Batch Operations](BATCH_OPERATIONS_BACKFILL.md) – tag designs with AI
- [Backup and Restore](BACKUP_RESTORE.md)
- [Supported Formats](SUPPORTED_FORMATS.md)
