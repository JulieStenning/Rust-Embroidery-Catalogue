# Code Signing for Windows Releases

This project can optionally code-sign the Windows desktop executable and installer, but a paid certificate is **not required yet** while development and testing are still in progress.

## Why code signing matters

Code signing helps Windows, SmartScreen, and antivirus products such as Norton treat the application as more trustworthy. It can reduce warning prompts and repeated reputation scans for:

- `dist\EmbroideryCatalogue\EmbroideryCatalogue.exe`
- `installer\Output\EmbroideryCatalogueSetup.exe`

Without a certificate, the app will still build and run, but Windows security products may inspect it more aggressively.

---

## Current status

`build_desktop.bat` already supports **optional** signing.

If no signing configuration is present, the build completes normally and leaves the files unsigned.

---

## What will be needed later

When ready to publish more widely, obtain:

1. a **code-signing certificate** from a trusted certificate authority
2. Microsoft **`signtool.exe`** from the Windows SDK

Common certificate providers include:

- DigiCert
- Sectigo
- GlobalSign
- SSL.com

For best reputation with SmartScreen and antivirus tools, an **EV code-signing certificate** is ideal, but it is more expensive.

---

## Supported signing methods

The build script supports these options:

### 1. Sign with a `.pfx` certificate file

```bat
set SIGN_CERT_FILE=C:\path\to\your-certificate.pfx
set SIGN_CERT_PASSWORD=your-password
set SIGN_TIMESTAMP_URL=http://timestamp.digicert.com
build_desktop.bat
```

### 2. Sign with a certificate from the Windows certificate store by subject name

```bat
set SIGN_CERT_SUBJECT=Your Company Name
set SIGN_TIMESTAMP_URL=http://timestamp.digicert.com
build_desktop.bat
```

### 3. Sign with a certificate from the Windows certificate store by thumbprint

```bat
set SIGN_CERT_SHA1=YOURCERTTHUMBPRINT
set SIGN_TIMESTAMP_URL=http://timestamp.digicert.com
build_desktop.bat
```

### 4. Explicitly set the path to `signtool.exe`

```bat
set SIGNTOOL_EXE=C:\Program Files (x86)\Windows Kits\10\App Certification Kit\signtool.exe
```

---

## Recommended future setup

If a `.pfx` certificate is purchased later, the simplest setup is:

```bat
set SIGNTOOL_EXE=C:\Program Files (x86)\Windows Kits\10\App Certification Kit\signtool.exe
set SIGN_CERT_FILE=C:\path\to\certificate.pfx
set SIGN_CERT_PASSWORD=your-password
set SIGN_TIMESTAMP_URL=http://timestamp.digicert.com
build_desktop.bat
```

---

## How to confirm signing worked

A signed build should print messages like:

```text
Signing dist\EmbroideryCatalogue\EmbroideryCatalogue.exe ...
Signing installer\Output\EmbroideryCatalogueSetup.exe ...
```

If signing is not configured, the script will instead print:

```text
Code signing is not configured, so the EXE and installer were left unsigned.
```

---

## Important security note

Do **not** commit any of the following into the repository:

- `.pfx` certificate files
- certificate passwords
- private keys

Keep those only on the release machine or in a secure secret store.
