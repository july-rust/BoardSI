# BoardSI Windows x64 developer launcher (source build or unpacked developer artifact).
# Bundled packages contain Qt runtime. Source builds may use QT_PREFIX. Not a final installer.
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
    $stackupValidator = Join-Path $PSScriptRoot 'boardsi-stackup-config.exe'
    $a = Join-Path $PSScriptRoot 'qt-model-a.frame'
    $b = Join-Path $PSScriptRoot 'qt-model-b.frame'
    $fixture = Join-Path $PSScriptRoot 'public-pads-v94-synthetic.asc'
    $defaultWorkingDirectory = $PSScriptRoot
} else {
    $root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
    $preview = Join-Path $root 'build-qt/boardsi-preview-p0.exe'
    $worker = Join-Path $root 'target/debug/boardsi-worker.exe'
    $importer = Join-Path $root 'target/debug/boardsi-import-artwork.exe'
    $stackupValidator = Join-Path $root 'target/debug/boardsi-stackup-config.exe'
    $a = Join-Path $root 'build-qt/qt-model-a.frame'
    $b = Join-Path $root 'build-qt/qt-model-b.frame'
    $fixture = Join-Path $root 'crates/boardsi-import/tests/fixtures/pads_route_straight_v94.asc'
    $defaultWorkingDirectory = Join-Path $root 'build-qt'
}
# Bundled-mode binaries require no Qt installation, QT_PREFIX or compiler SDK.
# Source-developer builds continue to use the pinned separately installed SDK.
$bundled = $packed -and (Test-Path -LiteralPath (Join-Path $PSScriptRoot 'Qt6Core.dll') -PathType Leaf)
if ($bundled) {
    $qtBin = $PSScriptRoot
    $platformDll = Join-Path $PSScriptRoot 'platforms/qwindows.dll'
    foreach ($file in @('Qt6Core.dll','Qt6Gui.dll','Qt6Network.dll','Qt6OpenGL.dll','Qt6OpenGLWidgets.dll','Qt6Widgets.dll','platforms/qwindows.dll',
                       'qt.conf','LICENSES/LGPL-3.0-only.txt','LICENSES/GPL-3.0-only.txt',
                       'source/qtbase-v6.8.3-source.zip', 'SHA256SUMS.txt')) {
        $expected = Join-Path $PSScriptRoot $file
        if (-not (Test-Path -LiteralPath $expected -PathType Leaf)) {
            throw ('Missing bundled Qt dependency, license or source: ' + $expected)
        }
    }
    # The package was assembled by the pinned CI. Verify its entire payload,
    # including Qt DLLs, source and notices; do not trust an incomplete copy.
    foreach ($entry in (Get-Content -LiteralPath (Join-Path $PSScriptRoot 'SHA256SUMS.txt'))) {
        $pair = $entry -split '  ', 2
        if ($pair.Count -ne 2 -or $pair[0] -notmatch '^[0-9a-fA-F]{64}$') {
            throw 'Malformed bundled SHA256SUMS.txt'
        }
        $file = Join-Path $PSScriptRoot $pair[1]
        if (-not (Test-Path -LiteralPath $file -PathType Leaf)) {
            throw ('Missing bundled manifest payload: ' + $pair[1])
        }
        if ((Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash -ne $pair[0]) {
            throw ('Bundled integrity failure: ' + $pair[1])
        }
    }
    $kitLabel = 'bundled Qt 6.8.3 MSVC 2022 x64'
} else {
    $qt = $env:QT_PREFIX
    if ([string]::IsNullOrWhiteSpace($qt)) {
        throw 'Qt runtime not bundled: install Qt 6.8.3 MSVC x64 SDK and set QT_PREFIX.'
    }
    $qt = [System.IO.Path]::GetFullPath($qt)
    $qtBin = Join-Path $qt 'bin'
    $platformDll = Join-Path $qt 'plugins/platforms/qwindows.dll'
    $qmake = Join-Path $qtBin 'qmake.exe'
    foreach ($required in @(
        (Join-Path $qtBin 'Qt6Core.dll'),(Join-Path $qtBin 'Qt6Gui.dll'),
        (Join-Path $qtBin 'Qt6Widgets.dll'),$platformDll,$qmake
    )) {
        if (-not (Test-Path -LiteralPath $required -PathType Leaf)) {
            throw ('Missing Qt SDK runtime/plugin: ' + $required)
        }
    }
    $qtVersion = (& $qmake -query QT_VERSION 2>&1 | Out-String).Trim()
    if ($LASTEXITCODE -ne 0 -or $qtVersion -ne '6.8.3') {
        throw ('Expected Qt 6.8.3, got: ' + $qtVersion)
    }
    $kitSpec = (& $qmake -query QMAKE_XSPEC 2>&1 | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) { throw 'Cannot query Qt target mkspec' }
    $archData = (& $qmake -query QT_INSTALL_ARCHDATA 2>&1 | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) { throw 'Cannot query Qt target architecture data' }
    $kitName = [System.IO.Path]::GetFileName($archData.TrimEnd([char[]]@('/', '\')))
    if ($kitSpec -ne 'win32-msvc' -or $kitName -ne 'msvc2022_64') {
        throw ('Expected MSVC 2022 x64 Qt 6.8.3, got spec=' + $kitSpec + ', kit=' + $kitName)
    }
    $kitLabel = 'Qt 6.8.3 MSVC 2022 x64 developer SDK'
}
foreach ($required in @($preview, $worker, $importer, $stackupValidator, $a, $b)) {
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
foreach ($native in @($preview, $worker, $importer, $stackupValidator,
                       (Join-Path $qtBin 'Qt6Core.dll'),
                       (Join-Path $qtBin 'Qt6Gui.dll'),
                       (Join-Path $qtBin 'Qt6Network.dll'),
                       (Join-Path $qtBin 'Qt6Widgets.dll'), $platformDll)) {
    Assert-Amd64PeFile $native
}
# The bundled GPU-enabled package must include both extra Qt DLLs, while
# source developer builds without Qt OpenGLWidgets remain runnable via CPU.
if ($bundled) {
    Assert-Amd64PeFile (Join-Path $qtBin 'Qt6OpenGL.dll')
    Assert-Amd64PeFile (Join-Path $qtBin 'Qt6OpenGLWidgets.dll')
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
Write-Host ('Qt runtime: ' + $kitLabel + ' ; Windows x64 preview, NOT a certified release')
Write-Host ('GUI: ' + $preview)
if ($ValidateOnly) {
    Write-Host 'VALIDATION ONLY - no GUI started or native execution claimed.'
    return
}
$launchArgs = @($worker, $a, $b, $importer)
if ($SelfTest) { $launchArgs = @('--selftest') + $launchArgs + @($fixture) }
$oldPath = $env:PATH
$oldPluginPath = $env:QT_PLUGIN_PATH
$oldValidator = $env:BOARDSI_STACKUP_VALIDATOR
$oldPlatform = $env:QT_QPA_PLATFORM
try {
    $env:PATH = $qtBin + ';' + $oldPath
    $env:QT_PLUGIN_PATH = if ($bundled) { $PSScriptRoot } else { Join-Path $qt 'plugins' }
    $env:QT_QPA_PLATFORM = 'windows'
    $env:BOARDSI_STACKUP_VALIDATOR = $stackupValidator
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
    $env:BOARDSI_STACKUP_VALIDATOR = $oldValidator
}
