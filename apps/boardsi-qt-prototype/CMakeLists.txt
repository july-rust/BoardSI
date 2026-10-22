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
  # Optional P2-A import smoke: actual Rust binary and a public synthetic fixture.
  set(BOARDSI_IMPORTER "" CACHE FILEPATH "Absolute built PADS route importer")
  if(BOARDSI_IMPORTER)
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
