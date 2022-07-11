cmake_minimum_required(VERSION 3.10)
set(CMAKE_CXX_STANDARD 14)
set(CMAKE_CXX_STANDARD_REQURIED ON)

project(play_cpp_sdk VERSION 0.0.1)

# Find bindings source files
file(GLOB_RECURSE DEFI_WALLET_CORE_CPP_BINDINGS include/defi-wallet-core-cpp/src/*.cc)
file(GLOB_RECURSE EXTRA_CPP_BINDINGS include/extra-cpp-bindings/src/*.cc)
file(GLOB PLAY_CPP_SDK_SROUCES include/*.cc)

# Find the rust types binding library
find_library(RUST_LIB cxxbridge1 REQUIRED PATHS ${CMAKE_CURRENT_SOURCE_DIR}/lib)

# Find the prebuilt static and dynamic libraries
if (WIN32)
  find_library(PLAY_CPP_SDK_LIB play_cpp_sdk.lib PATHS ${CMAKE_CURRENT_SOURCE_DIR}/lib)
  find_library(PLAY_CPP_SDK_DYLIB play_cpp_sdk.dll.lib PATHS ${CMAKE_CURRENT_SOURCE_DIR}/lib)
  find_file(PLAY_CPP_SDK_DLL NAME play_cpp_sdk.dll PATHS ${CMAKE_CURRENT_SOURCE_DIR}/lib)
  # Add static library play_cpp_sdk built from bindings source files
  add_library(play_cpp_sdk STATIC ${DEFI_WALLET_CORE_CPP_BINDINGS} ${EXTRA_CPP_BINDINGS} ${PLAY_CPP_SDK_SROUCES})
  target_link_libraries(play_cpp_sdk ${RUST_LIB} ${PLAY_CPP_SDK_DYLIB} Ncrypt userenv ntdll Secur32 crypt32)
endif()

if (APPLE)
  find_library(PLAY_CPP_SDK_LIB libplay_cpp_sdk.a PATHS ${CMAKE_CURRENT_SOURCE_DIR}/lib)
  find_library(PLAY_CPP_SDK_DYLIB libplay_cpp_sdk.dylib PATHS ${CMAKE_CURRENT_SOURCE_DIR}/lib)
  # Add static library play_cpp_sdk built from bindings source files
  add_library(play_cpp_sdk STATIC ${DEFI_WALLET_CORE_CPP_BINDINGS} ${EXTRA_CPP_BINDINGS} ${PLAY_CPP_SDK_SROUCES})
  target_link_libraries(play_cpp_sdk ${RUST_LIB} ${PLAY_CPP_SDK_DYLIB})
endif()

if (UNIX AND NOT APPLE)
  find_library(PLAY_CPP_SDK_LIB libplay_cpp_sdk.a PATHS ${CMAKE_CURRENT_SOURCE_DIR}/lib)
  # Add dynamic library play_cpp_sdk built from bindings source files
  add_library(play_cpp_sdk SHARED ${DEFI_WALLET_CORE_CPP_BINDINGS} ${EXTRA_CPP_BINDINGS} ${PLAY_CPP_SDK_SROUCES})
  target_link_libraries(play_cpp_sdk ${PLAY_CPP_SDK_LIB} pthread dl)
endif()
