Add-Type -AssemblyName System.Drawing

# Create a 32x32 indigo PNG/ICO placeholder
$bmp = New-Object System.Drawing.Bitmap 32, 32
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.Clear([System.Drawing.Color]::FromArgb(79, 70, 229))
$g.Dispose()

# Create icons directory
New-Item -ItemType Directory -Force -Path "icons" | Out-Null

# Save as ICO
$iconHandle = $bmp.GetHicon()
$icon = [System.Drawing.Icon]::FromHandle($iconHandle)
$stream = New-Object System.IO.FileStream "icons\icon.ico", "Create"
$icon.Save($stream)
$stream.Close()
$icon.Dispose()
[System.Runtime.InteropServices.Marshal]::DestroyIcon($iconHandle) | Out-Null
$bmp.Dispose()

Write-Host "icons/icon.ico created successfully"
