# Architecture decision log (M0)

Decisions still open: Qt Widgets/Quick; Rust binding with C ABI vs CXX-Qt; versioned async IPC, snapshot/Run ID/heartbeat; toolchain versions and true three-native-platform portability. All inputs normalize via format-specific adapters to a Canonical Board Model preserving geometry/provenance. Viewer/Port/Solver must use identical physical truth. Do not claim a numerical result for unsupported 3D discontinuities. Record decisions and measurable prototypes here.
