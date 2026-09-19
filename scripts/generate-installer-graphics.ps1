Add-Type -AssemblyName System.Drawing

if (-not (Test-Path "icons")) { New-Item -ItemType Directory -Path "icons" | Out-Null }
if (-not (Test-Path "src-tauri/icons")) { New-Item -ItemType Directory -Path "src-tauri/icons" | Out-Null }

# 1. Generate NSIS Sidebar Image (164 x 314 BMP)
$sbWidth = 164
$sbHeight = 314
$sidebar = [System.Drawing.Bitmap]::new($sbWidth, $sbHeight)
$g = [System.Drawing.Graphics]::FromImage($sidebar)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic

$rect = [System.Drawing.Rectangle]::new(0, 0, $sbWidth, $sbHeight)
$brush = [System.Drawing.Drawing2D.LinearGradientBrush]::new($rect, [System.Drawing.Color]::FromArgb(24, 124, 132), [System.Drawing.Color]::FromArgb(12, 71, 104), 90.0)
$g.FillRectangle($brush, $rect)
$brush.Dispose()

$srcIcon = [System.Drawing.Image]::FromFile("icons/icon.png")
$iconSize = 112
$iconX = [int](($sbWidth - $iconSize) / 2)
$iconY = 60
$g.DrawImage($srcIcon, $iconX, $iconY, $iconSize, $iconSize)
$srcIcon.Dispose()

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

$sidebar.Save("icons/nsis-sidebar.bmp", [System.Drawing.Imaging.ImageFormat]::Bmp)
$sidebar.Save("src-tauri/icons/nsis-sidebar.bmp", [System.Drawing.Imaging.ImageFormat]::Bmp)
$sidebar.Dispose()

# 2. Generate NSIS Header Image (150 x 57 BMP)
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

$header.Save("icons/nsis-header.bmp", [System.Drawing.Imaging.ImageFormat]::Bmp)
$header.Save("src-tauri/icons/nsis-header.bmp", [System.Drawing.Imaging.ImageFormat]::Bmp)
$header.Dispose()

# 3. Generate WiX Banner Image (493 x 58 BMP)
$wbWidth = 493
$wbHeight = 58
$wixBanner = [System.Drawing.Bitmap]::new($wbWidth, $wbHeight)
$gwb = [System.Drawing.Graphics]::FromImage($wixBanner)
$gwb.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$gwb.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
$gwb.Clear([System.Drawing.Color]::White)

$srcIcon3 = [System.Drawing.Image]::FromFile("icons/icon.png")
$wbIconSize = 50
$wbIconX = $wbWidth - $wbIconSize - 8
$wbIconY = [int](($wbHeight - $wbIconSize) / 2)
$gwb.DrawImage($srcIcon3, $wbIconX, $wbIconY, $wbIconSize, $wbIconSize)
$srcIcon3.Dispose()
$gwb.Dispose()

$wixBanner.Save("icons/wix-banner.bmp", [System.Drawing.Imaging.ImageFormat]::Bmp)
$wixBanner.Save("src-tauri/icons/wix-banner.bmp", [System.Drawing.Imaging.ImageFormat]::Bmp)
$wixBanner.Dispose()

# 4. Generate WiX Dialog Image (493 x 312 BMP)
$wdWidth = 493
$wdHeight = 312
$wixDialog = [System.Drawing.Bitmap]::new($wdWidth, $wdHeight)
$gwd = [System.Drawing.Graphics]::FromImage($wixDialog)
$gwd.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$gwd.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic

# Left side background (164 wide)
$leftRect = [System.Drawing.Rectangle]::new(0, 0, 164, $wdHeight)
$leftBrush = [System.Drawing.Drawing2D.LinearGradientBrush]::new($leftRect, [System.Drawing.Color]::FromArgb(24, 124, 132), [System.Drawing.Color]::FromArgb(12, 71, 104), 90.0)
$gwd.FillRectangle($leftBrush, $leftRect)
$leftBrush.Dispose()

# Right side white background
$rightRect = [System.Drawing.Rectangle]::new(164, 0, ($wdWidth - 164), $wdHeight)
$rightBrush = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::White)
$gwd.FillRectangle($rightBrush, $rightRect)
$rightBrush.Dispose()

# Draw icon on left side
$srcIcon4 = [System.Drawing.Image]::FromFile("icons/icon.png")
$wdIconSize = 112
$wdIconX = [int]((164 - $wdIconSize) / 2)
$wdIconY = 60
$gwd.DrawImage($srcIcon4, $wdIconX, $wdIconY, $wdIconSize, $wdIconSize)
$srcIcon4.Dispose()

# Left side text
$wdFont = [System.Drawing.Font]::new("Segoe UI", [float]11.0, [System.Drawing.FontStyle]::Bold)
$wdBrush = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(240, 248, 255))
$wdsf = [System.Drawing.StringFormat]::new()
$wdsf.Alignment = [System.Drawing.StringAlignment]::Center
$gwd.DrawString("Embroidery", $wdFont, $wdBrush, [float](164 / 2), [float]($wdIconY + $wdIconSize + 20), $wdsf)
$gwd.DrawString("Catalogue", $wdFont, $wdBrush, [float](164 / 2), [float]($wdIconY + $wdIconSize + 40), $wdsf)
$wdFont.Dispose()
$wdBrush.Dispose()
$wdsf.Dispose()
$gwd.Dispose()

$wixDialog.Save("icons/wix-dialog.bmp", [System.Drawing.Imaging.ImageFormat]::Bmp)
$wixDialog.Save("src-tauri/icons/wix-dialog.bmp", [System.Drawing.Imaging.ImageFormat]::Bmp)
$wixDialog.Dispose()

Write-Host "Successfully generated NSIS and WiX installer bitmaps"
