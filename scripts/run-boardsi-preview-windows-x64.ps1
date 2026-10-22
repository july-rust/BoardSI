# BoardSI Windows x64 developer launcher (source build or unpacked developer artifact).
# Qt 6.8.3 MSVC x64 must be installed separately. NOT a redistribution installer.
param(
    [switch]$SelfTest,
    [switch]$ValidateOnly,
    [string]$EvidenceDirectory = ''
)
$ErrorActionPreference = 'Stop'
if ([System.Environment]::OSVersion.Platform.ToString() -ne 'Win32NT' -or
    [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString() -ne 'X64') {
    throw 'BoardSI Preview developer launcher requires Windows x64.'
}
# Package detection is anchored to the script directory, not to the caller's CWD.
$packed = Test-Path -LiteralPath (Join-Path $PSScriptRoot 'boardsi-preview-p0.exe') -PathType Leaf
if ($packed) {
    $preview = Join-Path $PSScriptRoot 'boardsi-preview-p0.exe'
    $worker = Join-Path $PSScriptRoot 'boardsi-worker.exe'
    $importer = Join-Path $PSScriptRoot 'boardsi-import-artwork.exe'
    $a = Join-Path $PSScriptRoot 'qt-model-a.frame'
    $b = Join-Path $PSScriptRoot 'qt-model-b.frame'
    $fixture = Join-Path $PSScriptRoot 'public-pads-v94-synthetic.asc'
    $defaultWorkingDirectory = $PSScriptRoot
} else {
    $root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
    $preview = Join-Path $root 'build-qt/boardsi-preview-p0.exe'
    $worker = Join-Path $root 'target/debug/boardsi-worker.exe'
    $importer = Join-Path $root 'target/debug/boardsi-import-artwork.exe'
    $a = Join-Path $root 'build-qt/qt-model-a.frame'
    $b = Join-Path $root 'build-qt/qt-model-b.frame'
    $fixture = Join-Path $root 'crates/boardsi-import/tests/fixtures/pads_route_straight_v94.asc'
    $defaultWorkingDirectory = Join-Path $root 'build-qt'
}
$qt = $env:QT_PREFIX
if ([string]::IsNullOrWhiteSpace($qt)) {
    throw 'QT_PREFIX must point to a separately installed Qt 6.8.3 MSVC 2022 x64 SDK. Qt is not bundled.'
}
$qt = [System.IO.Path]::GetFullPath($qt)
$qtBin = Join-Path $qt 'bin'
$qmake = Join-Path $qtBin 'qmake.exe'
$qtFiles = @(
    (Join-Path $qtBin 'Qt6Core.dll'),
    (Join-Path $qtBin 'Qt6Gui.dll'),
    (Join-Path $qtBin 'Qt6Widgets.dll'),
    (Join-Path $qt 'plugins/platforms/qwindows.dll'),
    $qmake
)
foreach ($required in $qtFiles) {
    if (-not (Test-Path -LiteralPath $required -PathType Leaf)) {
        throw ('Missing Qt SDK runtime/plugin component: ' + $required)
    }
}
$qtVersion = (& $qmake -query QT_VERSION 2>&1 | Out-String).Trim()
if ($LASTEXITCODE -ne 0 -or $qtVersion -ne '6.8.3') {
    throw ('Expected Qt 6.8.3 MSVC x64 SDK, qmake reports: ' + $qtVersion)
}
# QMAKE_XSPEC and architecture data distinguish the pinned MSVC x64 kit
# from a Qt 6.8.3 MinGW or ARM64 kit that has the same DLL filenames.
$kitSpec = (& $qmake -query QMAKE_XSPEC 2>&1 | Out-String).Trim()
if ($LASTEXITCODE -ne 0) { throw 'Cannot query Qt target mkspec' }
$archData = (& $qmake -query QT_INSTALL_ARCHDATA 2>&1 | Out-String).Trim()
if ($LASTEXITCODE -ne 0) { throw 'Cannot query Qt target architecture data' }
$kitName = [System.IO.Path]::GetFileName($archData.TrimEnd([char[]]@('/', '\')))
if ($kitSpec -ne 'win32-msvc' -or $kitName -ne 'msvc2022_64') {
    throw ('Incompatible Qt kit; expected Qt 6.8.3 MSVC 2022 x64, got spec=' +
           $kitSpec + ', archdata=' + $archData)
}
foreach ($required in @($preview, $worker, $importer, $a, $b)) {
    if (-not (Test-Path -LiteralPath $required -PathType Leaf)) {
        throw ('Missing BoardSI executable/model frame: ' + $required)
    }
}
if ($SelfTest -and -not (Test-Path -LiteralPath $fixture -PathType Leaf)) {
    throw ('Missing public synthetic test fixture: ' + $fixture)
}
# Validate executable and runtime PE headers rather than trusting the folder name.
function Assert-Amd64PeFile([string]$path) {
    $stream = [System.IO.File]::OpenRead($path)
    try {
        $reader = [System.IO.BinaryReader]::new($stream)
        if ($reader.ReadUInt16() -ne 0x5A4D) { throw 'Missing MZ header' }
        $stream.Position = 0x3C
        $peOffset = $reader.ReadInt32()
        if ($peOffset -lt 64 -or $peOffset -gt ($stream.Length - 6)) {
            throw 'Invalid PE header offset'
        }
        $stream.Position = $peOffset
        if ($reader.ReadUInt32() -ne 0x00004550) { throw 'Missing PE signature' }
        if ($reader.ReadUInt16() -ne 0x8664) { throw 'Not an AMD64 executable' }
    } catch {
        throw ('Windows x64 PE architecture check failed: ' + $path + ' / ' + $_.Exception.Message)
    } finally {
        $stream.Dispose()
    }
}
foreach ($native in @($preview, $worker, $importer,
                       (Join-Path $qtBin 'Qt6Core.dll'),
                       (Join-Path $qt 'plugins/platforms/qwindows.dll'))) {
    Assert-Amd64PeFile $native
}
if ([string]::IsNullOrWhiteSpace($EvidenceDirectory)) {
    $workingDirectory = $defaultWorkingDirectory
} else {
    $workingDirectory = [System.IO.Path]::GetFullPath(
        [System.IO.Path]::Combine((Get-Location).Path, $EvidenceDirectory))
}
if (-not (Test-Path -LiteralPath $workingDirectory -PathType Container)) {
    throw ('Evidence/working directory not found: ' + $workingDirectory)
}
Write-Host ('BoardSI developer launcher: ' + $(if ($packed) { 'unpacked artifact' } else { 'source tree' }))
Write-Host ('Qt SDK: ' + $qt + ' (6.8.3); Windows x64; NO Qt redistribution approval')
Write-Host ('GUI: ' + $preview)
if ($ValidateOnly) {
    Write-Host 'VALIDATION ONLY - no GUI started or native execution claimed.'
    return
}
$launchArgs = @($worker, $a, $b, $importer)
if ($SelfTest) { $launchArgs = @('--selftest') + $launchArgs + @($fixture) }
$oldPath = $env:PATH
$oldPluginPath = $env:QT_PLUGIN_PATH
$oldPlatform = $env:QT_QPA_PLATFORM
try {
    $env:PATH = $qtBin + ';' + $oldPath
    $env:QT_PLUGIN_PATH = Join-Path $qt 'plugins'
    $env:QT_QPA_PLATFORM = 'windows'
    # Smoke writes qt-preview-p0-results.json in the process working directory.
    # CI uses a directory outside the staged artifact to keep SHA256SUMS complete.
    Push-Location -LiteralPath $workingDirectory
    try {
        & $preview @launchArgs
        if ($LASTEXITCODE -ne 0) {
            throw ('BoardSI Preview exited with code ' + $LASTEXITCODE)
        }
        if ($SelfTest -and -not (Test-Path -LiteralPath 'qt-preview-p0-results.json' -PathType Leaf)) {
            throw 'Native GUI smoke receipt missing'
        }
    } finally {
        Pop-Location
    }
} finally {
    $env:PATH = $oldPath
    $env:QT_PLUGIN_PATH = $oldPluginPath
    $env:QT_QPA_PLATFORM = $oldPlatform
}
