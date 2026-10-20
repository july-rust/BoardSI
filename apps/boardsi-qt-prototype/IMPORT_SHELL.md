# BoardSI M1 Open PCB signature shell — restricted prototype

This separately built Qt Widgets application opens a local file selector and invokes the offline Rust `boardsi-import-probe` executable through asynchronous `QProcess`. The browser/UI does **not** parse PCB geometry, create a Canonical Board Model, display Artwork, or compute impedance/RLGC/S-parameters. Its result is always explicitly labeled SIGNATURE_ONLY / NOT_PARSED / NOT_RUN.

## Build

```sh
cargo build --locked -p boardsi-import --bin boardsi-import-probe
cmake -S apps/boardsi-qt-prototype -B build-qt -DCMAKE_PREFIX_PATH=/path/to/qt -DBUILD_TESTING=OFF
cmake --build build-qt --parallel 2
./build-qt/boardsi-qt-import-shell /absolute/path/to/target/debug/boardsi-import-probe
```

For a synthetic local test (do not put confidential CAD files in public CI):

```sh
./build-qt/boardsi-qt-import-shell /absolute/path/to/target/debug/boardsi-import-probe --selftest crates/boardsi-import/tests/fixtures/pads_header_only.asc PCB
./build-qt/boardsi-qt-import-shell /absolute/path/to/target/debug/boardsi-import-probe --selftest crates/boardsi-import/tests/fixtures/not_a_pcb.asc UNKNOWN
```

The optional selftest returns zero only for the expected exact Rust CLI response. It tests Qt/CLI async handoff with synthetic input; it does not establish PADS revision support or satisfy T01–T54 acceptance.

**Evidence boundary:** the existing Qt native workflow compiles CMake's default targets, including this shell, but does not currently run these new integration selftests. Do not claim cross-platform Qt/CLI runtime PASS until independently executed in the approved GitHub environment. Existing M0 F8 1000-ms timer tests are unchanged. No PH-08, protected ruleset, or proprietary artwork changes.