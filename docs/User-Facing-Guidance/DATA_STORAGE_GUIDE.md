## Data Storage & External Drives Guide

To run **Embroidery Catalogue** smoothly, your storage device needs to store two main things:

1. **Your Embroidery Files:** The original design files (`.jef`, `.pes`, `.hus`, etc.) that you collect and organize.
2. **The Catalogue Database & Image Cache:** The local database storing your tags, information about the design file, and fast-loading thumbnail previews created by the app.

While storing your library directly on your computer’s main internal drive (SSD or Hard Drive) works automatically, using external storage—like an SD card or an external USB drive—is a great choice if internal space is limited.

---

## 1. Choosing the Right External Storage Device

Not all external storage devices perform at the same speed. Because **Embroidery Catalogue** reads hundreds or thousands of tiny design files to extract information and render thumbnails, **fast random read speeds are crucial**.

### Recommended: SD Cards (Application Performance Class)

If your computer has a built-in SD card slot (or a quality USB 3.0 card reader), an SD card is a compact, convenient way to host your library.

* **Look for the "A2" Specification:** Always choose an **Application Performance Class 2 (A2)** SD card.
* **Why A2 matters:** Standard SD cards are designed for writing continuous video streams (camera use) and are often slow at loading thousands of tiny files. **A2 cards are engineered specifically for rapid reading and writing of small files** (high IOPS—Input/Output Operations Per Second), making app responsiveness much faster.


* **Speed Rating:** Look for **V30**, **U3**, or **Class 10** ratings alongside the A2 logo.
* **Recommended Brands:** SanDisk (e.g., Extreme or Extreme Pro A2), Samsung (e.g., PRO Plus or EVO Plus A2), or Kingston.

> **Visual Check:** Look for a small symbol that says **A2** printed directly on the front of the SD card or packaging. Avoid older "A1" or unrated cards if possible, as thumbing through large libraries will feel noticeably slower.

---

### Alternative Storage Options

* **External Solid State Drives (Portable SSDs):**
* *Performance:* **Excellent.**
* Connecting a portable SSD via USB 3.0, USB-C, or Thunderbolt provides speed equal to or greater than internal drives. Fast library scanning and quick image thumbnail loading are guaranteed.


* **USB Flash Drives ("Thumb Drives"):**
* *Performance:* **Variable / Moderate.**
* Standard USB flash drives often suffer from slow random read speeds, even if labeled "USB 3.0". If using a flash drive, select a high-performance model explicitly designed for fast data transfers (e.g., SanDisk Extreme PRO USB).


* **Traditional External Hard Drives (Spinning HDDs):**
* *Performance:* **Not Recommended.**
* Traditional spinning hard drives are significantly slower when reading thousands of individual thumbnail images, which will cause delayed scrolling in the browse view.



---

## 2. Setting Up Your Library on External Storage

### Formatting Your Drive

Ensure your external drive or SD card is formatted with a modern filesystem supported by your operating system:

* **Windows:** NTFS or exFAT
* **macOS:** APFS or exFAT

*(Note: exFAT is ideal if you plan to move your SD card between Windows and Mac computers).*

### Managing the Drive Letter or Volume Name

When using external storage, your operating system assigns it a location (e.g., `D:\` on Windows or `/Volumes/MySDCard` on macOS).

* **Keep the Drive Plugged In:** Ensure your SD card or external drive is connected **before** opening **Embroidery Catalogue**.
* **Maintain Drive Letters (Windows):** If Windows assigns a new drive letter to your SD card after unplugging it, **Embroidery Catalogue** may not locate your library files. You can assign a permanent drive letter to your SD card using Windows Disk Management.

---

## 3. Best Practices for Data Safety

1. **Unplug Safely:** Always use your operating system's "Eject" or "Safely Remove Hardware" option before unplugging your SD card or external drive to prevent database corruption.
2. **Regular Backups:** External cards and drives can be lost or damaged. Use the built-in backup tools in **Embroidery Catalogue** ([Backup](#/admin/maintenance/backup)) or copy your database and designs to a second drive periodically.
3. **Keep Original Files Untouched:** **Embroidery Catalogue** is an offline-first tool that **never moves, modifies, or alters** your original embroidery files. However, keeping an independent backup of your source embroidery purchases is always recommended.