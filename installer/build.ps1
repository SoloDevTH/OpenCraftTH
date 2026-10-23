<#
.SYNOPSIS
  Build the OpenCraft Studio bundle installer: one setup.exe that installs any of the craft apps.

.DESCRIPTION
  Collects each app's MSI, as built by <app>/packaging/windows/package.ps1 into
  <app>/dist/release (or $env:DIST), and chains them in a WiX Burn bundle (opencraft.wxs) whose
  first page has a checkbox per app (theme.xml). Produces one file, in dist/ next to this script:
    OpenCraft-Studio-Setup-<version>-<arch>.exe

  An app without an MSI is left out (its checkbox is hidden), unless -Build is given: then its
  package.ps1 runs first (Rust + the Windows SDK, as package.ps1 says). The newest MSI per app is used.

  WiX v5 is used from PATH, or else fetched from nuget.org into tools/ (needs the .NET 6+ runtime).
  Double-clicking build.cmd runs this with -Build.

.EXAMPLE
  pwsh installer/build.ps1
  pwsh installer/build.ps1 -Arch arm64 -Build
  pwsh installer/build.ps1 -Apps photocraft,vectorcraft
  pwsh installer/build.ps1 -Build -ScratchDir C:\acb     # compiles on another drive, deleted per app
#>
param(
  [ValidateSet('x64', 'x86', 'arm64')] [string] $Arch = 'x64',
  # Burn versions are numeric; the date and the minute of the (UTC) day make every build an upgrade
  # of the one before, also when two are made on the same day.
  [string] $Version = $(
    $now = (Get-Date).ToUniversalTime()
    '{0}.{1}' -f $now.ToString('yyyy.M.d'), [int][math]::Floor($now.TimeOfDay.TotalMinutes)),
  [string[]] $Apps = @('cadcraft', 'designcraft', 'effectcraft', 'filmcraft', 'gridcraft',
    'lightcraft', 'pdfcraft', 'photocraft', 'vectorcraft', 'wordcraft'),
  [switch] $Build,
  # With -Build: compile each app in <ScratchDir>\<app> and delete it once its MSI is made, so a
  # full build needs room for one app's release build rather than all ten.
  [string] $ScratchDir
)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$Stage = Join-Path $PSScriptRoot "obj\$Arch"
$Dist = Join-Path $PSScriptRoot 'dist'
Remove-Item -Recurse -Force $Stage -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $Stage, $Dist | Out-Null

# WiX 5.0.2 and the extensions the apps' MSIs and this bundle use. package.ps1 calls `wix`, so a
# fetched copy goes on PATH through a wix.cmd shim.
$WixVersion = '5.0.2'
if (-not (Get-Command wix -ErrorAction SilentlyContinue)) {
  $tools = Join-Path $PSScriptRoot 'tools\wix'
  if (-not (Test-Path (Join-Path $tools 'wix.cmd'))) {
    Write-Output "==> fetching WiX $WixVersion from nuget.org"
    New-Item -ItemType Directory -Force -Path $tools | Out-Null
    $nupkg = Join-Path $tools 'wix.zip'
    Invoke-WebRequest "https://www.nuget.org/api/v2/package/wix/$WixVersion" -OutFile $nupkg -UseBasicParsing
    Expand-Archive $nupkg (Join-Path $tools 'pkg') -Force
    Remove-Item $nupkg
    Set-Content (Join-Path $tools 'wix.cmd') '@dotnet "%~dp0pkg\tools\net6.0\any\wix.dll" %*' -Encoding ascii
  }
  $env:PATH = "$tools;$env:PATH"
}
$haveExt = (wix extension list -g) -join "`n"
foreach ($ext in 'WixToolset.UI.wixext', 'WixToolset.Util.wixext', 'WixToolset.BootstrapperApplications.wixext') {
  if ($haveExt -notmatch [regex]::Escape("$ext $WixVersion")) {
    wix extension add -g "$ext/$WixVersion"
    if ($LASTEXITCODE -ne 0) { throw "wix extension add $ext failed" }
  }
}

function Find-Msi([string] $App) {
  $dir = if ($env:DIST) { $env:DIST } else { Join-Path $Root "$App\dist\release" }
  Get-ChildItem -Path (Join-Path $dir "$App-*-windows-$Arch.msi") -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending | Select-Object -First 1
}

$included = @()
foreach ($app in $Apps) {
  $msi = Find-Msi $app
  if (-not $msi -and $Build) {
    Write-Output "==> ${app}: package.ps1 -Arch $Arch"
    # package.ps1 asks git for the build's commit; outside a checkout, Windows PowerShell 5.1 turns
    # git's error into a terminating one, so say "unknown" up front.
    $shaVar = "$($app.ToUpper())_BUILD_SHA"
    if (-not (Test-Path (Join-Path $Root "$app\.git")) -and -not [Environment]::GetEnvironmentVariable($shaVar)) {
      [Environment]::SetEnvironmentVariable($shaVar, 'unknown')
    }
    $savedTarget = $env:CARGO_TARGET_DIR
    if ($ScratchDir) { $env:CARGO_TARGET_DIR = Join-Path $ScratchDir $app }
    # package.ps1 runs cargo in the current directory: the app's workspace root.
    Push-Location (Join-Path $Root "$app")
    try { & (Join-Path $Root "$app\packaging\windows\package.ps1") -Arch $Arch }
    catch { Write-Warning "${app}: package.ps1 failed: $_" }
    finally {
      Pop-Location
      if ($ScratchDir) { Remove-Item -Recurse -Force $env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue }
      $env:CARGO_TARGET_DIR = $savedTarget
    }
    $msi = Find-Msi $app
  }
  if (-not $msi) { Write-Warning "${app}: no $Arch MSI in $app\dist\release; left out (use -Build to make one)"; continue }
  Write-Output "ok ${app}: $($msi.Name)"
  Copy-Item $msi.FullName (Join-Path $Stage "$app.msi")
  $included += $app
}
if (-not $included) { throw "no app MSIs found for $Arch" }

# The bundle's icon and the 64 px logo on its pages: OpenCraft Studio's mark, drawn here (four
# rounded tiles on a dark rounded square: the suite's apps). The .ico holds one 256 px PNG image,
# which Windows Vista and later read.
Add-Type -AssemblyName System.Drawing
function New-RoundedRect([single] $X, [single] $Y, [single] $W, [single] $R) {
  $p = New-Object System.Drawing.Drawing2D.GraphicsPath
  $d = 2 * $R
  $p.AddArc($X, $Y, $d, $d, 180, 90); $p.AddArc($X + $W - $d, $Y, $d, $d, 270, 90)
  $p.AddArc($X + $W - $d, $Y + $W - $d, $d, $d, 0, 90); $p.AddArc($X, $Y + $W - $d, $d, $d, 90, 90)
  $p.CloseFigure(); $p
}
function Save-Mark([int] $Size, [string] $Path) {
  $bmp = New-Object System.Drawing.Bitmap $Size, $Size
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
  $u = $Size / 64.0
  $g.FillPath((New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 0x1f, 0x24, 0x33))), (New-RoundedRect 0 0 ($Size - 1) (14 * $u)))
  $colors = @(@(0xf2, 0x6b, 0x3a), @(0x3a, 0x9b, 0xf2), @(0x5c, 0xc9, 0x7b), @(0xf2, 0xc1, 0x3a))
  for ($i = 0; $i -lt 4; $i++) {
    $c = $colors[$i]
    $brush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, $c[0], $c[1], $c[2]))
    $g.FillPath($brush, (New-RoundedRect ((12 + ($i % 2) * 22) * $u) ((12 + [math]::Floor($i / 2) * 22) * $u) (18 * $u) (5 * $u)))
  }
  $g.Dispose(); $bmp.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png); $bmp.Dispose()
}
$Logo = Join-Path $Stage 'logo.png'
Save-Mark 64 $Logo
Save-Mark 256 (Join-Path $Stage 'icon.png')
$png = [IO.File]::ReadAllBytes((Join-Path $Stage 'icon.png'))
$ico = New-Object IO.MemoryStream
$w = New-Object IO.BinaryWriter $ico
$w.Write([uint16]0); $w.Write([uint16]1); $w.Write([uint16]1)                 # ICONDIR: 1 image
$w.Write([byte]0); $w.Write([byte]0); $w.Write([byte]0); $w.Write([byte]0)   # 256x256, no palette
$w.Write([uint16]1); $w.Write([uint16]32); $w.Write([uint32]$png.Length); $w.Write([uint32]22)
$w.Write($png)
$Icon = Join-Path $Stage 'opencraft.ico'
[IO.File]::WriteAllBytes($Icon, $ico.ToArray())

$Exe = Join-Path $Dist "OpenCraft-Studio-Setup-$Version-$Arch.exe"
Write-Output "==> wix build ($($included -join ', '))"
wix build (Join-Path $PSScriptRoot 'opencraft.wxs') -arch $Arch `
  -ext WixToolset.BootstrapperApplications.wixext/5.0.2 `
  -d "Version=$Version" -d "Apps=$($included -join ';')" -d "MsiDir=$Stage" -d "IconPath=$Icon" -d "LogoPath=$Logo" `
  -o $Exe
if ($LASTEXITCODE -ne 0) { throw "wix build failed with exit code $LASTEXITCODE" }
Remove-Item -Force -ErrorAction SilentlyContinue ([IO.Path]::ChangeExtension($Exe, '.wixpdb'))
# ponytail: unsigned. Signing a bundle means `wix burn detach`, sign the engine, `wix burn reattach`,
# then sign the exe; add it with the apps' sign.ps1 secrets when releases ship the bundle.
Get-Item $Exe | Format-Table Name, Length
