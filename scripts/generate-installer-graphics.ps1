Add-Type -AssemblyName System.Drawing

# 1. Generate Sidebar Image (164 x 314 BMP)
$sbWidth = 164
$sbHeight = 314
$sidebar = [System.Drawing.Bitmap]::new($sbWidth, $sbHeight)
$g = [System.Drawing.Graphics]::FromImage($sidebar)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic

# Background Gradient (teal to dark navy matching app icon)
$rect = [System.Drawing.Rectangle]::new(0, 0, $sbWidth, $sbHeight)
$brush = [System.Drawing.Drawing2D.LinearGradientBrush]::new($rect, [System.Drawing.Color]::FromArgb(24, 124, 132), [System.Drawing.Color]::FromArgb(12, 71, 104), 90.0)
$g.FillRectangle($brush, $rect)
$brush.Dispose()

# Load icon
$srcIcon = [System.Drawing.Image]::FromFile("icons/icon.png")
$iconSize = 112
$iconX = [int](($sbWidth - $iconSize) / 2)
$iconY = 60
$g.DrawImage($srcIcon, $iconX, $iconY, $iconSize, $iconSize)
$srcIcon.Dispose()

# Text
$titleFont = [System.Drawing.Font]::new("Segoe UI", [float]11.0, [System.Drawing.FontStyle]::Bold)
$titleBrush = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(240, 248, 255))
$sf = [System.Drawing.StringFormat]::new()
$sf.Alignment = [System.Drawing.StringAlignment]::Center
$g.DrawString("Embroidery", $titleFont, $titleBrush, [float]($sbWidth / 2), [float]($iconY + $iconSize + 20), $sf)
$g.DrawString("Catalogue", $titleFont, $titleBrush, [float]($sbWidth / 2), [float]($iconY + $iconSize + 40), $sf)
$titleFont.Dispose()
$titleBrush.Dispose()
$sf.Dispose()
$g.Dispose()

# Ensure directories exist
if (-not (Test-Path "icons")) { New-Item -ItemType Directory -Path "icons" | Out-Null }
if (-not (Test-Path "src-tauri/icons")) { New-Item -ItemType Directory -Path "src-tauri/icons" | Out-Null }

# Save sidebar
$sidebar.Save("icons/nsis-sidebar.bmp", [System.Drawing.Imaging.ImageFormat]::Bmp)
$sidebar.Save("src-tauri/icons/nsis-sidebar.bmp", [System.Drawing.Imaging.ImageFormat]::Bmp)
$sidebar.Dispose()

# 2. Generate Header Image (150 x 57 BMP)
$hWidth = 150
$hHeight = 57
$header = [System.Drawing.Bitmap]::new($hWidth, $hHeight)
$gh = [System.Drawing.Graphics]::FromImage($header)
$gh.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$gh.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
$gh.Clear([System.Drawing.Color]::White)

$srcIcon2 = [System.Drawing.Image]::FromFile("icons/icon.png")
$hIconSize = 48
$hIconX = $hWidth - $hIconSize - 8
$hIconY = [int](($hHeight - $hIconSize) / 2)
$gh.DrawImage($srcIcon2, $hIconX, $hIconY, $hIconSize, $hIconSize)
$srcIcon2.Dispose()

$gh.Dispose()

# Save header
$header.Save("icons/nsis-header.bmp", [System.Drawing.Imaging.ImageFormat]::Bmp)
$header.Save("src-tauri/icons/nsis-header.bmp", [System.Drawing.Imaging.ImageFormat]::Bmp)
$header.Dispose()

Write-Host "Successfully generated nsis-sidebar.bmp and nsis-header.bmp"
