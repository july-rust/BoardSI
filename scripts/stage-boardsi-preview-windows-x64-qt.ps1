# Build Qt-bundled Windows x64 Preview test package. QtBase source is included.
param([Parameter(Mandatory=$true)][string]$StagePath)
$ErrorActionPreference='Stop'
$stage=[IO.Path]::GetFullPath($StagePath)
$qt=[IO.Path]::GetFullPath($env:QT_PREFIX)
$bin=Join-Path $qt 'bin'
$qmake=Join-Path $bin 'qmake.exe'
$windeployqt=Join-Path $bin 'windeployqt.exe'
if (-not (Test-Path -LiteralPath $windeployqt)) { throw 'Qt deployment tool missing' }
$version=(& $qmake -query QT_VERSION | Out-String).Trim()
$spec=(& $qmake -query QMAKE_XSPEC | Out-String).Trim()
$archData=(& $qmake -query QT_INSTALL_ARCHDATA | Out-String).Trim()
$kit=[IO.Path]::GetFileName($archData.TrimEnd([char[]]@('/','\')))
if ($version -ne '6.8.3' -or $spec -ne 'win32-msvc' -or $kit -ne 'msvc2022_64') {
    throw "Wrong Qt SDK: $version $spec $kit"
}
if (Test-Path -LiteralPath $stage) { Remove-Item -LiteralPath $stage -Recurse -Force }
New-Item -ItemType Directory -Path $stage -Force | Out-Null
$items=@{
 'build-qt/boardsi-preview-p0.exe'='boardsi-preview-p0.exe'
 'target/debug/boardsi-worker.exe'='boardsi-worker.exe'
 'target/debug/boardsi-import-artwork.exe'='boardsi-import-artwork.exe'
 'build-qt/qt-model-a.frame'='qt-model-a.frame'
 'build-qt/qt-model-b.frame'='qt-model-b.frame'
 'crates/boardsi-import/tests/fixtures/pads_route_straight_v94.asc'='public-pads-v94-synthetic.asc'
 'crates/boardsi-import/tests/fixtures/not_a_pcb.asc'='not_a_pcb.asc'
 'scripts/run-boardsi-preview-windows-x64.ps1'='run-preview.ps1'
}
foreach ($path in $items.Keys) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {throw "Missing build input: $path"}
    Copy-Item -LiteralPath $path -Destination (Join-Path $stage $items[$path])
}
$priorPath=$env:PATH
try {
    $env:PATH=$bin+';'+$env:PATH
    & $windeployqt --release --no-translations --no-compiler-runtime --no-system-d3d-compiler --no-opengl-sw --no-plugins --dir $stage (Join-Path $stage 'boardsi-preview-p0.exe')
    if ($LASTEXITCODE -ne 0) {throw "windeployqt failed with $LASTEXITCODE"}
} finally {$env:PATH=$priorPath}
New-Item -ItemType Directory -Path (Join-Path $stage 'platforms') -Force | Out-Null
Copy-Item -LiteralPath (Join-Path $qt 'plugins/platforms/qwindows.dll') -Destination (Join-Path $stage 'platforms/qwindows.dll')
$expected=@('Qt6Core.dll','Qt6Gui.dll','Qt6Network.dll','Qt6Widgets.dll') | Sort-Object
$found=@(Get-ChildItem -LiteralPath $stage -File -Filter 'Qt6*.dll' | Select-Object -ExpandProperty Name | Sort-Object)
if (@(Compare-Object $expected $found).Count -gt 0) { throw "Unexpected Qt module set: $($found -join ',')" }
$crt=@(Get-ChildItem -LiteralPath $stage -Recurse -File | Where-Object {$_.Name -match '^(vcruntime|msvcp|concrt|ucrtbase|api-ms-win-).*\.dll$'})
if ($crt.Count -gt 0) {throw 'Do not redistribute individual MSVC CRT DLL files'}
@'
[Paths]
Prefix=.
Plugins=.
'@ | Set-Content -LiteralPath (Join-Path $stage 'qt.conf') -Encoding ascii
# Pin official Qt Project's v6.8.3 source commit rather than depending
# on third-party release mirrors that can return HTTP 404 in the CI runner.
$sourceName='qtbase-v6.8.3-source.zip'
$sourceCommit='c07c2d5a527a644d36e7853d55132ae38921682f'
$sourceDir=Join-Path $stage 'source'
$licenseDir=Join-Path $stage 'LICENSES'
New-Item -ItemType Directory -Path $sourceDir,$licenseDir -Force | Out-Null
$archive=Join-Path $sourceDir $sourceName
$checkout=Join-Path $env:RUNNER_TEMP 'boardsi-qtbase-6.8.3-source'
if (Test-Path -LiteralPath $checkout) {Remove-Item -LiteralPath $checkout -Force -Recurse}
& git -c core.longpaths=true clone --depth 1 --single-branch --branch v6.8.3 https://github.com/qt/qtbase.git $checkout
if ($LASTEXITCODE -ne 0) {throw 'Cannot fetch official QtBase release tag for source compliance'}
$actualCommit=(& git -C $checkout rev-parse HEAD | Out-String).Trim()
if ($LASTEXITCODE -ne 0 -or $actualCommit -ne $sourceCommit) {
    throw ('QtBase v6.8.3 commit identity mismatch: '+$actualCommit)
}
& git -C $checkout archive --format=zip --prefix=qtbase-v6.8.3/ --output=$archive HEAD
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $archive -PathType Leaf)) {
    throw 'Cannot archive official verified QtBase source'
}
$sourceSha=(Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
@"
Upstream: https://github.com/qt/qtbase/tree/v6.8.3
Exact git commit: $sourceCommit
Packaged source file: $sourceName
Packaged source SHA256: $sourceSha
Qt release: 6.8.3
No local source modifications.
"@ | Set-Content -LiteralPath (Join-Path $sourceDir 'SOURCE-PROVENANCE.txt') -Encoding utf8
$entries=@(& tar -tf $archive)
if ($LASTEXITCODE -ne 0) {throw 'Qt source archive index invalid'}
$extractRoot=Join-Path $env:RUNNER_TEMP 'boardsi-qt-license-extract'
New-Item -ItemType Directory -Path $extractRoot -Force | Out-Null
foreach ($name in @('LGPL-3.0-only.txt','GPL-3.0-only.txt')) {
    $match=@($entries | Where-Object {$_ -match ('(^|/)LICENSES/'+[regex]::Escape($name)+'$')})
    if ($match.Count -ne 1) {throw ('Exact LGPL/GPL license not found in source: '+$name)}
    & tar -xf $archive -C $extractRoot $match[0]
    if ($LASTEXITCODE -ne 0) {throw 'Qt license extraction failed'}
    $origin=Join-Path $extractRoot ($match[0] -replace '/', [IO.Path]::DirectorySeparatorChar)
    Copy-Item -LiteralPath $origin -Destination (Join-Path $licenseDir $name)
}
@'
BoardSI P2-A Windows x64 QT-BUNDLED TEST PACKAGE (NOT A RELEASE)
1. Extract all files from the archive.
2. Double-click boardsi-preview-p0.exe. No Qt SDK or QT_PREFIX needed.
3. Optional: PowerShell .\run-preview.ps1 -ValidateOnly
   then .\run-preview.ps1 -SelfTest or .\run-preview.ps1
4. Open PCB: restricted PADS V9.4 MILS straight-route subset only.
   No pads/vias/connectivity/stackup or physical PCB SI results.
5. If an MSVC runtime DLL is missing, install the official Microsoft
   Visual C++ 2015-2022 x64 Redistributable:
   https://aka.ms/vs/17/release/vc_redist.x64.exe

Qt 6.8.3 Core, Gui, Network, Widgets and qwindows are dynamically linked
and user-replaceable. Qt is copyright The Qt Company Ltd and contributors.
Full LGPLv3 and GPLv3 texts: LICENSES/ directory.
The COMPLETE EXACT QtBase 6.8.3 upstream source (including bundled
third-party code and license/attribution records) is INCLUDED:
source/qtbase-v6.8.3-source.zip
Official tagged source: https://github.com/qt/qtbase/tree/v6.8.3
Pinned source commit: c07c2d5a527a644d36e7853d55132ae38921682f
Source SHA256 and provenance: source/SOURCE-PROVENANCE.txt
The LGPL grants rights to modify/rebuild and substitute the Qt DLLs.
The application is not locked to unmodified Qt DLLs. No standalone
Microsoft compiler runtime DLL is intentionally distributed.
Qt/third-party legal compliance for general release is NOT_VERIFIED.
This is a development Preview only, NOT an SI analysis product.
'@ | Set-Content -LiteralPath (Join-Path $stage 'README.txt') -Encoding utf8
@'
NOTICE: BoardSI uses Qt 6.8.3 under LGPLv3. The Windows x64 test
package includes unmodified dynamically linked QtCore, QtGui, QtNetwork, QtWidgets
and the Windows qwindows platform plugin. Qt and other license terms,
attributions and the complete upstream QtBase source code are included
in the same package. Qt DLLs can be replaced with ABI-compatible
modified versions. Read ../README.txt and the full LGPLv3/GPLv3 texts.
'@ | Set-Content -LiteralPath (Join-Path $licenseDir 'QT-NOTICE.txt') -Encoding utf8
$payload=@(Get-ChildItem -LiteralPath $stage -File -Recurse | Where-Object {$_.Name -ne 'SHA256SUMS.txt'} | Sort-Object FullName)
if ($payload.Count -lt 17 -or $payload.Count -gt 50) {throw "Unexpected Qt package file count: $($payload.Count)"}
$lines=@($payload | ForEach-Object {
    $relative=$_.FullName.Substring($stage.Length+1).Replace('\','/')
    ('{0}  {1}' -f (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash,$relative)
})
$manifest=Join-Path $stage 'SHA256SUMS.txt'
$lines | Set-Content -LiteralPath $manifest -Encoding ascii
foreach ($line in (Get-Content -LiteralPath $manifest)) {
    $parts=$line -split '  ',2
    if ($parts.Count -ne 2 -or $parts[0] -notmatch '^[a-fA-F0-9]{64}$') {throw 'Bad SHA256 manifest format'}
    if ((Get-FileHash -LiteralPath (Join-Path $stage $parts[1]) -Algorithm SHA256).Hash -ne $parts[0]) {
        throw ('Mismatch of packaged SHA256: '+$parts[1])
    }
}
Write-Host "Qt-bundled test stage validated: $($payload.Count) payloads including QtBase source."
