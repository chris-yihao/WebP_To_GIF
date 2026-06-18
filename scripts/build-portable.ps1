$ErrorActionPreference = 'Stop'

$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$releaseDir = Join-Path $root 'src-tauri\target\release'
$appPath = Join-Path $releaseDir 'webPToGif.exe'
$dllPath = Join-Path $releaseDir 'WebView2Loader.dll'
$launcherSource = Join-Path $root 'scripts\portable_launcher.rs'
$launcherPath = Join-Path $releaseDir 'webPToGif-portable-launcher.exe'
$portablePath = Join-Path $releaseDir 'WebP_To_GIF_Portable.exe'
$iconPath = Join-Path $root 'src-tauri\icons\icon.ico'
$resourceScript = Join-Path $releaseDir 'portable_launcher.rc'
$resourceObject = Join-Path $releaseDir 'portable_launcher_res.o'

if (-not (Test-Path $appPath)) {
  throw "Missing release app: $appPath"
}

if (-not (Test-Path $dllPath)) {
  throw "Missing WebView2 loader: $dllPath"
}

if (-not (Test-Path $iconPath)) {
  throw "Missing app icon: $iconPath"
}

$cargoHome = Join-Path $root '.tools\cargo'
$rustupHome = Join-Path $root '.tools\rustup'
$w64Bin = Join-Path $root '.tools\w64devkit\bin'
$rustTarget = Join-Path $rustupHome 'toolchains\stable-x86_64-pc-windows-gnu\lib\rustlib\x86_64-pc-windows-gnu'
$gnuBin = Join-Path $rustTarget 'bin'
$gnuSelfBin = Join-Path $gnuBin 'self-contained'
$gnuSelfLib = Join-Path $rustTarget 'lib\self-contained'
$rustc = Join-Path $cargoHome 'bin\rustc.exe'

if (-not (Test-Path $rustc)) {
  throw "Missing rustc: $rustc"
}

$windres = Join-Path $w64Bin 'windres.exe'
if (-not (Test-Path $windres)) {
  throw "Missing windres: $windres"
}

$env:RUSTUP_HOME = $rustupHome
$env:CARGO_HOME = $cargoHome
$env:PATH = "$w64Bin;$cargoHome\bin;$gnuBin;$gnuSelfBin;$env:PATH"
$env:LIBRARY_PATH = "$gnuSelfLib;$env:LIBRARY_PATH"

$iconForResource = $iconPath -replace '\\', '/'
Set-Content -LiteralPath $resourceScript -Encoding ASCII -Value "1 ICON `"$iconForResource`""
& $windres $resourceScript -O coff -o $resourceObject

& $rustc $launcherSource `
  --edition 2021 `
  -C opt-level=z `
  -C panic=abort `
  -C linker="$w64Bin\gcc.exe" `
  -C link-arg="$resourceObject" `
  -o $launcherPath

$launcherBytes = [System.IO.File]::ReadAllBytes($launcherPath)
$appBytes = [System.IO.File]::ReadAllBytes($appPath)
$dllBytes = [System.IO.File]::ReadAllBytes($dllPath)
$magic = [System.Text.Encoding]::ASCII.GetBytes('WPGIFPK1')
$appLength = [System.BitConverter]::GetBytes([UInt64]$appBytes.Length)
$dllLength = [System.BitConverter]::GetBytes([UInt64]$dllBytes.Length)

$stream = [System.IO.File]::Open($portablePath, [System.IO.FileMode]::Create, [System.IO.FileAccess]::Write)
try {
  $stream.Write($launcherBytes, 0, $launcherBytes.Length)
  $stream.Write($appBytes, 0, $appBytes.Length)
  $stream.Write($dllBytes, 0, $dllBytes.Length)
  $stream.Write($magic, 0, $magic.Length)
  $stream.Write($appLength, 0, $appLength.Length)
  $stream.Write($dllLength, 0, $dllLength.Length)
} finally {
  $stream.Dispose()
}

Remove-Item -LiteralPath $launcherPath -Force
Remove-Item -LiteralPath $resourceScript -Force
Remove-Item -LiteralPath $resourceObject -Force
Write-Host "Portable exe created: $portablePath"
