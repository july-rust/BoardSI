cmake_minimum_required(VERSION 3.21)
project(BoardSIWorkerPrototype LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
find_package(Qt6 6.5 REQUIRED COMPONENTS Core Gui Widgets)
add_executable(boardsi-qt-prototype main.cpp)
target_link_libraries(boardsi-qt-prototype PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
# Isolated opt-in F4 proof: synthetic Canonical Model identity/receipt, no physical solver.
add_executable(boardsi-qt-model-v2 model_v2.cpp)
target_link_libraries(boardsi-qt-model-v2 PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
add_executable(boardsi-preview-p0 preview_p0.cpp)
target_link_libraries(boardsi-preview-p0 PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
# D_056 independent opt-in source-layer via viewer; legacy Qt artwork unchanged.
add_executable(boardsi-d056-via-viewer d056_via_viewer.cpp)
target_link_libraries(boardsi-d056-via-viewer PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
if(MSVC)
  target_compile_options(boardsi-d056-via-viewer PRIVATE /W4 /WX)
else()
  target_compile_options(boardsi-d056-via-viewer PRIVATE -Wall -Wextra -Wpedantic -Werror)
endif()
if(WIN32)
  target_link_libraries(boardsi-preview-p0 PRIVATE psapi)
endif()
# D_056_04 opt-in strict Canonical via IPC decoder; main Artwork V1–V14 unchanged.
add_executable(boardsi-d05604-canonical-ipc-test d056_canonical_ipc_test.cpp)
target_link_libraries(boardsi-d05604-canonical-ipc-test PRIVATE Qt6::Core)
if(MSVC)
  target_compile_options(boardsi-d05604-canonical-ipc-test PRIVATE /W4 /WX)
else()
  target_compile_options(boardsi-d05604-canonical-ipc-test PRIVATE -Wall -Wextra -Wpedantic -Werror)
endif()
# GPU-first Artwork Viewer uses the stable Qt Graphics View OpenGL viewport.
# Optional at build time so machines without OpenGLWidgets can compile CPU fallback.
# This does NOT change the Rust Worker, Canonical data or SI capabilities.
# A_014: Qt Quick scene-graph / QRhi backend is opt-in at build capability.
# A CPU/OpenGL-only SDK continues to build the exact existing Preview target.
find_package(Qt6 6.5 QUIET COMPONENTS Quick)
if(TARGET Qt6::Quick)
  target_link_libraries(boardsi-preview-p0 PRIVATE Qt6::Quick)
  target_compile_definitions(boardsi-preview-p0 PRIVATE BOARDSI_RHI_ARTWORK=1)
  message(STATUS "BoardSI Qt Quick / RHI artwork renderer: AVAILABLE (runtime verification required)")
  if(APPLE)
    enable_language(OBJCXX)
    target_sources(boardsi-preview-p0 PRIVATE rhi_metal_probe.mm)
    set_source_files_properties(rhi_metal_probe.mm PROPERTIES COMPILE_OPTIONS "-fobjc-arc")
    target_link_libraries(boardsi-preview-p0 PRIVATE "-framework Metal" "-framework Foundation")
  endif()
else()
  message(WARNING "BoardSI Qt Quick / RHI unavailable, retaining existing artwork renderer")
endif()
find_package(Qt6 6.5 QUIET COMPONENTS OpenGLWidgets)
if(TARGET Qt6::OpenGLWidgets)
  target_link_libraries(boardsi-preview-p0 PRIVATE Qt6::OpenGLWidgets)
  target_compile_definitions(boardsi-preview-p0 PRIVATE BOARDSI_OPENGL_VIEWPORT=1)
  message(STATUS "BoardSI Artwork GPU viewport: ENABLED (runtime hardware probe, CPU fallback)")
else()
  message(WARNING "BoardSI Artwork GPU viewport: OpenGLWidgets unavailable, CPU fallback only")
endif()
if(MSVC)
  target_compile_options(boardsi-preview-p0 PRIVATE /W4 /WX)
  target_compile_options(boardsi-qt-prototype PRIVATE /W4 /WX)
  target_compile_options(boardsi-qt-model-v2 PRIVATE /W4 /WX)
else()
  target_compile_options(boardsi-preview-p0 PRIVATE -Wall -Wextra -Wpedantic -Werror)
  target_compile_options(boardsi-qt-prototype PRIVATE -Wall -Wextra -Wpedantic -Werror)
  target_compile_options(boardsi-qt-model-v2 PRIVATE -Wall -Wextra -Wpedantic -Werror)
endif()
include(CTest)
if(BUILD_TESTING)
  add_test(NAME qt-d05604-canonical-padstack-ipc
    COMMAND boardsi-d05604-canonical-ipc-test --selftest)
  set_tests_properties(qt-d05604-canonical-padstack-ipc PROPERTIES TIMEOUT 30)
  add_test(NAME qt-preview-d056-source-shaped-via-layers
    COMMAND boardsi-d056-via-viewer --selftest)
  set_tests_properties(qt-preview-d056-source-shaped-via-layers PROPERTIES
    TIMEOUT 30 ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
  find_package(Threads REQUIRED)
  find_package(Python3 REQUIRED COMPONENTS Interpreter)
  add_executable(boardsi-qt-runtime-validation tests/runtime.cpp)
  target_link_libraries(boardsi-qt-runtime-validation PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets Threads::Threads)
  if(MSVC)
    target_compile_options(boardsi-qt-runtime-validation PRIVATE /W4 /WX)
  else()
    target_compile_options(boardsi-qt-runtime-validation PRIVATE -Wall -Wextra -Wpedantic -Werror)
  endif()
  set(BOARDSI_WORKER "" CACHE FILEPATH "Absolute built Rust worker path")
  if(NOT BOARDSI_WORKER)
    message(FATAL_ERROR "BUILD_TESTING requires -DBOARDSI_WORKER=/absolute/path/to/boardsi-worker")
  endif()
  add_test(NAME qt-worker-runtime COMMAND boardsi-qt-runtime-validation
    "${BOARDSI_WORKER}" "${Python3_EXECUTABLE}" "${CMAKE_CURRENT_SOURCE_DIR}/tests/fault_worker.py")
  set_tests_properties(qt-worker-runtime PROPERTIES TIMEOUT 120 ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
  # Actual QApplication desktop controls and native Rust process exercised by P0.
  add_test(NAME qt-preview-p0-smoke COMMAND boardsi-preview-p0 --selftest
    "${BOARDSI_WORKER}" "${CMAKE_CURRENT_BINARY_DIR}/qt-model-a.frame" "${CMAKE_CURRENT_BINARY_DIR}/qt-model-b.frame")
  set_tests_properties(qt-preview-p0-smoke PROPERTIES TIMEOUT 35 ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
  # B_001: actual Qt editor and safe source-bound overlay roundtrip, not physical SI.
  add_test(NAME qt-preview-b001-stackup-reference
    COMMAND boardsi-preview-p0 --b001-stackup-selftest)
  set_tests_properties(qt-preview-b001-stackup-reference PROPERTIES TIMEOUT 30
    ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
  # A_020: separate synthetic UI + GPU/CPU source-filter regression, no real PCB data.
  add_test(NAME qt-preview-a020-explorer COMMAND boardsi-preview-p0 --explorer-selftest)
  set_tests_properties(qt-preview-a020-explorer PROPERTIES TIMEOUT 35
    ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
  add_test(NAME qt-preview-gpu-first-cpu-fallback
    COMMAND ${CMAKE_COMMAND} -E env BOARDSI_ARTWORK_RENDERER=cpu
    $<TARGET_FILE:boardsi-preview-p0> --renderer-selftest
    "${BOARDSI_WORKER}" "${CMAKE_CURRENT_BINARY_DIR}/qt-model-a.frame"
    "${CMAKE_CURRENT_BINARY_DIR}/qt-model-b.frame")
  set_tests_properties(qt-preview-gpu-first-cpu-fallback PROPERTIES TIMEOUT 20
    ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
  if(TARGET Qt6::Quick)
    # A_019 is a model-only capacity and source-precision check, not GPU FPS.
    add_test(NAME qt-preview-a019-rhi-model-scale
      COMMAND boardsi-preview-p0 --rhi-scale-selftest)
    set_tests_properties(qt-preview-a019-rhi-model-scale PROPERTIES TIMEOUT 120
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
  endif()
  # Optional P2-A import smoke: actual Rust binary and a public synthetic fixture.
  set(BOARDSI_IMPORTER "" CACHE FILEPATH "Absolute built PADS route importer")
  if(BOARDSI_IMPORTER)
    # C_005: source-verified Rust topology → Qt CPU/RHI marker and stale removal.
    add_test(NAME qt-preview-c005-reference-risk
      COMMAND boardsi-preview-p0 --c005-risk-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-c005-reference-risk PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
        # D_011: re-materialize source-only graph from independently verified imported geometry.
    add_test(NAME qt-preview-d011-source-contact-graph
      COMMAND boardsi-preview-p0 --d011-source-contact-graph-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d011-source-contact-graph PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_031 exact steep 3:4 non-45 oblique Via-A Source-owned endpoint, reuse Qt runtime workflow.
    add_executable(boardsi-d031-native-qt d031_qt_runner.cpp)
    target_link_libraries(boardsi-d031-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d031-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d031-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d031-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d031-non45-steep-via-a-endpoint-source
      COMMAND boardsi-d031-native-qt "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d031-non45-steep-via-a-endpoint-source PROPERTIES TIMEOUT 60
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_033 exact mirrored negative steep 3:4 non-45 oblique Via-A Source-owned endpoint, reuse Qt runtime workflow.
    add_executable(boardsi-d033-native-qt d033_qt_runner.cpp)
    target_link_libraries(boardsi-d033-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d033-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d033-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d033-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d033-non45-steep-mirrored-via-a-endpoint-source
      COMMAND boardsi-d033-native-qt "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d033-non45-steep-mirrored-via-a-endpoint-source PROPERTIES TIMEOUT 90
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_036: dual signed-negative 4:3 Via-A/Via-B SOURCE-only graph, existing CI.
    add_executable(boardsi-d036-native-qt d036_qt_runner.cpp)
    target_link_libraries(boardsi-d036-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d036-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d036-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d036-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d036-dual-mirrored-via-source
      COMMAND boardsi-d036-native-qt "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d036-dual-mirrored-via-source PROPERTIES TIMEOUT 90
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_037: mixed-sign negative-A positive-B 4:3 Via-A/Via-B SOURCE-only graph, existing CI.
    add_executable(boardsi-d037-native-qt d037_qt_runner.cpp)
    target_link_libraries(boardsi-d037-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d037-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d037-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d037-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d037-mixed-sign-dual-via-source
      COMMAND boardsi-d037-native-qt "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d037-mixed-sign-dual-via-source PROPERTIES TIMEOUT 90
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_053: Native SOURCE-authored finite-width bridge repair, no global electrical claims.
    add_executable(boardsi-d053-native-qt d053_qt_runner.cpp)
    target_link_libraries(boardsi-d053-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d053-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d053-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d053-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    if((WIN32 OR APPLE) AND "$ENV{GITHUB_ACTIONS}" STREQUAL "true")
      add_custom_command(TARGET boardsi-d053-native-qt POST_BUILD
        COMMAND "${CMAKE_COMMAND}" -E env "QT_QPA_PLATFORM=offscreen"
          "$<TARGET_FILE:boardsi-d053-native-qt>" "${BOARDSI_IMPORTER}"
          "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d052_dual_bridge_midspan_break_both_v94.asc"
        VERBATIM)
    endif()
    add_test(NAME qt-preview-d053-dual-bridge-real-midspan-repair-source
      COMMAND boardsi-d053-native-qt "${BOARDSI_IMPORTER}"
        "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d052_dual_bridge_midspan_break_both_v94.asc")
    set_tests_properties(qt-preview-d053-dual-bridge-real-midspan-repair-source PROPERTIES TIMEOUT 240
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_052: SOURCE-real finite-width dual bridge midspan cuts.
    add_executable(boardsi-d052-native-qt d052_qt_runner.cpp)
    target_link_libraries(boardsi-d052-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d052-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d052-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d052-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    if((WIN32 OR APPLE) AND "$ENV{GITHUB_ACTIONS}" STREQUAL "true")
      add_custom_command(TARGET boardsi-d052-native-qt POST_BUILD
        COMMAND "${CMAKE_COMMAND}" -E env "QT_QPA_PLATFORM=offscreen"
          "$<TARGET_FILE:boardsi-d052-native-qt>" "${BOARDSI_IMPORTER}"
          "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d052_dual_bridge_midspan_break_both_v94.asc"
        VERBATIM)
    endif()
    add_test(NAME qt-preview-d052-dual-bridge-midspan-break-source
      COMMAND boardsi-d052-native-qt "${BOARDSI_IMPORTER}"
        "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d052_dual_bridge_midspan_break_both_v94.asc")
    set_tests_properties(qt-preview-d052-dual-bridge-midspan-break-source PROPERTIES TIMEOUT 180
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_051: SOURCE-local dual finite-width bridge loop / virtual single-cut.
    add_executable(boardsi-d051-native-qt d051_qt_runner.cpp)
    target_link_libraries(boardsi-d051-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d051-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d051-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d051-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    if((WIN32 OR APPLE) AND "$ENV{GITHUB_ACTIONS}" STREQUAL "true")
      add_custom_command(TARGET boardsi-d051-native-qt POST_BUILD
        COMMAND "${CMAKE_COMMAND}" -E env "QT_QPA_PLATFORM=offscreen"
          "$<TARGET_FILE:boardsi-d051-native-qt>" "${BOARDSI_IMPORTER}"
          "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d051_mixed_pad_dual_finite_width_loop_v94.asc"
        VERBATIM)
    endif()
    add_test(NAME qt-preview-d051-mixed-pad-dual-bridge-loop-source
      COMMAND boardsi-d051-native-qt "${BOARDSI_IMPORTER}"
        "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d051_mixed_pad_dual_finite_width_loop_v94.asc")
    set_tests_properties(qt-preview-d051-mixed-pad-dual-bridge-loop-source PROPERTIES TIMEOUT 150
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_050: SOURCE finite-width bridge, one connected mixed-pad component.
    add_executable(boardsi-d050-native-qt d050_qt_runner.cpp)
    target_link_libraries(boardsi-d050-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d050-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d050-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d050-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    if((WIN32 OR APPLE) AND "$ENV{GITHUB_ACTIONS}" STREQUAL "true")
      add_custom_command(TARGET boardsi-d050-native-qt POST_BUILD
        COMMAND "${CMAKE_COMMAND}" -E env "QT_QPA_PLATFORM=offscreen"
          "$<TARGET_FILE:boardsi-d050-native-qt>" "${BOARDSI_IMPORTER}"
          "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d050_mixed_pad_finite_width_bridge_v94.asc"
        VERBATIM)
    endif()
    add_test(NAME qt-preview-d050-mixed-pad-finite-width-bridge-source
      COMMAND boardsi-d050-native-qt "${BOARDSI_IMPORTER}"
        "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d050_mixed_pad_finite_width_bridge_v94.asc")
    set_tests_properties(qt-preview-d050-mixed-pad-finite-width-bridge-source PROPERTIES TIMEOUT 90
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_049 mixed round/square SMD Pad dual-contact SOURCE composition; use existing CI matrix.
    add_executable(boardsi-d049-native-qt d049_qt_runner.cpp)
    target_link_libraries(boardsi-d049-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d049-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d049-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d049-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    if((WIN32 OR APPLE) AND "$ENV{GITHUB_ACTIONS}" STREQUAL "true")
      add_custom_command(TARGET boardsi-d049-native-qt POST_BUILD
        COMMAND "${CMAKE_COMMAND}" -E env "QT_QPA_PLATFORM=offscreen"
          "$<TARGET_FILE:boardsi-d049-native-qt>" "${BOARDSI_IMPORTER}"
          "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d049_mixed_round_square_dual_width_v94.asc"
        VERBATIM)
    endif()
    add_test(NAME qt-preview-d049-mixed-pad-dual-finite-width-source
      COMMAND boardsi-d049-native-qt "${BOARDSI_IMPORTER}"
        "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d049_mixed_round_square_dual_width_v94.asc")
    set_tests_properties(qt-preview-d049-mixed-pad-dual-finite-width-source PROPERTIES TIMEOUT 90
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_048 bounded finite-width round SMD Pad SOURCE contacts; use existing CI matrix.
    add_executable(boardsi-d048-native-qt d048_qt_runner.cpp)
    target_link_libraries(boardsi-d048-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d048-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d048-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d048-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    if((WIN32 OR APPLE) AND "$ENV{GITHUB_ACTIONS}" STREQUAL "true")
      add_custom_command(TARGET boardsi-d048-native-qt POST_BUILD
        COMMAND "${CMAKE_COMMAND}" -E env "QT_QPA_PLATFORM=offscreen"
          "$<TARGET_FILE:boardsi-d048-native-qt>" "${BOARDSI_IMPORTER}"
          "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d048_trace_round_pad_width_v94.asc"
        VERBATIM)
    endif()
    add_test(NAME qt-preview-d048-trace-round-pad-finite-width-source
      COMMAND boardsi-d048-native-qt "${BOARDSI_IMPORTER}"
        "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d048_trace_round_pad_width_v94.asc")
    set_tests_properties(qt-preview-d048-trace-round-pad-finite-width-source PROPERTIES TIMEOUT 90
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_047 bounded finite-width oblique SOURCE contacts; use existing CI matrix.
    add_executable(boardsi-d047-native-qt d047_qt_runner.cpp)
    target_link_libraries(boardsi-d047-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d047-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d047-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d047-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    if((WIN32 OR APPLE) AND "$ENV{GITHUB_ACTIONS}" STREQUAL "true")
      add_custom_command(TARGET boardsi-d047-native-qt POST_BUILD
        COMMAND "${CMAKE_COMMAND}" -E env "QT_QPA_PLATFORM=offscreen"
          "$<TARGET_FILE:boardsi-d047-native-qt>" "${BOARDSI_IMPORTER}"
          "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d047_trace_square_pad_width_v94.asc"
        VERBATIM)
    endif()
    add_test(NAME qt-preview-d047-trace-square-pad-finite-width-source
      COMMAND boardsi-d047-native-qt "${BOARDSI_IMPORTER}"
        "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d047_trace_square_pad_width_v94.asc")
    set_tests_properties(qt-preview-d047-trace-square-pad-finite-width-source PROPERTIES TIMEOUT 90
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_046 bounded finite-width oblique SOURCE contacts; use existing CI matrix.
    add_executable(boardsi-d046-native-qt d046_qt_runner.cpp)
    target_link_libraries(boardsi-d046-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d046-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d046-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d046-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    if((WIN32 OR APPLE) AND "$ENV{GITHUB_ACTIONS}" STREQUAL "true")
      add_custom_command(TARGET boardsi-d046-native-qt POST_BUILD
        COMMAND "${CMAKE_COMMAND}" -E env "QT_QPA_PLATFORM=offscreen"
          "$<TARGET_FILE:boardsi-d046-native-qt>" "${BOARDSI_IMPORTER}"
          "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d046_oblique_width_v94.asc"
        VERBATIM)
    endif()
    add_test(NAME qt-preview-d046-oblique-finite-width-source
      COMMAND boardsi-d046-native-qt "${BOARDSI_IMPORTER}"
        "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d046_oblique_width_v94.asc")
    set_tests_properties(qt-preview-d046-oblique-finite-width-source PROPERTIES TIMEOUT 90
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_045 bounded finite-width perpendicular SOURCE contacts; use existing CI matrix.
    add_executable(boardsi-d045-native-qt d045_qt_runner.cpp)
    target_link_libraries(boardsi-d045-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d045-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d045-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d045-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    if((WIN32 OR APPLE) AND "$ENV{GITHUB_ACTIONS}" STREQUAL "true")
      add_custom_command(TARGET boardsi-d045-native-qt POST_BUILD
        COMMAND "${CMAKE_COMMAND}" -E env "QT_QPA_PLATFORM=offscreen"
          "$<TARGET_FILE:boardsi-d045-native-qt>" "${BOARDSI_IMPORTER}"
          "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d045_perpendicular_width_v94.asc"
        VERBATIM)
    endif()
    add_test(NAME qt-preview-d045-perpendicular-finite-width-source
      COMMAND boardsi-d045-native-qt "${BOARDSI_IMPORTER}"
        "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d045_perpendicular_width_v94.asc")
    set_tests_properties(qt-preview-d045-perpendicular-finite-width-source PROPERTIES TIMEOUT 90
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_044 bounded finite-width parallel SOURCE contacts; use existing CI matrix.
    add_executable(boardsi-d044-native-qt d044_qt_runner.cpp)
    target_link_libraries(boardsi-d044-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d044-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d044-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d044-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    if((WIN32 OR APPLE) AND "$ENV{GITHUB_ACTIONS}" STREQUAL "true")
      add_custom_command(TARGET boardsi-d044-native-qt POST_BUILD
        COMMAND "${CMAKE_COMMAND}" -E env "QT_QPA_PLATFORM=offscreen"
          "$<TARGET_FILE:boardsi-d044-native-qt>" "${BOARDSI_IMPORTER}"
          "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d044_parallel_width_v94.asc"
        VERBATIM)
    endif()
    add_test(NAME qt-preview-d044-parallel-finite-width-source
      COMMAND boardsi-d044-native-qt "${BOARDSI_IMPORTER}"
        "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d044_parallel_width_v94.asc")
    set_tests_properties(qt-preview-d044-parallel-finite-width-source PROPERTIES TIMEOUT 90
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_043 independent SOURCE-local counterfactual cut-set regression.
    add_executable(boardsi-d043-native-qt d043_qt_runner.cpp)
    target_link_libraries(boardsi-d043-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d043-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d043-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d043-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    if((WIN32 OR APPLE) AND "$ENV{GITHUB_ACTIONS}" STREQUAL "true")
      add_custom_command(TARGET boardsi-d043-native-qt POST_BUILD
        COMMAND "${CMAKE_COMMAND}" -E env "QT_QPA_PLATFORM=offscreen"
          "$<TARGET_FILE:boardsi-d043-native-qt>" "${BOARDSI_IMPORTER}"
          "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d042_local_closed_loop_v94.asc"
        VERBATIM)
    endif()
    add_test(NAME qt-preview-d043-local-trace-cutset-source
      COMMAND boardsi-d043-native-qt "${BOARDSI_IMPORTER}"
        "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d042_local_closed_loop_v94.asc")
    set_tests_properties(qt-preview-d043-local-trace-cutset-source PROPERTIES TIMEOUT 90
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_042 source-owned second path and one local cycle; existing native CI.
    add_executable(boardsi-d042-native-qt d042_qt_runner.cpp)
    target_link_libraries(boardsi-d042-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d042-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d042-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d042-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    if((WIN32 OR APPLE) AND "$ENV{GITHUB_ACTIONS}" STREQUAL "true")
      add_custom_command(TARGET boardsi-d042-native-qt POST_BUILD
        COMMAND "${CMAKE_COMMAND}" -E env "QT_QPA_PLATFORM=offscreen"
          "$<TARGET_FILE:boardsi-d042-native-qt>" "${BOARDSI_IMPORTER}"
          "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d042_local_closed_loop_v94.asc"
        VERBATIM)
    endif()
    add_test(NAME qt-preview-d042-local-closed-loop-source
      COMMAND boardsi-d042-native-qt "${BOARDSI_IMPORTER}"
        "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d042_local_closed_loop_v94.asc")
    set_tests_properties(qt-preview-d042-local-closed-loop-source PROPERTIES TIMEOUT 90
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_041 source-owned exact same-layer bridge; existing native CI.
    add_executable(boardsi-d041-native-qt d041_qt_runner.cpp)
    target_link_libraries(boardsi-d041-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d041-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d041-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d041-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    # Native Windows/macOS Actions run the same regression after building.
    # Linux already executes the registered CTest in its existing workflow.
    if((WIN32 OR APPLE) AND "$ENV{GITHUB_ACTIONS}" STREQUAL "true")
      add_custom_command(TARGET boardsi-d041-native-qt POST_BUILD
        COMMAND "${CMAKE_COMMAND}" -E env "QT_QPA_PLATFORM=offscreen"
          "$<TARGET_FILE:boardsi-d041-native-qt>" "${BOARDSI_IMPORTER}"
          "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d041_local_bridge_v94.asc"
        VERBATIM)
    endif()
    add_test(NAME qt-preview-d041-local-source-bridge
      COMMAND boardsi-d041-native-qt "${BOARDSI_IMPORTER}"
        "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d041_local_bridge_v94.asc")
    set_tests_properties(qt-preview-d041-local-source-bridge PROPERTIES TIMEOUT 90
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_040 same-layer multiple local Copper components; same existing native CI matrix.
    add_executable(boardsi-d040-native-qt d040_qt_runner.cpp)
    target_link_libraries(boardsi-d040-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d040-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d040-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d040-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d040-multi-component-copper-source
      COMMAND boardsi-d040-native-qt "${BOARDSI_IMPORTER}"
        "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d040_multi_component_v94.asc")
    set_tests_properties(qt-preview-d040-multi-component-copper-source PROPERTIES TIMEOUT 90
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_039: bounded same-layer two-Pad/two-Trace local copper geometry; existing native CI.
    add_executable(boardsi-d039-native-qt d039_qt_runner.cpp)
    target_link_libraries(boardsi-d039-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d039-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d039-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d039-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d039-local-copper-source
      COMMAND boardsi-d039-native-qt "${BOARDSI_IMPORTER}"
        "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/d039_local_copper_v94.asc")
    set_tests_properties(qt-preview-d039-local-copper-source PROPERTIES TIMEOUT 90
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_038: mixed-sign positive-A negative-B 4:3 Via-A/Via-B SOURCE-only graph, existing CI.
    add_executable(boardsi-d038-native-qt d038_qt_runner.cpp)
    target_link_libraries(boardsi-d038-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d038-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d038-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d038-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d038-mixed-sign-dual-via-source
      COMMAND boardsi-d038-native-qt "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d038-mixed-sign-dual-via-source PROPERTIES TIMEOUT 90
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_035 exact mirrored negative shallow 4:3 non-45 oblique Via-A Source-owned endpoint, reuse Qt runtime workflow.
    add_executable(boardsi-d035-native-qt d035_qt_runner.cpp)
    target_link_libraries(boardsi-d035-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d035-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d035-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d035-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d035-non45-oblique-mirrored-via-a-endpoint-source
      COMMAND boardsi-d035-native-qt "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d035-non45-oblique-mirrored-via-a-endpoint-source PROPERTIES TIMEOUT 90
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_034 exact mirrored negative shallow 4:3 non-45 oblique Via-B Source-owned endpoint, reuse Qt runtime workflow.
    add_executable(boardsi-d034-native-qt d034_qt_runner.cpp)
    target_link_libraries(boardsi-d034-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d034-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d034-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d034-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d034-non45-oblique-mirrored-via-b-endpoint-source
      COMMAND boardsi-d034-native-qt "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d034-non45-oblique-mirrored-via-b-endpoint-source PROPERTIES TIMEOUT 90
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_032 exact mirrored negative steep 3:4 non-45 oblique Via-B Source-owned endpoint, reuse Qt runtime workflow.
    add_executable(boardsi-d032-native-qt d032_qt_runner.cpp)
    target_link_libraries(boardsi-d032-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d032-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d032-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d032-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d032-non45-steep-mirrored-via-b-endpoint-source
      COMMAND boardsi-d032-native-qt "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d032-non45-steep-mirrored-via-b-endpoint-source PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_030 exact steep 3:4 non-45 oblique Via-B Source-owned endpoint, reuse Qt runtime workflow.
    add_executable(boardsi-d030-native-qt d030_qt_runner.cpp)
    target_link_libraries(boardsi-d030-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d030-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d030-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d030-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d030-non45-steep-via-b-endpoint-source
      COMMAND boardsi-d030-native-qt "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d030-non45-steep-via-b-endpoint-source PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_029 exact 4:3 non-45 oblique Via-A Source-owned endpoint, reuse Qt runtime workflow.
    add_executable(boardsi-d029-native-qt d029_qt_runner.cpp)
    target_link_libraries(boardsi-d029-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d029-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d029-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d029-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d029-non45-oblique-via-a-endpoint-source
      COMMAND boardsi-d029-native-qt "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d029-non45-oblique-via-a-endpoint-source PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_028 exact 4:3 non-45 oblique Via-B Source-owned endpoint, reuse Qt runtime workflow.
    add_executable(boardsi-d028-native-qt d028_qt_runner.cpp)
    target_link_libraries(boardsi-d028-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d028-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d028-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d028-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d028-non45-oblique-via-b-endpoint-source
      COMMAND boardsi-d028-native-qt "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d028-non45-oblique-via-b-endpoint-source PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_027 45-degree oblique Via-A Source-owned endpoint, reuse Qt runtime workflow.
    add_executable(boardsi-d027-native-qt d027_qt_runner.cpp)
    target_link_libraries(boardsi-d027-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d027-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d027-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d027-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d027-oblique-via-a-endpoint-source
      COMMAND boardsi-d027-native-qt "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d027-oblique-via-a-endpoint-source PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_026 45-degree oblique Via-B Source-owned endpoint, reuse Qt runtime workflow.
    add_executable(boardsi-d026-native-qt d026_qt_runner.cpp)
    target_link_libraries(boardsi-d026-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d026-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d026-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d026-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d026-oblique-via-b-endpoint-source
      COMMAND boardsi-d026-native-qt "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d026-oblique-via-b-endpoint-source PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_025 orthogonal Via-A Source-owned endpoint, reuse Qt runtime workflow.
    add_executable(boardsi-d025-native-qt d025_qt_runner.cpp)
    target_link_libraries(boardsi-d025-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d025-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d025-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d025-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d025-orthogonal-via-a-endpoint-source
      COMMAND boardsi-d025-native-qt "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d025-orthogonal-via-a-endpoint-source PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_024 orthogonal Via-B Source-owned endpoint, reuse Qt runtime workflow.
    add_executable(boardsi-d024-native-qt d024_qt_runner.cpp)
    target_link_libraries(boardsi-d024-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d024-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d024-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d024-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d024-orthogonal-endpoint-source
      COMMAND boardsi-d024-native-qt "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d024-orthogonal-endpoint-source PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_023 opposite Via-A Source-owned endpoint, reuse Qt runtime workflow.
    add_executable(boardsi-d023-native-qt d023_qt_runner.cpp)
    target_link_libraries(boardsi-d023-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d023-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d023-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d023-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d023-opposite-endpoint-source
      COMMAND boardsi-d023-native-qt "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d023-opposite-endpoint-source PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_022 real Source-only proper endpoint collinear Trace contact; no new workflow.
    add_executable(boardsi-d022-native-qt d022_qt_runner.cpp)
    target_link_libraries(boardsi-d022-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d022-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d022-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d022-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d022-endpoint-contact-source
      COMMAND boardsi-d022-native-qt "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d022-endpoint-contact-source PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_021 real Source-only proper partial collinear overlap; no new workflow.
    add_executable(boardsi-d021-native-qt d021_qt_runner.cpp)
    target_link_libraries(boardsi-d021-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d021-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d021-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d021-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d021-partial-overlap-source
      COMMAND boardsi-d021-native-qt "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d021-partial-overlap-source PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_020 real Source-only strictly contained collinear Trace overlap.
    add_executable(boardsi-d020-native-qt d020_qt_runner.cpp)
    target_link_libraries(boardsi-d020-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d020-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d020-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d020-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d020-contained-overlap-source
      COMMAND boardsi-d020-native-qt "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d020-contained-overlap-source PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_019 independent real-source oblique T (existing Native CI, no new workflow).
    add_executable(boardsi-d019-native-qt d019_qt_runner.cpp)
    target_link_libraries(boardsi-d019-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d019-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d019-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d019-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d019-oblique-t-junction-source
      COMMAND boardsi-d019-native-qt "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d019-oblique-t-junction-source PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_017 real-source oblique X-junction qualification, no new Actions workflow.
    add_executable(boardsi-d017-native-qt d017_qt_runner.cpp)
    target_link_libraries(boardsi-d017-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d017-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d017-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d017-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d017-oblique-x-junction-source
      COMMAND boardsi-d017-native-qt "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d017-oblique-x-junction-source PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_016 real-source X-junction qualification, no new Actions workflow.
    add_executable(boardsi-d016-native-qt d016_qt_runner.cpp)
    target_link_libraries(boardsi-d016-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d016-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d016-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d016-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d016-x-junction-source
      COMMAND boardsi-d016-native-qt "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d016-x-junction-source PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_015 independent real-source T-junction qualification. Existing Actions only.
    add_executable(boardsi-d015-native-qt d015_qt_runner.cpp)
    target_link_libraries(boardsi-d015-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d015-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d015-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d015-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d015-t-junction-source
      COMMAND boardsi-d015-native-qt "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d015-t-junction-source PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_014 independently compiled Qt Source importer proof. No new Actions workflow.
    add_executable(boardsi-d014-native-qt d014_qt_runner.cpp)
    target_link_libraries(boardsi-d014-native-qt PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
    if(WIN32)
      target_link_libraries(boardsi-d014-native-qt PRIVATE psapi)
    endif()
    if(MSVC)
      target_compile_options(boardsi-d014-native-qt PRIVATE /W4 /WX)
    else()
      target_compile_options(boardsi-d014-native-qt PRIVATE -Wall -Wextra -Wpedantic -Werror)
    endif()
    add_test(NAME qt-preview-d014-two-via-source-chain
      COMMAND boardsi-d014-native-qt "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d014-two-via-source-chain PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_013: compatibility barrel projection must not be accepted as electrical.
    add_test(NAME qt-preview-d013-legacy-barrel-provenance
      COMMAND boardsi-preview-p0 --d013-legacy-barrel-provenance-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d013-legacy-barrel-provenance PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_012: Source Via span evidence is not physical plated-barrel continuity.
    add_test(NAME qt-preview-d012-via-span-provenance
      COMMAND boardsi-preview-p0 --d012-via-span-provenance-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d012-via-span-provenance PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_010: independently imported two-trace/four-area candidate, source geometry only.
    add_test(NAME qt-preview-d010-two-trace-four-area
      COMMAND boardsi-preview-p0 --d010-dual-layer-two-trace-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d010-two-trace-four-area PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_009: independently imported dual-layer source candidate.
    add_test(NAME qt-preview-d009-dual-layer-via-trace
      COMMAND boardsi-preview-p0 --d009-dual-layer-via-trace-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d009-dual-layer-via-trace PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_008: actual source-imported two-layer Via pad/COPCLS proof (geometry only).
    add_test(NAME qt-preview-d008-dual-layer-via-area
      COMMAND boardsi-preview-p0 --d008-dual-layer-via-area-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d008-dual-layer-via-area PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_007: independent Source Via Pad -> Straight Trace -> distinct COPCLS.
    add_test(NAME qt-preview-d007-via-trace-area
      COMMAND boardsi-preview-p0 --d007-via-trace-three-area-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d007-via-trace-area PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_006: independent Source plated Via pad / three COPCLS-COPCUT validation.
    add_test(NAME qt-preview-d006-via-three-area
      COMMAND boardsi-preview-p0 --d006-via-three-area-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d006-via-three-area PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_005: source Pad -> Trace -> Area independent imported-Qt readback.
    add_test(NAME qt-preview-d005-pad-trace-area
      COMMAND boardsi-preview-p0 --d005-pad-trace-area-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d005-pad-trace-area PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_004: source Trace endpoint attachment / independent native Qt readback.
    add_test(NAME qt-preview-d004-endpoint-attachment
      COMMAND boardsi-preview-p0 --d004-endpoint-attachment-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d004-endpoint-attachment PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_003: independent Rust 3-area COPCUT graph with stale-source guards.
    add_test(NAME qt-preview-d003-three-area
      COMMAND boardsi-preview-p0 --d003-three-area-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d003-three-area PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_002: independent Rust source COPCUT readback and source-change rejection.
    add_test(NAME qt-preview-d002-copcut-contact
      COMMAND boardsi-preview-p0 --d002-copcut-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d002-copcut-contact PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # D_001: actual Rust Source Pad witness -> Qt independent source geometry/stale.
    add_test(NAME qt-preview-d001-pad-endpoint
      COMMAND boardsi-preview-p0 --d001-pad-endpoint-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-d001-pad-endpoint PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # C_023 V15: exact source interval gap and Qt independent readback/stale.
    add_test(NAME qt-preview-c023-reference-corridor
      COMMAND boardsi-preview-p0 --c023-corridor-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-c023-reference-corridor PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # C_022: 3..6 Source rectangles, Qt independent component groups and stale.
    add_test(NAME qt-preview-c022-reference-multi-area
      COMMAND boardsi-preview-p0 --c022-area-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-c022-reference-multi-area PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # C_021: exactly two source rectangles, actual Rust->Qt warning/stale.
    add_test(NAME qt-preview-c021-reference-two-area
      COMMAND boardsi-preview-p0 --c021-area-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-c021-reference-two-area PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # C_014: Source 0-8 rectangular COPCUT, Qt independent decode, stale state.
    add_test(NAME qt-preview-c014-multi-cutout
      COMMAND boardsi-preview-p0 --c014-multi-cutout-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-c014-multi-cutout PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # C_013: Signal Via Pad versus explicitly source-owned Reference COPCUT.
    add_test(NAME qt-preview-c013-signal-via-cutout
      COMMAND boardsi-preview-p0 --c013-cutout-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-c013-signal-via-cutout PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # C_012: exact Source Signal/Stitch Via Pad-to-Pad rim and stale GUI.
    add_test(NAME qt-preview-c012-via-pad-clearance
      COMMAND boardsi-preview-p0 --c012-clearance-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-c012-via-pad-clearance PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # C_011: explicit circular Stitch Via Pad rim vs source Copper and Cutout.
    add_test(NAME qt-preview-c011-reference-pad-footprint
      COMMAND boardsi-preview-p0 --c011-pad-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-c011-reference-pad-footprint PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # C_010: Source-owned Corridor to Reference Stitch same-Area risk and stale.
    add_test(NAME qt-preview-c010-reference-corridor-stitch
      COMMAND boardsi-preview-p0 --c010-corridor-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-c010-reference-corridor-stitch PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # C_009: strict Reference Stitch candidate and real Rust->Qt stale smoke.
    add_test(NAME qt-preview-c009-reference-stitch-candidate
      COMMAND boardsi-preview-p0 --c009-stitch-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-c009-reference-stitch-candidate PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # C_008: two-layer plated-via actual importer + Qt stale/risk smoke.
    add_test(NAME qt-preview-c008-via-reference-risk
      COMMAND boardsi-preview-p0 --c008-via-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-c008-via-reference-risk PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    # C_007: source-route actual importer and stale selection.
    add_test(NAME qt-preview-c007-route-reference-risk
      COMMAND boardsi-preview-p0 --c007-route-selftest "${BOARDSI_IMPORTER}")
    set_tests_properties(qt-preview-c007-route-reference-risk PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    add_test(NAME qt-preview-p2a-partial-import COMMAND boardsi-preview-p0 --selftest
      "${BOARDSI_WORKER}" "${CMAKE_CURRENT_BINARY_DIR}/qt-model-a.frame"
      "${CMAKE_CURRENT_BINARY_DIR}/qt-model-b.frame" "${BOARDSI_IMPORTER}"
      "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/pads_route_straight_v94.asc")
    set_tests_properties(qt-preview-p2a-partial-import PROPERTIES TIMEOUT 35
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    add_test(NAME qt-preview-p2b-through-via COMMAND boardsi-preview-p0 --via-selftest
      "${BOARDSI_WORKER}" "${CMAKE_CURRENT_BINARY_DIR}/qt-model-a.frame"
      "${CMAKE_CURRENT_BINARY_DIR}/qt-model-b.frame" "${BOARDSI_IMPORTER}"
      "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/pads_via_round_through_v94.asc")
    set_tests_properties(qt-preview-p2b-through-via PROPERTIES TIMEOUT 30
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    add_test(NAME qt-preview-p2c-component-pads COMMAND boardsi-preview-p0 --pad-selftest
      "${BOARDSI_WORKER}" "${CMAKE_CURRENT_BINARY_DIR}/qt-model-a.frame"
      "${CMAKE_CURRENT_BINARY_DIR}/qt-model-b.frame" "${BOARDSI_IMPORTER}"
      "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/pads_component_simple_v94.asc")
    set_tests_properties(qt-preview-p2c-component-pads PROPERTIES TIMEOUT 30
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    add_test(NAME qt-preview-p2d-rf-rectangles COMMAND boardsi-preview-p0 --rect-selftest
      "${BOARDSI_WORKER}" "${CMAKE_CURRENT_BINARY_DIR}/qt-model-a.frame"
      "${CMAKE_CURRENT_BINARY_DIR}/qt-model-b.frame" "${BOARDSI_IMPORTER}"
      "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/pads_rf_rectangle_v94.asc")
    set_tests_properties(qt-preview-p2d-rf-rectangles PROPERTIES TIMEOUT 30
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
    add_test(NAME qt-preview-p2e-bottom-mirror COMMAND boardsi-preview-p0 --bottom-selftest
      "${BOARDSI_WORKER}" "${CMAKE_CURRENT_BINARY_DIR}/qt-model-a.frame"
      "${CMAKE_CURRENT_BINARY_DIR}/qt-model-b.frame" "${BOARDSI_IMPORTER}"
      "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/boardsi-import/tests/fixtures/pads_bottom_mirror_v94.asc")
    set_tests_properties(qt-preview-p2e-bottom-mirror PROPERTIES TIMEOUT 30
      ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
  endif()
  # Files are produced from boardsi_model by the Rust example before ctest.
  add_test(NAME qt-model-v2-identity COMMAND boardsi-qt-model-v2 --selftest
    "${BOARDSI_WORKER}" "${CMAKE_CURRENT_BINARY_DIR}/qt-model-a.frame" "${CMAKE_CURRENT_BINARY_DIR}/qt-model-b.frame")
  set_tests_properties(qt-model-v2-identity PROPERTIES TIMEOUT 30 ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
endif()
