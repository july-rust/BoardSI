# Local developer-only launcher for the tested Windows x64 Qt + Rust Preview.
# Requires the pinned Qt SDK and binaries already built per qt-runtime.yml.
# This is not a Qt DLL redistribution bundle or public installer.
param(
    [switch]$SelfTest
)
$ErrorActionPreference = 'Stop'
$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
if (-not [System.Environment]::OSVersion.Platform.ToString().Equals('Win32NT') -or
    [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString() -ne 'X64') {
    throw 'This launcher is specifically for Windows x64.'
}
$qt = $env:QT_PREFIX
if ([string]::IsNullOrWhiteSpace($qt)) {
    throw 'QT_PREFIX is required: point it to the pinned Qt 6.8.3 MSVC x64 SDK.'
}
$qtBin = Join-Path $qt 'bin'
$qtPlatforms = Join-Path $qt 'plugins/platforms'
if (-not (Test-Path (Join-Path $qtBin 'Qt6Core.dll')) -or
    -not (Test-Path (Join-Path $qtPlatforms 'qwindows.dll'))) {
    throw 'Qt runtime or Windows platform plugin missing from QT_PREFIX.'
}
$preview = Join-Path $root 'build-qt/boardsi-preview-p0.exe'
$worker = Join-Path $root 'target/debug/boardsi-worker.exe'
$importer = Join-Path $root 'target/debug/boardsi-import-artwork.exe'
$a = Join-Path $root 'build-qt/qt-model-a.frame'
$b = Join-Path $root 'build-qt/qt-model-b.frame'
foreach ($required in @($preview, $worker, $importer, $a, $b)) {
    if (-not (Test-Path -PathType Leaf $required)) {
        throw ('Missing developer build input. Reproduce qt-runtime.yml build: ' + $required)
    }
}
$env:PATH = $qtBin + ';' + $env:PATH
$env:QT_PLUGIN_PATH = Join-Path $qt 'plugins'
$env:QT_QPA_PLATFORM = 'windows'
$launchArgs = @($worker, $a, $b, $importer)
if ($SelfTest) {
    $fixture = Join-Path $root 'crates/boardsi-import/tests/fixtures/pads_route_straight_v94.asc'
    if (-not (Test-Path -PathType Leaf $fixture)) { throw 'Public test fixture missing.' }
    $launchArgs = @('--selftest') + $launchArgs + @($fixture)
}
& $preview @launchArgs
if ($LASTEXITCODE -ne 0) {
    throw ('BoardSI Preview exited with code ' + $LASTEXITCODE)
}
